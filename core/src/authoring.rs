//! File operations of the lesson editor: create, save, copy and remove user categories and
//! lessons, attach videos and pictures.
//!
//! Only the user root is ever written. Saving is atomic (write a temporary file, then
//! rename), so a crash never leaves a half-written `lesson.toml`.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{self, Error, Result};
use crate::guide;
use crate::library::subdirs;
use crate::model::{
    Accent, CATEGORY_FILE, Canvas, Category, CheckKind, FORMAT, Guide, LESSON_FILE, Lesson, Range, Skill, Step, Task, Theory, category_to_toml, lesson_to_toml,
    load_category, load_lesson,
};
use crate::package::ensure_inside;
use crate::paths::{self, IMAGE_EXTENSIONS, VIDEO_EXTENSIONS};
use crate::validate::{self, Issue};

/// Largest video the editor copies into a lesson.
pub const MAX_VIDEO_BYTES: u64 = 1 << 30;
/// Largest picture the editor copies into a lesson.
pub const MAX_IMAGE_BYTES: u64 = 64 << 20;
const SLUG_MAX: usize = 40;

/// A folder-name-safe id from a (usually Russian) title: `Свет и тень` → `svet-i-ten`.
pub fn slugify(title: &str) -> String {
    let mut out = String::new();
    for ch in title.to_lowercase().chars() {
        let piece: &str = match ch {
            'а' => "a",
            'б' => "b",
            'в' => "v",
            'г' => "g",
            'д' => "d",
            'е' | 'э' => "e",
            'ё' => "yo",
            'ж' => "zh",
            'з' => "z",
            'и' => "i",
            'й' => "y",
            'к' => "k",
            'л' => "l",
            'м' => "m",
            'н' => "n",
            'о' => "o",
            'п' => "p",
            'р' => "r",
            'с' => "s",
            'т' => "t",
            'у' => "u",
            'ф' => "f",
            'х' => "kh",
            'ц' => "ts",
            'ч' => "ch",
            'ш' => "sh",
            'щ' => "shch",
            'ы' => "y",
            'ю' => "yu",
            'я' => "ya",
            'ъ' | 'ь' => "",
            c if c.is_ascii_alphanumeric() => {
                out.push(c);
                continue;
            }
            _ => "-",
        };
        out.push_str(piece);
    }
    let mut slug = String::new();
    for part in out.split('-').filter(|p| !p.is_empty()) {
        if slug.len() + part.len() + 1 > SLUG_MAX {
            break;
        }
        if !slug.is_empty() {
            slug.push('-');
        }
        slug.push_str(part);
    }
    if slug.is_empty() {
        // A title of only symbols, or one word longer than the limit.
        slug = out.replace('-', "").chars().take(SLUG_MAX).collect();
    }
    slug
}

/// `base`, or `base-2`, `base-3`… whichever `taken` does not hold.
fn unique(base: &str, taken: impl Fn(&str) -> bool) -> String {
    let base = if base.is_empty() { "lesson" } else { base };
    if !taken(base) {
        return base.into();
    }
    (2..).map(|n| format!("{base}-{n}")).find(|s| !taken(s)).unwrap_or_else(|| base.into())
}

/// Writes `bytes` to `path` through a temporary file in the same folder.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let tmp = path.with_file_name(format!(".{name}.tmp"));
    fs::write(&tmp, bytes).map_err(error::io(&tmp))?;
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        Error::Io { path: path.to_path_buf(), source: e }
    })
}

