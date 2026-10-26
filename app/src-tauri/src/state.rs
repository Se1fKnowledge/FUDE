//! Shared state: where lessons and data live, the progress database, the running editor
//! and the practice attempt in progress.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard};

use base64::Engine;
use fude_core::library::{Library, Root};
use fude_core::paths;

use crate::db::Db;
use crate::editor::{self, Editor};
use crate::error::{AppError, AppResult};
use crate::practice::Practice;
use crate::window::{self, Rect};
use fude_core::control::Call;
use serde_json::json;

pub struct AppState {
    /// Lessons that ship with the app (read-only).
    pub builtin: PathBuf,
    /// The student's own and imported lessons.
    pub user: PathBuf,
    /// Database, editor token, attempt snapshots.
    pub data: PathBuf,
    pub executable: Option<PathBuf>,
    pub db: Mutex<Db>,
    pub editor: Mutex<Option<Editor>>,
    pub practice: Mutex<Option<Practice>>,
    pub guide_cancel: Arc<AtomicBool>,
    /// Where the editor's window goes: inside the FUDE window or on its own.
    pub layout: Mutex<Layout>,
}

/// The editor window's place. Lock order: `editor` before `layout`.
#[derive(Debug)]
pub struct Layout {
    /// Show the editor inside the FUDE window (else as its own window).
    pub embedded: bool,
    /// FUDE's own window handle (0 where embedding is not supported).
    pub host: isize,
    /// The area the interface reserves for the editor, in physical pixels.
    pub rect: Option<Rect>,
    /// Whether the current screen shows the editor at all.
    pub visible: bool,
    /// The editor window while it is a child of FUDE's window, with what to restore.
    pub attached: Option<(isize, window::Saved)>,
}

impl Layout {
    pub fn new(host: isize) -> Self {
        Layout { embedded: window::EMBEDDING_SUPPORTED && host != 0, host, rect: None, visible: false, attached: None }
    }
}

/// Locks a mutex, recovering the data if a panic poisoned it.
pub fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl AppState {
    pub fn library(&self) -> Library {
        Library::scan(&[Root::builtin(&self.builtin), Root::user(&self.user)])
    }

    pub fn attempts_dir(&self) -> PathBuf {
        self.data.join("attempts")
    }

    /// An editor whose read root is `read_root`, reusing the running one when it fits.
    pub fn editor_for(&self, read_root: &Path) -> AppResult<MutexGuard<'_, Option<Editor>>> {
        let mut guard = lock(&self.editor);
        let reusable = guard.as_mut().is_some_and(|e| e.alive() && e.read_root == read_root);
        if !reusable {
            // Close the old one first: two editors would fight over the same files.
            *guard = None;
            let exe = self.executable.as_ref().ok_or_else(|| {
                AppError::msg(
                    "Редактор PhotoCraft не найден. Соберите его (cargo build --release -p photocraft) или укажите путь в переменной FUDE_PHOTOCRAFT.",
                )
            })?;
            let editor = Editor::launch(exe, &self.data.join("control.token"), read_root, &self.attempts_dir())?;
            lock(&self.layout).attached = None;
            *guard = Some(editor);
        }
        if let Some(editor) = guard.as_mut() {
            self.arrange(editor);
        }
        Ok(guard)
    }

    /// Puts the editor's window where the layout says: inside FUDE over the reserved area, or
    /// on its own. Called with the editor locked.
    pub fn arrange(&self, editor: &mut Editor) {
        let mut layout = lock(&self.layout);
        let wanted_inside = layout.embedded && layout.host != 0 && layout.rect.is_some_and(|r| r.is_usable());
        let current = layout.attached.map(|(w, _)| w).filter(|w| window::alive(*w));
        // A freshly started editor opens its window shortly after its channel: wait for it.
        let found = current.or_else(|| {
            (0..40).find_map(|_| {
                let w = window::main_window(editor.pid());
                if w.is_none() {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                w
            })
        });
        let Some(win) = found else { return };
        if wanted_inside {
            let Some(rect) = layout.rect else { return };
            if layout.attached.is_none() {
                let saved = window::embed(win, layout.host, rect);
                layout.attached = Some((win, saved));
                // No caption buttons or self-moving inside FUDE (photocraft/docs/control-protocol.md).
                let _ = editor.call(&Call::new("ui.set", json!({ "hostEmbedded": true })));
            } else {
                window::place(win, rect);
            }
            window::show(win, layout.visible);
        } else if let Some((w, saved)) = layout.attached.take() {
            window::detach(w, saved);
            let _ = editor.call(&Call::new("ui.set", json!({ "hostEmbedded": false })));
        }
    }

    pub fn close_editor(&self) {
        *lock(&self.editor) = None;
    }
}

/// The folder of a lesson the editor may touch: it must lie inside the user root.
pub fn user_dir(state: &AppState, dir: &str) -> AppResult<PathBuf> {
    let path = PathBuf::from(dir);
    fude_core::package::ensure_inside(&state.user, &path)?;
    Ok(path)
}

pub fn mime(path: &str) -> &'static str {
    match paths::extension(path).as_deref() {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    }
}

/// A picture of a lesson as a `data:` URL; `None` if it is missing or not a picture.
pub fn picture(dir: &Path, rel: &str) -> Option<String> {
    if !paths::has_extension(rel, paths::IMAGE_EXTENSIONS) {
        return None;
    }
    let path = paths::resolve(dir, rel).ok()?;
    let bytes = std::fs::read(path).ok()?;
    Some(data_url(mime(rel), &bytes))
}

pub fn data_url(mime: &str, bytes: &[u8]) -> String {
    format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes))
}

/// Finds PhotoCraft next to the running app or in the repository.
pub fn find_editor() -> Option<PathBuf> {
    let beside = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf));
    editor::find_executable(beside.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pictures_become_data_urls_and_paths_cannot_escape() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.png"), [1u8, 2, 3]).unwrap();
        assert_eq!(picture(dir.path(), "a.png").as_deref(), Some("data:image/png;base64,AQID"));
        assert_eq!(picture(dir.path(), "../a.png"), None);
        assert_eq!(picture(dir.path(), "missing.png"), None);
        assert_eq!(picture(dir.path(), "notes.md"), None);
        assert_eq!(mime("x.JPG"), "image/jpeg");
    }
}
