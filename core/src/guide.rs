//! Animated guides: `guide.json`, a script of PhotoCraft control calls with captions.
//!
//! Lessons can be imported from strangers, so a script may only use an allowlist of
//! methods and engine commands: painting, layers, selections, undo and brush settings.
//! Opening or saving files, scripts, shortcuts and anything else is refused.

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::control::{self, Call};
use crate::error::{self, Error, Result};
use crate::model::{Canvas, FORMAT, parse_hex};
use crate::paths::{self, IMAGE_EXTENSIONS};
use crate::validate::{Issue, check_target};

/// Engine commands a guide may run through `engine.execute`.
pub const ALLOWED_COMMANDS: &[&str] = &[
    "paint.stroke",
    "layer.new.layer",
    "layer.setProps",
    "layer.select",
    "layer.selectTop",
    "layer.selectBottom",
    "layer.delete",
    "select.rect",
    "select.all",
    "select.deselect",
    "select.inverse",
    "edit.fill",
    "edit.undo",
    "edit.redo",
    "tools.setBrush",
    "tools.setColors",
    "tools.defaultColors",
    "tools.swapColors",
    "file.new",
];

/// `ui.set` fields a guide may change.
pub const ALLOWED_UI_SET: &[&str] = &["tool", "zoom", "center", "rotation", "fit", "brushSize"];
/// `ui.pointer` fields a guide may pass.
const ALLOWED_POINTER: &[&str] = &["events", "tool", "modifiers", "button"];

pub const MAX_STEPS: usize = 400;
pub const MAX_CALLS: usize = 2000;
pub const MAX_POINTS: usize = 5000;
pub const MAX_PAUSE_MS: u32 = 15_000;
pub const DEFAULT_PAUSE_MS: u32 = 600;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Script {
    pub format: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// A picture to open first, relative to the lesson folder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open: Option<String>,
    /// Or a blank canvas to start on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canvas: Option<Canvas>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    #[serde(default)]
    pub caption: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight: Option<String>,
    #[serde(default)]
    pub calls: Vec<Call>,
    /// Milliseconds to wait after the step; [`DEFAULT_PAUSE_MS`] when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pause: Option<u32>,
    /// The task step this part of the guide shows (`task.steps[].id`), for the author's
    /// timeline. The player ignores it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_step: Option<String>,
}

impl Step {
    pub fn pause_ms(&self) -> u32 {
        self.pause.unwrap_or(DEFAULT_PAUSE_MS).min(MAX_PAUSE_MS)
    }
}

impl Script {
    /// Calls that prepare the editor before the first step.
    pub fn setup(&self) -> Vec<Call> {
        match (&self.open, &self.canvas) {
            (Some(open), _) => vec![Call::new("app.open", serde_json::json!({ "path": open }))],
            (None, Some(canvas)) => {
                let task = crate::model::Task { brief: String::new(), open: None, reference: None, canvas: Some(canvas.clone()), steps: vec![] };
                control::task_setup(&task)
            }
            (None, None) => vec![],
        }
    }

    /// Total of the pauses: roughly how long the guide plays, not counting the editor's own time.
    pub fn duration_ms(&self) -> u64 {
        self.steps.iter().map(|s| u64::from(s.pause_ms())).sum()
    }

    pub fn call_count(&self) -> usize {
        self.steps.iter().map(|s| s.calls.len()).sum()
    }
}

pub fn parse(text: &str, path: &Path) -> Result<Script> {
    serde_json::from_str(text).map_err(|e| Error::Json { path: path.to_path_buf(), message: e.to_string() })
}

pub fn load(path: &Path) -> Result<Script> {
    let text = std::fs::read_to_string(path).map_err(error::io(path))?;
    parse(&text, path)
}

pub fn to_json(script: &Script) -> Result<String> {
    serde_json::to_string_pretty(script).map_err(|e| Error::Serialize(e.to_string()))
}

