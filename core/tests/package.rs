#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use fude_core::library::{Library, Root};
use fude_core::package::{self, ExportItem, ImportOptions, Limits, MANIFEST};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

struct World {
    _tmp: tempfile::TempDir,
    builtin: PathBuf,
    user: PathBuf,
    out: PathBuf,
}

fn world() -> World {
    let tmp = tempfile::tempdir().unwrap();
    let builtin = tmp.path().join("builtin");
    let user = tmp.path().join("user");
    let out = tmp.path().join("out");
    fs::create_dir_all(&builtin).unwrap();
    fs::create_dir_all(&out).unwrap();
    World { builtin, user, out, _tmp: tmp }
}

fn library(w: &World) -> Library {
    Library::scan(&[Root::builtin(&w.builtin), Root::user(&w.user)])
}

/// A category `water` with two lessons, one with a video and a picture, outside the library.
fn authored(w: &World) -> PathBuf {
    let src = w.out.join("src");
    let c = common::category(&src, "water", "Акварель");
    let a = common::lesson(&c, "01-wash", "water.wash", 1);
    fs::create_dir_all(a.join("media")).unwrap();
    fs::write(a.join("media").join("guide.mp4"), b"not really a video").unwrap();
    common::png(&a.join("assets").join("task.png"), [255, 255, 255, 255]);
    fs::write(a.join("notes.exe"), b"MZ").unwrap();
    fs::write(a.join(".DS_Store"), b"junk").unwrap();
    common::lesson(&c, "02-glaze", "water.glaze", 2);
    c
}

fn pack(w: &World, dir: &Path) -> PathBuf {
    let file = w.out.join("water.fude");
    let report = package::export(&file, "Акварель", &[ExportItem { category_dir: dir.to_path_buf(), lessons: vec![] }]).unwrap();
    assert_eq!(report.lessons, ["water.wash", "water.glaze"]);
    assert_eq!(report.skipped.len(), 1, "the .exe is left out: {:?}", report.skipped);
    file
}

#[test]
fn export_then_import_restores_the_lessons() {
    let w = world();
    let file = pack(&w, &authored(&w));
    let report = package::import(&file, &library(&w), &w.user, &ImportOptions::default(), Limits::default()).unwrap();
    assert_eq!(report.title, "Акварель");
    assert_eq!(report.categories_added, ["water"]);
    assert_eq!(report.lessons_added, ["water.wash", "water.glaze"]);
    assert!(report.lessons_replaced.is_empty());

    let lib = library(&w);
    assert!(lib.problems.is_empty(), "{:?}", lib.problems);
    let wash = lib.lesson("water.wash").unwrap();
    assert!(wash.editable);
    assert!(wash.dir.join("media").join("guide.mp4").is_file());
    assert!(wash.dir.join("assets").join("task.png").is_file());
    assert!(!wash.dir.join("notes.exe").exists());
    assert!(!wash.dir.join(".DS_Store").exists());
    let leftovers: Vec<_> = fs::read_dir(&w.user).unwrap().map(|e| e.unwrap().file_name()).collect();
    assert_eq!(leftovers, ["water"], "the staging folder is gone");
}

#[test]
fn inspect_reads_a_package_without_touching_the_library() {
    let w = world();
    let file = pack(&w, &authored(&w));
    let scratch = w.out.join("scratch");
    fs::create_dir_all(&scratch).unwrap();
    let info = package::inspect(&file, &scratch, Limits::default()).unwrap();
    assert_eq!(info.manifest.categories, ["water"]);
    assert!(info.manifest.created_by.unwrap().starts_with("FUDE "));
    assert_eq!(info.lessons.len(), 2);
    assert!(!info.issues.iter().any(|i| i.is_error()));
    assert!(!w.user.exists());
    assert_eq!(fs::read_dir(&scratch).unwrap().count(), 0);
}

