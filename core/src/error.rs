//! Errors. Messages are in the current language ([`crate::i18n`]) because the lessons app
//! shows them to the student or the lesson author as they are.

use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
    Io { path: PathBuf, source: std::io::Error },
    Toml { path: PathBuf, message: String },
    Json { path: PathBuf, message: String },
    Image { path: PathBuf, message: String },
    Serialize(String),
    Invalid(String),
    Package(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Error::Toml { path, message } => {
                write!(f, "{}", crate::tr!("{}: не удалось прочитать TOML: {message}", "{}: could not read TOML: {message}", path.display()))
            }
            Error::Json { path, message } => {
                write!(f, "{}", crate::tr!("{}: не удалось прочитать JSON: {message}", "{}: could not read JSON: {message}", path.display()))
            }
            Error::Image { path, message } => {
                write!(f, "{}", crate::tr!("{}: не удалось прочитать картинку: {message}", "{}: could not read the picture: {message}", path.display()))
            }
            Error::Serialize(e) => write!(f, "{}", crate::tr!("не удалось записать TOML: {e}", "could not write TOML: {e}")),
            Error::Invalid(e) => write!(f, "{e}"),
            Error::Package(e) => write!(f, "{}", crate::tr!("пакет: {e}", "package: {e}")),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// `map_err` adapter that attaches the path to an I/O error.
pub(crate) fn io(path: &Path) -> impl FnOnce(std::io::Error) -> Error + '_ {
    move |source| Error::Io { path: path.to_path_buf(), source }
}