/// Checks a script; with `dir`, also that the picture it opens exists.
pub fn validate(script: &Script, dir: Option<&Path>) -> Vec<Issue> {
    let mut issues = Vec::new();
    if script.format != FORMAT {
        issues.push(Issue::error(
            "format",
            crate::tr!("версия формата {} не поддерживается (нужна {FORMAT})", "format version {} is not supported ({FORMAT} is needed)", script.format),
        ));
    }
    if script.open.is_some() && script.canvas.is_some() {
        issues.push(Issue::error("open", crate::i18n::pick("укажите что-то одно: open или canvas", "give only one of open or canvas")));
    }
    if let Some(open) = &script.open {
        match paths::check_relative(open) {
            Err(e) => issues.push(Issue::error("open", e)),
            Ok(()) if !paths::has_extension(open, IMAGE_EXTENSIONS) => {
                issues.push(Issue::error("open", crate::i18n::pick("нужна картинка png, jpg или webp", "a png, jpg or webp picture is needed")))
            }
            Ok(()) => {
                if let Some(dir) = dir
                    && !paths::resolve(dir, open).is_ok_and(|p| p.is_file())
                {
                    issues.push(Issue::error("open", crate::tr!("файл «{open}» не найден", "file «{open}» not found")));
                }
            }
        }
    }
    if let Some(canvas) = &script.canvas
        && (!(16..=8192).contains(&canvas.width) || !(16..=8192).contains(&canvas.height) || parse_hex(&canvas.background).is_none())
    {
        issues
            .push(Issue::error("canvas", crate::i18n::pick("холст от 16 до 8192 пикселей, фон #rrggbb", "a canvas of 16 to 8192 pixels, background #rrggbb")));
    }
    if script.steps.is_empty() {
        issues.push(Issue::error("steps", crate::i18n::pick("в гайде нет шагов", "the guide has no steps")));
    }
    if script.steps.len() > MAX_STEPS {
        issues.push(Issue::error("steps", crate::tr!("больше {MAX_STEPS} шагов", "more than {MAX_STEPS} steps")));
    }
    if script.call_count() > MAX_CALLS {
        issues.push(Issue::error("steps", crate::tr!("больше {MAX_CALLS} команд", "more than {MAX_CALLS} commands")));
    }
    for (i, step) in script.steps.iter().enumerate() {
        let field = format!("steps[{i}]");
        if let Some(target) = &step.highlight
            && let Err(e) = check_target(target)
        {
            issues.push(Issue::error(format!("{field}.highlight"), e));
        }
        if step.pause.is_some_and(|p| p > MAX_PAUSE_MS) {
            issues.push(Issue::error(format!("{field}.pause"), crate::tr!("пауза больше {MAX_PAUSE_MS} мс", "a pause longer than {MAX_PAUSE_MS} ms")));
        }
        if step.caption.trim().is_empty() && step.calls.is_empty() {
            issues.push(Issue::warning(&field, crate::i18n::pick("пустой шаг: ни подписи, ни команд", "an empty step: no caption and no commands")));
        }
        for (j, call) in step.calls.iter().enumerate() {
            if let Err(e) = check_call(call) {
                issues.push(Issue::error(format!("{field}.calls[{j}]"), e));
            }
        }
    }
    issues
}

