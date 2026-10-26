//! A practice attempt: the task set up in the editor, steps checked in order, hints drawn and
//! removed, and the result written to the progress database.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use chrono::NaiveDateTime;
use fude_core::assess::{self, Context, EditorState, Progress, Snapshot, TaskOutcome};
use fude_core::control::{self, Call};
use fude_core::hint;
use fude_core::model::Lesson;
use serde::Serialize;
use serde_json::Value;

use crate::db::Db;
use crate::editor::{Client, Editor};
use crate::error::{AppError, AppResult};
use crate::progress;

/// Something that answers control calls: the real editor, or a test double.
pub trait Channel {
    fn call(&mut self, call: &Call) -> AppResult<Value>;
}

impl Channel for Editor {
    fn call(&mut self, call: &Call) -> AppResult<Value> {
        Editor::call(self, call)
    }
}

impl Channel for Client {
    fn call(&mut self, call: &Call) -> AppResult<Value> {
        Client::call(self, call)
    }
}

pub struct Practice {
    pub attempt: i64,
    pub lesson: Lesson,
    pub dir: PathBuf,
    /// Where the editor saves snapshots (its automation write root).
    pub write_root: PathBuf,
    context: Context,
    pub progress: Progress,
    hinted: HashSet<String>,
    confirmed: HashSet<String>,
    pub hints: u32,
    started: Instant,
    hint_shown: bool,
    finished: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReport {
    pub attempt: i64,
    pub outcome: TaskOutcome,
    /// Steps done so far (they stay done).
    pub done: usize,
    pub total: usize,
    pub finished: bool,
    /// Steps completed by this check.
    pub advanced: usize,
    pub xp: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HintReport {
    pub step: String,
    pub text: String,
    pub target: Option<String>,
    pub shapes: usize,
    pub hints: u32,
}

impl Practice {
    pub fn new(attempt: i64, lesson: Lesson, dir: &Path, write_root: &Path) -> AppResult<Practice> {
        let context = Context::for_lesson(&lesson, dir)?;
        Ok(Practice {
            attempt,
            lesson,
            dir: dir.to_path_buf(),
            write_root: write_root.to_path_buf(),
            context,
            progress: Progress::default(),
            hinted: HashSet::new(),
            confirmed: HashSet::new(),
            hints: 0,
            started: Instant::now(),
            hint_shown: false,
            finished: false,
        })
    }

    pub fn snapshot_name(&self) -> String {
        format!("attempt-{}.png", self.attempt)
    }

    pub fn snapshot_path(&self) -> PathBuf {
        self.write_root.join(self.snapshot_name())
    }

    /// Opens the task picture or creates the blank canvas.
    pub fn setup(&self, ch: &mut dyn Channel) -> AppResult<()> {
        for call in control::task_setup(&self.lesson.task) {
            ch.call(&call)?;
        }
        Ok(())
    }

    fn total(&self) -> usize {
        self.lesson.task.steps.len()
    }

    /// Whether any open step reads the canvas, so a snapshot is worth saving.
    fn needs_canvas(&self) -> bool {
        self.lesson.task.steps.iter().skip(self.progress.done).any(|s| s.checks.iter().any(|c| c.kind.needs_canvas()))
    }

    fn snapshot(&self, ch: &mut dyn Channel, with_canvas: bool) -> AppResult<Snapshot> {
        let reply = ch.call(&control::inspect())?;
        let canvas = if with_canvas {
            ch.call(&control::snapshot(&self.snapshot_name()))?;
            Some(assess::load_image(&self.snapshot_path())?)
        } else {
            None
        };
        Ok(Snapshot { state: EditorState::from_inspect(&reply), canvas })
    }

    /// Checks the task now, moves past the steps that pass and finishes the attempt once
    /// every step is done.
    pub fn check(&mut self, ch: &mut dyn Channel, db: &Db, now: NaiveDateTime) -> AppResult<CheckReport> {
        if self.finished {
            return Err(AppError::msg("задание уже выполнено"));
        }
        let snapshot = self.snapshot(ch, self.needs_canvas())?;
        let outcome = assess::task(&self.lesson.task, &self.context, &snapshot, &self.confirmed);
        let before = self.progress.done;
        let advanced = self.progress.update(&outcome);
        for step in &self.lesson.task.steps[before..self.progress.done] {
            db.record_step(self.attempt, &step.id, self.hinted.contains(&step.id), now)?;
        }
        if advanced > 0 && self.hint_shown {
            self.hide_hint(ch)?;
        }
        let mut xp = None;
        if self.progress.finished(self.total()) {
            xp = Some(self.finish(ch, db, now, snapshot.canvas.is_some())?);
        }
        Ok(CheckReport { attempt: self.attempt, outcome, done: self.progress.done, total: self.total(), finished: self.finished, advanced, xp })
    }

    fn finish(&mut self, ch: &mut dyn Channel, db: &Db, now: NaiveDateTime, have_canvas: bool) -> AppResult<u32> {
        // Keep a picture of the finished work for the result screen.
        if !have_canvas {
            ch.call(&control::snapshot(&self.snapshot_name()))?;
        }
        let similarity = match (&self.context.task_reference, assess::load_image(&self.snapshot_path())) {
            (Some(path), Ok(canvas)) => self.context.references.get(path).map(|reference| assess::similarity(&canvas, reference)),
            _ => None,
        };
        let first = !db.attempts()?.iter().any(|a| a.passed && a.lesson == self.lesson.id);
        let xp = progress::xp_for(self.lesson.difficulty, self.hints, first);
        let seconds = u32::try_from(self.started.elapsed().as_secs()).unwrap_or(u32::MAX);
        db.finish_attempt(self.attempt, true, seconds, xp, similarity, now)?;
        let detail = if self.hints == 0 { format!("+{xp} XP · без подсказок") } else { format!("+{xp} XP · подсказок: {}", self.hints) };
        db.log(now, "pass", &format!("Пройден урок «{}»", self.lesson.title), &detail)?;
        db.clear_stage(&self.lesson.id)?;
        self.finished = true;
        Ok(xp)
    }

    /// Shows the hint of the current step: counts it, and draws its ghost shapes on a hint
    /// layer, returning the student to the layer they were on.
    pub fn hint(&mut self, ch: &mut dyn Channel, db: &Db) -> AppResult<HintReport> {
        let Some(step) = self.lesson.task.steps.get(self.progress.done).cloned() else {
            return Err(AppError::msg("все шаги уже выполнены"));
        };
        let Some(h) = step.hint.clone() else {
            return Err(AppError::msg("у этого шага нет подсказки"));
        };
        if self.hinted.insert(step.id.clone()) {
            self.hints += 1;
            db.add_hint(self.attempt)?;
        }
        if !h.shapes.is_empty() && !self.hint_shown {
            let state = ch.call(&control::inspect())?;
            let doc = state.get("document").filter(|d| d.is_object());
            let width = doc.and_then(|d| d.get("width")).and_then(Value::as_u64).unwrap_or(1200) as u32;
            let height = doc.and_then(|d| d.get("height")).and_then(Value::as_u64).unwrap_or(800) as u32;
            let active = doc.and_then(|d| d.get("activeLayer")).and_then(Value::as_u64);
            for call in hint::show(&h, width, height, active) {
                ch.call(&call)?;
            }
            self.hint_shown = true;
        }
        Ok(HintReport { step: step.id, text: h.text, target: h.target, shapes: h.shapes.len(), hints: self.hints })
    }

    /// Removes every hint layer.
    pub fn hide_hint(&mut self, ch: &mut dyn Channel) -> AppResult<()> {
        let state = ch.call(&control::inspect())?;
        let layers = state.pointer("/document/layers").cloned().unwrap_or(Value::Null);
        for call in hint::hide(&layers) {
            ch.call(&call)?;
        }
        self.hint_shown = false;
        Ok(())
    }

    /// Marks a step without checks as done by the student.
    pub fn confirm(&mut self, step: &str) {
        self.confirmed.insert(step.to_string());
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::tests::fake_editor;
    use chrono::NaiveDate;
    use fude_core::library::{Library, Root};
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    fn now() -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 10, 26).and_then(|d| d.and_hms_opt(10, 0, 0)).unwrap()
    }

    fn builtin(id: &str) -> (Lesson, PathBuf) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("lessons");
        let library = Library::scan(&[Root::builtin(root)]);
        let e = library.lesson(id).unwrap();
        (e.lesson.clone(), e.dir.clone())
    }

