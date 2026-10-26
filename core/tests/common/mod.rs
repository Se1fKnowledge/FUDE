//! Lesson folders for the integration tests.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

pub const GUIDE: &str = r##"{
  "format": 1,
  "canvas": {"width": 400, "height": 300, "background": "#FFFFFF"},
  "steps": [
    {"caption": "Берём кисть", "highlight": "tool:Brush",
     "calls": [{"method": "ui.set", "params": {"tool": "Brush"}}]}
  ]
}"##;

pub fn category(root: &Path, id: &str, title: &str) -> PathBuf {
    let dir = root.join(id);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("category.toml"), format!("id = \"{id}\"\ntitle = \"{title}\"\naccent = \"shu\"\n")).unwrap();
    dir
}

/// A valid lesson `<category>.<slug>` in `<category dir>/<folder>`.
pub fn lesson(category_dir: &Path, folder: &str, id: &str, order: u32) -> PathBuf {
    lesson_with(category_dir, folder, id, order, "")
}

/// Same, with extra top-level TOML lines (before the tables).
pub fn lesson_with(category_dir: &Path, folder: &str, id: &str, order: u32, extra: &str) -> PathBuf {
    let dir = category_dir.join(folder);
    fs::create_dir_all(dir.join("assets")).unwrap();
    fs::write(dir.join("theory.md"), "# Теория\n").unwrap();
    fs::write(dir.join("guide.json"), GUIDE).unwrap();
    let text = format!(
        r##"format = 1
id = "{id}"
title = "Урок {id}"
difficulty = 2
order = {order}
skills = ["tools"]
{extra}

[guide]
script = "guide.json"

[task]
brief = "Нарисуйте линию."

[task.canvas]
width = 400
height = 300

[[task.steps]]
id = "tool"
text = "Возьмите кисть"
  [[task.steps.checks]]
  kind = "tool"
  tool = "Brush"
"##
    );
    fs::write(dir.join("lesson.toml"), text).unwrap();
    dir
}

/// A tiny PNG.
pub fn png(path: &Path, rgba: [u8; 4]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    image::RgbaImage::from_pixel(8, 8, image::Rgba(rgba)).save(path).unwrap();
}