/// Creates a category folder in the user root.
pub fn create_category(user_root: &Path, title: &str, kanji: Option<&str>, accent: Accent, description: &str) -> Result<(PathBuf, Category)> {
    fs::create_dir_all(user_root).map_err(error::io(user_root))?;
    let id = unique(&slugify(title), |s| user_root.join(s).exists());
    let category = Category {
        id: id.clone(),
        title: title.trim().into(),
        kanji: kanji.map(str::trim).filter(|k| !k.is_empty()).map(Into::into),
        description: description.trim().into(),
        order: 100 + subdirs(user_root).len() as u32,
        accent,
        author: None,
    };
    let issues = validate::category(&category, Some(&id));
    if validate::has_errors(&issues) {
        return Err(Error::Invalid(issues.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")));
    }
    let dir = user_root.join(&id);
    fs::create_dir_all(&dir).map_err(error::io(&dir))?;
    save_category(&dir, &category)?;
    Ok((dir, category))
}

pub fn save_category(dir: &Path, category: &Category) -> Result<()> {
    write_atomic(&dir.join(CATEGORY_FILE), category_to_toml(category)?.as_bytes())
}

/// A starting lesson: a blank canvas and one step to edit.
pub fn template(id: &str, title: &str, difficulty: u8) -> Lesson {
    Lesson {
        format: FORMAT,
        id: id.into(),
        title: title.trim().into(),
        summary: String::new(),
        difficulty: difficulty.clamp(1, 5),
        minutes: 15,
        order: 0,
        skills: vec![Skill::Tools],
        requires: vec![],
        author: None,
        theory: Theory::default(),
        guide: Guide::default(),
        quiz: vec![],
        task: Task {
            brief: "Опишите, что ученик должен нарисовать.".into(),
            open: None,
            reference: None,
            canvas: Some(Canvas::default()),
            steps: vec![Step {
                id: "draw".into(),
                text: "Нарисуйте что-нибудь кистью.".into(),
                hint: None,
                checks: vec![
                    CheckKind::Tool { tool: "Brush".into() }.into(),
                    CheckKind::Coverage { region: [0.0, 0.0, 1.0, 1.0], changed: Range::at_least(0.01) }.into(),
                ],
            }],
        },
    }
}

/// Creates a lesson folder `NN-slug` in a user category with a template lesson and theory.
pub fn create_lesson(category_dir: &Path, title: &str, difficulty: u8) -> Result<(PathBuf, Lesson)> {
    let category = load_category(category_dir)?;
    let existing: Vec<Lesson> = subdirs(category_dir).iter().filter_map(|d| load_lesson(d).ok()).collect();
    let slug = unique(&slugify(title), |s| existing.iter().any(|l| l.slug() == s));
    let mut lesson = template(&format!("{}.{slug}", category.id), title, difficulty);
    lesson.order = existing.iter().map(|l| l.order).max().unwrap_or(0) + 1;
    let number = subdirs(category_dir).len() + 1;
    let dir = unique_dir(category_dir, &format!("{number:02}-{slug}"));
    fs::create_dir_all(&dir).map_err(error::io(&dir))?;
    write_atomic(&dir.join(&lesson.theory.file), format!("# {}\n\nНапишите здесь теорию урока.\n", lesson.title).as_bytes())?;
    save_lesson(&dir, &lesson)?;
    Ok((dir, lesson))
}

fn unique_dir(parent: &Path, name: &str) -> PathBuf {
    parent.join(unique(name, |s| parent.join(s).exists()))
}

/// Saves a lesson and returns what validation says about it. Drafts with errors are saved
/// too, so an author never loses work; the library lists them as problems until fixed.
pub fn save_lesson(dir: &Path, lesson: &Lesson) -> Result<Vec<Issue>> {
    write_atomic(&dir.join(LESSON_FILE), lesson_to_toml(lesson)?.as_bytes())?;
    let category = dir.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str());
    Ok(validate::lesson(lesson, Some(dir), category))
}

pub fn read_theory(dir: &Path, lesson: &Lesson) -> Result<String> {
    let path = paths::resolve(dir, &lesson.theory.file).map_err(Error::Invalid)?;
    fs::read_to_string(&path).map_err(error::io(&path))
}

pub fn save_theory(dir: &Path, lesson: &Lesson, text: &str) -> Result<()> {
    let path = paths::resolve(dir, &lesson.theory.file).map_err(Error::Invalid)?;
    write_atomic(&path, text.as_bytes())
}

pub fn save_guide(dir: &Path, lesson: &mut Lesson, script: &guide::Script) -> Result<Vec<Issue>> {
    let rel = lesson.guide.script.clone().unwrap_or_else(|| "guide.json".into());
    let path = paths::resolve(dir, &rel).map_err(Error::Invalid)?;
    write_atomic(&path, guide::to_json(script)?.as_bytes())?;
    lesson.guide.script = Some(rel);
    save_lesson(dir, lesson)
}

/// Copies a file into `<lesson>/<subdir>/` under a free name and returns its lesson path.
pub fn attach(dir: &Path, source: &Path, subdir: &str, allowed: &[&str], max_bytes: u64) -> Result<String> {
    let name = source.file_name().and_then(|n| n.to_str()).ok_or_else(|| Error::Invalid(format!("{}: нет имени файла", source.display())))?;
    let ext = paths::extension(name)
        .filter(|e| allowed.contains(&e.as_str()))
        .ok_or_else(|| Error::Invalid(format!("«{name}»: нужен файл {}", allowed.join(", "))))?;
    let size = fs::metadata(source).map_err(error::io(source))?.len();
    if size > max_bytes {
        return Err(Error::Invalid(format!("«{name}» больше {} МиБ", max_bytes >> 20)));
    }
    let stem = slugify(name.rsplit_once('.').map_or(name, |(s, _)| s));
    let folder = dir.join(subdir);
    fs::create_dir_all(&folder).map_err(error::io(&folder))?;
    let stem = unique(&stem, |s| folder.join(format!("{s}.{ext}")).exists());
    let file = format!("{stem}.{ext}");
    let target = folder.join(&file);
    fs::copy(source, &target).map_err(error::io(&target))?;
    Ok(format!("{subdir}/{file}"))
}

/// Copies an uploaded video into the lesson and makes it the lesson's video guide.
pub fn attach_video(dir: &Path, lesson: &mut Lesson, source: &Path) -> Result<Vec<Issue>> {
    let rel = attach(dir, source, "media", VIDEO_EXTENSIONS, MAX_VIDEO_BYTES)?;
    lesson.guide.video = Some(rel);
    save_lesson(dir, lesson)
}

/// Copies a picture into `assets/` and returns its lesson path (for `task.open`,
/// `task.reference` or `guide.poster`).
pub fn attach_image(dir: &Path, source: &Path) -> Result<String> {
    attach(dir, source, "assets", IMAGE_EXTENSIONS, MAX_IMAGE_BYTES)
}

/// Copies a lesson (usually a built-in one) into a user category, under a new id, so the
/// author can change it.
pub fn duplicate_lesson(source_dir: &Path, category_dir: &Path) -> Result<(PathBuf, Lesson)> {
    let mut lesson = load_lesson(source_dir)?;
    let category = load_category(category_dir)?;
    let existing: Vec<Lesson> = subdirs(category_dir).iter().filter_map(|d| load_lesson(d).ok()).collect();
    let slug = unique(&format!("{}-copy", lesson.slug()), |s| existing.iter().any(|l| l.slug() == s));
    lesson.id = format!("{}.{slug}", category.id);
    lesson.title = format!("{} (копия)", lesson.title);
    lesson.requires.clear();
    let dir = unique_dir(category_dir, &format!("{:02}-{slug}", subdirs(category_dir).len() + 1));
    copy_tree(source_dir, &dir)?;
    save_lesson(&dir, &lesson)?;
    Ok((dir, lesson))
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to).map_err(error::io(to))?;
    for entry in fs::read_dir(from).map_err(error::io(from))? {
        let entry = entry.map_err(error::io(from))?;
        let kind = entry.file_type().map_err(error::io(&entry.path()))?;
        let target = to.join(entry.file_name());
        if kind.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), &target).map_err(error::io(&target))?;
        }
    }
    Ok(())
}

