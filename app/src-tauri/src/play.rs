//! Commands that drive the editor: playing a guide, and a practice attempt with checks and
//! hints.

use std::sync::atomic::Ordering;
use std::time::Duration;

use fude_core::guide;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::error::{AppError, AppResult};
use crate::learn::{now, refresh_achievements};
use crate::practice::{CheckReport, HintReport, Practice};
use crate::state::{AppState, lock};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorStatus {
    pub found: bool,
    pub path: Option<String>,
    pub running: bool,
}

#[tauri::command]
pub fn editor_status(state: State<'_, AppState>) -> EditorStatus {
    let running = lock(&state.editor).as_mut().is_some_and(|e| e.alive());
    EditorStatus { found: state.executable.is_some(), path: state.executable.as_ref().map(|p| p.display().to_string()), running }
}

#[tauri::command]
pub fn editor_close(state: State<'_, AppState>) {
    state.guide_cancel.store(true, Ordering::SeqCst);
    *lock(&state.practice) = None;
    state.close_editor();
}

// ------------------------------------------------------------------ guide

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuideEvent {
    pub lesson: String,
    /// `starting`, `step`, `done`, `stopped` or `error`.
    pub state: &'static str,
    pub index: usize,
    pub total: usize,
    pub caption: String,
    pub highlight: Option<String>,
    pub message: Option<String>,
}

/// Plays a lesson's guide in the editor on a background thread, reporting each step as a
/// `guide` event.
#[tauri::command]
pub fn guide_play(app: AppHandle, state: State<'_, AppState>, id: String) -> AppResult<()> {
    let library = state.library();
    let entry = library.lesson(&id).ok_or_else(|| AppError::msg("урок не найден"))?;
    let rel = entry.lesson.guide.script.clone().ok_or_else(|| AppError::msg("у урока нет анимированного гайда"))?;
    let dir = entry.dir.clone();
    let script = guide::load(&dir.join(&rel))?;
    if guide::validate(&script, Some(&dir)).iter().any(|i| i.is_error()) {
        return Err(AppError::msg("сценарий гайда не прошёл проверку"));
    }
    // A guide and a practice share the editor; the guide ends the practice.
    *lock(&state.practice) = None;
    state.guide_cancel.store(false, Ordering::SeqCst);
    let cancel = state.guide_cancel.clone();
    let total = script.steps.len();
    let event = move |app: &AppHandle, state: &'static str, index: usize, caption: String, highlight: Option<String>, message: Option<String>| {
        let _ = app.emit("guide", GuideEvent { lesson: id.clone(), state, index, total, caption, highlight, message });
    };
    std::thread::spawn(move || {
        let st = app.state::<AppState>();
        event(&app, "starting", 0, "Открываем редактор…".into(), None, None);
        let run = || -> AppResult<bool> {
            {
                let mut guard = st.editor_for(&dir)?;
                let editor = guard.as_mut().ok_or_else(|| AppError::msg("редактор не запущен"))?;
                editor.calls(&script.setup())?;
            }
            for (i, step) in script.steps.iter().enumerate() {
                if cancel.load(Ordering::SeqCst) {
                    return Ok(false);
                }
                event(&app, "step", i, step.caption.clone(), step.highlight.clone(), None);
                {
                    let mut guard = lock(&st.editor);
                    let editor = guard.as_mut().ok_or_else(|| AppError::msg("редактор закрыт"))?;
                    editor.calls(&step.calls)?;
                }
                // Sleep in small slices so «Стоп» answers at once.
                let mut left = step.pause_ms();
                while left > 0 && !cancel.load(Ordering::SeqCst) {
                    let slice = left.min(100);
                    std::thread::sleep(Duration::from_millis(u64::from(slice)));
                    left -= slice;
                }
            }
            Ok(true)
        };
        match run() {
            Ok(true) => event(&app, "done", total, "Гайд закончился. Попробуйте сами!".into(), None, None),
            Ok(false) => event(&app, "stopped", 0, String::new(), None, None),
            Err(e) => event(&app, "error", 0, String::new(), None, Some(e.to_string())),
        }
    });
    Ok(())
}

