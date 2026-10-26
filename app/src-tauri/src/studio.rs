//! The lesson editor («Мастерская»): the student's own categories and lessons, attachments
//! and `.fude` packages. Only the user root is ever written.

use std::path::{Path, PathBuf};

use fude_core::authoring;
use fude_core::guide;
use fude_core::library::{self, subdirs};
use fude_core::model::{self, Accent, Category, Lesson};
use fude_core::package::{self, ExportItem, ImportOptions, ImportReport, Limits, PackageInfo};
use fude_core::validate::{self, Issue};
use serde::Serialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::learn::{now, refresh_achievements};
use crate::state::{AppState, lock, picture, user_dir};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftView {
    pub dir: String,
    pub id: String,
    pub title: String,
    pub difficulty: u8,
    pub errors: usize,
    pub warnings: usize,
    pub broken: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioCategory {
    pub dir: String,
    pub category: Category,
    pub lessons: Vec<DraftView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Studio {
    pub root: String,
    pub categories: Vec<StudioCategory>,
    /// Built-in lessons that can be copied for editing.
    pub builtin: Vec<(String, String)>,
}

/// Every user category and lesson, drafts with errors included.
#[tauri::command]
pub fn studio(state: State<'_, AppState>) -> AppResult<Studio> {
    std::fs::create_dir_all(&state.user)?;
    let mut categories = Vec::new();
    for dir in subdirs(&state.user) {
        let Ok(category) = model::load_category(&dir) else { continue };
        let lessons = subdirs(&dir)
            .into_iter()
            .filter(|d| d.join(model::LESSON_FILE).is_file())
            .map(|d| match model::load_lesson(&d) {
                Ok(l) => {
                    let issues = validate::lesson(&l, Some(&d), Some(&category.id));
                    DraftView {
                        dir: d.display().to_string(),
                        id: l.id.clone(),
                        title: l.title.clone(),
                        difficulty: l.difficulty,
                        errors: issues.iter().filter(|i| i.is_error()).count(),
                        warnings: issues.iter().filter(|i| !i.is_error()).count(),
                        broken: None,
                    }
                }
                Err(e) => DraftView {
                    dir: d.display().to_string(),
                    id: String::new(),
                    title: d.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
                    difficulty: 1,
                    errors: 1,
                    warnings: 0,
                    broken: Some(e.to_string()),
                },
            })
            .collect();
        categories.push(StudioCategory { dir: dir.display().to_string(), category, lessons });
    }
    let builtin = state.library().lessons().filter(|e| !e.editable).map(|e| (e.lesson.id.clone(), e.lesson.title.clone())).collect();
    Ok(Studio { root: state.user.display().to_string(), categories, builtin })
}

fn accent(name: &str) -> Accent {
    match name {
        "shu" => Accent::Shu,
        "kaki" => Accent::Kaki,
        "kin" => Accent::Kin,
        _ => Accent::Ai,
    }
}

#[tauri::command]
pub fn studio_category_create(state: State<'_, AppState>, title: String, kanji: Option<String>, accent_name: String, description: String) -> AppResult<String> {
    let (dir, _) = authoring::create_category(&state.user, &title, kanji.as_deref(), accent(&accent_name), &description)?;
    Ok(dir.display().to_string())
}

#[tauri::command]
pub fn studio_category_save(state: State<'_, AppState>, dir: String, category: Category) -> AppResult<Vec<Issue>> {
    let dir = user_dir(&state, &dir)?;
    let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
    if category.id != name {
        return Err(AppError::msg("id категории менять нельзя: от него зависят id её уроков"));
    }
    let issues = validate::category(&category, Some(&name));
    if !validate::has_errors(&issues) {
        authoring::save_category(&dir, &category)?;
    }
    Ok(issues)
}

#[tauri::command]
pub fn studio_lesson_create(state: State<'_, AppState>, category_dir: String, title: String, difficulty: u8) -> AppResult<String> {
    let dir = user_dir(&state, &category_dir)?;
    let (lesson_dir, lesson) = authoring::create_lesson(&dir, &title, difficulty)?;
    lock(&state.db).log(now(), "author", &format!("Создан урок «{}»", lesson.title), "Мастерская")?;
    refresh_achievements(&state)?;
    Ok(lesson_dir.display().to_string())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftDetail {
    pub dir: String,
    pub category: Category,
    pub lesson: Lesson,
    pub theory: String,
    /// The guide script as JSON text, if the lesson has one.
    pub guide: Option<String>,
    pub issues: Vec<Issue>,
    pub task_image: Option<String>,
    pub reference_image: Option<String>,
    pub video: Option<String>,
    /// Every lesson id in the library, for «требует урок».
    pub lesson_ids: Vec<(String, String)>,
}

#[tauri::command]
pub fn studio_lesson(state: State<'_, AppState>, dir: String) -> AppResult<DraftDetail> {
    let dir = user_dir(&state, &dir)?;
    let category_dir = dir.parent().map(Path::to_path_buf).ok_or_else(|| AppError::msg("урок вне категории"))?;
    let category = model::load_category(&category_dir)?;
    let lesson = model::load_lesson(&dir)?;
    let issues = validate::lesson(&lesson, Some(&dir), Some(&category.id));
    let theory = authoring::read_theory(&dir, &lesson).unwrap_or_default();
    let guide = lesson.guide.script.as_ref().and_then(|s| fude_core::paths::resolve(&dir, s).ok()).and_then(|p| std::fs::read_to_string(p).ok());
    Ok(DraftDetail {
        task_image: lesson.task.open.as_ref().and_then(|p| picture(&dir, p)),
        reference_image: lesson.task.reference.as_ref().and_then(|p| picture(&dir, p)),
        video: lesson.guide.video.as_ref().and_then(|v| fude_core::paths::resolve(&dir, v).ok()).filter(|p| p.is_file()).map(|p| p.display().to_string()),
        lesson_ids: state.library().lessons().map(|e| (e.lesson.id.clone(), e.lesson.title.clone())).collect(),
        dir: dir.display().to_string(),
        category,
        lesson,
        theory,
        guide,
        issues,
    })
}

/// Saves the lesson, its theory and (if given) its guide script; returns validation issues.
#[tauri::command]
pub fn studio_lesson_save(state: State<'_, AppState>, dir: String, lesson: Lesson, theory: String, guide: Option<String>) -> AppResult<Vec<Issue>> {
    let dir = user_dir(&state, &dir)?;
    let mut lesson = lesson;
    let mut issues = Vec::new();
    match guide.as_deref().map(str::trim) {
        Some(text) if !text.is_empty() => match guide::parse(text, &dir.join("guide.json")) {
            Ok(script) => {
                lesson.guide.script.get_or_insert_with(|| "guide.json".into());
                issues.extend(authoring::save_guide(&dir, &mut lesson, &script)?);
            }
            Err(e) => issues.push(Issue::error("guide.json", e.to_string())),
        },
        Some(_) => lesson.guide.script = None,
        None => {}
    }
    authoring::save_theory(&dir, &lesson, &theory)?;
    let saved = authoring::save_lesson(&dir, &lesson)?;
    issues.extend(saved);
    issues.dedup();
    Ok(issues)
}

/// Copies a picture or video into the lesson; returns its lesson path.
#[tauri::command]
pub fn studio_attach(state: State<'_, AppState>, dir: String, kind: String, source: String) -> AppResult<String> {
    let dir = user_dir(&state, &dir)?;
    let source = PathBuf::from(source);
    match kind.as_str() {
        "video" => Ok(authoring::attach(&dir, &source, "media", fude_core::paths::VIDEO_EXTENSIONS, authoring::MAX_VIDEO_BYTES)?),
        _ => Ok(authoring::attach_image(&dir, &source)?),
    }
}

#[tauri::command]
pub fn studio_picture(state: State<'_, AppState>, dir: String, path: String) -> AppResult<Option<String>> {
    let dir = user_dir(&state, &dir)?;
    Ok(picture(&dir, &path))
}

#[tauri::command]
pub fn studio_duplicate(state: State<'_, AppState>, lesson: String, category_dir: String) -> AppResult<String> {
    let library = state.library();
    let entry = library.lesson(&lesson).ok_or_else(|| AppError::msg("урок не найден"))?;
    let target = user_dir(&state, &category_dir)?;
    let (dir, _) = authoring::duplicate_lesson(&entry.dir, &target)?;
    Ok(dir.display().to_string())
}

#[tauri::command]
pub fn studio_remove(state: State<'_, AppState>, dir: String) -> AppResult<()> {
    let dir = user_dir(&state, &dir)?;
    authoring::remove(&state.user, &dir)?;
    Ok(())
}

// ------------------------------------------------------------------ packages

/// Exports a category (all lessons, or the chosen lesson folders) to a `.fude` file.
#[tauri::command]
pub fn package_export(state: State<'_, AppState>, category: String, lessons: Vec<String>, out: String) -> AppResult<usize> {
    let library = state.library();
    let entry = library.category(&category).ok_or_else(|| AppError::msg("категория не найдена"))?;
    let category_dir = entry.user_dir.clone().or_else(|| entry.dirs.first().cloned()).ok_or_else(|| AppError::msg("у категории нет папки"))?;
    let dirs: Vec<PathBuf> = entry.lessons.iter().filter(|l| lessons.is_empty() || lessons.contains(&l.lesson.id)).map(|l| l.dir.clone()).collect();
    // Lessons may sit in the built-in and the user folder of the same category: one item
    // per folder, each with its own lessons.
    let mut items: Vec<ExportItem> = Vec::new();
    for dir in dirs {
        let parent = dir.parent().map(Path::to_path_buf).unwrap_or_else(|| category_dir.clone());
        match items.iter_mut().find(|i| i.category_dir == parent) {
            Some(item) => item.lessons.push(dir),
            None => items.push(ExportItem { category_dir: parent, lessons: vec![dir] }),
        }
    }
    if items.len() > 1 {
        return Err(AppError::msg("уроки категории лежат в двух местах — экспортируйте встроенные и свои по отдельности"));
    }
    let report = package::export(Path::new(&out), &entry.category.title, &items)?;
    Ok(report.lessons.len())
}

#[tauri::command]
pub fn package_inspect(state: State<'_, AppState>, path: String) -> AppResult<PackageInfo> {
    std::fs::create_dir_all(&state.data)?;
    Ok(package::inspect(Path::new(&path), &state.data, Limits::default())?)
}

#[tauri::command]
pub fn package_import(state: State<'_, AppState>, path: String, replace: bool) -> AppResult<ImportReport> {
    let library = state.library();
    let report = package::import(Path::new(&path), &library, &state.user, &ImportOptions { replace }, Limits::default())?;
    let detail = format!("уроков: {}", report.lessons_added.len() + report.lessons_replaced.len());
    lock(&state.db).log(now(), "import", &format!("Импортирован пакет «{}»", report.title), &detail)?;
    Ok(report)
}

/// Problems of the library (broken lessons in either root), for the studio.
#[tauri::command]
pub fn library_problems(state: State<'_, AppState>) -> Vec<library::Problem> {
    state.library().problems
}
