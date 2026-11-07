//! Lessons in other languages: an overlay file `en.toml` next to `lesson.toml` (or next to
//! `category.toml`) and an optional `theory.en.md`.
//!
//! The overlay is one table of the lesson's own texts and their translations:
//!
//! ```toml
//! [text]
//! "Падающая тень от сферы" = "A sphere's cast shadow"
//! "Создайте слой «Тень»." = "Create a «Shadow» layer."
//! ```
//!
//! Any text of the lesson found in the table is replaced: the title, summary, brief, step
//! texts, hints, check messages, quiz questions, options and explanations, and the captions of
//! `guide.json`. Keying by the original text keeps the overlay right when steps move. Texts
//! with no entry stay as written. The lesson files themselves are never changed: the studio
//! edits the original, and the library applies the overlay when it reads lessons for the
//! student (see [`crate::library::Library::scan`]).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::guide::Script;
use crate::i18n::Lang;
use crate::model::{Category, Lesson};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overlay {
    #[serde(default)]
    pub text: HashMap<String, String>,
}

impl Overlay {
    /// The overlay for `lang` in `dir`, if there is one (Russian, the source, has none).
    pub fn load(dir: &Path, lang: Lang) -> Option<Overlay> {
        if lang == Lang::Ru {
            return None;
        }
        let text = std::fs::read_to_string(file(dir, lang)).ok()?;
        toml::from_str(&text).ok()
    }

    /// The translation of `s`, or `s` itself.
    fn put(&self, s: &mut String) {
        if let Some(t) = self.text.get(s.trim()).filter(|t| !t.trim().is_empty()) {
            *s = t.clone();
        }
    }

    pub fn apply_lesson(&self, l: &mut Lesson) {
        self.put(&mut l.title);
        self.put(&mut l.summary);
        self.put(&mut l.task.brief);
        for step in &mut l.task.steps {
            self.put(&mut step.text);
            if let Some(h) = &mut step.hint {
                self.put(&mut h.text);
            }
            for c in &mut step.checks {
                if let Some(m) = &mut c.message {
                    self.put(m);
                }
            }
        }
        for q in &mut l.quiz {
            self.put(&mut q.question);
            q.options.iter_mut().for_each(|o| self.put(o));
            if let Some(e) = &mut q.explain {
                self.put(e);
            }
        }
    }

    pub fn apply_category(&self, c: &mut Category) {
        self.put(&mut c.title);
        self.put(&mut c.description);
    }

    pub fn apply_script(&self, s: &mut Script) {
        s.steps.iter_mut().for_each(|st| self.put(&mut st.caption));
    }

    /// Entries that match no text of the lesson or its guide: likely typos or stale after edits.
    pub fn unused(&self, lesson: &Lesson, script: Option<&Script>) -> Vec<String> {
        let texts = texts(lesson, script);
        let mut out: Vec<String> = self.text.keys().filter(|k| !texts.contains(&k.as_str())).cloned().collect();
        out.sort();
        out
    }

    /// Texts of the lesson and its guide that have no entry yet. Texts without Russian letters
    /// (shortcuts like «Ctrl+0», colour codes) read the same in English and need none.
    pub fn missing(&self, lesson: &Lesson, script: Option<&Script>) -> Vec<String> {
        let russian = |t: &str| t.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));
        texts(lesson, script).into_iter().filter(|t| russian(t) && !self.text.contains_key(*t)).map(str::to_string).collect()
    }
}

/// Every text a student reads in a lesson and its guide, trimmed, without empties and repeats.
pub fn texts<'a>(lesson: &'a Lesson, script: Option<&'a Script>) -> Vec<&'a str> {
    let mut all: Vec<&str> = vec![&lesson.title, &lesson.summary, &lesson.task.brief];
    for step in &lesson.task.steps {
        all.push(&step.text);
        if let Some(h) = &step.hint {
            all.push(&h.text);
        }
        all.extend(step.checks.iter().filter_map(|c| c.message.as_deref()));
    }
    for q in &lesson.quiz {
        all.push(&q.question);
        all.extend(q.options.iter().map(String::as_str));
        all.extend(q.explain.as_deref());
    }
    if let Some(s) = script {
        all.extend(s.steps.iter().map(|st| st.caption.as_str()));
    }
    let mut out: Vec<&str> = Vec::new();
    for t in all.into_iter().map(str::trim).filter(|t| !t.is_empty()) {
        if !out.contains(&t) {
            out.push(t);
        }
    }
    out
}

