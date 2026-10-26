#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::fs;

use fude_core::authoring;
use fude_core::guide;
use fude_core::library::{Library, Root};
use fude_core::model::{Accent, load_lesson};

#[test]
fn a_new_category_and_lesson_show_up_in_the_library() {
    let tmp = tempfile::tempdir().unwrap();
    let user = tmp.path().join("user");
    let (dir, category) = authoring::create_category(&user, "Свет и тень", Some("影"), Accent::Shu, "Мои уроки").unwrap();
    assert_eq!(category.id, "svet-i-ten");
    assert_eq!(dir, user.join("svet-i-ten"));
    let (lesson_dir, lesson) = authoring::create_lesson(&dir, "Падающая тень", 3).unwrap();
    assert_eq!(lesson.id, "svet-i-ten.padayushchaya-ten");
    assert_eq!(lesson_dir.file_name().unwrap(), "01-padayushchaya-ten");
    assert!(authoring::read_theory(&lesson_dir, &lesson).unwrap().starts_with("# Падающая тень"));

    let library = Library::scan(&[Root::user(&user)]);
    assert!(library.problems.is_empty(), "{:?}", library.problems);
    let entry = library.lesson(&lesson.id).unwrap();
    assert!(entry.editable);
    assert_eq!(entry.lesson.difficulty, 3);
}

#[test]
fn same_titles_get_numbered_ids() {
    let tmp = tempfile::tempdir().unwrap();
    let (a, _) = authoring::create_category(tmp.path(), "Акварель", None, Accent::Ai, "").unwrap();
    let (b, second) = authoring::create_category(tmp.path(), "Акварель", None, Accent::Ai, "").unwrap();
    assert_ne!(a, b);
    assert_eq!(second.id, "akvarel-2");
    let (_, one) = authoring::create_lesson(&a, "Заливка", 1).unwrap();
    let (dir_two, two) = authoring::create_lesson(&a, "Заливка", 1).unwrap();
    assert_eq!(one.id, "akvarel.zalivka");
    assert_eq!(two.id, "akvarel.zalivka-2");
    assert_eq!(two.order, 2);
    assert_eq!(dir_two.file_name().unwrap(), "02-zalivka-2");
    let (_, kanji_only) = authoring::create_lesson(&a, "影", 1).unwrap();
    assert_eq!(kanji_only.id, "akvarel.lesson");
}

#[test]
fn saving_reports_issues_but_keeps_the_draft() {
    let tmp = tempfile::tempdir().unwrap();
    let (dir, _) = authoring::create_category(tmp.path(), "Мои", None, Accent::Kin, "").unwrap();
    let (lesson_dir, mut lesson) = authoring::create_lesson(&dir, "Черновик", 2).unwrap();
    lesson.difficulty = 0;
    lesson.title = "Черновик 2".into();
    let issues = authoring::save_lesson(&lesson_dir, &lesson).unwrap();
    assert!(issues.iter().any(|i| i.is_error() && i.field == "difficulty"));
    assert_eq!(load_lesson(&lesson_dir).unwrap().title, "Черновик 2", "saved anyway");
    let library = Library::scan(&[Root::user(tmp.path())]);
    assert_eq!(library.lessons().count(), 0);
    assert_eq!(library.problems.len(), 1, "the draft is listed for the editor");
    assert!(!lesson_dir.join(".lesson.toml.tmp").exists());
}