/// Whether a guide may send this call; the error says why not.
pub fn check_call(call: &Call) -> std::result::Result<(), String> {
    let Some(params) = call.params.as_object() else {
        return Err(crate::tr!("params должен быть объектом", "params must be an object"));
    };
    match call.method.as_str() {
        "ui.set" => only_keys(params, ALLOWED_UI_SET, "ui.set"),
        "ui.pointer" => {
            only_keys(params, ALLOWED_POINTER, "ui.pointer")?;
            let events = params
                .get("events")
                .and_then(Value::as_array)
                .ok_or(crate::i18n::pick("ui.pointer: нужен массив events", "ui.pointer: an events array is needed"))?;
            if events.len() > MAX_POINTS {
                return Err(crate::tr!("ui.pointer: больше {MAX_POINTS} событий", "ui.pointer: more than {MAX_POINTS} events"));
            }
            for event in events {
                let kind = event.get("kind").and_then(Value::as_str);
                if !matches!(kind, Some("down" | "move" | "up")) {
                    return Err(crate::tr!("ui.pointer: kind — down, move или up", "ui.pointer: kind is down, move or up"));
                }
                if !(event.get("x").is_some_and(Value::is_number) && event.get("y").is_some_and(Value::is_number)) {
                    return Err(crate::tr!("ui.pointer: у события нужны числа x и y", "ui.pointer: an event needs numbers x and y"));
                }
            }
            Ok(())
        }
        "engine.execute" => {
            only_keys(params, &["command", "params"], "engine.execute")?;
            let command = params
                .get("command")
                .and_then(Value::as_str)
                .ok_or(crate::i18n::pick("engine.execute: нужна строка command", "engine.execute: a command string is needed"))?;
            if !ALLOWED_COMMANDS.contains(&command) {
                return Err(crate::tr!(
                    "команда «{command}» в гайде запрещена; разрешены: {}",
                    "the «{command}» command is not allowed in a guide; allowed: {}",
                    ALLOWED_COMMANDS.join(", ")
                ));
            }
            let inner = params.get("params").cloned().unwrap_or_else(|| serde_json::json!({}));
            if !inner.is_object() {
                return Err(crate::tr!("{command}: params должен быть объектом", "{command}: params must be an object"));
            }
            match command {
                "paint.stroke" => {
                    let points = inner
                        .get("points")
                        .and_then(Value::as_array)
                        .ok_or(crate::i18n::pick("paint.stroke: нужен массив points", "paint.stroke: a points array is needed"))?;
                    if points.is_empty() || points.len() > MAX_POINTS {
                        return Err(crate::tr!("paint.stroke: от 1 до {MAX_POINTS} точек", "paint.stroke: 1 to {MAX_POINTS} points"));
                    }
                    let well_formed = points.iter().all(|p| p.as_array().is_some_and(|xy| xy.len() >= 2 && xy.iter().all(Value::is_number)));
                    if !well_formed {
                        return Err(crate::tr!("paint.stroke: точка — массив [x, y, нажим?]", "paint.stroke: a point is an array [x, y, pressure?]"));
                    }
                    Ok(())
                }
                "file.new" => {
                    let size = |key: &str| inner.get(key).and_then(Value::as_u64).unwrap_or(1);
                    if size("width") > 8192 || size("height") > 8192 {
                        return Err(crate::tr!("file.new: холст больше 8192 пикселей", "file.new: a canvas larger than 8192 pixels"));
                    }
                    Ok(())
                }
                _ => Ok(()),
            }
        }
        other => Err(crate::tr!(
            "метод «{other}» в гайде запрещён; разрешены ui.set, ui.pointer, engine.execute",
            "the «{other}» method is not allowed in a guide; allowed: ui.set, ui.pointer, engine.execute"
        )),
    }
}