    /// Editor state the fake answers `ui.inspect` with; tests change it between checks.
    fn state(tool: &str, layers: Value) -> Value {
        json!({"tool": tool, "brush": {"size": 60, "hardness": 0.0, "opacity": 0.4}, "views": [{"zoom": 1.0}], "session": {"active": 0},
               "document": {"width": 1200, "height": 800, "activeLayer": 2, "layers": layers, "hasSelection": false, "history": [], "canRedo": false}})
    }

    #[test]
    fn a_practice_runs_through_check_hint_and_finish_against_a_fake_editor() {
        let (lesson, dir) = builtin("light.cast-shadow");
        let write = tempfile::tempdir().unwrap();
        let write_root = write.path().to_path_buf();
        let current = Arc::new(Mutex::new(state("Brush", json!([{"id": 1, "name": "Фон", "kind": "Pixel", "visible": true, "opacity": 1.0}]))));
        let canvas_source = Arc::new(Mutex::new(dir.join("assets").join("task.png")));
        let (cur, src, root) = (current.clone(), canvas_source.clone(), write_root.clone());
        let (addr, log, _h) = fake_editor("t", move |method, params| match method {
            "ui.inspect" => Ok(cur.lock().unwrap().clone()),
            "app.save" => {
                // The "editor" saves whatever picture the test says the canvas looks like.
                let name = params["path"].as_str().unwrap();
                std::fs::copy(&*src.lock().unwrap(), root.join(name)).unwrap();
                Ok(json!({"path": name}))
            }
            _ => Ok(json!({})),
        });
        let mut ch = Client::connect(addr, "t").unwrap();
        let db = Db::in_memory(now()).unwrap();
        let attempt = db.start_attempt(&lesson.id, now()).unwrap();
        let mut p = Practice::new(attempt, lesson, &dir, &write_root).unwrap();
        p.setup(&mut ch).unwrap();

        // Step 1 needs a layer named «Тень»: not yet.
        let r = p.check(&mut ch, &db, now()).unwrap();
        assert_eq!((r.done, r.total, r.finished), (0, 3, false));
        assert!(!r.outcome.steps[0].passed);

        // A hint on step 1 has no shapes: counted, nothing drawn.
        let h = p.hint(&mut ch, &db).unwrap();
        assert_eq!((h.step.as_str(), h.shapes, h.hints), ("layer", 0, 1));
        assert_eq!(p.hint(&mut ch, &db).unwrap().hints, 1, "the same step's hint counts once");

        *current.lock().unwrap() = state(
            "Brush",
            json!([{"id": 2, "name": "Тень", "kind": "Pixel", "visible": true, "opacity": 1.0}, {"id": 1, "name": "Фон", "kind": "Pixel", "visible": true, "opacity": 1.0}]),
        );
        let r = p.check(&mut ch, &db, now()).unwrap();
        assert_eq!((r.done, r.advanced), (1, 1));

        // Step 2 has a ghost ellipse: drawn on a hint layer, then the student's layer again.
        let before = log.lock().unwrap().len();
        let h = p.hint(&mut ch, &db).unwrap();
        assert_eq!((h.shapes, h.hints), (1, 2));
        let calls: Vec<String> = log.lock().unwrap()[before..].iter().map(|(m, p)| format!("{m} {}", p["command"].as_str().unwrap_or(""))).collect();
        assert_eq!(calls, ["ui.inspect ", "engine.execute layer.new.layer", "engine.execute paint.stroke", "engine.execute layer.select"]);

        // The canvas becomes the model answer: the remaining steps pass, the hint layer goes,
        // and the attempt is finished with experience for a hinted first pass.
        *current.lock().unwrap() = state(
            "Brush",
            json!([{"id": 9, "name": hint::LAYER_NAME, "kind": "Pixel", "visible": true, "opacity": 0.35}, {"id": 2, "name": "Тень", "kind": "Pixel", "visible": true, "opacity": 1.0}]),
        );
        *canvas_source.lock().unwrap() = dir.join("assets").join("reference.png");
        let r = p.check(&mut ch, &db, now()).unwrap();
        assert!(r.finished, "{:?}", r.outcome);
        assert_eq!(r.xp, Some(progress::xp_for(3, 2, true)));
        assert!(log.lock().unwrap().iter().any(|(m, p)| m == "engine.execute" && p["command"] == "layer.delete" && p["params"]["layer"] == 9));
        let a = db.attempt(attempt).unwrap().unwrap();
        assert!(a.passed);
        assert_eq!(a.hints, 2);
        assert!(a.similarity.unwrap() > 0.99, "the canvas is the reference");
        assert_eq!(db.steps(attempt).unwrap(), vec![("layer".into(), true), ("cast".into(), true), ("contact".into(), false)]);
        assert!(db.journal(5).unwrap()[0].title.contains("Падающая тень"));
        assert!(p.check(&mut ch, &db, now()).is_err(), "a finished attempt is not checked again");
    }