/// Deletes a lesson or category folder, but only inside the user root.
pub fn remove(user_root: &Path, dir: &Path) -> Result<()> {
    ensure_inside(user_root, dir)?;
    if !(dir.join(LESSON_FILE).is_file() || dir.join(CATEGORY_FILE).is_file()) {
        return Err(Error::Invalid(format!("{}: это не папка урока или категории", dir.display())));
    }
    fs::remove_dir_all(dir).map_err(error::io(dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn russian_titles_become_latin_slugs() {
        assert_eq!(slugify("Свет и тень"), "svet-i-ten");
        assert_eq!(slugify("Падающая тень от сферы"), "padayushchaya-ten-ot-sfery");
        assert_eq!(slugify("Щука, ёж и Юла!"), "shchuka-yozh-i-yula");
        assert_eq!(slugify("  Layer 2 — opacity  "), "layer-2-opacity");
        assert_eq!(slugify("影"), "");
        assert!(slugify("Рефлекс и отражённый свет на сложной драпировке в мастерской").len() <= SLUG_MAX);
        assert_eq!(slugify(&"а".repeat(60)).len(), SLUG_MAX);
        for title in ["Свет и тень", "Основы интерфейса", "Холст и масштаб", "a--b"] {
            assert!(validate::is_slug(&slugify(title)), "{title}");
        }
    }

    #[test]
    fn unique_names_count_up() {
        let taken = ["ten", "ten-2"];
        assert_eq!(unique("ten", |s| taken.contains(&s)), "ten-3");
        assert_eq!(unique("svet", |s| taken.contains(&s)), "svet");
        assert_eq!(unique("", |_| false), "lesson");
    }

    #[test]
    fn the_template_lesson_is_valid() {
        let lesson = template("my.first", "Первый урок", 9);
        assert_eq!(lesson.difficulty, 5);
        let issues = validate::lesson(&lesson, None, Some("my"));
        assert!(!validate::has_errors(&issues), "{issues:?}");
    }
}
