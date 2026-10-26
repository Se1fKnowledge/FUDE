//! Relative paths inside a lesson folder or package.
//!
//! Lessons come from other people (imported `.fude` files), so every path a lesson names is
//! checked here before it touches the file system: relative, `/`-separated, no `..`, no
//! drive letters, and only file types a lesson needs.

use std::path::{Path, PathBuf};

/// File types a lesson folder may contain.
pub const ALLOWED_EXTENSIONS: &[&str] = &["md", "json", "toml", "txt", "png", "jpg", "jpeg", "webp", "mp4", "webm"];
/// Pictures: task images, references, posters.
pub const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp"];
/// Uploaded guide videos.
pub const VIDEO_EXTENSIONS: &[&str] = &["mp4", "webm"];

const MAX_LEN: usize = 255;

/// Checks that `path` is a safe relative path; the error explains why it is not.
pub fn check_relative(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("пустой путь".into());
    }
    if path.len() > MAX_LEN {
        return Err(format!("путь длиннее {MAX_LEN} символов"));
    }
    if path.contains('\\') {
        return Err(format!("«{path}»: разделяйте папки знаком «/»"));
    }
    if path.starts_with('/') {
        return Err(format!("«{path}»: путь должен быть относительным"));
    }
    if path.contains(':') {
        return Err(format!("«{path}»: двоеточие в пути запрещено"));
    }
    if path.chars().any(char::is_control) {
        return Err(format!("«{path}»: управляющие символы в пути запрещены"));
    }
    for part in path.split('/') {
        match part {
            "" => return Err(format!("«{path}»: пустая часть пути")),
            "." | ".." => return Err(format!("«{path}»: «.» и «..» в пути запрещены")),
            _ => {}
        }
    }
    Ok(())
}

/// Lower-case extension of `path`, if any.
pub fn extension(path: &str) -> Option<String> {
    let name = path.rsplit('/').next().unwrap_or(path);
    let (stem, ext) = name.rsplit_once('.')?;
    (!stem.is_empty() && !ext.is_empty()).then(|| ext.to_ascii_lowercase())
}

/// Whether `path` ends with one of `allowed` (case-insensitive).
pub fn has_extension(path: &str, allowed: &[&str]) -> bool {
    extension(path).is_some_and(|ext| allowed.contains(&ext.as_str()))
}

/// Joins a checked relative path onto `dir`.
pub fn resolve(dir: &Path, path: &str) -> Result<PathBuf, String> {
    check_relative(path)?;
    Ok(path.split('/').fold(dir.to_path_buf(), |acc, part| acc.join(part)))
}

/// `path` relative to `base`, `/`-separated, if it lies inside `base`.
pub fn relative_to(base: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(base).ok()?;
    let parts: Option<Vec<&str>> = rel.components().map(|c| c.as_os_str().to_str()).collect();
    let joined = parts?.join("/");
    (!joined.is_empty()).then_some(joined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_relative_paths_are_accepted() {
        for ok in ["theory.md", "assets/task.png", "media/guide video.mp4", "a/b/c/d.json", "урок/теория.md"] {
            assert_eq!(check_relative(ok), Ok(()), "{ok}");
        }
    }

    #[test]
    fn escaping_and_absolute_paths_are_refused() {
        for bad in [
            "",
            "/etc/passwd",
            "../secret.png",
            "assets/../../x.png",
            "./theory.md",
            "C:/Windows/win.ini",
            "C:evil.png",
            "assets\\task.png",
            "assets//task.png",
            "assets/",
            "file.md:stream",
            "a\u{0}b.md",
        ] {
            assert!(check_relative(bad).is_err(), "{bad:?} must be refused");
        }
        assert!(check_relative(&"a".repeat(MAX_LEN + 1)).is_err());
    }

    #[test]
    fn extensions_are_lower_cased_and_need_a_stem() {
        assert_eq!(extension("media/Guide.MP4").as_deref(), Some("mp4"));
        assert_eq!(extension("theory.md").as_deref(), Some("md"));
        assert_eq!(extension(".hidden"), None);
        assert_eq!(extension("noext"), None);
        assert_eq!(extension("dir.d/noext"), None);
        assert!(has_extension("a/B.PNG", IMAGE_EXTENSIONS));
        assert!(!has_extension("a/b.exe", ALLOWED_EXTENSIONS));
        assert!(has_extension("v.webm", VIDEO_EXTENSIONS));
    }

    #[test]
    fn resolve_joins_inside_the_folder() {
        let dir = Path::new("lesson");
        assert_eq!(resolve(dir, "assets/task.png"), Ok(Path::new("lesson").join("assets").join("task.png")));
        assert!(resolve(dir, "../x").is_err());
    }

    #[test]
    fn relative_to_uses_forward_slashes() {
        let base = Path::new("root");
        assert_eq!(relative_to(base, &base.join("a").join("b.png")).as_deref(), Some("a/b.png"));
        assert_eq!(relative_to(base, base), None);
        assert_eq!(relative_to(base, Path::new("other/x")), None);
    }
}