    #[test]
    fn steps_without_canvas_checks_skip_the_snapshot_and_manual_steps_need_confirmation() {
        let (mut lesson, dir) = builtin("basics.canvas");
        lesson.task.steps[0].checks.clear();
        let write = tempfile::tempdir().unwrap();
        let (addr, log, _h) = fake_editor("t", |method, _| match method {
            "ui.inspect" => {
                Ok(json!({"tool": "Zoom", "views": [{"zoom": 3.0}], "session": {"active": 0}, "document": {"width": 10, "height": 10, "layers": []}}))
            }
            _ => Ok(json!({})),
        });
        let mut ch = Client::connect(addr, "t").unwrap();
        let db = Db::in_memory(now()).unwrap();
        let attempt = db.start_attempt(&lesson.id, now()).unwrap();
        let mut p = Practice::new(attempt, lesson, &dir, write.path()).unwrap();
        assert_eq!(p.check(&mut ch, &db, now()).unwrap().done, 0, "a manual step waits for «Готово»");
        p.confirm("zoom-tool");
        assert_eq!(p.check(&mut ch, &db, now()).unwrap().done, 2, "confirmed, and 300% passes the zoom step");
        assert!(!log.lock().unwrap().iter().any(|(m, _)| m == "app.save"), "no canvas checks, no snapshots");
    }
}
