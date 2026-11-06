//! The lesson studio's live tools: proving checks on pictures, showing a step's hint in the
//! editor, recording a guide while the author draws in PhotoCraft, and playing a draft's guide.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use fude_core::assess::{self, CheckProof};
use fude_core::control::{self, Call};
use fude_core::model::Lesson;
use fude_core::record::{self, Brush, Event, Recorded, Recording};
use serde::Serialize;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::error::{AppError, AppResult};
use crate::play::{self, PracticeStart};
use crate::practice::{Channel, HintReport};
use crate::state::{AppState, lock, user_dir};

/// How the checks of a draft behave on its reference picture and its untouched task.
#[tauri::command]
pub fn studio_prove(state: State<'_, AppState>, dir: String, lesson: Lesson) -> AppResult<Vec<CheckProof>> {
    let dir = user_dir(&state, &dir)?;
    Ok(assess::prove(&lesson, &dir)?)
}

/// Opens the draft's task in the editor and draws the hint of step `step`, as the student
/// would see it after reaching that step. Nothing is recorded.
#[tauri::command]
pub async fn studio_hint_preview(app: AppHandle, dir: String, lesson: Lesson, step: usize) -> AppResult<HintReport> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let dir = user_dir(&state, &dir)?;
        if step >= lesson.task.steps.len() {
            return Err(AppError::msg(fude_core::i18n::pick("нет такого шага", "there is no such step")));
        }
        let _: PracticeStart = play::start(&state, lesson, &dir, true)?;
        let mut practice = lock(&state.practice);
        let p = practice.as_mut().ok_or_else(|| AppError::msg(fude_core::i18n::pick("задание не открылось", "the task did not open")))?;
        p.progress.done = step;
        let mut editor = lock(&state.editor);
        let e = editor.as_mut().ok_or_else(|| AppError::msg(fude_core::i18n::pick("редактор закрыт", "the editor is closed")))?;
        let db = lock(&state.scratch_db);
        p.hint(e, &db)
    })
    .await
    .map_err(|e| AppError::msg(e.to_string()))?
}

/// Plays the guide of the lesson saved in `dir`, draft or not.
#[tauri::command]
pub fn guide_play_draft(app: AppHandle, state: State<'_, AppState>, dir: String) -> AppResult<()> {
    let dir = user_dir(&state, &dir)?;
    let lesson = fude_core::model::load_lesson(&dir)?;
    play::play_guide(app.clone(), &state, &lesson, dir)
}

// ---------------------------------------------------------------- recording

/// A recording in progress: what the author did so far.
pub struct Recorder {
    stop: Arc<AtomicBool>,
    events: Arc<std::sync::Mutex<Vec<Recorded>>>,
    brush: Brush,
    worker: Option<std::thread::JoinHandle<()>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordTick {
    pub elapsed_ms: u64,
    pub strokes: usize,
    pub actions: usize,
    pub tool: Option<String>,
}

/// How often the recorder reads the editor's journal.
const POLL: Duration = Duration::from_millis(300);
/// Pauses longer than this (the author thinking) are shortened in the guide.
const LONGEST_PAUSE_MS: u32 = 2000;

/// Opens the draft's task in the editor and starts writing down what the author does there.
/// Progress arrives as `guide-record` events.
#[tauri::command]
pub async fn guide_record_start(app: AppHandle, dir: String, lesson: Lesson) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let dir = user_dir(&state, &dir)?;
        stop_recorder(&state);
        state.guide_cancel.store(true, Ordering::SeqCst);
        *lock(&state.practice) = None;
        let (mark, brush, tool) = {
            let mut guard = state.editor_for(&dir)?;
            let editor = guard.as_mut().ok_or_else(|| AppError::msg(fude_core::i18n::pick("редактор не запущен", "the editor is not running")))?;
            for call in control::task_setup(&lesson.task) {
                editor.call(&call)?;
            }
            let mark = journal_end(editor)?;
            let inspect = editor.call(&control::inspect())?;
            (mark, brush_of(&inspect), tool_of(&inspect))
        };
        let stop = Arc::new(AtomicBool::new(false));
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let worker = {
            let (stop, events, app) = (stop.clone(), events.clone(), app.clone());
            std::thread::spawn(move || record_loop(&app, mark, tool, &stop, &events))
        };
        *lock(&state.recording) = Some(Recorder { stop, events, brush, worker: Some(worker) });
        Ok(())
    })
    .await
    .map_err(|e| AppError::msg(e.to_string()))?
}

