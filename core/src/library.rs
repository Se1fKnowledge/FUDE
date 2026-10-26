//! The lesson library: categories and lessons found under one or more roots.
//!
//! The built-in root ships with the app and is read-only; the user root holds lessons made
//! in the lesson editor or imported from `.fude` packages. A broken lesson never breaks the
//! catalog: it is left out and listed in [`Library::problems`] for the lesson editor.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::model::{CATEGORY_FILE, Category, LESSON_FILE, Lesson, load_category, load_lesson};
use crate::validate::{self, Issue, has_errors};

/// A folder of categories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Root {
    pub path: PathBuf,
    /// Whether the lesson editor may change lessons here.
    pub editable: bool,
}

impl Root {
    pub fn builtin(path: impl Into<PathBuf>) -> Self {
        Root { path: path.into(), editable: false }
    }

    pub fn user(path: impl Into<PathBuf>) -> Self {
        Root { path: path.into(), editable: true }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LessonEntry {
    pub lesson: Lesson,
    pub dir: PathBuf,
    pub editable: bool,
    /// Warnings from validation (errors keep a lesson out of the library).
    pub warnings: Vec<Issue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryEntry {
    pub category: Category,
    /// Every folder of this category, one per root that has it.
    pub dirs: Vec<PathBuf>,
    /// The category folder in the editable root, if there is one.
    pub user_dir: Option<PathBuf>,
    pub lessons: Vec<LessonEntry>,
}

/// Something wrong with a file in the library.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Problem {
    pub path: PathBuf,
    pub message: String,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Library {
    pub categories: Vec<CategoryEntry>,
    pub problems: Vec<Problem>,
}

impl Library {
    /// Reads every root in order. A missing root is skipped (a fresh install has no user
    /// lessons yet); on duplicate lesson ids the first root wins.
    pub fn scan(roots: &[Root]) -> Library {
        let mut library = Library::default();
        let mut index: HashMap<String, usize> = HashMap::new();
        let mut seen: HashSet<String> = HashSet::new();
        for root in roots {
            for dir in subdirs(&root.path) {
                if !dir.join(CATEGORY_FILE).is_file() {
                    continue;
                }
                let dir_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
                let category = match load_category(&dir) {
                    Ok(c) => c,
                    Err(e) => {
                        library.problem(&dir.join(CATEGORY_FILE), e.to_string(), vec![]);
                        continue;
                    }
                };
                let issues = validate::category(&category, Some(&dir_name));
                if has_errors(&issues) {
                    library.problem(&dir.join(CATEGORY_FILE), "категория не прошла проверку".into(), issues);
                    continue;
                }
                let slot = *index.entry(category.id.clone()).or_insert_with(|| {
                    library.categories.push(CategoryEntry { category: category.clone(), dirs: vec![], user_dir: None, lessons: vec![] });
                    library.categories.len() - 1
                });
                let id = category.id.clone();
                let mut lessons = Vec::new();
                for lesson_dir in subdirs(&dir) {
                    if !lesson_dir.join(LESSON_FILE).is_file() {
                        continue;
                    }
                    match read_lesson(&lesson_dir, &id) {
                        Ok((lesson, warnings)) => {
                            if !seen.insert(lesson.id.clone()) {
                                library.problem(&lesson_dir.join(LESSON_FILE), format!("урок «{}» уже есть в библиотеке", lesson.id), vec![]);
                                continue;
                            }
                            lessons.push(LessonEntry { lesson, dir: lesson_dir, editable: root.editable, warnings });
                        }
                        Err((message, issues)) => library.problem(&lesson_dir.join(LESSON_FILE), message, issues),
                    }
                }
                if let Some(entry) = library.categories.get_mut(slot) {
                    entry.dirs.push(dir.clone());
                    if root.editable && entry.user_dir.is_none() {
                        entry.user_dir = Some(dir.clone());
                    }
                    entry.lessons.extend(lessons);
                }
            }
        }
        library.finish();
        library
    }

    fn problem(&mut self, path: &Path, message: String, issues: Vec<Issue>) {
        self.problems.push(Problem { path: path.to_path_buf(), message, issues });
    }

    /// Sorts the catalog and reports `requires` that point nowhere.
    fn finish(&mut self) {
        self.categories.sort_by(|a, b| (a.category.order, &a.category.title).cmp(&(b.category.order, &b.category.title)));
        for entry in &mut self.categories {
            entry.lessons.sort_by(|a, b| (a.lesson.order, &a.lesson.title).cmp(&(b.lesson.order, &b.lesson.title)));
        }
        let ids: HashSet<String> = self.lessons().map(|l| l.lesson.id.clone()).collect();
        let mut missing = Vec::new();
        for entry in self.lessons() {
            for required in &entry.lesson.requires {
                if !ids.contains(required) {
                    missing.push((entry.dir.join(LESSON_FILE), format!("урок «{}» требует «{required}», которого нет в библиотеке", entry.lesson.id)));
                }
            }
        }
        for (path, message) in missing {
            self.problem(&path, message, vec![]);
        }
    }

    pub fn lessons(&self) -> impl Iterator<Item = &LessonEntry> {
        self.categories.iter().flat_map(|c| &c.lessons)
    }

    pub fn lesson(&self, id: &str) -> Option<&LessonEntry> {
        self.lessons().find(|l| l.lesson.id == id)
    }

    pub fn category(&self, id: &str) -> Option<&CategoryEntry> {
        self.categories.iter().find(|c| c.category.id == id)
    }

    /// Whether a lesson is open: every lesson it requires is in `done`.
    pub fn is_unlocked(&self, id: &str, done: &HashSet<String>) -> bool {
        self.lesson(id).is_some_and(|l| l.lesson.requires.iter().all(|r| done.contains(r)))
    }
}

/// Loads and validates one lesson folder.
pub fn read_lesson(dir: &Path, category_id: &str) -> Result<(Lesson, Vec<Issue>), (String, Vec<Issue>)> {
    let lesson = load_lesson(dir).map_err(|e| (e.to_string(), vec![]))?;
    let issues = validate::lesson(&lesson, Some(dir), Some(category_id));
    if has_errors(&issues) {
        return Err((format!("урок «{}» не прошёл проверку", lesson.id), issues));
    }
    Ok((lesson, issues))
}

/// Sub-folders in name order; nothing when `dir` is missing or unreadable.
pub fn subdirs(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else { return vec![] };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.path())
        .filter(|p| !p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with('.')))
        .collect();
    dirs.sort();
    dirs
}
