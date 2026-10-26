//! Errors. Messages are in Russian because the lessons app shows them to the student or
//! the lesson author as they are.

use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}: {source}", path.display())]
    Io { path: PathBuf, source: std::io::Error },
    #[error("{}: не удалось прочитать TOML: {message}", path.display())]
    Toml { path: PathBuf, message: String },
    #[error("{}: не удалось прочитать JSON: {message}", path.display())]
    Json { path: PathBuf, message: String },
    #[error("{}: не удалось прочитать картинку: {message}", path.display())]
    Image { path: PathBuf, message: String },
    #[error("не удалось записать TOML: {0}")]
    Serialize(String),
    #[error("{0}")]
    Invalid(String),
    #[error("пакет: {0}")]
    Package(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// `map_err` adapter that attaches the path to an I/O error.
pub(crate) fn io(path: &Path) -> impl FnOnce(std::io::Error) -> Error + '_ {
    move |source| Error::Io { path: path.to_path_buf(), source }
}
