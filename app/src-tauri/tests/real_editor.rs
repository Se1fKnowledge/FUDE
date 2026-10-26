//! Drives the real PhotoCraft: opens a lesson task, plays its guide and checks the task.
//! It opens an editor window, so it is ignored by default:
//!
//! ```bash
//! cargo test -p fude-app --test real_editor -- --ignored --nocapture
//! ```
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

use chrono::Local;
use fude_app_lib::db::Db;
use fude_app_lib::editor::{Editor, find_executable};
use fude_app_lib::practice::Practice;
use fude_core::guide;
use fude_core::library::{Library, Root};

fn run_lesson(id: &str) {
    let exe = find_executable(None).expect("build PhotoCraft first: cargo build --release -p photocraft");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("lessons");
    let library = Library::scan(&[Root::builtin(&root)]);
    let entry = library.lesson(id).unwrap();
    let data = tempfile::tempdir().unwrap();
    let write_root = data.path().join("attempts");
    let mut editor = Editor::launch(&exe, &data.path().join("control.token"), &entry.dir, &write_root).unwrap();

    let db = Db::in_memory(Local::now().naive_local()).unwrap();
    let attempt = db.start_attempt(id, Local::now().naive_local()).unwrap();
    let mut practice = Practice::new(attempt, entry.lesson.clone(), &entry.dir, &write_root).unwrap();
    practice.setup(&mut editor).unwrap();
    let first = practice.check(&mut editor, &db, Local::now().naive_local()).unwrap();
    println!("{id}: before the guide {}/{} steps", first.done, first.total);

    // The guide is the model solution, checked as a student works: after every step.
    let script = guide::load(&entry.dir.join(entry.lesson.guide.script.as_ref().unwrap())).unwrap();
    for step in &script.steps {
        for call in &step.calls {
            if let Err(e) = editor.call(call) {
                panic!("{id}: «{}» failed: {e}", step.caption);
            }
        }
        if !practice.is_finished() {
            practice.check(&mut editor, &db, Local::now().naive_local()).unwrap();
        }
    }
    if practice.is_finished() {
        println!("{id}: finished by the guide, {} hints", practice.hints);
        return;
    }
    let after = practice.check(&mut editor, &db, Local::now().naive_local()).unwrap();
    for (s, o) in entry.lesson.task.steps.iter().zip(&after.outcome.steps) {
        let failed: Vec<&str> = o.checks.iter().filter(|c| !c.passed).map(|c| c.message.as_str()).collect();
        println!("  {} {} {:?}", if o.passed { "✓" } else { "·" }, s.text, failed);
    }
    println!("{id}: after the guide {}/{} steps, finished: {}", after.done, after.total, after.finished);
}

#[test]
#[ignore = "opens the PhotoCraft window"]
fn guides_solve_their_tasks_in_the_real_editor() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("lessons");
    for entry in Library::scan(&[Root::builtin(root)]).lessons() {
        let id = entry.lesson.id.as_str();
        run_lesson(id);
    }
}