fn only_keys(params: &serde_json::Map<String, Value>, allowed: &[&str], method: &str) -> std::result::Result<(), String> {
    match params.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(key) => Err(crate::tr!(
            "{method}: поле «{key}» в гайде запрещено; разрешены: {}",
            "{method}: the «{key}» field is not allowed in a guide; allowed: {}",
            allowed.join(", ")
        )),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const SCRIPT: &str = r##"{
      "format": 1,
      "title": "Кисть",
      "canvas": {"width": 800, "height": 600, "background": "#F3EEE4"},
      "steps": [
        {"caption": "Берём кисть", "highlight": "tool:Brush",
         "calls": [{"method": "ui.set", "params": {"tool": "Brush"}}], "pause": 1000},
        {"caption": "Штрих",
         "calls": [{"method": "engine.execute", "params": {"command": "paint.stroke",
                    "params": {"points": [[100, 100, 1], [300, 120, 0.8]], "size": 20, "color": "#16161A"}}}]}
      ]
    }"##;

    fn script() -> Script {
        parse(SCRIPT, Path::new("guide.json")).unwrap()
    }

    #[test]
    fn a_script_reads_and_is_valid() {
        let s = script();
        assert!(validate(&s, None).is_empty(), "{:?}", validate(&s, None));
        assert_eq!(s.duration_ms(), 1000 + u64::from(DEFAULT_PAUSE_MS));
        assert_eq!(s.call_count(), 2);
        let setup = s.setup();
        assert_eq!(setup[0].command(), Some("file.new"));
        assert_eq!(setup[1].command(), Some("edit.fill"));
    }

    #[test]
    fn a_script_round_trips() {
        let s = script();
        assert_eq!(parse(&to_json(&s).unwrap(), Path::new("g")).unwrap(), s);
    }

    #[test]
    fn file_and_script_commands_are_refused() {
        for (method, params) in [
            ("app.open", json!({"path": "x.png"})),
            ("app.save", json!({"path": "x.png"})),
            ("ui.key", json!({"key": "Ctrl+S"})),
            ("ui.menu.invoke", json!({"id": "file.save"})),
            ("engine.execute", json!({"command": "file.save", "params": {}})),
            ("engine.execute", json!({"command": "file.automate.batch", "params": {}})),
            ("engine.execute", json!({"command": "help.github"})),
            ("engine.execute", json!({"command": "paint.stroke", "params": {"points": []}})),
            ("engine.execute", json!({"command": "paint.stroke", "params": {"points": [["a", 1]]}})),
            ("engine.execute", json!({"command": "file.new", "params": {"width": 100000, "height": 10}})),
            ("engine.execute", json!({"command": "edit.fill", "extra": 1})),
            ("engine.execute", json!({"command": 5})),
            ("ui.set", json!({"theme": "classic"})),
            ("ui.set", json!({"strokeTrace": "C:/x"})),
            ("ui.pointer", json!({"events": [{"kind": "tap", "x": 0, "y": 0}]})),
            ("ui.pointer", json!({"events": [{"kind": "down", "x": "0", "y": 0}]})),
            ("ui.pointer", json!({})),
        ] {
            let call = Call::new(method, params.clone());
            assert!(check_call(&call).is_err(), "{method} {params} must be refused");
        }
        assert!(check_call(&Call { method: "ui.set".into(), params: json!([1]) }).is_err());
    }

    #[test]
    fn allowed_calls_pass() {
        for call in [
            Call::new("ui.set", json!({"tool": "Eraser", "brushSize": 30})),
            Call::new("ui.pointer", json!({"tool": "Brush", "events": [{"kind": "down", "x": 1, "y": 2}, {"kind": "up", "x": 3, "y": 4}]})),
            Call::engine("layer.new.layer", json!({"name": "Тень"})),
            Call::engine("tools.setBrush", json!({"size": 60, "hardness": 0})),
            Call::engine("select.rect", json!({"x": 0, "y": 0, "width": 10, "height": 10, "ellipse": true})),
            Call::new("engine.execute", json!({"command": "edit.undo"})),
        ] {
            assert_eq!(check_call(&call), Ok(()), "{call:?}");
        }
    }

    #[test]
    fn script_level_problems_are_reported() {
        let mut s = script();
        s.format = 9;
        s.open = Some("../x.png".into());
        s.steps[0].highlight = Some("tool:Laser".into());
        s.steps[0].pause = Some(MAX_PAUSE_MS + 1);
        s.steps[1].calls.push(Call::new("app.save", json!({})));
        let fields: Vec<_> = validate(&s, None).into_iter().filter(Issue::is_error).map(|i| i.field).collect();
        assert_eq!(fields, vec!["format", "open", "open", "steps[0].highlight", "steps[0].pause", "steps[1].calls[1]"]);
        let empty = Script { format: 1, title: None, open: None, canvas: None, steps: vec![] };
        assert_eq!(validate(&empty, None).len(), 1);
        assert!(empty.setup().is_empty());
    }

    #[test]
    fn pauses_are_capped() {
        let step = Step { caption: String::new(), highlight: None, calls: vec![], pause: Some(u32::MAX), task_step: None };
        assert_eq!(step.pause_ms(), MAX_PAUSE_MS);
    }

    #[test]
    fn unknown_fields_in_a_script_are_errors() {
        let text = SCRIPT.replace("\"title\"", "\"autorun\": true, \"title\"");
        assert!(parse(&text, Path::new("g")).is_err());
    }
}