/// Warnings about a lesson's overlays for the CLI and the library: a file that does not read,
/// entries that match nothing.
pub fn check(dir: &Path, lesson: &Lesson) -> Vec<crate::validate::Issue> {
    use crate::validate::Issue;
    let mut issues = Vec::new();
    for lang in [Lang::En] {
        let path = file(dir, lang);
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        let name = format!("{}.toml", lang.code());
        match toml::from_str::<Overlay>(&text) {
            Err(e) => issues.push(Issue::error(&name, crate::tr!("не читается: {}", "does not read: {}", e.message()))),
            Ok(o) => {
                let script = lesson.guide.script.as_ref().and_then(|s| crate::guide::load(&dir.join(s)).ok());
                for k in o.unused(lesson, script.as_ref()) {
                    issues.push(Issue::warning(&name, crate::tr!("«{k}» нет в уроке — устаревшая строка?", "“{k}” is not in the lesson — a stale entry?")));
                }
            }
        }
    }
    issues
}

/// `en.toml` in `dir`.
pub fn file(dir: &Path, lang: Lang) -> PathBuf {
    dir.join(format!("{}.toml", lang.code()))
}

/// The theory file to show: `theory.en.md` for English when it exists, else the original.
pub fn theory_path(dir: &Path, lesson: &Lesson, lang: Lang) -> PathBuf {
    let original = dir.join(&lesson.theory.file);
    if lang == Lang::Ru {
        return original;
    }
    let stem = Path::new(&lesson.theory.file).file_stem().and_then(|s| s.to_str()).unwrap_or("theory");
    let ext = Path::new(&lesson.theory.file).extension().and_then(|s| s.to_str()).unwrap_or("md");
    let localized = dir.join(format!("{stem}.{}.{ext}", lang.code()));
    if localized.is_file() { localized } else { original }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{load_lesson, parse_lesson};

    const LESSON: &str = r##"
format = 1
id = "light.sphere"
title = "Тень сферы"
summary = "Кратко"
difficulty = 2

[task]
brief = "Нарисуйте тень"
canvas = { width = 100, height = 100 }

[[task.steps]]
id = "layer"
text = "Создайте слой"
hint = { text = "Ctrl+Shift+N" }

[[task.steps.checks]]
kind = "layers"
count = { min = 2 }
message = "Нужен второй слой"

[[quiz]]
question = "Где тень?"
options = ["Слева", "Справа"]
answer = 1
"##;

    fn overlay() -> Overlay {
        toml::from_str(
            r#"
[text]
"Тень сферы" = "Sphere shadow"
"Создайте слой" = "Create a layer"
"Нужен второй слой" = "A second layer is needed"
"Справа" = "On the right"
"Лишняя строка" = "Stale"
"#,
        )
        .unwrap()
    }

    #[test]
    fn replaces_texts_that_have_an_entry_and_keeps_the_rest() {
        let mut l = parse_lesson(LESSON, Path::new("lesson.toml")).unwrap();
        overlay().apply_lesson(&mut l);
        assert_eq!(l.title, "Sphere shadow");
        assert_eq!(l.task.steps[0].text, "Create a layer");
        assert_eq!(l.task.steps[0].checks[0].message.as_deref(), Some("A second layer is needed"));
        assert_eq!(l.quiz[0].options, ["Слева", "On the right"]);
        assert_eq!(l.summary, "Кратко", "no entry, no change");
    }

    #[test]
    fn names_entries_that_match_nothing() {
        let l = parse_lesson(LESSON, Path::new("lesson.toml")).unwrap();
        assert_eq!(overlay().unused(&l, None), ["Лишняя строка"]);
    }

    #[test]
    fn loads_the_overlay_and_the_english_theory_from_the_lesson_folder() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("lesson.toml"), LESSON).unwrap();
        std::fs::write(dir.path().join("theory.md"), "Русский").unwrap();
        let l = load_lesson(dir.path()).unwrap();
        assert!(Overlay::load(dir.path(), Lang::En).is_none(), "no en.toml yet");
        assert_eq!(theory_path(dir.path(), &l, Lang::En), dir.path().join("theory.md"));
        std::fs::write(dir.path().join("en.toml"), "[text]\n\"Тень сферы\" = \"Sphere shadow\"\n").unwrap();
        std::fs::write(dir.path().join("theory.en.md"), "English").unwrap();
        assert_eq!(Overlay::load(dir.path(), Lang::En).unwrap().text.len(), 1);
        assert!(Overlay::load(dir.path(), Lang::Ru).is_none(), "Russian is the source");
        assert_eq!(theory_path(dir.path(), &l, Lang::En), dir.path().join("theory.en.md"));
        assert_eq!(theory_path(dir.path(), &l, Lang::Ru), dir.path().join("theory.md"));
    }
}
