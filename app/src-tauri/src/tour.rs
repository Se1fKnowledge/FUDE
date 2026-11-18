//! The editor tour: one chapter running in the editor. FUDE tells the editor which place to
//! light and what the card says (`tour.set`), reads what the student did (`tour.poll`), checks
//! the step's small task and moves on. Steps are remembered, so a chapter left halfway resumes
//! at its first open step. The tour gives no experience: it is an introduction, not a lesson.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use chrono::NaiveDateTime;
use fude_core::assess::{self, Context, EditorState, Snapshot};
use fude_core::control::{self, Call};
use fude_core::i18n::pick;
use fude_core::model::Task;
use fude_core::tour::{self, Chapter};
use serde::Serialize;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State};

use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::learn::now;
use crate::practice::Channel;
use crate::state::{AppState, lock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StepState {
    Pending,
    Done,
    Skipped,
}

impl StepState {
    pub fn code(self) -> &'static str {
        match self {
            StepState::Pending => "pending",
            StepState::Done => "done",
            StepState::Skipped => "skipped",
        }
    }

    pub fn from_code(code: &str) -> StepState {
        match code {
            "done" => StepState::Done,
            "skipped" => StepState::Skipped,
            _ => StepState::Pending,
        }
    }
}

const SNAPSHOT: &str = "tour.png";

