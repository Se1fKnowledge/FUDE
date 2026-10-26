//! The built-in lessons in `lessons/`: valid, complete, and assessable.
//!
//! Each lesson's `assets/reference.png` is its finished result. Every canvas check must pass
//! on it, and every step with canvas checks must fail on the untouched task: otherwise a
//! check either cannot be met or is met without doing anything.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::HashSet;
use std::path::PathBuf;

use fude_core::assess::{self, Base, Context, EditorState, Snapshot};
use fude_core::guide;
use fude_core::hint;
use fude_core::library::{LessonEntry, Library, Root};
use fude_core::model::Skill;
use fude_core::package::{self, ExportItem, Limits};
use image::{Rgba, RgbaImage};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("lessons")
}

fn library() -> Library {
    Library::scan(&[Root::builtin(root())])
}

#[test]
fn the_builtin_library_has_no_problems_or_warnings() {
    let library = library();
    for p in &library.problems {
        eprintln!("{}: {}", p.path.display(), p.message);
        for i in &p.issues {
            eprintln!("  {i}");
        }
    }
    assert!(library.problems.is_empty());
    for entry in library.lessons() {
        assert!(entry.warnings.is_empty(), "{}: {:?}", entry.lesson.id, entry.warnings);
    }
}

#[test]
fn two_categories_of_six_lessons_with_rising_difficulty() {
    let library = library();
    let ids: Vec<_> = library.categories.iter().map(|c| c.category.id.as_str()).collect();
    assert_eq!(ids, ["basics", "light"]);
    for c in &library.categories {
        assert_eq!(c.lessons.len(), 6, "{}", c.category.id);
        assert!(c.category.kanji.is_some());
        let difficulty: Vec<u8> = c.lessons.iter().map(|l| l.lesson.difficulty).collect();
        assert!(difficulty.windows(2).all(|w| w[0] <= w[1]), "{}: {difficulty:?}", c.category.id);
    }
}

#[test]
fn lessons_form_one_path_from_the_first() {
    let library = library();
    let all: Vec<&LessonEntry> = library.lessons().collect();
    assert!(all[0].lesson.requires.is_empty(), "the first lesson is open");
    let mut done = HashSet::new();
    for entry in &all {
        assert!(library.is_unlocked(&entry.lesson.id, &done), "{} is locked after its predecessors", entry.lesson.id);
        done.insert(entry.lesson.id.clone());
    }
    let open: Vec<_> = all.iter().filter(|e| e.lesson.requires.is_empty()).collect();
    assert_eq!(open.len(), 1, "only the first lesson starts unlocked");
}

#[test]
fn every_lesson_teaches_theory_guide_and_quiz() {
    for entry in library().lessons() {
        let l = &entry.lesson;
        let theory = std::fs::read_to_string(entry.dir.join(&l.theory.file)).unwrap();
        assert!(theory.starts_with(&format!("# {}", l.title)), "{}: theory starts with the title", l.id);
        assert!(theory.chars().count() > 1200, "{}: theory is substantial", l.id);
        assert!(theory.contains("## Совет сэнсэя"), "{}", l.id);
        assert!(l.quiz.len() >= 2, "{}", l.id);
        assert!(!l.summary.is_empty() && l.minutes > 0, "{}", l.id);
        assert!(l.task.steps.len() >= 3, "{}", l.id);
        for step in &l.task.steps {
            assert!(step.hint.is_some(), "{}: step {} has a hint", l.id, step.id);
            assert!(!step.checks.is_empty(), "{}: step {} is checked", l.id, step.id);
        }
        let script = guide::load(&entry.dir.join(l.guide.script.as_deref().unwrap())).unwrap();
        assert!(guide::validate(&script, Some(&entry.dir)).is_empty(), "{}", l.id);
        assert!(script.steps.len() >= 4, "{}", l.id);
        assert!(script.duration_ms() >= 5000, "{}: the guide is long enough to follow", l.id);
    }
}

#[test]
fn the_skills_of_the_statistics_screen_are_trained() {
    let trained: HashSet<Skill> = library().lessons().flat_map(|l| l.lesson.skills.clone()).collect();
    for skill in [Skill::Tools, Skill::Line, Skill::Form, Skill::Light, Skill::Color, Skill::Composition] {
        assert!(trained.contains(&skill), "{skill:?}");
    }
}

fn untouched(context: &Context, width: u32, height: u32) -> RgbaImage {
    match &context.base {
        Base::Image(img) => img.clone(),
        Base::Color([r, g, b]) => RgbaImage::from_pixel(width, height, Rgba([*r, *g, *b, 255])),
    }
}

#[test]
fn canvas_checks_pass_on_the_reference_and_fail_on_the_untouched_task() {
    let mut checked = 0;
    for entry in library().lessons() {
        let l = &entry.lesson;
        let has_canvas_checks = l.task.steps.iter().any(|s| s.checks.iter().any(|c| c.kind.needs_canvas()));
        let Some(reference) = &l.task.reference else {
            assert!(!has_canvas_checks, "{}: canvas checks need a reference to prove them", l.id);
            continue;
        };
        let context = Context::for_lesson(l, &entry.dir).unwrap();
        let done = assess::load_image(&entry.dir.join(reference)).unwrap();
        let before = untouched(&context, done.width(), done.height());
        let state = EditorState::default();
        for step in &l.task.steps {
            let canvas: Vec<_> = step.checks.iter().filter(|c| c.kind.needs_canvas()).collect();
            if canvas.is_empty() {
                continue;
            }
            let finished = Snapshot { state: state.clone(), canvas: Some(done.clone()) };
            for check in &canvas {
                let out = assess::check(check, &context, &finished);
                assert!(out.passed, "{} / {}: {} fails on the reference: {} ({:?})", l.id, step.id, out.kind, out.message, out.value);
            }
            let start = Snapshot { state: state.clone(), canvas: Some(before.clone()) };
            let any_fails = canvas.iter().any(|c| !assess::check(c, &context, &start).passed);
            // A step may also be held by an editor-state check (layer opacity, a tool…).
            let guarded = step.checks.iter().any(|c| !c.kind.needs_canvas());
            assert!(any_fails || guarded, "{} / {}: passes without doing anything", l.id, step.id);
            checked += 1;
        }
    }
    assert!(checked >= 12, "only {checked} canvas steps checked");
}

#[test]
fn hints_draw_with_guide_safe_calls() {
    for entry in library().lessons() {
        for step in &entry.lesson.task.steps {
            let Some(h) = &step.hint else { continue };
            for call in hint::show(h, 1200, 800, Some(1)) {
                assert_eq!(guide::check_call(&call), Ok(()), "{} / {}", entry.lesson.id, step.id);
            }
        }
    }
}

#[test]
fn the_builtin_categories_pack_and_check_as_packages() {
    let tmp = tempfile::tempdir().unwrap();
    for category in library().categories {
        let file = tmp.path().join(format!("{}.{}", category.category.id, package::EXTENSION));
        let report = package::export(&file, &category.category.title, &[ExportItem { category_dir: category.dirs[0].clone(), lessons: vec![] }]).unwrap();
        assert_eq!(report.lessons.len(), 6);
        assert!(report.skipped.is_empty(), "{:?}", report.skipped);
        let info = package::inspect(&file, tmp.path(), Limits::default()).unwrap();
        assert_eq!(info.lessons.len(), 6);
        assert!(info.issues.is_empty(), "{:?}", info.issues);
    }
}
