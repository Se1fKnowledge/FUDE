//! Turning what an author did in PhotoCraft into guide steps.
//!
//! While the author records, FUDE reads PhotoCraft's journal of executed commands and watches
//! the active tool. Here those events become [`Step`]s: a tool change highlights and selects
//! the tool, every stroke is its own step (so it shows as one clip on the timeline), and other
//! commands the guide allowlist permits join the step before them. Commands a guide may not
//! run are left out and reported.

use serde::Serialize;
use serde_json::{Value, json};

use crate::control::Call;
use crate::guide::{ALLOWED_COMMANDS, MAX_PAUSE_MS, MAX_POINTS, MAX_STEPS, Step};
use crate::validate::canonical_tool;

/// One thing the author did, `at_ms` after recording started.
#[derive(Debug, Clone, PartialEq)]
pub struct Recorded {
    pub at_ms: u64,
    pub event: Event,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The active tool became this one (PhotoCraft's spelling).
    Tool(String),
    /// A journaled engine command.
    Command { id: String, params: Value },
}

/// The brush when recording starts; strokes carry it explicitly so the guide does not depend
/// on the student's brush.
#[derive(Debug, Clone, PartialEq)]
pub struct Brush {
    pub size: f64,
    pub hardness: f64,
    pub opacity: f64,
    /// `#rrggbb`.
    pub color: String,
}

impl Default for Brush {
    fn default() -> Self {
        Brush { size: 30.0, hardness: 0.8, opacity: 1.0, color: "#000000".into() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Recording {
    pub steps: Vec<Step>,
    pub strokes: usize,
    /// Commands left out because a guide may not run them, one entry per kind.
    pub skipped: Vec<String>,
}

/// Shortest pause kept between recorded steps.
const MIN_PAUSE_MS: u64 = 150;

/// What the author chose before recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// Pauses longer than this (the author thinking) shrink to it; [`MAX_PAUSE_MS`] keeps them.
    pub longest_pause_ms: u32,
    /// Leave out undo and redo, so the guide shows only the work that stayed.
    pub skip_undo: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options { longest_pause_ms: 2000, skip_undo: true }
    }
}

/// Builds guide steps from recorded events.
pub fn to_guide(events: &[Recorded], start: &Brush, options: Options) -> Recording {
    let mut brush = start.clone();
    let mut steps: Vec<(u64, Step, bool)> = Vec::new(); // (start time, step, holds a stroke)
    let mut skipped: Vec<String> = Vec::new();
    let mut strokes = 0;
    let push = |steps: &mut Vec<(u64, Step, bool)>, at: u64, call: Call, highlight: Option<String>, stroke: bool| {
        let joins = !stroke && highlight.is_none() && steps.last().is_some_and(|(_, _, s)| !s);
        if joins && let Some((_, step, _)) = steps.last_mut() {
            step.calls.push(call);
            return;
        }
        steps.push((at, Step { caption: String::new(), highlight, calls: vec![call], pause: None, task_step: None }, stroke));
    };
    for e in events {
        match &e.event {
            Event::Tool(tool) => {
                let Some(name) = canonical_tool(tool) else { continue };
                push(&mut steps, e.at_ms, Call::new("ui.set", json!({ "tool": name })), Some(format!("tool:{name}")), false);
            }
            Event::Command { id, params } => match id.as_str() {
                "paint.stroke" => {
                    let Some(points) = stroke_points(params) else { continue };
                    let mut p = json!({ "points": points, "size": brush.size, "hardness": brush.hardness, "opacity": brush.opacity, "flow": 1.0, "color": brush.color });
                    if params.get("erase").and_then(Value::as_bool) == Some(true) {
                        p["erase"] = json!(true);
                    }
                    strokes += 1;
                    push(&mut steps, e.at_ms, Call::engine("paint.stroke", p), None, true);
                }
                "tools.setBrush" => {
                    let patch = params.get("brush").unwrap_or(params);
                    let num = |k: &str| patch.get(k).and_then(Value::as_f64);
                    if let Some(v) = num("size") {
                        brush.size = v;
                    }
                    if let Some(v) = num("hardness") {
                        brush.hardness = v.clamp(0.0, 1.0);
                    }
                    if let Some(v) = num("opacity") {
                        brush.opacity = v.clamp(0.0, 1.0);
                    }
                    push(&mut steps, e.at_ms, Call::engine(id, params.clone()), None, false);
                }
                "tools.setColors" => {
                    if let Some(hex) = params.get("foreground").and_then(Value::as_str).filter(|h| crate::model::parse_hex(h).is_some()) {
                        brush.color = hex.to_string();
                    }
                    push(&mut steps, e.at_ms, Call::engine(id, params.clone()), None, false);
                }
                // Recording starts on the lesson's own picture; a new document would leave it.
                "file.new" => note(&mut skipped, id),
                "edit.undo" | "edit.redo" if options.skip_undo => {}
                _ if ALLOWED_COMMANDS.contains(&id.as_str()) => push(&mut steps, e.at_ms, Call::engine(id, params.clone()), None, false),
                _ => note(&mut skipped, id),
            },
        }
    }
    let longest = u64::from(options.longest_pause_ms.clamp(MIN_PAUSE_MS as u32, MAX_PAUSE_MS));
    let times: Vec<u64> = steps.iter().map(|(t, ..)| *t).collect();
    let mut out: Vec<Step> = steps.into_iter().map(|(_, s, _)| s).collect();
    for (i, step) in out.iter_mut().enumerate() {
        let gap = times.get(i + 1).map_or(600, |next| next.saturating_sub(times[i]));
        step.pause = Some(u32::try_from(gap.clamp(MIN_PAUSE_MS, longest)).unwrap_or(MAX_PAUSE_MS));
    }
    out.truncate(MAX_STEPS);
    Recording { steps: out, strokes, skipped }
}

fn note(skipped: &mut Vec<String>, id: &str) {
    if !skipped.iter().any(|s| s == id) {
        skipped.push(id.to_string());
    }
}

/// The stroke's points as `[x, y, pressure]`, thinned to the guide limit.
fn stroke_points(params: &Value) -> Option<Vec<Value>> {
    let raw = params.get("points")?.as_array()?;
    let points: Vec<[f64; 3]> = raw
        .iter()
        .filter_map(|p| {
            let a = p.as_array()?;
            let (x, y) = (a.first()?.as_f64()?, a.get(1)?.as_f64()?);
            let pressure = a.get(2).and_then(Value::as_f64).unwrap_or(1.0);
            Some([round(x), round(y), (pressure * 100.0).round() / 100.0])
        })
        .collect();
    if points.is_empty() {
        return None;
    }
    let step = points.len().div_ceil(MAX_POINTS);
    let mut kept: Vec<[f64; 3]> = points.iter().step_by(step).copied().collect();
    if let (Some(last), Some(end)) = (kept.last(), points.last())
        && last != end
    {
        kept.push(*end);
    }
    kept.truncate(MAX_POINTS);
    Some(kept.into_iter().map(|p| json!(p)).collect())
}

fn round(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guide::check_call;

    fn cmd(at_ms: u64, id: &str, params: Value) -> Recorded {
        Recorded { at_ms, event: Event::Command { id: id.into(), params } }
    }

    fn tool(at_ms: u64, name: &str) -> Recorded {
        Recorded { at_ms, event: Event::Tool(name.into()) }
    }

    #[test]
    fn a_session_becomes_tool_action_and_stroke_steps() {
        let events = vec![
            tool(0, "Brush"),
            cmd(300, "tools.setBrush", json!({"brush": {"size": 60, "hardness": 0}})),
            cmd(500, "tools.setBrush", json!({"brush": {"opacity": 0.3}})),
            cmd(1800, "paint.stroke", json!({"points": [[10.0, 20.0, 0.5], [30.04, 40.0, 0.75]], "zoom": 1.0, "target": "layer"})),
            cmd(9000, "paint.stroke", json!({"points": [[1, 2], [3, 4]], "erase": true})),
            cmd(9400, "layer.new.layer", json!({"name": "Тень"})),
            cmd(9500, "file.open", json!({"path": "C:/x.png"})),
            cmd(9600, "app.quit", json!({})),
        ];
        let r = to_guide(&events, &Brush { color: "#3B3A40".into(), ..Brush::default() }, Options::default());
        assert_eq!(r.strokes, 2);
        assert_eq!(r.skipped, vec!["file.open".to_string(), "app.quit".to_string()]);
        assert_eq!(r.steps.len(), 4, "tool with its brush settings, two strokes, the new layer");
        assert_eq!(r.steps[0].highlight.as_deref(), Some("tool:Brush"));
        assert_eq!(r.steps[0].calls.len(), 3, "brush settings join the tool step");
        let stroke = &r.steps[1].calls[0].params["params"];
        assert_eq!(stroke["points"], json!([[10.0, 20.0, 0.5], [30.0, 40.0, 0.75]]));
        assert_eq!((stroke["size"].as_f64(), stroke["hardness"].as_f64(), stroke["opacity"].as_f64()), (Some(60.0), Some(0.0), Some(0.3)));
        assert_eq!(stroke["color"], "#3B3A40");
        assert_eq!(r.steps[2].calls[0].params["params"]["erase"], true);
        assert_eq!(r.steps[2].calls[0].params["params"]["points"], json!([[1.0, 2.0, 1.0], [3.0, 4.0, 1.0]]), "missing pressure is full pressure");
        assert_eq!(r.steps.iter().map(|s| s.pause).collect::<Vec<_>>(), vec![Some(1800), Some(2000), Some(400), Some(600)], "the 7 s pause shrinks to 2 s");
        for s in &r.steps {
            for c in &s.calls {
                assert_eq!(check_call(c), Ok(()), "{c:?}");
            }
        }
    }

    #[test]
    fn colours_follow_the_foreground_and_unknown_tools_are_ignored() {
        let events = vec![
            tool(0, "NoSuchTool"),
            cmd(10, "tools.setColors", json!({"foreground": "#BC3823"})),
            cmd(20, "paint.stroke", json!({"points": [[0, 0]]})),
            cmd(30, "paint.stroke", json!({"points": []})),
        ];
        let r = to_guide(&events, &Brush::default(), Options::default());
        assert_eq!(r.steps.len(), 2);
        assert_eq!(r.steps[1].calls[0].params["params"]["color"], "#BC3823");
        assert_eq!(r.strokes, 1, "a stroke without points is dropped");
    }

    #[test]
    fn long_strokes_are_thinned_to_the_guide_limit_keeping_the_end() {
        let points: Vec<Value> = (0..12_001).map(|i| json!([f64::from(i), 0.0, 1.0])).collect();
        let r = to_guide(&[cmd(0, "paint.stroke", json!({ "points": points }))], &Brush::default(), Options::default());
        let kept = r.steps[0].calls[0].params["params"]["points"].as_array().unwrap().clone();
        assert!(kept.len() <= MAX_POINTS);
        assert_eq!(kept.last().unwrap()[0], 12_000.0);
        assert_eq!(check_call(&r.steps[0].calls[0]), Ok(()));
    }

    #[test]
    fn undo_is_left_out_by_default_and_long_pauses_can_stay() {
        let events = vec![
            cmd(0, "layer.new.layer", json!({})),
            cmd(100, "edit.undo", json!({})),
            cmd(200, "edit.redo", json!({})),
            cmd(9000, "paint.stroke", json!({"points": [[0, 0], [1, 1]]})),
        ];
        let r = to_guide(&events, &Brush::default(), Options::default());
        assert_eq!(r.steps[0].calls.len(), 1, "undo and redo are not in the guide");
        assert!(r.skipped.is_empty(), "leaving them out is a choice, not a refusal");
        let kept = to_guide(&events, &Brush::default(), Options { longest_pause_ms: MAX_PAUSE_MS, skip_undo: false });
        assert_eq!(kept.steps[0].calls.len(), 3);
        assert_eq!(kept.steps[0].pause, Some(9000), "the pause before the stroke stays as recorded");
    }
}