#[tauri::command]
pub fn guide_stop(state: State<'_, AppState>) {
    state.guide_cancel.store(true, Ordering::SeqCst);
}

// ------------------------------------------------------------------ practice

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeStart {
    pub attempt: i64,
    pub steps: Vec<StepView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepView {
    pub id: String,
    pub text: String,
    pub has_hint: bool,
    pub manual: bool,
}

/// Opens the task in the editor and starts an attempt.
#[tauri::command]
pub async fn practice_start(app: AppHandle, id: String) -> AppResult<PracticeStart> {
    // Starting the editor can take seconds: keep it off the main thread.
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        state.guide_cancel.store(true, Ordering::SeqCst);
        let library = state.library();
        let entry = library.lesson(&id).ok_or_else(|| AppError::msg("урок не найден"))?;
        // Locks are taken practice → editor → db everywhere; here the old practice goes first.
        *lock(&state.practice) = None;
        let practice = {
            let mut guard = state.editor_for(&entry.dir)?;
            let editor = guard.as_mut().ok_or_else(|| AppError::msg("редактор не запущен"))?;
            let db = lock(&state.db);
            let attempt = db.start_attempt(&id, now())?;
            db.set_stage(&id, "practice", now())?;
            let practice = Practice::new(attempt, entry.lesson.clone(), &entry.dir, &state.attempts_dir())?;
            practice.setup(editor)?;
            practice
        };
        let attempt = practice.attempt;
        let steps = entry
            .lesson
            .task
            .steps
            .iter()
            .map(|s| StepView { id: s.id.clone(), text: s.text.clone(), has_hint: s.hint.is_some(), manual: s.checks.is_empty() })
            .collect();
        *lock(&state.practice) = Some(practice);
        Ok(PracticeStart { attempt, steps })
    })
    .await
    .map_err(|e| AppError::msg(e.to_string()))?
}

fn with_practice<T>(state: &AppState, f: impl FnOnce(&mut Practice, &mut crate::editor::Editor) -> AppResult<T>) -> AppResult<T> {
    let mut practice = lock(&state.practice);
    let p = practice.as_mut().ok_or_else(|| AppError::msg("задание не начато"))?;
    let mut editor = lock(&state.editor);
    let e = editor.as_mut().filter(|e| e.read_root == p.dir).ok_or_else(|| AppError::msg("редактор закрыт — откройте задание заново"))?;
    if !e.alive() {
        return Err(AppError::msg("редактор закрыт — откройте задание заново"));
    }
    f(p, e)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckResult {
    #[serde(flatten)]
    pub report: CheckReport,
    pub hints: u32,
}

#[tauri::command]
pub async fn practice_check(app: AppHandle) -> AppResult<CheckResult> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let result = with_practice(&state, |p, e| {
            let db = lock(&state.db);
            let report = p.check(e, &db, now())?;
            Ok(CheckResult { report, hints: p.hints })
        })?;
        if result.report.finished {
            refresh_achievements(&state)?;
            *lock(&state.practice) = None;
        }
        Ok(result)
    })
    .await
    .map_err(|e| AppError::msg(e.to_string()))?
}

#[tauri::command]
pub async fn practice_hint(app: AppHandle) -> AppResult<HintReport> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        with_practice(&state, |p, e| {
            let db = lock(&state.db);
            p.hint(e, &db)
        })
    })
    .await
    .map_err(|e| AppError::msg(e.to_string()))?
}

#[tauri::command]
pub async fn practice_hide_hint(app: AppHandle) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        with_practice(&state, |p, e| p.hide_hint(e))
    })
    .await
    .map_err(|e| AppError::msg(e.to_string()))?
}

#[tauri::command]
pub fn practice_confirm(state: State<'_, AppState>, step: String) -> AppResult<()> {
    lock(&state.practice).as_mut().ok_or_else(|| AppError::msg("задание не начато"))?.confirm(&step);
    Ok(())
}

/// Leaves the task: the attempt stays unfinished, the editor closes.
#[tauri::command]
pub fn practice_stop(state: State<'_, AppState>) {
    *lock(&state.practice) = None;
    state.close_editor();
}