pub struct TourRun {
    pub chapter: Chapter,
    pub index: usize,
    pub states: Vec<StepState>,
    /// The folder the editor opens files from during the tour (its read root).
    pub dir: PathBuf,
    write_root: PathBuf,
    context: Context,
    /// Why the current step is not done yet, in the card's status line.
    status: String,
    started: Instant,
    last_canvas: Option<Instant>,
    /// The step changed in this poll: what the editor reported belongs to the previous one.
    moved: bool,
    pub finished: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepView {
    pub id: String,
    pub title: String,
    pub learned: String,
    pub key: Option<String>,
    pub state: StepState,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TourView {
    pub chapter: String,
    pub title: String,
    pub index: usize,
    pub total: usize,
    pub steps: Vec<StepView>,
    /// The current step's task is done and the card waits for «Дальше».
    pub done: bool,
    pub finished: bool,
    /// Presses outside the lit place since the last poll.
    pub outside: u32,
    pub seconds: u64,
    /// The sheet as the student left it, once the chapter is finished.
    pub picture: Option<String>,
    pub next: Option<NextChapter>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextChapter {
    pub id: String,
    pub title: String,
}

fn steps_view(chapter: &Chapter, states: &[StepState]) -> Vec<StepView> {
    chapter
        .steps
        .iter()
        .zip(states)
        .map(|(s, st)| StepView { id: s.id.clone(), title: s.title.clone(), learned: s.learned.clone(), key: s.key.clone(), state: *st })
        .collect()
}

impl TourRun {
    /// A chapter at its first open step; a chapter with nothing open starts over.
    pub fn new(chapter: Chapter, saved: &HashMap<String, StepState>, dir: &Path, write_root: &Path) -> TourRun {
        let mut states: Vec<StepState> = chapter.steps.iter().map(|s| saved.get(&s.id).copied().unwrap_or(StepState::Pending)).collect();
        if !states.contains(&StepState::Pending) {
            states.fill(StepState::Pending);
        }
        let index = states.iter().position(|s| *s == StepState::Pending).unwrap_or(0);
        TourRun {
            chapter,
            index,
            states,
            dir: dir.to_path_buf(),
            write_root: write_root.to_path_buf(),
            context: Context::blank(),
            status: String::new(),
            started: Instant::now(),
            last_canvas: None,
            moved: false,
            finished: false,
        }
    }

    /// A blank sheet, the chapter's own preparation, then the first card. A chapter taken up
    /// again at a later step starts on a fresh editor: what the earlier steps left (the brush in
    /// hand, its size) is put back first, or the step would have nothing to work on.
    pub fn setup(&mut self, ch: &mut dyn Channel) -> AppResult<()> {
        let task = Task { brief: String::new(), open: None, reference: None, canvas: Some(self.chapter.canvas()), steps: Vec::new() };
        for call in control::task_setup(&task) {
            ch.call(&call)?;
        }
        for call in &self.chapter.setup {
            ch.call(call)?;
        }
        for step in &self.chapter.steps[..self.index] {
            for call in &step.leaves {
                ch.call(call)?;
            }
        }
        self.status = self.waiting();
        self.push(ch)
    }

    fn waiting(&self) -> String {
        match self.chapter.steps.get(self.index) {
            Some(s) if s.hover.is_some() => pick("Ждём курсор", "Waiting for the pointer").to_string(),
            _ => pick("Ждём действия", "Waiting for the action").to_string(),
        }
    }

    fn current_done(&self) -> bool {
        self.states.get(self.index).is_some_and(|s| *s == StepState::Done)
    }

    /// The card and the lit place of the current step, as `tour.set` takes them.
    pub fn card(&self) -> Value {
        let step = &self.chapter.steps[self.index];
        let done = self.current_done();
        let last = self.index + 1 == self.chapter.steps.len();
        let mut buttons = Vec::new();
        if self.index > 0 {
            buttons.push(json!({ "id": "back", "label": pick("Назад", "Back") }));
        }
        if done {
            buttons.push(
                json!({ "id": "next", "label": if last { pick("Завершить главу", "Finish the chapter") } else { pick("Дальше", "Next") }, "primary": true }),
            );
        } else {
            buttons.push(json!({ "id": "skip", "label": pick("Пропустить", "Skip") }));
        }
        json!({
            "target": step.target,
            "title": step.title,
            "body": step.body,
            "task": step.task,
            "status": if done { pick("Сделано", "Done").to_string() } else { self.status.clone() },
            "done": done,
            "buttons": buttons,
            "block": true,
            "keys": step.keys,
            "toast": pick("Сейчас работает только подсвеченная область", "Only the lit place works right now"),
        })
    }

    fn push(&self, ch: &mut dyn Channel) -> AppResult<()> {
        ch.call(&Call::new("tour.set", self.card()))?;
        Ok(())
    }

    fn go(&mut self, index: usize) {
        self.index = index;
        self.moved = true;
        self.last_canvas = None;
        self.status = self.waiting();
    }

    fn advance(&mut self) {
        if self.index + 1 >= self.chapter.steps.len() {
            self.finished = true;
        } else {
            self.go(self.index + 1);
        }
    }

    fn mark(&mut self, state: StepState, db: &Db, now: NaiveDateTime) -> AppResult<()> {
        self.states[self.index] = state;
        db.set_tour_step(&self.chapter.id, &self.chapter.steps[self.index].id, state.code(), now)
    }

    /// Reads what the student did, checks the step's task and refreshes the card.
    pub fn poll(&mut self, ch: &mut dyn Channel, db: &Db, now: NaiveDateTime) -> AppResult<u32> {
        if self.finished {
            return Ok(0);
        }
        let reply = ch.call(&Call::new("tour.poll", json!({})))?;
        let outside = reply.get("outside").and_then(Value::as_u64).unwrap_or(0) as u32;
        let before = (self.index, self.states.clone(), self.status.clone());
        self.moved = false;
        for event in reply.get("events").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
            match event {
                "back" if self.index > 0 => self.go(self.index - 1),
                "skip" => {
                    if self.states[self.index] == StepState::Pending {
                        self.mark(StepState::Skipped, db, now)?;
                        // A skipped step is done for the student, so the next one finds the
                        // editor as it expects.
                        for call in &self.chapter.steps[self.index].leaves {
                            ch.call(call)?;
                        }
                    }
                    self.advance();
                }
                "next" if self.states[self.index] != StepState::Pending => self.advance(),
                _ => {}
            }
            if self.finished {
                break;
            }
        }
        if self.finished {
            return self.finish(ch, db, now).map(|()| outside);
        }
        // The editor answers `active: false` after a restart: show the card again.
        let lost = reply.get("active").and_then(Value::as_bool) == Some(false);
        if !self.moved && !lost && !self.current_done() {
            self.assess(ch, &reply, db, now)?;
        }
        if lost || before != (self.index, self.states.clone(), self.status.clone()) {
            self.push(ch)?;
        }
        Ok(outside)
    }

    fn assess(&mut self, ch: &mut dyn Channel, reply: &Value, db: &Db, now: NaiveDateTime) -> AppResult<()> {
        let step = self.chapter.steps[self.index].clone();
        if let Some(seconds) = step.hover {
            if reply.get("inside").and_then(Value::as_f64).unwrap_or(0.0) >= seconds {
                self.mark(StepState::Done, db, now)?;
            }
            return Ok(());
        }
        // A sheet snapshot costs a PNG each time: once a second is plenty for a stroke.
        let with_canvas = step.needs_canvas();
        if with_canvas && self.last_canvas.is_some_and(|t| t.elapsed() < Duration::from_millis(900)) {
            return Ok(());
        }
        let state = ch.call(&control::inspect())?;
        let canvas = if with_canvas {
            ch.call(&control::snapshot(SNAPSHOT))?;
            self.last_canvas = Some(Instant::now());
            Some(assess::load_image(&self.write_root.join(SNAPSHOT))?)
        } else {
            None
        };
        let outcome = assess::step(&step.as_step(), &self.context, &Snapshot { state: EditorState::from_inspect(&state), canvas }, false);
        if outcome.passed {
            self.mark(StepState::Done, db, now)?;
        } else if let Some(failed) = outcome.checks.iter().find(|c| !c.passed) {
            self.status = failed.message.clone();
        }
        Ok(())
    }

    fn finish(&mut self, ch: &mut dyn Channel, db: &Db, now: NaiveDateTime) -> AppResult<()> {
        // Keep the sheet for the chapter's last screen, then give the editor back.
        let _ = ch.call(&control::snapshot(SNAPSHOT));
        ch.call(&Call::new("tour.clear", json!({})))?;
        let skipped = self.states.iter().filter(|s| **s == StepState::Skipped).count();
        let detail = if skipped == 0 {
            fude_core::tr!("шагов: {}", "steps: {}", self.states.len())
        } else {
            fude_core::tr!("шагов: {}, пропущено: {skipped}", "steps: {}, skipped: {skipped}", self.states.len())
        };
        db.log(now, "tour", &fude_core::tr!("Пройдена глава путеводителя «{}»", "Tour chapter done “{}”", self.chapter.title), &detail)?;
        Ok(())
    }

    pub fn view(&self, outside: u32) -> TourView {
        let chapters = tour::builtin();
        let next = chapters
            .iter()
            .position(|c| c.id == self.chapter.id)
            .and_then(|i| chapters.get(i + 1))
            .map(|c| NextChapter { id: c.id.clone(), title: c.title.clone() });
        TourView {
            chapter: self.chapter.id.clone(),
            title: self.chapter.title.clone(),
            index: self.index,
            total: self.chapter.steps.len(),
            steps: steps_view(&self.chapter, &self.states),
            done: self.current_done(),
            finished: self.finished,
            outside,
            seconds: self.started.elapsed().as_secs(),
            picture: if self.finished { crate::state::picture(&self.write_root, SNAPSHOT) } else { None },
            next,
        }
    }
}

// ------------------------------------------------------------------ overview

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterView {
    pub id: String,
    pub title: String,
    pub short: String,
    pub summary: String,
    pub minutes: u32,
    pub steps: Vec<StepView>,
    /// `done` (nothing open), `started` or `new`.
    pub state: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub chapters: Vec<ChapterView>,
    /// The chapter to go on with: the first one with an open step.
    pub current: Option<String>,
    pub steps: usize,
    pub minutes: u32,
    pub done: usize,
}

pub fn overview(db: &Db) -> AppResult<Overview> {
    let saved = db.tour_steps()?;
    let chapters: Vec<ChapterView> = tour::builtin()
        .into_iter()
        .map(|c| {
            let states: Vec<StepState> =
                c.steps.iter().map(|s| saved.get(&(c.id.clone(), s.id.clone())).map_or(StepState::Pending, |code| StepState::from_code(code))).collect();
            let open = states.iter().filter(|s| **s == StepState::Pending).count();
            let state = if open == 0 {
                "done"
            } else if open == states.len() {
                "new"
            } else {
                "started"
            };
            ChapterView { steps: steps_view(&c, &states), id: c.id, title: c.title, short: c.short, summary: c.summary, minutes: c.minutes, state }
        })
        .collect();
    Ok(Overview {
        current: chapters.iter().find(|c| c.state != "done").map(|c| c.id.clone()),
        steps: chapters.iter().map(|c| c.steps.len()).sum(),
        minutes: chapters.iter().map(|c| c.minutes).sum(),
        done: chapters.iter().filter(|c| c.state == "done").count(),
        chapters,
    })
}

// ------------------------------------------------------------------ commands

#[tauri::command]
pub fn tour_overview(state: State<'_, AppState>) -> AppResult<Overview> {
    overview(&lock(&state.db))
}

fn tour_dir(state: &AppState) -> AppResult<PathBuf> {
    let dir = state.data.join("tour");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Starts a chapter in the editor, at its first open step.
#[tauri::command]
pub async fn tour_start(app: AppHandle, chapter: String) -> AppResult<TourView> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let found =
            tour::builtin().into_iter().find(|c| c.id == chapter).ok_or_else(|| AppError::msg(pick("нет такой главы путеводителя", "no such tour chapter")))?;
        state.guide_cancel.store(true, std::sync::atomic::Ordering::SeqCst);
        // Locks are taken practice → tour → editor → db everywhere.
        *lock(&state.practice) = None;
        *lock(&state.tour) = None;
        let dir = tour_dir(&state)?;
        let run = {
            let mut guard = state.editor_for(&dir)?;
            let editor = guard.as_mut().ok_or_else(|| AppError::msg(pick("редактор не запущен", "the editor is not running")))?;
            let saved: HashMap<String, StepState> =
                lock(&state.db).tour_steps()?.into_iter().filter(|((c, _), _)| *c == found.id).map(|((_, s), code)| (s, StepState::from_code(&code))).collect();
            let mut run = TourRun::new(found, &saved, &dir, &state.attempts_dir());
            run.setup(editor)?;
            run
        };
        let view = run.view(0);
        *lock(&state.tour) = Some(run);
        crate::window::focus_editor_soon();
        Ok(view)
    })
    .await
    .map_err(|e| AppError::msg(e.to_string()))?
}

