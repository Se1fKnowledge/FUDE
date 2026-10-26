//! `.fude` packages: zip archives for sharing lessons.
//!
//! ```text
//! fude-package.toml
//! <category id>/category.toml
//! <category id>/<lesson folder>/…
//! ```
//!
//! Import is fail-closed. Entries that could escape the target folder (zip-slip), links,
//! unknown file types and oversized archives are refused before anything is written; the
//! package is unpacked into a staging folder, every lesson is validated, and only then are
//! the folders moved into the user library.

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::error::{self, Error, Result};
use crate::library::{self, Library, subdirs};
use crate::model::{CATEGORY_FILE, Category, FORMAT, LESSON_FILE, Lesson, load_category};
use crate::paths::{self, ALLOWED_EXTENSIONS};
use crate::validate::{self, Issue, has_errors};

pub const MANIFEST: &str = "fude-package.toml";
pub const EXTENSION: &str = "fude";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: u32,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    pub categories: Vec<String>,
}

/// Size limits for an imported package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub max_files: usize,
    pub max_file_bytes: u64,
    pub max_total_bytes: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Limits { max_files: 4000, max_file_bytes: 1 << 30, max_total_bytes: 4 << 30 }
    }
}

/// One category to export: all its lessons, or only some lesson folders.
#[derive(Debug, Clone)]
pub struct ExportItem {
    pub category_dir: PathBuf,
    /// Lesson folders to include; empty means every lesson of the category folder.
    pub lessons: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ExportReport {
    pub files: usize,
    pub lessons: Vec<String>,
    /// Files left out because a lesson may not contain them.
    pub skipped: Vec<String>,
}

/// Writes a package with the given categories and lessons to `out`.
pub fn export(out: &Path, title: &str, items: &[ExportItem]) -> Result<ExportReport> {
    if items.is_empty() {
        return Err(Error::Package("нечего экспортировать".into()));
    }
    let mut report = ExportReport::default();
    let mut categories: Vec<(Category, PathBuf, Vec<PathBuf>)> = Vec::new();
    for item in items {
        let category = load_category(&item.category_dir)?;
        let lessons = if item.lessons.is_empty() {
            subdirs(&item.category_dir).into_iter().filter(|d| d.join(LESSON_FILE).is_file()).collect()
        } else {
            item.lessons.clone()
        };
        for dir in &lessons {
            let lesson = library::read_lesson(dir, &category.id).map_err(|(message, issues)| invalid(message, &issues))?.0;
            report.lessons.push(lesson.id);
        }
        categories.push((category, item.category_dir.clone(), lessons));
    }
    let mut ids = HashSet::new();
    if let Some((c, _, _)) = categories.iter().find(|(c, _, _)| !ids.insert(c.id.clone())) {
        return Err(Error::Package(format!("категория «{}» указана дважды", c.id)));
    }

    let file = File::create(out).map_err(error::io(out))?;
    let mut zip = ZipWriter::new(file);
    let manifest = Manifest {
        format: FORMAT,
        title: title.into(),
        created_by: Some(format!("FUDE {}", crate::VERSION)),
        categories: categories.iter().map(|(c, _, _)| c.id.clone()).collect(),
    };
    let text = toml::to_string_pretty(&manifest).map_err(|e| Error::Serialize(e.to_string()))?;
    add(&mut zip, MANIFEST, text.as_bytes(), out)?;
    report.files += 1;
    for (category, dir, lessons) in &categories {
        let path = dir.join(CATEGORY_FILE);
        let bytes = fs::read(&path).map_err(error::io(&path))?;
        add(&mut zip, &format!("{}/{CATEGORY_FILE}", category.id), &bytes, out)?;
        report.files += 1;
        for lesson_dir in lessons {
            let name =
                lesson_dir.file_name().and_then(|n| n.to_str()).ok_or_else(|| Error::Package(format!("{}: имя папки не в UTF-8", lesson_dir.display())))?;
            for file in walk(lesson_dir)? {
                let Some(rel) = paths::relative_to(lesson_dir, &file) else { continue };
                let entry = format!("{}/{name}/{rel}", category.id);
                if paths::check_relative(&entry).is_err() || !paths::has_extension(&rel, ALLOWED_EXTENSIONS) {
                    report.skipped.push(entry);
                    continue;
                }
                let bytes = fs::read(&file).map_err(error::io(&file))?;
                add(&mut zip, &entry, &bytes, out)?;
                report.files += 1;
            }
        }
    }
    zip.finish().map_err(|e| zip_error(out, e))?;
    Ok(report)
}

fn invalid(message: String, issues: &[Issue]) -> Error {
    let details: Vec<String> = issues.iter().filter(|i| i.is_error()).map(ToString::to_string).collect();
    if details.is_empty() { Error::Invalid(message) } else { Error::Invalid(format!("{message}:\n  {}", details.join("\n  "))) }
}

fn zip_error(path: &Path, e: zip::result::ZipError) -> Error {
    Error::Package(format!("{}: {e}", path.display()))
}

fn add(zip: &mut ZipWriter<File>, name: &str, bytes: &[u8], out: &Path) -> Result<()> {
    // Pictures and videos are already compressed.
    let method =
        if paths::has_extension(name, &["png", "jpg", "jpeg", "webp", "mp4", "webm"]) { CompressionMethod::Stored } else { CompressionMethod::Deflated };
    let options = SimpleFileOptions::default().compression_method(method).large_file(bytes.len() as u64 >= u64::from(u32::MAX));
    zip.start_file(name, options).map_err(|e| zip_error(out, e))?;
    zip.write_all(bytes).map_err(error::io(out))
}

/// Every file under `dir`, sorted, skipping hidden entries and links.
fn walk(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).map_err(error::io(&current))? {
            let entry = entry.map_err(error::io(&current))?;
            let path = entry.path();
            if entry.file_name().to_str().is_none_or(|n| n.starts_with('.')) {
                continue;
            }
            let kind = entry.file_type().map_err(error::io(&path))?;
            if kind.is_dir() {
                stack.push(path);
            } else if kind.is_file() {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// What a package holds, read without unpacking it to the library.
#[derive(Debug, Clone, Serialize)]
pub struct PackageInfo {
    pub manifest: Manifest,
    pub categories: Vec<Category>,
    pub lessons: Vec<Lesson>,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ImportOptions {
    /// Replace user lessons with the same id instead of refusing the import.
    pub replace: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ImportReport {
    pub title: String,
    pub categories_added: Vec<String>,
    pub lessons_added: Vec<String>,
    pub lessons_replaced: Vec<String>,
    pub warnings: Vec<String>,
}

/// Checks a package: unpacks it into a temporary folder next to `scratch` and validates it.
pub fn inspect(package: &Path, scratch: &Path, limits: Limits) -> Result<PackageInfo> {
    let staging = Staging::new(scratch)?;
    unpack(package, &staging.path, limits)?;
    let (manifest, staged) = read_staged(&staging.path)?;
    let mut info = PackageInfo { manifest, categories: vec![], lessons: vec![], issues: vec![] };
    for s in staged {
        info.issues.extend(s.issues);
        info.categories.push(s.category);
        info.lessons.extend(s.lessons.into_iter().map(|(l, _)| l));
    }
    Ok(info)
}

/// Imports a package into `user_root`, checking ids against the current `library`.
pub fn import(package: &Path, library: &Library, user_root: &Path, options: &ImportOptions, limits: Limits) -> Result<ImportReport> {
    fs::create_dir_all(user_root).map_err(error::io(user_root))?;
    let staging = Staging::new(user_root)?;
    unpack(package, &staging.path, limits)?;
    let (manifest, staged) = read_staged(&staging.path)?;
    let errors: Vec<&Issue> = staged.iter().flat_map(|s| &s.issues).filter(|i| i.is_error()).collect();
    if !errors.is_empty() {
        let list: Vec<String> = errors.iter().map(ToString::to_string).collect();
        return Err(Error::Package(format!("уроки в пакете не прошли проверку:\n  {}", list.join("\n  "))));
    }

    // Decide everything before moving anything.
    let mut report = ImportReport { title: manifest.title.clone(), ..ImportReport::default() };
    let mut replaced_dirs = Vec::new();
    for s in &staged {
        for (lesson, _) in &s.lessons {
            if let Some(existing) = library.lesson(&lesson.id) {
                if !existing.editable {
                    return Err(Error::Package(format!("урок «{}» уже есть среди встроенных, его нельзя заменить", lesson.id)));
                }
                if !options.replace {
                    return Err(Error::Package(format!("урок «{}» уже есть в библиотеке; выберите замену, чтобы обновить его", lesson.id)));
                }
                replaced_dirs.push(existing.dir.clone());
                report.lessons_replaced.push(lesson.id.clone());
            } else {
                report.lessons_added.push(lesson.id.clone());
            }
        }
    }
    report.warnings = staged.iter().flat_map(|s| &s.issues).map(ToString::to_string).collect();

    for dir in &replaced_dirs {
        ensure_inside(user_root, dir)?;
        fs::remove_dir_all(dir).map_err(error::io(dir))?;
    }
    for s in staged {
        let target = user_root.join(&s.category.id);
        if !target.join(CATEGORY_FILE).is_file() {
            fs::create_dir_all(&target).map_err(error::io(&target))?;
            let from = s.dir.join(CATEGORY_FILE);
            let to = target.join(CATEGORY_FILE);
            fs::copy(&from, &to).map_err(error::io(&to))?;
            report.categories_added.push(s.category.id.clone());
        }
        for (_, dir) in s.lessons {
            let name = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "lesson".into());
            let to = free_name(&target, &name);
            fs::rename(&dir, &to).map_err(error::io(&to))?;
        }
    }
    Ok(report)
}

/// A temporary folder that is removed when dropped, even on error.
struct Staging {
    path: PathBuf,
}

impl Staging {
    fn new(parent: &Path) -> Result<Self> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_nanos());
        let path = parent.join(format!(".import-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path).map_err(error::io(&path))?;
        Ok(Staging { path })
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct Staged {
    category: Category,
    dir: PathBuf,
    lessons: Vec<(Lesson, PathBuf)>,
    issues: Vec<Issue>,
}

fn read_staged(root: &Path) -> Result<(Manifest, Vec<Staged>)> {
    let path = root.join(MANIFEST);
    let text = fs::read_to_string(&path).map_err(|_| Error::Package(format!("нет файла {MANIFEST}")))?;
    let manifest: Manifest = toml::from_str(&text).map_err(|e| Error::Package(format!("{MANIFEST}: {}", e.message())))?;
    if manifest.format != FORMAT {
        return Err(Error::Package(format!("версия формата {} не поддерживается", manifest.format)));
    }
    if manifest.categories.is_empty() {
        return Err(Error::Package("в пакете нет категорий".into()));
    }
    let listed: HashSet<&str> = manifest.categories.iter().map(String::as_str).collect();
    for dir in subdirs(root) {
        let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if !listed.contains(name) {
            return Err(Error::Package(format!("папка «{name}» не указана в {MANIFEST}")));
        }
    }
    let mut staged = Vec::new();
    let mut ids = HashSet::new();
    for id in &manifest.categories {
        if !validate::is_slug(id) {
            return Err(Error::Package(format!("«{id}» — не id категории")));
        }
        let dir = root.join(id);
        let category = load_category(&dir).map_err(|e| Error::Package(e.to_string()))?;
        let mut issues = validate::category(&category, Some(id));
        let mut lessons = Vec::new();
        for lesson_dir in subdirs(&dir) {
            if !lesson_dir.join(LESSON_FILE).is_file() {
                issues.push(Issue::error(paths::relative_to(root, &lesson_dir).unwrap_or_default(), "папка без lesson.toml"));
                continue;
            }
            match library::read_lesson(&lesson_dir, id) {
                Ok((lesson, warnings)) => {
                    if !ids.insert(lesson.id.clone()) {
                        issues.push(Issue::error(&lesson.id, "урок с таким id в пакете дважды"));
                    }
                    issues.extend(prefixed(&lesson.id, warnings));
                    lessons.push((lesson, lesson_dir));
                }
                Err((message, errs)) => {
                    let rel = paths::relative_to(root, &lesson_dir).unwrap_or_default();
                    issues.push(Issue::error(&rel, message));
                    issues.extend(prefixed(&rel, errs));
                }
            }
        }
        if lessons.is_empty() && !has_errors(&issues) {
            issues.push(Issue::warning(id, "в категории нет уроков"));
        }
        staged.push(Staged { category, dir, lessons, issues });
    }
    Ok((manifest, staged))
}

fn prefixed(prefix: &str, issues: Vec<Issue>) -> impl Iterator<Item = Issue> + '_ {
    issues.into_iter().map(move |mut i| {
        i.field = if i.field.is_empty() { prefix.into() } else { format!("{prefix}: {}", i.field) };
        i
    })
}

/// Unpacks `package` into `target`, refusing anything unsafe before writing a byte.
pub fn unpack(package: &Path, target: &Path, limits: Limits) -> Result<()> {
    let file = File::open(package).map_err(error::io(package))?;
    let mut archive = ZipArchive::new(file).map_err(|e| Error::Package(format!("не zip-архив: {e}")))?;
    if archive.len() > limits.max_files {
        return Err(Error::Package(format!("больше {} файлов", limits.max_files)));
    }
    let mut total = 0u64;
    let mut names = HashSet::new();
    for i in 0..archive.len() {
        let entry = archive.by_index_raw(i).map_err(|e| zip_error(package, e))?;
        let name = entry.name().to_string();
        let trimmed = name.strip_suffix('/').unwrap_or(&name);
        paths::check_relative(trimmed).map_err(|e| Error::Package(format!("небезопасный путь: {e}")))?;
        if entry.enclosed_name().is_none() {
            return Err(Error::Package(format!("небезопасный путь «{name}»")));
        }
        if entry.unix_mode().is_some_and(|mode| mode & 0o170000 == 0o120000) {
            return Err(Error::Package(format!("«{name}»: ссылки в пакете запрещены")));
        }
        if entry.is_dir() {
            continue;
        }
        if !paths::has_extension(trimmed, ALLOWED_EXTENSIONS) {
            return Err(Error::Package(format!("«{name}»: такой тип файла в уроке запрещён")));
        }
        if !names.insert(trimmed.to_lowercase()) {
            return Err(Error::Package(format!("«{name}» в пакете дважды")));
        }
        if entry.size() > limits.max_file_bytes {
            return Err(Error::Package(format!("«{name}» больше {} МиБ", limits.max_file_bytes >> 20)));
        }
        total = total.saturating_add(entry.size());
        if total > limits.max_total_bytes {
            return Err(Error::Package(format!("пакет больше {} МиБ", limits.max_total_bytes >> 20)));
        }
    }
    let mut written = 0u64;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| zip_error(package, e))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        let out = paths::resolve(target, &name).map_err(Error::Package)?;
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).map_err(error::io(parent))?;
        }
        let mut file = File::create(&out).map_err(error::io(&out))?;
        // The declared size can lie: count the real bytes too.
        let budget = limits.max_file_bytes.min(limits.max_total_bytes.saturating_sub(written));
        let copied = io::copy(&mut (&mut entry).take(budget.saturating_add(1)), &mut file).map_err(error::io(&out))?;
        if copied > budget {
            return Err(Error::Package(format!("«{name}» больше, чем заявлено в архиве")));
        }
        written += copied;
    }
    Ok(())
}

/// `dir/name`, or `dir/name-2`, `dir/name-3`… if taken.
fn free_name(dir: &Path, name: &str) -> PathBuf {
    let first = dir.join(name);
    if !first.exists() {
        return first;
    }
    (2..).map(|n| dir.join(format!("{name}-{n}"))).find(|p| !p.exists()).unwrap_or(first)
}

/// Refuses to touch `path` unless it lies inside `root`.
pub fn ensure_inside(root: &Path, path: &Path) -> Result<()> {
    let root = fs::canonicalize(root).map_err(error::io(root))?;
    let path_c = fs::canonicalize(path).map_err(error::io(path))?;
    if path_c.starts_with(&root) && path_c != root {
        Ok(())
    } else {
        Err(Error::Invalid(format!("{}: вне папки пользовательских уроков", path.display())))
    }
}
