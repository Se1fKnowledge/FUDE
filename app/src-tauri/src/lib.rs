//! FUDE desktop app: the Tauri shell around the lesson core and the PhotoCraft editor.
//!
//! The interface (SvelteKit, `app/src`) calls the commands registered here. Lessons come
//! from `fude-core`; progress lives in SQLite (`db`, `progress`); the editor runs as its
//! own process driven over its control channel (`editor`, `practice`, `play`).

pub mod author;
pub mod db;
pub mod editor;
pub mod error;
pub mod learn;
pub mod play;
pub mod practice;
pub mod progress;
pub mod state;
pub mod studio;
pub mod window;

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::state::AppState;

/// The built-in lessons: bundled resources, or the repository during development.
fn builtin_lessons(app: &tauri::App) -> PathBuf {
    let bundled = app.path().resource_dir().ok().map(|d| d.join("lessons"));
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("lessons");
    match bundled {
        Some(dir) if dir.join("basics").is_dir() && !cfg!(debug_assertions) => dir,
        Some(dir) if !repo.is_dir() && dir.is_dir() => dir,
        _ => repo,
    }
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // FUDE_DATA_DIR keeps tests and demos away from the student's own progress.
    let data = match std::env::var_os("FUDE_DATA_DIR") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => app.path().app_data_dir()?,
    };
    std::fs::create_dir_all(&data)?;
    let user = data.join("lessons");
    std::fs::create_dir_all(&user)?;
    let builtin = builtin_lessons(app);
    // Uploaded videos are played by the webview through the asset protocol.
    let scope = app.asset_protocol_scope();
    scope.allow_directory(&builtin, true)?;
    scope.allow_directory(&user, true)?;
    let db = db::Db::open(&data.join("fude.db"), learn::now())?;
    let host = main_window_handle(app);
    let handle = app.handle().clone();
    window::install_host(
        host,
        Box::new(move || {
            use tauri::Emitter;
            let _ = handle.emit("hint-hotkey", ());
        }),
    );
    app.manage(AppState {
        builtin,
        user,
        data,
        executable: state::find_editor(),
        db: Mutex::new(db),
        scratch_db: Mutex::new(db::Db::in_memory(learn::now())?),
        editor: Mutex::new(None),
        practice: Mutex::new(None),
        guide_cancel: Arc::new(AtomicBool::new(false)),
        recording: Mutex::new(None),
        layout: Mutex::new(state::Layout::new(host)),
    });
    Ok(())
}

/// The native handle of FUDE's main window (Windows), or 0.
fn main_window_handle(app: &tauri::App) -> isize {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let Some(window) = app.get_webview_window("main") else { return 0 };
    match window.window_handle().map(|h| h.as_raw()) {
        Ok(RawWindowHandle::Win32(h)) => h.hwnd.get(),
        _ => 0,
    }
}

pub fn run() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(setup)
        .on_window_event(|window, event| {
            // Closing FUDE closes the editor it started.
            if let tauri::WindowEvent::Destroyed = event {
                window.state::<AppState>().close_editor();
            }
        })
        .invoke_handler(tauri::generate_handler![
            learn::boot,
            learn::profile_save,
            learn::catalog,
            learn::lesson,
            learn::lesson_stage,
            learn::home,
            learn::achievements,
            learn::activity,
            learn::stats,
            learn::result,
            play::editor_status,
            play::editor_close,
            play::editor_layout,
            play::editor_area,
            play::guide_play,
            play::guide_stop,
            play::practice_start,
            play::practice_preview,
            play::practice_check,
            play::practice_hint,
            play::practice_hide_hint,
            play::practice_confirm,
            play::practice_stop,
            play::practice_hint_key,
            studio::studio,
            studio::studio_category_create,
            studio::studio_category_save,
            studio::studio_lesson_create,
            studio::studio_lesson,
            studio::studio_lesson_save,
            studio::studio_attach,
            studio::studio_picture,
            studio::studio_duplicate,
            studio::studio_remove,
            studio::package_export,
            studio::package_inspect,
            studio::package_import,
            studio::library_problems,
            author::studio_prove,
            author::studio_hint_preview,
            author::guide_record_start,
            author::guide_record_stop,
            author::guide_play_draft,
        ])
        .run(tauri::generate_context!());
    if let Err(e) = result {
        eprintln!("FUDE: {e}");
        std::process::exit(1);
    }
}