#[test]
fn theory_guides_videos_and_pictures_are_stored_in_the_lesson() {
    let tmp = tempfile::tempdir().unwrap();
    let (dir, _) = authoring::create_category(tmp.path(), "Мои", None, Accent::Ai, "").unwrap();
    let (lesson_dir, mut lesson) = authoring::create_lesson(&dir, "Видео", 1).unwrap();

    authoring::save_theory(&lesson_dir, &lesson, "# Свет\n\nТекст.").unwrap();
    assert_eq!(authoring::read_theory(&lesson_dir, &lesson).unwrap(), "# Свет\n\nТекст.");

    let script = guide::parse(common::GUIDE, &lesson_dir).unwrap();
    let issues = authoring::save_guide(&lesson_dir, &mut lesson, &script).unwrap();
    assert!(!issues.iter().any(|i| i.is_error()), "{issues:?}");
    assert_eq!(lesson.guide.script.as_deref(), Some("guide.json"));

    let video = tmp.path().join("Мой ролик.MP4");
    fs::write(&video, b"video bytes").unwrap();
    let issues = authoring::attach_video(&lesson_dir, &mut lesson, &video).unwrap();
    assert!(!issues.iter().any(|i| i.is_error()), "{issues:?}");
    assert_eq!(lesson.guide.video.as_deref(), Some("media/moy-rolik.mp4"));
    authoring::attach_video(&lesson_dir, &mut lesson, &video).unwrap();
    assert_eq!(lesson.guide.video.as_deref(), Some("media/moy-rolik-2.mp4"), "no overwrite");
    assert_eq!(load_lesson(&lesson_dir).unwrap().guide.video, lesson.guide.video);

    let exe = tmp.path().join("setup.exe");
    fs::write(&exe, b"MZ").unwrap();
    assert!(authoring::attach_video(&lesson_dir, &mut lesson, &exe).is_err());

    let picture = tmp.path().join("task.png");
    common::png(&picture, [255, 0, 0, 255]);
    let rel = authoring::attach_image(&lesson_dir, &picture).unwrap();
    assert_eq!(rel, "assets/task.png");
    lesson.task.canvas = None;
    lesson.task.open = Some(rel);
    let issues = authoring::save_lesson(&lesson_dir, &lesson).unwrap();
    assert!(!issues.iter().any(|i| i.is_error()), "{issues:?}");
}

#[test]
fn a_builtin_lesson_is_edited_as_a_copy() {
    let tmp = tempfile::tempdir().unwrap();
    let builtin = tmp.path().join("builtin");
    let light = common::category(&builtin, "light", "Свет");
    let source = common::lesson_with(&light, "01-source", "light.source", 1, "");
    common::lesson_with(&light, "02-next", "light.next", 2, "requires = [\"light.source\"]");
    let user = tmp.path().join("user");
    let (mine, _) = authoring::create_category(&user, "Мои", None, Accent::Ai, "").unwrap();

    let (copy_dir, copy) = authoring::duplicate_lesson(&light.join("02-next"), &mine).unwrap();
    assert_eq!(copy.id, "moi.next-copy");
    assert!(copy.title.ends_with("(копия)"));
    assert!(copy.requires.is_empty());
    assert!(copy_dir.join("guide.json").is_file());
    assert!(source.join("lesson.toml").is_file(), "the original stays");

    let library = Library::scan(&[Root::builtin(&builtin), Root::user(&user)]);
    assert!(library.problems.is_empty(), "{:?}", library.problems);
    assert_eq!(library.lessons().count(), 3);
}

#[test]
fn removing_only_works_inside_the_user_root() {
    let tmp = tempfile::tempdir().unwrap();
    let builtin = tmp.path().join("builtin");
    let light = common::category(&builtin, "light", "Свет");
    let builtin_lesson = common::lesson(&light, "01-a", "light.a", 1);
    let user = tmp.path().join("user");
    let (mine, _) = authoring::create_category(&user, "Мои", None, Accent::Ai, "").unwrap();
    let (lesson_dir, _) = authoring::create_lesson(&mine, "Урок", 1).unwrap();

    assert!(authoring::remove(&user, &builtin_lesson).is_err());
    assert!(builtin_lesson.exists());
    assert!(authoring::remove(&user, &user).is_err(), "not the root itself");
    let stray = user.join("stray");
    fs::create_dir_all(&stray).unwrap();
    assert!(authoring::remove(&user, &stray).is_err(), "not arbitrary folders");
    authoring::remove(&user, &lesson_dir).unwrap();
    assert!(!lesson_dir.exists());
    authoring::remove(&user, &mine).unwrap();
    assert!(!mine.exists());
}
