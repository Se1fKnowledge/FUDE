//! Calls on PhotoCraft's JSON control channel (`photocraft/docs/control-protocol.md`).
//!
//! The core never talks to the editor itself; it produces [`Call`]s that the app sends.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::model::{Canvas, Task};

/// One control request: `{"method": …, "params": …}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Call {
    pub method: String,
    #[serde(default = "empty_object")]
    pub params: Value,
}

fn empty_object() -> Value {
    json!({})
}

impl Call {
    pub fn new(method: &str, params: Value) -> Self {
        Call { method: method.into(), params }
    }

    /// `engine.execute {command, params}`.
    pub fn engine(command: &str, params: Value) -> Self {
        Call::new("engine.execute", json!({ "command": command, "params": params }))
    }

    /// The engine command of an `engine.execute` call.
    pub fn command(&self) -> Option<&str> {
        (self.method == "engine.execute").then(|| self.params.get("command").and_then(Value::as_str)).flatten()
    }

    /// The JSON-line request with an id, as the control socket expects it.
    pub fn request(&self, id: u64) -> Value {
        json!({ "id": id, "method": self.method, "params": self.params })
    }
}

/// `file.new` for a blank canvas.
pub fn new_document(canvas: &Canvas) -> Call {
    Call::engine("file.new", json!({ "width": canvas.width, "height": canvas.height, "mode": "rgb", "depth": 8, "background": "white" }))
}

/// Calls that set the editor up for a task: open the task picture (relative to the lesson
/// folder, which the app passes to PhotoCraft as its automation read root) or create a blank
/// canvas filled with its background colour.
pub fn task_setup(task: &Task) -> Vec<Call> {
    if let Some(open) = &task.open {
        return vec![Call::new("app.open", json!({ "path": open }))];
    }
    let canvas = task.canvas.clone().unwrap_or_default();
    let mut calls = vec![new_document(&canvas)];
    if !canvas.background.eq_ignore_ascii_case("#FFFFFF") {
        calls.push(Call::engine("edit.fill", json!({ "contents": "color", "color": canvas.background })));
    }
    calls
}

/// `app.save` of the canvas to a PNG under the write root, for the canvas checks.
pub fn snapshot(path: &str) -> Call {
    Call::new("app.save", json!({ "path": path }))
}

/// `ui.inspect`, whose reply [`crate::assess::EditorState::from_inspect`] reads.
pub fn inspect() -> Call {
    Call::new("ui.inspect", json!({}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(open: Option<&str>, canvas: Option<Canvas>) -> Task {
        Task { brief: String::new(), open: open.map(Into::into), reference: None, canvas, steps: vec![] }
    }

    #[test]
    fn engine_calls_wrap_the_command() {
        let call = Call::engine("layer.new.layer", json!({"name": "Тень"}));
        assert_eq!(call.method, "engine.execute");
        assert_eq!(call.command(), Some("layer.new.layer"));
        assert_eq!(call.params["params"]["name"], "Тень");
        assert_eq!(Call::new("ui.set", json!({})).command(), None);
        assert_eq!(call.request(7), json!({"id": 7, "method": "engine.execute", "params": call.params}));
    }

    #[test]
    fn a_task_picture_is_opened_through_the_read_root() {
        let calls = task_setup(&task(Some("assets/task.png"), None));
        assert_eq!(calls, vec![Call::new("app.open", json!({"path": "assets/task.png"}))]);
    }

    #[test]
    fn a_blank_canvas_is_created_and_tinted() {
        let white = task_setup(&task(None, Some(Canvas { width: 800, height: 600, background: "#ffffff".into() })));
        assert_eq!(white.len(), 1);
        assert_eq!(white[0].params["params"]["width"], 800);
        let paper = task_setup(&task(None, Some(Canvas { width: 800, height: 600, background: "#F3EEE4".into() })));
        assert_eq!(paper.len(), 2);
        assert_eq!(paper[1].command(), Some("edit.fill"));
        assert_eq!(paper[1].params["params"]["color"], "#F3EEE4");
        assert_eq!(task_setup(&task(None, None))[0].params["params"]["height"], 800);
    }

    #[test]
    fn calls_read_without_params() {
        let call: Call = serde_json::from_str(r#"{"method":"ui.inspect"}"#).unwrap();
        assert_eq!(call, inspect());
        assert_eq!(snapshot("attempt/canvas.png").params["path"], "attempt/canvas.png");
    }
}
