//! FUDE desktop app: the Tauri shell around the lesson core and the PhotoCraft editor.
//!
//! The interface (SvelteKit, `app/src`) calls the commands registered here. Lessons come
//! from `fude-core`; progress lives in SQLite (`db`, `progress`); the editor runs as its
//! own process driven over its control channel (`editor`, `practice`, `play`).

pub mod db;
pub mod editor;
pub mod error;
pub mod learn;
pub mod play;
pub mod practice;
pub mod progress;
pub mod state;
pub mod studio;

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
    let data = app.path().app_data_dir()?;
    std::fs::create_dir_all(&data)?;
    let user = data.join("lessons");
    std::fs::create_dir_all(&user)?;
    let builtin = builtin_lessons(app);
    // Uploaded videos are played by the webview through the asset protocol.
    let scope = app.asset_protocol_scope();
    scope.allow_directory(&builtin, true)?;
    scope.allow_directory(&user, true)?;
    let db = db::Db::open(&data.join("fude.db"), learn::now())?;
    app.manage(AppState {
        builtin,
        user,
        data,
        executable: state::find_editor(),
        db: Mutex::new(db),
        editor: Mutex::new(None),
        practice: Mutex::new(None),
        guide_cancel: Arc::new(AtomicBool::new(false)),
    });
    Ok(())
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
            play::guide_play,
            play::guide_stop,
            play::practice_start,
            play::practice_check,
            play::practice_hint,
            play::practice_hide_hint,
            play::practice_confirm,
            play::practice_stop,
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
        ])
        .run(tauri::generate_context!());
    if let Err(e) = result {
        eprintln!("FUDE: {e}");
        std::process::exit(1);
    }
}
