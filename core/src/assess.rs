//! Task assessment: are a step's checks met by the editor state and the canvas?
//!
//! The app gathers a [`Snapshot`] (the `ui.inspect` reply and, for canvas checks, the
//! canvas saved to PNG by `app.save`) and asks [`task`] which steps pass. Nothing here
//! talks to the editor, so every check is testable with plain JSON and images.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use image::RgbaImage;
use serde::Serialize;
use serde_json::Value;

use crate::error::{Error, Result};
use crate::model::{Check, CheckKind, Lesson, Range, Region, Step, Task, num, parse_hex};
use crate::paths;
use crate::validate::canonical_tool;

/// What `ui.inspect` says, reduced to what checks read.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct EditorState {
    pub tool: Option<String>,
    pub brush: Option<BrushState>,
    /// Active view zoom: 1.0 is 100%.
    pub zoom: Option<f64>,
    pub document: Option<DocumentState>,
    /// 0–1 RGB.
    pub foreground: Option<[f64; 3]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BrushState {
    pub size: f64,
    pub hardness: f64,
    pub opacity: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct DocumentState {
    pub width: u32,
    pub height: u32,
    /// Every layer of the tree, groups included, top to bottom.
    pub layers: Vec<LayerState>,
    /// `[x, y, width, height]` in pixels.
    pub selection: Option<[f64; 4]>,
    pub history_entries: usize,
    pub can_redo: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LayerState {
    pub name: String,
    pub kind: String,
    pub visible: bool,
    pub opacity: f64,
}

impl EditorState {
    /// Reads the `result` of a `ui.inspect` reply (or the reply itself).
    pub fn from_inspect(reply: &Value) -> EditorState {
        let v = reply.get("result").unwrap_or(reply);
        let f = |v: &Value, key: &str| v.get(key).and_then(Value::as_f64);
        let brush = v.get("brush").and_then(|b| Some(BrushState { size: f(b, "size")?, hardness: f(b, "hardness")?, opacity: f(b, "opacity")? }));
        let active = v.pointer("/session/active").and_then(Value::as_u64).unwrap_or(0);
        let views = v.get("views").and_then(Value::as_array);
        let zoom = views.and_then(|views| views.get(usize::try_from(active).unwrap_or(0)).or(views.first())).and_then(|view| f(view, "zoom"));
        let foreground =
            v.pointer("/session/foreground").and_then(Value::as_array).and_then(|c| Some([c.first()?.as_f64()?, c.get(1)?.as_f64()?, c.get(2)?.as_f64()?]));
        let document = v.get("document").filter(|d| d.is_object()).map(|d| {
            let mut layers = Vec::new();
            flatten_layers(d.get("layers"), &mut layers);
            let selection = match d.get("hasSelection").and_then(Value::as_bool) {
                Some(true) => d
                    .get("selectionBounds")
                    .and_then(Value::as_array)
                    .and_then(|b| Some([b.first()?.as_f64()?, b.get(1)?.as_f64()?, b.get(2)?.as_f64()?, b.get(3)?.as_f64()?])),
                _ => None,
            };
            DocumentState {
                width: d.get("width").and_then(Value::as_u64).and_then(|w| u32::try_from(w).ok()).unwrap_or(0),
                height: d.get("height").and_then(Value::as_u64).and_then(|h| u32::try_from(h).ok()).unwrap_or(0),
                layers,
                selection,
                history_entries: d.get("history").and_then(Value::as_array).map_or(0, Vec::len),
                can_redo: d.get("canRedo").and_then(Value::as_bool).unwrap_or(false),
            }
        });
        EditorState { tool: v.get("tool").and_then(Value::as_str).map(Into::into), brush, zoom, document, foreground }
    }
}

fn flatten_layers(layers: Option<&Value>, out: &mut Vec<LayerState>) {
    let Some(list) = layers.and_then(Value::as_array) else { return };
    for layer in list {
        out.push(LayerState {
            name: layer.get("name").and_then(Value::as_str).unwrap_or_default().into(),
            kind: layer.get("kind").and_then(Value::as_str).unwrap_or_default().into(),
            visible: layer.get("visible").and_then(Value::as_bool).unwrap_or(true),
            opacity: layer.get("opacity").and_then(Value::as_f64).unwrap_or(1.0),
        });
        flatten_layers(layer.get("children"), out);
    }
}

/// What the canvas looked like before the student started.
#[derive(Debug, Clone)]
pub enum Base {
    Image(RgbaImage),
    Color([u8; 3]),
}

/// Everything about a lesson the checks compare against, loaded once per attempt.
#[derive(Debug, Clone)]
pub struct Context {
    pub base: Base,
    /// Reference pictures by their path in the lesson.
    pub references: HashMap<String, RgbaImage>,
    pub task_reference: Option<String>,
}

impl Context {
    /// Loads the task picture and the references a lesson's checks use.
    pub fn for_lesson(lesson: &Lesson, dir: &Path) -> Result<Context> {
        let task = &lesson.task;
        let base = match (&task.open, &task.canvas) {
            (Some(open), _) => Base::Image(load_image(&resolve(dir, open)?)?),
            (None, canvas) => Base::Color(rgb8(canvas.as_ref().map_or("#FFFFFF", |c| c.background.as_str()))),
        };
        let mut wanted: Vec<String> = task.reference.iter().cloned().collect();
        for step in &task.steps {
            for check in &step.checks {
                if let CheckKind::Similar { reference: Some(r), .. } = &check.kind {
                    wanted.push(r.clone());
                }
            }
        }
        let mut references = HashMap::new();
        for path in wanted {
            if let std::collections::hash_map::Entry::Vacant(slot) = references.entry(path) {
                let image = load_image(&resolve(dir, slot.key())?)?;
                slot.insert(image);
            }
        }
        Ok(Context { base, references, task_reference: task.reference.clone() })
    }

    /// A context without pictures: a blank white canvas.
    pub fn blank() -> Context {
        Context { base: Base::Color([255, 255, 255]), references: HashMap::new(), task_reference: None }
    }
}

fn resolve(dir: &Path, path: &str) -> Result<std::path::PathBuf> {
    paths::resolve(dir, path).map_err(Error::Invalid)
}

fn rgb8(hex: &str) -> [u8; 3] {
    let [r, g, b] = parse_hex(hex).unwrap_or([1.0, 1.0, 1.0]);
    [(r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8]
}

pub fn load_image(path: &Path) -> Result<RgbaImage> {
    image::open(path).map(|i| i.to_rgba8()).map_err(|e| Error::Image { path: path.to_path_buf(), message: e.to_string() })
}

/// The editor at the moment of checking.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub state: EditorState,
    pub canvas: Option<RgbaImage>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CheckOutcome {
    pub kind: &'static str,
    pub passed: bool,
    /// Why it failed (the author's `message` if set), or what was found when it passed.
    pub message: String,
    /// The measured value, when there is one.
    pub value: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StepOutcome {
    pub id: String,
    pub passed: bool,
    /// The step has no checks and passes when the student presses «Готово».
    pub manual: bool,
    pub checks: Vec<CheckOutcome>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskOutcome {
    pub passed: bool,
    /// Index of the first step that is not done yet.
    pub current: Option<usize>,
    pub steps: Vec<StepOutcome>,
}

/// Assesses every step. `confirmed` holds the ids of check-less steps the student marked done.
pub fn task(task: &Task, context: &Context, snapshot: &Snapshot, confirmed: &HashSet<String>) -> TaskOutcome {
    let steps: Vec<StepOutcome> = task.steps.iter().map(|s| step(s, context, snapshot, confirmed.contains(&s.id))).collect();
    let current = steps.iter().position(|s| !s.passed);
    TaskOutcome { passed: current.is_none() && !steps.is_empty(), current, steps }
}

/// Progress through a task. Steps are done in order and stay done: picking another tool
/// for step 3 must not undo step 1, which only asked to pick the brush.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Progress {
    pub done: usize,
}

impl Progress {
    /// Advances past every step that passes now, starting from the first open one; returns
    /// how many steps were completed by this update.
    pub fn update(&mut self, outcome: &TaskOutcome) -> usize {
        let before = self.done;
        while outcome.steps.get(self.done).is_some_and(|s| s.passed) {
            self.done += 1;
        }
        self.done - before
    }

    /// Index of the step to work on, or `None` when the task is finished.
    pub fn current(&self, steps: usize) -> Option<usize> {
        (self.done < steps).then_some(self.done)
    }

    pub fn finished(&self, steps: usize) -> bool {
        steps > 0 && self.done >= steps
    }
}

pub fn step(step: &Step, context: &Context, snapshot: &Snapshot, confirmed: bool) -> StepOutcome {
    let checks: Vec<CheckOutcome> = step.checks.iter().map(|c| check(c, context, snapshot)).collect();
    let manual = checks.is_empty();
    let passed = if manual { confirmed } else { checks.iter().all(|c| c.passed) };
    StepOutcome { id: step.id.clone(), passed, manual, checks }
}

pub fn check(check: &Check, context: &Context, snapshot: &Snapshot) -> CheckOutcome {
    let (passed, generated, value) = evaluate(&check.kind, context, snapshot);
    let message = match (&check.message, passed) {
        (Some(author), false) => author.clone(),
        _ => generated,
    };
    CheckOutcome { kind: check.kind.name(), passed, message, value }
}

type Verdict = (bool, String, Option<f64>);

fn verdict(passed: bool, ok: String, fail: String, value: Option<f64>) -> Verdict {
    (passed, if passed { ok } else { fail }, value)
}

fn pct(v: f64) -> String {
    num(v * 100.0)
}

fn evaluate(kind: &CheckKind, context: &Context, snapshot: &Snapshot) -> Verdict {
    let state = &snapshot.state;
    if kind.needs_canvas() {
        let Some(canvas) = &snapshot.canvas else {
            return (false, "Нет снимка холста для проверки.".into(), None);
        };
        return canvas_check(kind, context, canvas);
    }
    match kind {
        CheckKind::Tool { tool } => {
            let wanted = canonical_tool(tool).unwrap_or(tool);
            let current = state.tool.as_deref().and_then(canonical_tool);
            let name = tool_label(wanted);
            let fail = match current {
                Some(c) => format!("Выберите инструмент «{name}» — сейчас выбран «{}».", tool_label(c)),
                None => format!("Выберите инструмент «{name}»."),
            };
            verdict(current == Some(wanted), format!("Инструмент «{name}»."), fail, None)
        }
        CheckKind::Brush { size, hardness, opacity } => {
            let Some(brush) = state.brush else {
                return (false, "Редактор не сообщил параметры кисти.".into(), None);
            };
            let parts = [
                (size.as_ref(), brush.size, 1.0, "Размер кисти", " px"),
                (hardness.as_ref(), brush.hardness, 100.0, "Жёсткость кисти", "%"),
                (opacity.as_ref(), brush.opacity, 100.0, "Непрозрачность кисти", "%"),
            ];
            for (range, value, scale, label, unit) in parts {
                if let Some(range) = range
                    && !range.contains(value)
                {
                    let fail = format!("{label} — {}{unit}, нужно {}{unit}.", num(value * scale), range.scaled(scale));
                    return (false, fail, Some(value));
                }
            }
            (true, format!("Кисть {} px, жёсткость {}%, непрозрачность {}%.", num(brush.size), pct(brush.hardness), pct(brush.opacity)), None)
        }
        CheckKind::Zoom { zoom } => match state.zoom {
            Some(z) => verdict(zoom.contains(z), format!("Масштаб {}%.", pct(z)), format!("Масштаб {}%, нужно {}%.", pct(z), zoom.scaled(100.0)), Some(z)),
            None => (false, "Редактор не сообщил масштаб.".into(), None),
        },
        _ => {
            let Some(doc) = &state.document else {
                return (false, "В редакторе не открыт документ.".into(), None);
            };
            document_check(kind, doc, state)
        }
    }
}

fn document_check(kind: &CheckKind, doc: &DocumentState, state: &EditorState) -> Verdict {
    match kind {
        CheckKind::Layers { count } => {
            let n = doc.layers.len() as f64;
            verdict(count.contains(n), format!("Слоёв: {}.", num(n)), format!("Слоёв в документе: {}, нужно {count}.", num(n)), Some(n))
        }
        CheckKind::Layer { name, visible, opacity, layer_type } => {
            let found = doc.layers.iter().find(|l| {
                name.as_ref().is_none_or(|n| l.name.to_lowercase().contains(&n.to_lowercase()))
                    && visible.is_none_or(|v| l.visible == v)
                    && opacity.is_none_or(|r| r.contains(l.opacity))
                    && layer_type.as_ref().is_none_or(|k| l.kind.eq_ignore_ascii_case(k))
            });
            match found {
                Some(l) => (true, format!("Слой «{}».", l.name), Some(l.opacity)),
                None => (false, format!("Нет слоя{}.", describe_layer(name.as_deref(), *visible, opacity.as_ref())), None),
            }
        }
        CheckKind::Selection { present, width, height } => match (doc.selection, present) {
            (None, true) => (false, "Нет выделения.".into(), None),
            (None, false) => (true, "Выделения нет.".into(), None),
            (Some(_), false) => (false, "Снимите выделение: Выделение › Отменить выделение (Ctrl+D).".into(), None),
            (Some([_, _, w, h]), true) => {
                let (fw, fh) = (w / f64::from(doc.width.max(1)), h / f64::from(doc.height.max(1)));
                let size = format!("{}×{}% холста", pct(fw), pct(fh));
                if let Some(r) = width
                    && !r.contains(fw)
                {
                    return (false, format!("Ширина выделения — {}% холста, нужно {}%.", pct(fw), r.scaled(100.0)), Some(fw));
                }
                if let Some(r) = height
                    && !r.contains(fh)
                {
                    return (false, format!("Высота выделения — {}% холста, нужно {}%.", pct(fh), r.scaled(100.0)), Some(fh));
                }
                (true, format!("Выделение {size}."), None)
            }
        },
        CheckKind::History { entries, can_redo } => {
            let n = doc.history_entries as f64;
            if let Some(r) = entries
                && !r.contains(n)
            {
                return (false, format!("Шагов в истории: {}, нужно {r}.", num(n)), Some(n));
            }
            match can_redo {
                Some(true) if !doc.can_redo => (false, "Отмените последнее действие: Редактирование › Отменить (Ctrl+Z).".into(), Some(n)),
                Some(false) if doc.can_redo => (false, "Последнее действие отменено — повторите его (Ctrl+Shift+Z) или продолжайте рисовать.".into(), Some(n)),
                _ => (true, format!("Шагов в истории: {}.", num(n)), Some(n)),
            }
        }
        CheckKind::Foreground { color, tolerance } => {
            let (Some(have), Some(want)) = (state.foreground, parse_hex(color)) else {
                return (false, "Редактор не сообщил основной цвет.".into(), None);
            };
            let d = distance(have, want);
            verdict(d <= *tolerance, "Основной цвет подходит.".into(), format!("Основной цвет не тот: нужен {color}."), Some(d))
        }
        _ => (false, "Проверка не относится к документу.".into(), None),
    }
}

fn describe_layer(name: Option<&str>, visible: Option<bool>, opacity: Option<&Range>) -> String {
    let mut parts = Vec::new();
    if let Some(n) = name {
        parts.push(format!(" «{n}»"));
    }
    match visible {
        Some(true) => parts.push(", видимого".into()),
        Some(false) => parts.push(", скрытого".into()),
        None => {}
    }
    if let Some(r) = opacity {
        parts.push(format!(", с непрозрачностью {}%", r.scaled(100.0)));
    }
    parts.concat()
}

/// Russian names of the tools lessons use most, as in the PhotoCraft Russian catalog.
pub fn tool_label(tool: &str) -> &str {
    match tool {
        "Move" => "Перемещение",
        "RectMarquee" => "Прямоугольная область",
        "EllipseMarquee" => "Овальная область",
        "Lasso" => "Лассо",
        "MagicWand" => "Волшебная палочка",
        "Eyedropper" => "Пипетка",
        "Brush" => "Кисть",
        "Pencil" => "Карандаш",
        "Eraser" => "Ластик",
        "Gradient" => "Градиент",
        "PaintBucket" => "Заливка",
        "Hand" => "Рука",
        "Zoom" => "Масштаб",
        "Smudge" => "Палец",
        "Blur" => "Размытие",
        "Dodge" => "Осветлитель",
        "Burn" => "Затемнитель",
        other => other,
    }
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d: f64 = a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum();
    (d / 3.0).sqrt()
}

// ------------------------------------------------------------------ canvas checks

/// Change threshold per channel, out of 255.
pub const CHANGE_THRESHOLD: u8 = 24;
/// At most this many pixels are read per region; larger regions are sampled evenly.
const SAMPLE_BUDGET: u64 = 250_000;
const SIMILAR_SIZE: u32 = 64;

fn canvas_check(kind: &CheckKind, context: &Context, canvas: &RgbaImage) -> Verdict {
    match kind {
        CheckKind::Coverage { region, changed } => {
            let share = changed_share(canvas, &context.base, region);
            verdict(
                changed.contains(share),
                format!("Закрашено {}% области.", pct(share)),
                format!("Закрашено {}% области, нужно {}%.", pct(share), changed.scaled(100.0)),
                Some(share),
            )
        }
        CheckKind::Luminance { region, value } => {
            let l = luminance(mean(canvas, region));
            verdict(value.contains(l), format!("Яркость {}%.", pct(l)), format!("Яркость области — {}%, нужно {}%.", pct(l), value.scaled(100.0)), Some(l))
        }
        CheckKind::Darker { region, than, by } => {
            let (a, b) = (luminance(mean(canvas, region)), luminance(mean(canvas, than)));
            let d = b - a;
            verdict(
                d >= *by,
                format!("Область темнее соседней на {}%.", pct(d)),
                format!("Область должна быть темнее соседней хотя бы на {}%, сейчас разница {}%.", pct(*by), pct(d)),
                Some(d),
            )
        }
        CheckKind::Color { region, color, tolerance } => {
            let Some(want) = parse_hex(color) else {
                return (false, format!("Неверный цвет в уроке: {color}."), None);
            };
            let d = distance(mean(canvas, region), want);
            verdict(d <= *tolerance, "Цвет области подходит.".into(), format!("Цвет области далёк от нужного ({color})."), Some(d))
        }
        CheckKind::Similar { reference, score } => {
            let path = reference.as_ref().or(context.task_reference.as_ref());
            let Some(image) = path.and_then(|p| context.references.get(p)) else {
                return (false, "Нет эталона для сравнения.".into(), None);
            };
            let s = similarity(canvas, image);
            verdict(
                score.contains(s),
                format!("Похожесть на эталон {}%.", pct(s)),
                format!("Похожесть на эталон — {}%, нужно {}%.", pct(s), score.scaled(100.0)),
                Some(s),
            )
        }
        _ => (false, "Проверка не относится к холсту.".into(), None),
    }
}

/// Pixel bounds `[x0, y0, x1, y1)` of a fractional region, at least one pixel.
pub fn pixel_bounds(region: &Region, width: u32, height: u32) -> [u32; 4] {
    let [x, y, w, h] = *region;
    let span = |start: f64, len: f64, size: u32| {
        let size_f = f64::from(size.max(1));
        let a = ((start * size_f).floor().max(0.0) as u32).min(size.saturating_sub(1));
        let b = (((start + len) * size_f).ceil() as u32).clamp(a + 1, size.max(1));
        (a, b)
    };
    let (x0, x1) = span(x, w, width);
    let (y0, y1) = span(y, h, height);
    [x0, y0, x1, y1]
}

/// The pixels of a region to read: every pixel, or an even grid within the sample budget.
fn samples(region: &Region, width: u32, height: u32) -> impl Iterator<Item = (u32, u32)> {
    let [x0, y0, x1, y1] = pixel_bounds(region, width, height);
    let area = u64::from(x1 - x0) * u64::from(y1 - y0);
    let step = ((area as f64 / SAMPLE_BUDGET as f64).sqrt().ceil() as u32).max(1);
    (y0..y1).step_by(step as usize).flat_map(move |y| (x0..x1).step_by(step as usize).map(move |x| (x, y)))
}

/// A pixel laid over white, 0–255.
fn over_white(image: &RgbaImage, x: u32, y: u32) -> [f64; 3] {
    let [r, g, b, a] = image.get_pixel(x, y).0;
    let alpha = f64::from(a) / 255.0;
    [r, g, b].map(|c| f64::from(c) * alpha + 255.0 * (1.0 - alpha))
}

/// Mean 0–1 RGB of a region.
fn mean(image: &RgbaImage, region: &Region) -> [f64; 3] {
    let (mut sum, mut n) = ([0.0; 3], 0.0);
    for (x, y) in samples(region, image.width(), image.height()) {
        let p = over_white(image, x, y);
        for i in 0..3 {
            sum[i] += p[i];
        }
        n += 1.0;
    }
    if n == 0.0 { [1.0; 3] } else { sum.map(|s| s / n / 255.0) }
}

/// Rec. 709 luma of 0–1 RGB.
pub fn luminance([r, g, b]: [f64; 3]) -> f64 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Share (0–1) of the region's pixels that differ from the base.
fn changed_share(canvas: &RgbaImage, base: &Base, region: &Region) -> f64 {
    let (w, h) = canvas.dimensions();
    let (mut changed, mut n) = (0u64, 0u64);
    for (x, y) in samples(region, w, h) {
        let now = over_white(canvas, x, y);
        let before = match base {
            Base::Color(c) => c.map(f64::from),
            Base::Image(img) => {
                let bx = (u64::from(x) * u64::from(img.width()) / u64::from(w.max(1))) as u32;
                let by = (u64::from(y) * u64::from(img.height()) / u64::from(h.max(1))) as u32;
                over_white(img, bx.min(img.width().saturating_sub(1)), by.min(img.height().saturating_sub(1)))
            }
        };
        if now.iter().zip(before).any(|(a, b)| (a - b).abs() > f64::from(CHANGE_THRESHOLD)) {
            changed += 1;
        }
        n += 1;
    }
    if n == 0 { 0.0 } else { changed as f64 / n as f64 }
}

/// Grey thumbnail used by [`similarity`].
fn thumbnail(image: &RgbaImage) -> Vec<f64> {
    let mut cells = Vec::with_capacity((SIMILAR_SIZE * SIMILAR_SIZE) as usize);
    for cy in 0..SIMILAR_SIZE {
        for cx in 0..SIMILAR_SIZE {
            let s = f64::from(SIMILAR_SIZE);
            let region = [f64::from(cx) / s, f64::from(cy) / s, 1.0 / s, 1.0 / s];
            cells.push(luminance(mean(image, &region)));
        }
    }
    cells
}

/// 1 − mean absolute difference of 64×64 grey thumbnails: 1 for the same picture.
pub fn similarity(a: &RgbaImage, b: &RgbaImage) -> f64 {
    let (ta, tb) = (thumbnail(a), thumbnail(b));
    let diff: f64 = ta.iter().zip(&tb).map(|(x, y)| (x - y).abs()).sum();
    1.0 - diff / ta.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Range;
    use image::Rgba;
    use serde_json::json;

    fn inspect() -> Value {
        json!({"result": {
            "tool": "Brush",
            "brush": {"size": 40.0, "hardness": 0.0, "opacity": 0.3},
            "views": [{"zoom": 1.0}, {"zoom": 2.0}],
            "session": {"active": 1, "foreground": [0.0863, 0.0863, 0.102, 1.0]},
            "document": {
                "width": 1000, "height": 500,
                "layers": [
                    {"id": 3, "name": "Тень", "kind": "Pixel", "visible": true, "opacity": 0.5},
                    {"id": 2, "name": "Группа", "kind": "Group", "visible": false, "opacity": 1.0,
                     "children": [{"id": 4, "name": "Блик", "kind": "Pixel", "visible": true, "opacity": 1.0}]},
                    {"id": 1, "name": "Background", "kind": "Pixel", "visible": true, "opacity": 1.0}
                ],
                "hasSelection": true, "selectionBounds": [0, 0, 500, 100],
                "history": [{"name": "Open"}, {"name": "Brush"}, {"name": "Brush"}],
                "canRedo": true
            }
        }})
    }

    fn snap() -> Snapshot {
        Snapshot { state: EditorState::from_inspect(&inspect()), canvas: None }
    }

    fn run(kind: CheckKind) -> CheckOutcome {
        check(&kind.into(), &Context::blank(), &snap())
    }

    #[test]
    fn inspect_replies_are_read() {
        let s = EditorState::from_inspect(&inspect());
        assert_eq!(s.tool.as_deref(), Some("Brush"));
        assert_eq!(s.brush, Some(BrushState { size: 40.0, hardness: 0.0, opacity: 0.3 }));
        assert_eq!(s.zoom, Some(2.0), "the active document's view");
        let doc = s.document.unwrap();
        assert_eq!(doc.layers.len(), 4);
        assert_eq!(doc.layers[2].name, "Блик");
        assert_eq!(doc.selection, Some([0.0, 0.0, 500.0, 100.0]));
        assert_eq!(doc.history_entries, 3);
        assert!(doc.can_redo);
        assert_eq!(EditorState::from_inspect(&json!({})), EditorState::default());
    }

    #[test]
    fn tool_checks_match_photocraft_names() {
        assert!(run(CheckKind::Tool { tool: "brush".into() }).passed);
        let fail = run(CheckKind::Tool { tool: "Eraser".into() });
        assert!(!fail.passed);
        assert_eq!(fail.message, "Выберите инструмент «Ластик» — сейчас выбран «Кисть».");
    }

    #[test]
    fn brush_checks_report_the_first_mismatch_in_display_units() {
        assert!(run(CheckKind::Brush { size: Some(Range::between(30.0, 60.0)), hardness: Some(Range::at_most(0.2)), opacity: None }).passed);
        let fail = run(CheckKind::Brush { size: None, hardness: None, opacity: Some(Range::at_least(0.5)) });
        assert!(!fail.passed);
        assert_eq!(fail.message, "Непрозрачность кисти — 30%, нужно не меньше 50%.");
    }

    #[test]
    fn zoom_layers_and_layer_checks() {
        assert!(run(CheckKind::Zoom { zoom: Range::at_least(1.5) }).passed);
        assert_eq!(run(CheckKind::Zoom { zoom: Range::at_most(1.0) }).message, "Масштаб 200%, нужно не больше 100%.");
        assert!(run(CheckKind::Layers { count: Range::at_least(2.0) }).passed);
        assert!(!run(CheckKind::Layers { count: Range::at_least(5.0) }).passed);
        assert!(run(CheckKind::Layer { name: Some("тень".into()), visible: Some(true), opacity: Some(Range::between(0.4, 0.6)), layer_type: None }).passed);
        assert!(run(CheckKind::Layer { name: None, visible: Some(false), opacity: None, layer_type: Some("group".into()) }).passed);
        let fail = run(CheckKind::Layer { name: Some("Тень".into()), visible: Some(false), opacity: None, layer_type: None });
        assert_eq!(fail.message, "Нет слоя «Тень», скрытого.");
    }

    #[test]
    fn selection_and_history_checks() {
        assert!(run(CheckKind::Selection { present: true, width: Some(Range::between(0.4, 0.6)), height: Some(Range::at_most(0.25)) }).passed);
        assert!(!run(CheckKind::Selection { present: false, width: None, height: None }).passed);
        let fail = run(CheckKind::Selection { present: true, width: Some(Range::at_least(0.9)), height: None });
        assert_eq!(fail.message, "Ширина выделения — 50% холста, нужно не меньше 90%.");
        assert!(run(CheckKind::History { entries: Some(Range::at_least(2.0)), can_redo: Some(true) }).passed);
        assert!(!run(CheckKind::History { entries: None, can_redo: Some(false) }).passed);
        assert!(!run(CheckKind::History { entries: Some(Range::at_least(10.0)), can_redo: None }).passed);
    }

    #[test]
    fn foreground_colour_is_compared_with_tolerance() {
        assert!(run(CheckKind::Foreground { color: "#16161A".into(), tolerance: 0.05 }).passed);
        assert!(!run(CheckKind::Foreground { color: "#BC3823".into(), tolerance: 0.1 }).passed);
    }

    #[test]
    fn state_checks_without_a_document_fail_politely() {
        let empty = Snapshot::default();
        let out = check(&CheckKind::Layers { count: Range::at_least(1.0) }.into(), &Context::blank(), &empty);
        assert!(!out.passed);
        assert_eq!(out.message, "В редакторе не открыт документ.");
        let out = check(&CheckKind::Coverage { region: [0.0, 0.0, 1.0, 1.0], changed: Range::at_least(0.1) }.into(), &Context::blank(), &empty);
        assert_eq!(out.message, "Нет снимка холста для проверки.");
    }

    #[test]
    fn the_authors_message_replaces_the_generated_one_on_failure_only() {
        let c = Check { kind: CheckKind::Tool { tool: "Eraser".into() }, message: Some("Возьмите ластик!".into()) };
        assert_eq!(check(&c, &Context::blank(), &snap()).message, "Возьмите ластик!");
        let c = Check { kind: CheckKind::Tool { tool: "Brush".into() }, message: Some("Возьмите кисть!".into()) };
        assert_eq!(check(&c, &Context::blank(), &snap()).message, "Инструмент «Кисть».");
    }

    /// 100×100 white canvas with the left half painted `color`.
    fn half_painted(color: [u8; 4]) -> RgbaImage {
        RgbaImage::from_fn(100, 100, |x, _| if x < 50 { Rgba(color) } else { Rgba([255, 255, 255, 255]) })
    }

    fn run_canvas(kind: CheckKind, canvas: RgbaImage, context: &Context) -> CheckOutcome {
        check(&kind.into(), context, &Snapshot { state: EditorState::default(), canvas: Some(canvas) })
    }

    #[test]
    fn coverage_measures_change_against_the_base() {
        let ctx = Context::blank();
        let painted = half_painted([22, 22, 26, 255]);
        let whole = run_canvas(CheckKind::Coverage { region: [0.0, 0.0, 1.0, 1.0], changed: Range::between(0.45, 0.55) }, painted.clone(), &ctx);
        assert!(whole.passed, "{whole:?}");
        let left = run_canvas(CheckKind::Coverage { region: [0.0, 0.0, 0.5, 1.0], changed: Range::at_least(0.99) }, painted.clone(), &ctx);
        assert!(left.passed);
        let right = run_canvas(CheckKind::Coverage { region: [0.5, 0.0, 0.5, 1.0], changed: Range::at_most(0.01) }, painted.clone(), &ctx);
        assert!(right.passed);
        let base = Context { base: Base::Image(painted.clone()), ..Context::blank() };
        let none = run_canvas(CheckKind::Coverage { region: [0.0, 0.0, 1.0, 1.0], changed: Range::at_least(0.1) }, painted, &base);
        assert!(!none.passed, "the task picture itself is not the student's work");
        assert_eq!(none.message, "Закрашено 0% области, нужно не меньше 10%.");
    }

    #[test]
    fn a_base_picture_of_another_size_is_scaled() {
        let base = Context { base: Base::Image(RgbaImage::from_pixel(10, 10, Rgba([255, 255, 255, 255]))), ..Context::blank() };
        let out = run_canvas(CheckKind::Coverage { region: [0.0, 0.0, 1.0, 1.0], changed: Range::between(0.45, 0.55) }, half_painted([0, 0, 0, 255]), &base);
        assert!(out.passed, "{out:?}");
    }

    #[test]
    fn transparent_pixels_count_as_white() {
        let ctx = Context::blank();
        let clear = RgbaImage::from_pixel(20, 20, Rgba([0, 0, 0, 0]));
        assert!(run_canvas(CheckKind::Coverage { region: [0.0, 0.0, 1.0, 1.0], changed: Range::at_most(0.0) }, clear.clone(), &ctx).passed);
        assert!(run_canvas(CheckKind::Luminance { region: [0.0, 0.0, 1.0, 1.0], value: Range::at_least(0.99) }, clear, &ctx).passed);
    }

    #[test]
    fn luminance_darker_and_colour() {
        let ctx = Context::blank();
        let canvas = half_painted([188, 56, 35, 255]);
        let left = [0.0, 0.0, 0.5, 1.0];
        let right = [0.5, 0.0, 0.5, 1.0];
        assert!(run_canvas(CheckKind::Luminance { region: right, value: Range::at_least(0.99) }, canvas.clone(), &ctx).passed);
        assert!(run_canvas(CheckKind::Darker { region: left, than: right, by: 0.3 }, canvas.clone(), &ctx).passed);
        let fail = run_canvas(CheckKind::Darker { region: right, than: left, by: 0.05 }, canvas.clone(), &ctx);
        assert!(!fail.passed);
        assert!(fail.value.is_some_and(|d| d < 0.0));
        assert!(run_canvas(CheckKind::Color { region: left, color: "#BC3823".into(), tolerance: 0.02 }, canvas.clone(), &ctx).passed);
        assert!(!run_canvas(CheckKind::Color { region: left, color: "#2F5D8C".into(), tolerance: 0.12 }, canvas, &ctx).passed);
    }

    #[test]
    fn similarity_is_one_for_the_same_picture_and_drops_with_difference() {
        let a = half_painted([0, 0, 0, 255]);
        assert!((similarity(&a, &a) - 1.0).abs() < 1e-9);
        let white = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
        let s = similarity(&a, &white);
        assert!((s - 0.5).abs() < 0.02, "{s}");
        let bigger = RgbaImage::from_fn(400, 400, |x, _| if x < 200 { Rgba([0, 0, 0, 255]) } else { Rgba([255, 255, 255, 255]) });
        assert!(similarity(&a, &bigger) > 0.98, "size does not matter");
        let mut ctx = Context::blank();
        ctx.references.insert("assets/ref.png".into(), bigger);
        ctx.task_reference = Some("assets/ref.png".into());
        assert!(run_canvas(CheckKind::Similar { reference: None, score: Range::at_least(0.9) }, a.clone(), &ctx).passed);
        let missing = run_canvas(CheckKind::Similar { reference: Some("assets/other.png".into()), score: Range::at_least(0.9) }, a, &ctx);
        assert_eq!(missing.message, "Нет эталона для сравнения.");
    }

    #[test]
    fn pixel_bounds_stay_inside_and_are_never_empty() {
        assert_eq!(pixel_bounds(&[0.0, 0.0, 1.0, 1.0], 100, 50), [0, 0, 100, 50]);
        assert_eq!(pixel_bounds(&[0.25, 0.5, 0.5, 0.5], 100, 50), [25, 25, 75, 50]);
        assert_eq!(pixel_bounds(&[1.0, 1.0, 0.0, 0.0], 100, 50), [99, 49, 100, 50]);
        assert_eq!(pixel_bounds(&[0.0, 0.0, 0.001, 0.001], 10, 10), [0, 0, 1, 1]);
    }

    #[test]
    fn big_regions_are_sampled_within_budget() {
        let n = samples(&[0.0, 0.0, 1.0, 1.0], 4000, 3000).count() as u64;
        assert!(n <= SAMPLE_BUDGET && n > SAMPLE_BUDGET / 4, "{n}");
        assert_eq!(samples(&[0.0, 0.0, 1.0, 1.0], 10, 10).count(), 100);
    }

    fn task_with(steps: Vec<Step>) -> Task {
        Task { brief: String::new(), open: None, reference: None, canvas: None, steps }
    }

    fn step_with(id: &str, checks: Vec<Check>) -> Step {
        Step { id: id.into(), text: id.into(), hint: None, checks }
    }

    #[test]
    fn a_task_passes_when_every_step_does_and_points_at_the_first_open_step() {
        let t = task_with(vec![
            step_with("tool", vec![CheckKind::Tool { tool: "Brush".into() }.into()]),
            step_with("look", vec![]),
            step_with("zoom", vec![CheckKind::Zoom { zoom: Range::at_least(4.0) }.into()]),
        ]);
        let none = HashSet::new();
        let out = task(&t, &Context::blank(), &snap(), &none);
        assert!(!out.passed);
        assert_eq!(out.current, Some(1));
        assert!(out.steps[1].manual);
        let confirmed: HashSet<String> = ["look".to_string()].into();
        let out = task(&t, &Context::blank(), &snap(), &confirmed);
        assert_eq!(out.current, Some(2));
        let done = task(&task_with(vec![t.steps[0].clone()]), &Context::blank(), &snap(), &confirmed);
        assert!(done.passed);
        assert_eq!(done.current, None);
        assert!(!task(&task_with(vec![]), &Context::blank(), &snap(), &none).passed);
    }

    #[test]
    fn progress_latches_done_steps_and_keeps_order() {
        let t = task_with(vec![
            step_with("tool", vec![CheckKind::Tool { tool: "Brush".into() }.into()]),
            step_with("zoom", vec![CheckKind::Zoom { zoom: Range::at_least(2.0) }.into()]),
            step_with("eraser", vec![CheckKind::Tool { tool: "Eraser".into() }.into()]),
        ]);
        let none = HashSet::new();
        let mut progress = Progress::default();
        // Brush at 200%: the first two steps pass at once.
        assert_eq!(progress.update(&task(&t, &Context::blank(), &snap(), &none)), 2);
        assert_eq!(progress.current(3), Some(2));
        // Switching to the eraser fails step 1 now, but it stays done.
        let mut eraser = snap();
        eraser.state.tool = Some("Eraser".into());
        assert_eq!(progress.update(&task(&t, &Context::blank(), &eraser, &none)), 1);
        assert!(progress.finished(3));
        assert_eq!(progress.current(3), None);
        // A later step passing early does not skip an open one.
        let mut fresh = Progress::default();
        assert_eq!(fresh.update(&task(&t, &Context::blank(), &eraser, &none)), 0);
        assert!(!Progress::default().finished(0));
    }

    #[test]
    fn russian_tool_labels_cover_the_lesson_tools() {
        for tool in ["Brush", "Eraser", "RectMarquee", "EllipseMarquee", "Zoom", "Hand", "PaintBucket", "Move"] {
            assert_ne!(tool_label(tool), tool);
        }
        assert_eq!(tool_label("Frame"), "Frame");
    }
}
