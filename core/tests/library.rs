#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::collections::HashSet;
use std::fs;

use fude_core::library::{Library, Root};

#[test]
fn a_library_lists_categories_and_lessons_in_order() {
    let tmp = tempfile::tempdir().unwrap();
    let light = common::category(tmp.path(), "light", "Свет и тень");
    common::lesson(&light, "02-shadow", "light.shadow", 2);
    common::lesson(&light, "01-source", "light.source", 1);
    let basics = common::category(tmp.path(), "basics", "Основы");
    fs::write(basics.join("category.toml"), "id = \"basics\"\ntitle = \"Основы\"\norder = 0\n").unwrap();
    common::lesson(&basics, "01-canvas", "basics.canvas", 1);

    let library = Library::scan(&[Root::builtin(tmp.path())]);
    assert!(library.problems.is_empty(), "{:?}", library.problems);
    let ids: Vec<_> = library.categories.iter().map(|c| c.category.id.as_str()).collect();
    assert_eq!(ids, ["basics", "light"]);
    let lessons: Vec<_> = library.lessons().map(|l| l.lesson.id.as_str()).collect();
    assert_eq!(lessons, ["basics.canvas", "light.source", "light.shadow"]);
    assert!(!library.lesson("light.shadow").unwrap().editable);
    assert_eq!(library.category("light").unwrap().user_dir, None);
}

#[test]
fn a_broken_lesson_is_a_problem_not_a_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let light = common::category(tmp.path(), "light", "Свет");
    common::lesson(&light, "01-ok", "light.ok", 1);
    let bad = common::lesson(&light, "02-bad", "light.bad", 2);
    fs::remove_file(bad.join("theory.md")).unwrap();
    let wrong = common::lesson(&light, "03-wrong", "basics.wrong", 3);
    let garbage = light.join("04-garbage");
    fs::create_dir_all(&garbage).unwrap();
    fs::write(garbage.join("lesson.toml"), "это не toml = = =").unwrap();

    let library = Library::scan(&[Root::builtin(tmp.path())]);
    assert_eq!(library.lessons().count(), 1);
    assert_eq!(library.problems.len(), 3, "{:#?}", library.problems);
    let bad_problem = library.problems.iter().find(|p| p.path.starts_with(&bad)).unwrap();
    assert!(bad_problem.issues.iter().any(|i| i.field == "theory.file"));
    assert!(library.problems.iter().any(|p| p.path.starts_with(&wrong)));
}

#[test]
fn a_bad_category_and_a_missing_root_are_handled() {
    let tmp = tempfile::tempdir().unwrap();
    let misnamed = common::category(tmp.path(), "light", "Свет");
    fs::write(misnamed.join("category.toml"), "id = \"shadow\"\ntitle = \"Свет\"\n").unwrap();
    fs::create_dir_all(tmp.path().join("not-a-category")).unwrap();
    let library = Library::scan(&[Root::builtin(tmp.path()), Root::user(tmp.path().join("missing"))]);
    assert!(library.categories.is_empty());
    assert_eq!(library.problems.len(), 1);
}

#[test]
fn user_lessons_join_builtin_categories_and_cannot_shadow_builtin_ids() {
    let tmp = tempfile::tempdir().unwrap();
    let builtin = tmp.path().join("builtin");
    let user = tmp.path().join("user");
    let b = common::category(&builtin, "light", "Свет и тень");
    common::lesson(&b, "01-source", "light.source", 1);
    let u = common::category(&user, "light", "Мой свет");
    common::lesson(&u, "01-mine", "light.mine", 5);
    common::lesson(&u, "02-dupe", "light.source", 6);

    let library = Library::scan(&[Root::builtin(&builtin), Root::user(&user)]);
    assert_eq!(library.categories.len(), 1);
    let entry = &library.categories[0];
    assert_eq!(entry.category.title, "Свет и тень", "the first root names the category");
    assert_eq!(entry.dirs.len(), 2);
    assert_eq!(entry.user_dir.as_deref(), Some(u.as_path()));
    assert_eq!(entry.lessons.len(), 2);
    assert!(library.lesson("light.mine").unwrap().editable);
    assert!(!library.lesson("light.source").unwrap().editable);
    assert_eq!(library.problems.len(), 1);
    assert!(library.problems[0].message.contains("уже есть"));
}

#[test]
fn requirements_unlock_lessons_and_missing_ones_are_reported() {
    let tmp = tempfile::tempdir().unwrap();
    let c = common::category(tmp.path(), "light", "Свет");
    common::lesson(&c, "01-a", "light.a", 1);
    common::lesson_with(&c, "02-b", "light.b", 2, "requires = [\"light.a\"]");
    common::lesson_with(&c, "03-c", "light.c", 3, "requires = [\"light.zzz\"]");
    let library = Library::scan(&[Root::builtin(tmp.path())]);
    assert_eq!(library.lessons().count(), 3);
    assert_eq!(library.problems.len(), 1);
    assert!(library.problems[0].message.contains("light.zzz"));
    let mut done = HashSet::new();
    assert!(library.is_unlocked("light.a", &done));
    assert!(!library.is_unlocked("light.b", &done));
    done.insert("light.a".to_string());
    assert!(library.is_unlocked("light.b", &done));
    assert!(!library.is_unlocked("light.nope", &done));
}

#[test]
fn hidden_folders_are_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    let c = common::category(tmp.path(), "light", "Свет");
    common::lesson(&c, ".draft", "light.draft", 1);
    common::category(tmp.path(), ".import-1", "Staging");
    let library = Library::scan(&[Root::builtin(tmp.path())]);
    assert_eq!(library.categories.len(), 1);
    assert_eq!(library.lessons().count(), 0);
}
