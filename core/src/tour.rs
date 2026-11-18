//! The editor tour: chapters that walk a student through the PhotoCraft window one lit place
//! at a time. Each step names the place to light, says what it is and sets a small task; the
//! task is checked either by the same checks lessons use or by the pointer resting on the place.
//!
//! The chapters ship inside the app (`core/tour/*.toml`), in Russian with English beside it.

use serde::{Deserialize, Serialize};

use crate::control::Call;
use crate::i18n::Lang;
use crate::model::{Canvas, Check, Step};

/// Places of the editor window a step can light besides a single tool (`tool:Brush`).
pub const ZONES: [&str; 6] = ["menu", "options", "toolbar", "canvas", "dock", "status"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    /// One word for the route line.
    pub short: String,
    pub summary: String,
    pub minutes: u32,
    /// The blank document the chapter starts on (1200 × 800 white when not given).
    #[serde(default)]
    pub canvas: Option<Canvas>,
    /// Control calls that bring the editor to a known state before the first step.
    #[serde(default)]
    pub setup: Vec<Call>,
    pub steps: Vec<TourStep>,
    #[serde(default, skip_serializing)]
    pub en: Option<ChapterText>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChapterText {
    pub title: String,
    pub short: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TourStep {
    pub id: String,
    pub title: String,
    pub body: String,
    pub task: String,
    /// What the student can do after the step, for the chapter's summary.
    pub learned: String,
    /// A key to show beside the step (`B`, `Ctrl+Z`).
    #[serde(default)]
    pub key: Option<String>,
    /// The lit place: one of [`ZONES`] or `tool:<Tool>`.
    pub target: String,
    /// Keys that work during the step; everything else on the keyboard is held back.
    #[serde(default)]
    pub keys: Vec<String>,
    /// Seconds the pointer must spend over the lit place, for a look-around step.
    #[serde(default)]
    pub hover: Option<f64>,
    /// Lesson checks that must all pass, for a do-it step.
    #[serde(default)]
    pub checks: Vec<Check>,
    /// Control calls that put the editor where a finished step leaves it (the brush picked, the
    /// size set). A chapter taken up again at a later step replays them for the steps before
    /// it, and a skipped step runs its own, so the next step always has what it needs.
    #[serde(default, skip_serializing)]
    pub leaves: Vec<Call>,
    #[serde(default, skip_serializing)]
    pub en: Option<StepText>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepText {
    pub title: String,
    pub body: String,
    pub task: String,
    pub learned: String,
}

impl TourStep {
    /// The step as a lesson step, for [`crate::assess::step`].
    pub fn as_step(&self) -> Step {
        Step { id: self.id.clone(), text: self.task.clone(), hint: None, checks: self.checks.clone() }
    }

    pub fn needs_canvas(&self) -> bool {
        self.checks.iter().any(|c| c.kind.needs_canvas())
    }
}

impl Chapter {
    pub fn canvas(&self) -> Canvas {
        self.canvas.clone().unwrap_or(Canvas { width: 1200, height: 800, background: "#FFFFFF".into() })
    }

    /// The chapter with its texts in `lang`.
    pub fn localized(mut self, lang: Lang) -> Chapter {
        if lang == Lang::En {
            if let Some(en) = self.en.take() {
                self.title = en.title;
                self.short = en.short;
                self.summary = en.summary;
            }
            for step in &mut self.steps {
                if let Some(en) = step.en.take() {
                    step.title = en.title;
                    step.body = en.body;
                    step.task = en.task;
                    step.learned = en.learned;
                }
            }
        }
        self
    }
}

const SOURCES: [(&str, &str); 3] = [
    ("01-window.toml", include_str!("../tour/01-window.toml")),
    ("02-brush.toml", include_str!("../tour/02-brush.toml")),
    ("03-eraser.toml", include_str!("../tour/03-eraser.toml")),
];

/// The chapters that ship with the app, in route order, in the current language.
pub fn builtin() -> Vec<Chapter> {
    let lang = crate::i18n::lang();
    // A chapter that does not parse is left out; the tests keep every shipped one sound.
    SOURCES.iter().filter_map(|(_, text)| parse(text).ok()).map(|c| c.localized(lang)).collect()
}

pub fn parse(text: &str) -> Result<Chapter, String> {
    let chapter: Chapter = toml::from_str(text).map_err(|e| e.to_string())?;
    let problems = check(&chapter);
    if problems.is_empty() { Ok(chapter) } else { Err(problems.join("; ")) }
}

/// What is wrong with a chapter, as plain messages; empty when it can run.
pub fn check(chapter: &Chapter) -> Vec<String> {
    let mut out = Vec::new();
    if chapter.steps.is_empty() {
        out.push("a chapter needs at least one step".to_string());
    }
    let mut seen = std::collections::HashSet::new();
    for step in &chapter.steps {
        let at = format!("step `{}`", step.id);
        if !seen.insert(step.id.as_str()) {
            out.push(format!("{at} is there twice"));
        }
        if !ZONES.contains(&step.target.as_str()) {
            match step.target.strip_prefix("tool:") {
                Some(_) => {
                    if let Err(e) = crate::validate::check_target(&step.target) {
                        out.push(format!("{at}: {e}"));
                    }
                }
                None => out.push(format!("{at}: target `{}` is neither a zone ({}) nor tool:<Tool>", step.target, ZONES.join(", "))),
            }
        }
        match (step.hover, step.checks.is_empty()) {
            (Some(s), true) if s > 0.0 && s <= 10.0 => {}
            (Some(_), true) => out.push(format!("{at}: hover must be between 0 and 10 seconds")),
            (None, false) => {}
            (Some(_), false) => out.push(format!("{at}: give either hover or checks, not both")),
            (None, true) => out.push(format!("{at}: a step needs hover or checks, or it can never be done")),
        }
        for text in [&step.title, &step.body, &step.task, &step.learned] {
            if text.trim().is_empty() {
                out.push(format!("{at}: title, body, task and learned must not be empty"));
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::with_lang;

    fn cyrillic(s: &str) -> bool {
        s.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c))
    }

    #[test]
    fn every_builtin_chapter_parses_and_has_unique_ids() {
        let chapters = with_lang(Lang::Ru, builtin);
        assert_eq!(chapters.len(), SOURCES.len());
        let ids: std::collections::HashSet<&str> = chapters.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids.len(), chapters.len());
        for c in &chapters {
            assert!(check(c).is_empty(), "{}: {:?}", c.id, check(c));
            assert!(c.minutes > 0 && !c.short.contains(' '), "{}: `short` is one word for the route line", c.id);
        }
    }

    #[test]
    fn a_do_it_step_that_changes_the_editor_says_how_to_leave_it_done() {
        // A later step builds on it: the brush must be in hand before its size can change.
        for c in with_lang(Lang::Ru, builtin) {
            for (i, s) in c.steps.iter().enumerate() {
                let later = i + 1 < c.steps.len();
                let builds_on = s.checks.iter().any(|k| matches!(k.kind.name(), "tool" | "brush"));
                if later && builds_on {
                    assert!(!s.leaves.is_empty(), "{}/{}: a later step needs what this one leaves", c.id, s.id);
                }
            }
        }
        let brush = with_lang(Lang::Ru, builtin).into_iter().find(|c| c.id == "brush").unwrap();
        assert_eq!(brush.steps[0].leaves[0].method, "ui.set");
    }

    #[test]
    fn every_builtin_chapter_is_fully_in_english() {
        for c in with_lang(Lang::En, builtin) {
            let mut texts = vec![c.title.clone(), c.short.clone(), c.summary.clone()];
            for s in &c.steps {
                texts.extend([s.title.clone(), s.body.clone(), s.task.clone(), s.learned.clone()]);
            }
            for t in texts {
                assert!(!cyrillic(&t), "{}: not translated: {t}", c.id);
                assert!(!t.contains('«'), "{}: Russian quotes in English text: {t}", c.id);
            }
        }
        let ru = with_lang(Lang::Ru, builtin);
        assert!(cyrillic(&ru[0].title));
    }

    #[test]
    fn a_step_is_done_by_hover_or_by_checks_never_by_neither() {
        let base = include_str!("../tour/01-window.toml");
        assert!(parse(base).is_ok());
        let no_way = base.replacen("hover = 0.6", "", 1);
        assert!(parse(&no_way).unwrap_err().contains("can never be done"));
        let bad_target = base.replacen("target = \"menu\"", "target = \"ceiling\"", 1);
        assert!(parse(&bad_target).unwrap_err().contains("neither a zone"));
        let bad_tool = include_str!("../tour/02-brush.toml").replacen("target = \"tool:Brush\"", "target = \"tool:Spoon\"", 1);
        assert!(parse(&bad_tool).is_err());
        let twice = base.replacen("id = \"options\"", "id = \"menu\"", 1);
        assert!(parse(&twice).unwrap_err().contains("twice"));
    }

    #[test]
    fn a_do_it_step_becomes_a_lesson_step_for_the_checks() {
        let brush = with_lang(Lang::Ru, builtin).into_iter().find(|c| c.id == "brush").unwrap();
        let pick = &brush.steps[0];
        assert_eq!(pick.as_step().checks.len(), 1);
        assert!(!pick.needs_canvas());
        assert!(brush.steps.iter().find(|s| s.id == "stroke").unwrap().needs_canvas());
        assert_eq!(brush.canvas().width, 1200);
        assert!(!brush.setup.is_empty());
    }
}