#[test]
fn exporting_single_lessons_keeps_their_category() {
    let w = world();
    let c = authored(&w);
    let file = w.out.join("one.fude");
    let report = package::export(&file, "Один", &[ExportItem { category_dir: c.clone(), lessons: vec![c.join("02-glaze")] }]).unwrap();
    assert_eq!(report.lessons, ["water.glaze"]);
    package::import(&file, &library(&w), &w.user, &ImportOptions::default(), Limits::default()).unwrap();
    let lib = library(&w);
    assert_eq!(lib.lessons().count(), 1);
    assert_eq!(lib.category("water").unwrap().category.title, "Акварель");
}

#[test]
fn an_invalid_lesson_cannot_be_exported() {
    let w = world();
    let c = authored(&w);
    fs::remove_file(c.join("01-wash").join("theory.md")).unwrap();
    let err = package::export(&w.out.join("x.fude"), "x", &[ExportItem { category_dir: c, lessons: vec![] }]).unwrap_err().to_string();
    assert!(err.contains("theory.file"), "{err}");
}

#[test]
fn importing_twice_needs_replace_and_replace_swaps_the_lesson() {
    let w = world();
    let c = authored(&w);
    let file = pack(&w, &c);
    package::import(&file, &library(&w), &w.user, &ImportOptions::default(), Limits::default()).unwrap();
    let again = package::import(&file, &library(&w), &w.user, &ImportOptions::default(), Limits::default()).unwrap_err().to_string();
    assert!(again.contains("уже есть"), "{again}");

    // Change the source lesson's title, re-export and replace.
    let toml = c.join("01-wash").join("lesson.toml");
    fs::write(&toml, fs::read_to_string(&toml).unwrap().replace("Урок water.wash", "Заливка")).unwrap();
    let file = pack(&w, &c);
    let report = package::import(&file, &library(&w), &w.user, &ImportOptions { replace: true }, Limits::default()).unwrap();
    assert_eq!(report.lessons_replaced, ["water.wash", "water.glaze"]);
    assert!(report.categories_added.is_empty());
    let lib = library(&w);
    assert_eq!(lib.lessons().count(), 2, "{:?}", lib.problems);
    assert_eq!(lib.lesson("water.wash").unwrap().lesson.title, "Заливка");
}

#[test]
fn builtin_lessons_are_never_replaced() {
    let w = world();
    let c = authored(&w);
    let file = pack(&w, &c);
    let builtin = common::category(&w.builtin, "water", "Акварель");
    common::lesson(&builtin, "01-wash", "water.wash", 1);
    let err = package::import(&file, &library(&w), &w.user, &ImportOptions { replace: true }, Limits::default()).unwrap_err().to_string();
    assert!(err.contains("встроенных"), "{err}");
    assert!(!w.user.join("water").exists(), "nothing was written");
}