/// Called a few times a second while a chapter runs.
#[tauri::command]
pub async fn tour_poll(app: AppHandle) -> AppResult<TourView> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let mut tour = lock(&state.tour);
        let run = tour.as_mut().ok_or_else(|| AppError::msg(pick("путеводитель не запущен", "the tour is not running")))?;
        if run.finished {
            return Ok(run.view(0));
        }
        let closed = || AppError::msg(pick("редактор закрыт — откройте главу заново", "the editor is closed — open the chapter again"));
        let mut editor = lock(&state.editor);
        let e = editor.as_mut().filter(|e| e.read_root == run.dir).ok_or_else(closed)?;
        if !e.alive() {
            return Err(closed());
        }
        let outside = run.poll(e, &lock(&state.db), now())?;
        Ok(run.view(outside))
    })
    .await
    .map_err(|e| AppError::msg(e.to_string()))?
}

/// Leaves the tour: what is done stays done, the editor closes.
#[tauri::command]
pub fn tour_stop(state: State<'_, AppState>) {
    *lock(&state.tour) = None;
    state.close_editor();
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use fude_core::i18n::{Lang, with_lang};
    use std::sync::{Arc, Mutex};

    /// An editor double: remembers the cards, answers `tour.poll` and `ui.inspect` from a script.
    #[derive(Default)]
    struct Fake {
        cards: Vec<Value>,
        polls: Arc<Mutex<Vec<Value>>>,
        tool: String,
        cleared: bool,
        calls: Vec<String>,
        sent: Vec<Call>,
    }

    impl Channel for Fake {
        fn call(&mut self, call: &Call) -> AppResult<Value> {
            self.calls.push(call.method.clone());
            self.sent.push(call.clone());
            Ok(match call.method.as_str() {
                "tour.set" => {
                    self.cards.push(call.params.clone());
                    Value::Null
                }
                "tour.clear" => {
                    self.cleared = true;
                    Value::Null
                }
                "tour.poll" => {
                    let mut polls = self.polls.lock().unwrap();
                    if polls.is_empty() { json!({"active": true, "events": [], "outside": 0, "inside": 0.0}) } else { polls.remove(0) }
                }
                "ui.inspect" => json!({ "tool": self.tool }),
                _ => Value::Null,
            })
        }
    }

    fn at() -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, 16).unwrap().and_hms_opt(12, 0, 0).unwrap()
    }

    fn chapter(id: &str) -> Chapter {
        chapter_in(Lang::Ru, id)
    }

    fn chapter_in(lang: Lang, id: &str) -> Chapter {
        with_lang(lang, tour::builtin).into_iter().find(|c| c.id == id).unwrap()
    }

    fn poll(events: &[&str], inside: f64, outside: u32) -> Value {
        json!({"active": true, "events": events, "outside": outside, "inside": inside})
    }

    #[test]
    fn a_look_around_step_is_done_by_the_pointer_and_waits_for_next() {
        let db = Db::in_memory(at()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let mut ed = Fake::default();
        let mut run = TourRun::new(chapter("window"), &HashMap::new(), dir.path(), dir.path());
        with_lang(Lang::Ru, || run.setup(&mut ed)).unwrap();
        assert_eq!(ed.cards[0]["target"], "menu");
        assert_eq!(ed.cards[0]["block"], true);
        assert_eq!(ed.cards[0]["buttons"], json!([{"id": "skip", "label": "Пропустить"}]), "no «Назад» on the first step");
        ed.polls.lock().unwrap().extend([poll(&[], 0.2, 1), poll(&[], 0.7, 0)]);
        assert_eq!(with_lang(Lang::Ru, || run.poll(&mut ed, &db, at())).unwrap(), 1, "a press outside is reported");
        assert!(!run.current_done());
        with_lang(Lang::Ru, || run.poll(&mut ed, &db, at())).unwrap();
        assert!(run.current_done());
        let card = ed.cards.last().unwrap();
        assert_eq!(card["done"], true);
        assert_eq!(card["buttons"], json!([{"id": "next", "label": "Дальше", "primary": true}]));
        // «Дальше» moves on; the time over the previous place does not count for the new one.
        ed.polls.lock().unwrap().push(poll(&["next"], 0.7, 0));
        with_lang(Lang::Ru, || run.poll(&mut ed, &db, at())).unwrap();
        assert_eq!((run.index, run.current_done()), (1, false));
        assert_eq!(ed.cards.last().unwrap()["target"], "options");
        assert_eq!(db.tour_steps().unwrap().get(&("window".to_string(), "menu".to_string())).map(String::as_str), Some("done"));
    }

    /// What the editor was told after the chapter's own preparation and before the first card.
    fn between_setup_and_card(ed: &Fake, chapter: &Chapter) -> Vec<Call> {
        let start = ed.sent.iter().position(|c| *c == chapter.setup[chapter.setup.len() - 1]).unwrap() + 1;
        let end = ed.sent.iter().position(|c| c.method == "tour.set").unwrap();
        ed.sent[start..end].to_vec()
    }

    #[test]
    fn a_chapter_taken_up_again_puts_back_what_the_earlier_steps_left() {
        let dir = tempfile::tempdir().unwrap();
        let brush = chapter("brush");
        let saved: HashMap<String, StepState> = [("pick".to_string(), StepState::Done), ("size".to_string(), StepState::Skipped)].into();
        let mut ed = Fake::default();
        let mut run = TourRun::new(brush.clone(), &saved, dir.path(), dir.path());
        assert_eq!(run.index, 2, "goes on from «Жёсткость»");
        with_lang(Lang::Ru, || run.setup(&mut ed)).unwrap();
        // The setup ends on the Move tool; the brush comes back in hand at size 40.
        let replayed = between_setup_and_card(&ed, &brush);
        assert_eq!(replayed, [brush.steps[0].leaves.clone(), brush.steps[1].leaves.clone()].concat());
        assert_eq!(replayed[0].params, json!({"tool": "Brush"}));
        // A fresh start replays nothing.
        let mut ed = Fake::default();
        let mut run = TourRun::new(brush.clone(), &HashMap::new(), dir.path(), dir.path());
        with_lang(Lang::Ru, || run.setup(&mut ed)).unwrap();
        assert!(between_setup_and_card(&ed, &brush).is_empty());
    }

    #[test]
    fn a_skipped_step_leaves_the_editor_as_if_it_was_done() {
        let db = Db::in_memory(at()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let brush = chapter("brush");
        let mut ed = Fake { tool: "Move".into(), ..Fake::default() };
        let mut run = TourRun::new(brush.clone(), &HashMap::new(), dir.path(), dir.path());
        with_lang(Lang::Ru, || run.setup(&mut ed)).unwrap();
        let before = ed.sent.len();
        ed.polls.lock().unwrap().push(poll(&["skip"], 0.0, 0));
        with_lang(Lang::Ru, || run.poll(&mut ed, &db, at())).unwrap();
        assert_eq!(run.index, 1);
        assert!(ed.sent[before..].contains(&brush.steps[0].leaves[0]), "skipping «Кисть» puts the brush in hand");
    }

    #[test]
    fn a_do_it_step_is_checked_and_says_what_is_missing() {
        let db = Db::in_memory(at()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let mut ed = Fake { tool: "Move".into(), ..Fake::default() };
        let mut run = TourRun::new(chapter("brush"), &HashMap::new(), dir.path(), dir.path());
        with_lang(Lang::Ru, || run.setup(&mut ed)).unwrap();
        assert!(ed.calls.iter().filter(|m| *m == "ui.set").count() >= 2, "the chapter's own preparation ran");
        assert_eq!(ed.cards[0]["keys"], json!(["B"]));
        with_lang(Lang::Ru, || run.poll(&mut ed, &db, at())).unwrap();
        assert!(!run.current_done());
        assert!(ed.cards.last().unwrap()["status"].as_str().unwrap().contains("Кисть"), "the failed check explains itself");
        ed.tool = "Brush".into();
        with_lang(Lang::Ru, || run.poll(&mut ed, &db, at())).unwrap();
        assert!(run.current_done());
        assert_eq!(ed.cards.last().unwrap()["status"], "Сделано");
    }

    #[test]
    fn skip_and_back_work_and_the_last_step_finishes_the_chapter() {
        let db = Db::in_memory(at()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let mut ed = Fake::default();
        let mut run = TourRun::new(chapter_in(Lang::En, "window"), &HashMap::new(), dir.path(), dir.path());
        with_lang(Lang::En, || run.setup(&mut ed)).unwrap();
        ed.polls.lock().unwrap().extend([poll(&["skip"], 0.0, 0), poll(&["back"], 0.0, 0), poll(&["next"], 0.0, 0)]);
        with_lang(Lang::En, || run.poll(&mut ed, &db, at())).unwrap();
        assert_eq!((run.index, run.states[0]), (1, StepState::Skipped));
        assert_eq!(ed.cards.last().unwrap()["buttons"][0]["label"], "Back");
        with_lang(Lang::En, || run.poll(&mut ed, &db, at())).unwrap();
        assert_eq!(run.index, 0);
        // A skipped step can be left with «Дальше»… there is none: it still offers the task.
        with_lang(Lang::En, || run.poll(&mut ed, &db, at())).unwrap();
        assert_eq!(run.index, 1, "next works on a skipped step");
        for _ in 0..4 {
            ed.polls.lock().unwrap().push(poll(&["skip"], 0.0, 0));
            with_lang(Lang::En, || run.poll(&mut ed, &db, at())).unwrap();
        }
        assert!(run.finished && ed.cleared);
        let view = with_lang(Lang::En, || run.view(0));
        assert_eq!(view.next.unwrap().id, "brush");
        assert_eq!(db.journal(5).unwrap()[0].title, "Tour chapter done “The editor window”");
        // Nothing open: the overview says done, and starting again starts over.
        let o = with_lang(Lang::En, || overview(&db)).unwrap();
        assert_eq!((o.chapters[0].state, o.current.as_deref(), o.done), ("done", Some("brush"), 1));
        let saved: HashMap<String, StepState> =
            db.tour_steps().unwrap().into_iter().filter(|((c, _), _)| c == "window").map(|((_, s), code)| (s, StepState::from_code(&code))).collect();
        let again = TourRun::new(chapter("window"), &saved, dir.path(), dir.path());
        assert_eq!((again.index, again.states[0]), (0, StepState::Pending));
    }

    #[test]
    fn a_chapter_left_halfway_resumes_at_its_first_open_step() {
        let saved = HashMap::from([("pick".to_string(), StepState::Done), ("size".to_string(), StepState::Skipped)]);
        let dir = tempfile::tempdir().unwrap();
        let run = TourRun::new(chapter("brush"), &saved, dir.path(), dir.path());
        assert_eq!(run.index, 2);
        let v = with_lang(Lang::Ru, || run.view(0));
        assert_eq!((v.total, v.steps[1].state, v.finished), (6, StepState::Skipped, false));
    }
}