/// Stops recording and returns the guide steps made of what the author did.
#[tauri::command]
pub async fn guide_record_stop(app: AppHandle, keep_pauses: bool, keep_undo: bool) -> AppResult<Recording> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let recorder = stop_recorder(&state).ok_or_else(|| AppError::msg(fude_core::i18n::pick("запись не идёт", "nothing is being recorded")))?;
        let events = lock(&recorder.events).clone();
        let options = record::Options { longest_pause_ms: if keep_pauses { fude_core::guide::MAX_PAUSE_MS } else { LONGEST_PAUSE_MS }, skip_undo: !keep_undo };
        Ok(record::to_guide(&events, &recorder.brush, options))
    })
    .await
    .map_err(|e| AppError::msg(e.to_string()))?
}

fn stop_recorder(state: &AppState) -> Option<Recorder> {
    let mut recorder = lock(&state.recording).take()?;
    recorder.stop.store(true, Ordering::SeqCst);
    if let Some(worker) = recorder.worker.take() {
        let _ = worker.join();
    }
    Some(recorder)
}

fn record_loop(app: &AppHandle, mut mark: u64, mut tool: Option<String>, stop: &AtomicBool, events: &std::sync::Mutex<Vec<Recorded>>) {
    let started = Instant::now();
    let state = app.state::<AppState>();
    let mut last_round = false;
    loop {
        // One more read after «Стоп», so the last stroke is not lost.
        if stop.load(Ordering::SeqCst) {
            last_round = true;
        }
        let at_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let read = {
            let mut guard = lock(&state.editor);
            let Some(editor) = guard.as_mut() else { break };
            if !editor.alive() {
                break;
            }
            read_round(editor, &mut mark, &mut tool)
        };
        let Ok(new) = read else { break };
        let mut all = lock(events);
        all.extend(new.into_iter().map(|event| Recorded { at_ms, event }));
        let strokes = all.iter().filter(|r| matches!(&r.event, Event::Command { id, .. } if id == "paint.stroke")).count();
        let tick = RecordTick { elapsed_ms: at_ms, strokes, actions: all.len() - strokes, tool: tool.clone() };
        drop(all);
        let _ = app.emit("guide-record", tick);
        if last_round {
            break;
        }
        std::thread::sleep(POLL);
    }
}

/// New events since `mark`: a changed tool first, then the journal entries.
pub fn read_round(ch: &mut dyn Channel, mark: &mut u64, tool: &mut Option<String>) -> AppResult<Vec<Event>> {
    let mut out = Vec::new();
    let inspect = ch.call(&control::inspect())?;
    let now = tool_of(&inspect);
    if now.is_some() && now != *tool {
        out.push(Event::Tool(now.clone().unwrap_or_default()));
        *tool = now;
    }
    loop {
        let reply = ch.call(&Call::new("journal.read", json!({ "from": *mark })))?;
        let entries = reply.get("entries").and_then(Value::as_array).cloned().unwrap_or_default();
        *mark = reply.get("next").and_then(Value::as_u64).unwrap_or(*mark);
        let full = entries.len() >= 500;
        for e in entries {
            let Some(id) = e.get("id").and_then(Value::as_str) else { continue };
            out.push(Event::Command { id: id.to_string(), params: e.get("params").cloned().unwrap_or(Value::Null) });
        }
        if !full {
            break;
        }
    }
    Ok(out)
}