/// Writes a zip by hand: `(name, bytes)` entries.
fn raw_zip(path: &Path, entries: &[(&str, &[u8])]) {
    let mut zip = ZipWriter::new(File::create(path).unwrap());
    for (name, bytes) in entries {
        zip.start_file(*name, SimpleFileOptions::default()).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
}

const MANIFEST_TEXT: &[u8] = b"format = 1\ntitle = \"x\"\ncategories = [\"water\"]\n";
const CATEGORY_TEXT: &[u8] = b"id = \"water\"\ntitle = \"x\"\n";

fn refused(entries: &[(&str, &[u8])], limits: Limits) -> String {
    let w = world();
    let file = w.out.join("evil.fude");
    raw_zip(&file, entries);
    let err = package::import(&file, &library(&w), &w.user, &ImportOptions::default(), limits).unwrap_err().to_string();
    let written: Vec<_> = fs::read_dir(&w.user).map(|d| d.map(|e| e.unwrap().file_name()).collect()).unwrap_or_default();
    assert!(written.is_empty(), "nothing may be left behind: {written:?}");
    assert!(!w.out.parent().unwrap().join("evil.md").exists());
    err
}

#[test]
fn zip_slip_entries_are_refused() {
    for name in ["../evil.md", "water/../../evil.md", "/evil.md", "C:/evil.md", "water\\evil.md", "water//evil.md", "./water/x.md"] {
        let err = refused(&[(MANIFEST, MANIFEST_TEXT), (name, b"x")], Limits::default());
        assert!(err.contains("путь"), "{name}: {err}");
    }
}

#[test]
fn unexpected_file_types_and_duplicates_are_refused() {
    let err = refused(&[(MANIFEST, MANIFEST_TEXT), ("water/run.exe", b"MZ")], Limits::default());
    assert!(err.contains("тип файла"), "{err}");
    let err = refused(&[(MANIFEST, MANIFEST_TEXT), ("water/A.md", b"1"), ("water/a.md", b"2")], Limits::default());
    assert!(err.contains("дважды"), "{err}");
}

#[test]
fn links_are_refused() {
    let w = world();
    let file = w.out.join("link.fude");
    let mut zip = ZipWriter::new(File::create(&file).unwrap());
    zip.start_file(MANIFEST, SimpleFileOptions::default()).unwrap();
    zip.write_all(MANIFEST_TEXT).unwrap();
    zip.add_symlink("water/theory.md", "../../secret.md", SimpleFileOptions::default()).unwrap();
    zip.finish().unwrap();
    let err = package::import(&file, &library(&w), &w.user, &ImportOptions::default(), Limits::default()).unwrap_err().to_string();
    assert!(err.contains("ссылки"), "{err}");
}

#[test]
fn size_and_count_limits_hold() {
    let tight = Limits { max_files: 2, max_file_bytes: 1 << 20, max_total_bytes: 2 << 20 };
    let err = refused(&[(MANIFEST, MANIFEST_TEXT), ("water/category.toml", CATEGORY_TEXT), ("water/a.md", b"x")], tight);
    assert!(err.contains("файлов"), "{err}");
    let big = vec![b'x'; 4096];
    let small = Limits { max_files: 10, max_file_bytes: 1024, max_total_bytes: 1 << 20 };
    let err = refused(&[(MANIFEST, MANIFEST_TEXT), ("water/big.md", &big)], small);
    assert!(err.contains("МиБ"), "{err}");
    let total = Limits { max_files: 10, max_file_bytes: 4096, max_total_bytes: 5000 };
    let err = refused(&[(MANIFEST, MANIFEST_TEXT), ("water/a.md", &big), ("water/b.md", &big)], total);
    assert!(err.contains("пакет больше"), "{err}");
}

#[test]
fn broken_packages_are_refused_whole() {
    assert!(refused(&[("water/category.toml", CATEGORY_TEXT)], Limits::default()).contains(MANIFEST));
    let err = refused(&[(MANIFEST, MANIFEST_TEXT), ("water/category.toml", CATEGORY_TEXT), ("other/category.toml", CATEGORY_TEXT)], Limits::default());
    assert!(err.contains("«other»"), "{err}");
    let err = refused(
        &[
            (MANIFEST, MANIFEST_TEXT),
            ("water/category.toml", CATEGORY_TEXT),
            ("water/01/lesson.toml", b"format = 1\nid = \"water.a\"\ntitle = \"a\"\ndifficulty = 9\n[task]\nbrief = \"\"\n"),
        ],
        Limits::default(),
    );
    assert!(err.contains("difficulty"), "{err}");
    let w = world();
    let not_zip = w.out.join("plain.fude");
    fs::write(&not_zip, b"hello").unwrap();
    let err = package::import(&not_zip, &library(&w), &w.user, &ImportOptions::default(), Limits::default()).unwrap_err().to_string();
    assert!(err.contains("zip"), "{err}");
}

#[test]
fn imported_lessons_merge_into_an_existing_user_category() {
    let w = world();
    let mine = common::category(&w.user, "water", "Моя акварель");
    common::lesson(&mine, "01-wash", "water.mine", 1);
    let file = pack(&w, &authored(&w));
    let report = package::import(&file, &library(&w), &w.user, &ImportOptions::default(), Limits::default()).unwrap();
    assert!(report.categories_added.is_empty());
    let lib = library(&w);
    assert_eq!(lib.lessons().count(), 3, "{:?}", lib.problems);
    assert_eq!(lib.category("water").unwrap().category.title, "Моя акварель", "the user's category file is kept");
    assert!(mine.join("01-wash-2").join("lesson.toml").is_file(), "folder names do not collide");
}