fn journal_end(ch: &mut dyn Channel) -> AppResult<u64> {
    let reply = ch.call(&Call::new("journal.read", json!({ "from": u64::MAX })))?;
    reply.get("next").and_then(Value::as_u64).ok_or_else(|| {
        AppError::msg(fude_core::i18n::pick("редактор не умеет читать журнал — обновите PhotoCraft", "the editor cannot read its journal — update PhotoCraft"))
    })
}

fn tool_of(inspect: &Value) -> Option<String> {
    inspect.get("tool").and_then(Value::as_str).map(str::to_string)
}

/// The brush and foreground colour PhotoCraft reports.
pub fn brush_of(inspect: &Value) -> Brush {
    let b = inspect.get("brush");
    let num = |k: &str, d: f64| b.and_then(|b| b.get(k)).and_then(Value::as_f64).unwrap_or(d);
    let color = inspect
        .pointer("/session/foreground")
        .and_then(Value::as_array)
        .and_then(|c| {
            let ch = |i: usize| c.get(i).and_then(Value::as_f64).map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
            Some(format!("#{:02X}{:02X}{:02X}", ch(0)?, ch(1)?, ch(2)?))
        })
        .unwrap_or_else(|| "#000000".into());
    Brush { size: num("size", 30.0), hardness: num("hardness", 0.8), opacity: num("opacity", 1.0), color }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake {
        journal: Vec<Value>,
        tool: &'static str,
    }

    impl Channel for Fake {
        fn call(&mut self, call: &Call) -> AppResult<Value> {
            match call.method.as_str() {
                "ui.inspect" => {
                    Ok(json!({"tool": self.tool, "brush": {"size": 12.0, "hardness": 0.5, "opacity": 0.4}, "session": {"foreground": [0.2, 0.4, 0.6, 1.0]}}))
                }
                "journal.read" => {
                    let from = call.params["from"].as_u64().unwrap_or(0).min(self.journal.len() as u64) as usize;
                    let entries: Vec<Value> = self.journal[from..].to_vec();
                    Ok(json!({"next": self.journal.len(), "entries": entries}))
                }
                other => Err(AppError::msg(format!("unexpected {other}"))),
            }
        }
    }

    #[test]
    fn a_round_reads_the_tool_change_and_new_journal_entries_once() {
        let mut fake = Fake { journal: vec![json!({"id": "tools.setBrush", "params": {"brush": {"size": 5}}})], tool: "Brush" };
        let mut mark = journal_end(&mut fake).unwrap();
        assert_eq!(mark, 1, "recording starts after what is already in the journal");
        let mut tool = Some("Move".to_string());
        fake.journal.push(json!({"id": "paint.stroke", "params": {"points": [[1, 2]]}}));
        let events = read_round(&mut fake, &mut mark, &mut tool).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], Event::Tool("Brush".into()));
        assert!(matches!(&events[1], Event::Command { id, .. } if id == "paint.stroke"));
        assert!(read_round(&mut fake, &mut mark, &mut tool).unwrap().is_empty(), "nothing twice");
    }

    #[test]
    fn the_brush_and_colour_come_from_inspect() {
        let mut fake = Fake { journal: vec![], tool: "Brush" };
        let inspect = fake.call(&control::inspect()).unwrap();
        assert_eq!(brush_of(&inspect), Brush { size: 12.0, hardness: 0.5, opacity: 0.4, color: "#336699".into() });
        assert_eq!(brush_of(&json!({})), Brush { size: 30.0, hardness: 0.8, opacity: 1.0, color: "#000000".into() });
    }

    #[test]
    fn an_editor_without_the_journal_is_reported() {
        struct Old;
        impl Channel for Old {
            fn call(&mut self, _: &Call) -> AppResult<Value> {
                Ok(json!({}))
            }
        }
        assert!(journal_end(&mut Old).unwrap_err().to_string().contains("обновите PhotoCraft"));
    }
}
