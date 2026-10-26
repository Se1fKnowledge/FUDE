//! The lesson format: `category.toml` and `lesson.toml` (see `docs/lesson-format.md`).

use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{self, Error, Result};

/// Lesson format version this core reads and writes.
pub const FORMAT: u32 = 1;
pub const CATEGORY_FILE: &str = "category.toml";
pub const LESSON_FILE: &str = "lesson.toml";

/// Category accent colour: a FUDE design token family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Accent {
    /// 藍 indigo.
    #[default]
    Ai,
    /// 朱 vermilion.
    Shu,
    /// 柿 persimmon.
    Kaki,
    /// 金 gold.
    Kin,
}

impl Accent {
    pub const ALL: [Accent; 4] = [Accent::Ai, Accent::Shu, Accent::Kaki, Accent::Kin];

    /// The `500` shade of the token family.
    pub fn hex(self) -> &'static str {
        match self {
            Accent::Ai => "#2F5D8C",
            Accent::Shu => "#BC3823",
            Accent::Kaki => "#E8893B",
            Accent::Kin => "#C29A4E",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Category {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kanji: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub order: u32,
    #[serde(default)]
    pub accent: Accent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

/// A side of the skill tracked on the statistics screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Skill {
    Tools,
    Line,
    Form,
    Light,
    Color,
    Composition,
}

impl Skill {
    pub const ALL: [Skill; 6] = [Skill::Tools, Skill::Line, Skill::Form, Skill::Light, Skill::Color, Skill::Composition];

    pub fn label(self) -> &'static str {
        match self {
            Skill::Tools => "Инструменты",
            Skill::Line => "Линия",
            Skill::Form => "Форма",
            Skill::Light => "Свет и тень",
            Skill::Color => "Цвет",
            Skill::Composition => "Композиция",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub format: u32,
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub summary: String,
    pub difficulty: u8,
    #[serde(default)]
    pub minutes: u16,
    #[serde(default)]
    pub order: u32,
    #[serde(default)]
    pub skills: Vec<Skill>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default)]
    pub theory: Theory,
    #[serde(default)]
    pub guide: Guide,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quiz: Vec<Question>,
    pub task: Task,
}

impl Lesson {
    /// The part of the id after the category: `light.sphere-shadow` → `sphere-shadow`.
    pub fn slug(&self) -> &str {
        self.id.split_once('.').map_or(self.id.as_str(), |(_, slug)| slug)
    }

    /// The category part of the id.
    pub fn category_id(&self) -> &str {
        self.id.split_once('.').map_or("", |(category, _)| category)
    }

    /// Russian word for the difficulty shown on lesson cards.
    pub fn difficulty_label(&self) -> &'static str {
        match self.difficulty {
            0 | 1 => "Первый шаг",
            2 => "Лёгкий",
            3 => "Средний",
            4 => "Сложный",
            _ => "Мастер",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Theory {
    #[serde(default = "default_theory")]
    pub file: String,
}

impl Default for Theory {
    fn default() -> Self {
        Theory { file: default_theory() }
    }
}

fn default_theory() -> String {
    "theory.md".into()
}

/// The video guide: a PhotoCraft command script, an uploaded video, or both.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Guide {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poster: Option<String>,
}

impl Guide {
    pub fn is_empty(&self) -> bool {
        self.script.is_none() && self.video.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    pub question: String,
    pub options: Vec<String>,
    pub answer: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explain: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub brief: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canvas: Option<Canvas>,
    #[serde(default)]
    pub steps: Vec<Step>,
}

/// A blank document to start from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    #[serde(default = "default_background")]
    pub background: String,
}

fn default_background() -> String {
    "#FFFFFF".into()
}

impl Default for Canvas {
    fn default() -> Self {
        Canvas { width: 1200, height: 800, background: default_background() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub id: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<Hint>,
    #[serde(default)]
    pub checks: Vec<Check>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hint {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shapes: Vec<Shape>,
}

/// `[x, y, width, height]` as fractions (0–1) of the canvas.
pub type Region = [f64; 4];
/// `[x, y]` as fractions (0–1) of the canvas.
pub type Point = [f64; 2];

/// A ghost shape drawn on the canvas by a hint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Shape {
    Ellipse { rect: Region },
    Rect { rect: Region },
    Line { from: Point, to: Point },
    Path { points: Vec<Point> },
}

/// An inclusive range; either end may be open.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Range {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

impl Range {
    pub fn at_least(min: f64) -> Self {
        Range { min: Some(min), max: None }
    }

    pub fn at_most(max: f64) -> Self {
        Range { min: None, max: Some(max) }
    }

    pub fn between(min: f64, max: f64) -> Self {
        Range { min: Some(min), max: Some(max) }
    }

    pub fn contains(&self, value: f64) -> bool {
        self.min.is_none_or(|min| value >= min) && self.max.is_none_or(|max| value <= max)
    }

    /// The same range with both ends multiplied (for showing fractions as percent).
    pub fn scaled(&self, factor: f64) -> Self {
        Range { min: self.min.map(|v| v * factor), max: self.max.map(|v| v * factor) }
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.min, self.max) {
            (Some(a), Some(b)) if a == b => write!(f, "ровно {}", num(a)),
            (Some(a), Some(b)) => write!(f, "от {} до {}", num(a), num(b)),
            (Some(a), None) => write!(f, "не меньше {}", num(a)),
            (None, Some(b)) => write!(f, "не больше {}", num(b)),
            (None, None) => write!(f, "любое"),
        }
    }
}

/// A number rounded to hundredths, without a trailing `,0`, with a Russian decimal comma.
pub fn num(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    let text = if rounded.fract() == 0.0 { format!("{rounded:.0}") } else { format!("{rounded}") };
    text.replace('.', ",")
}

/// One assessment check of a task step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Check {
    #[serde(flatten)]
    pub kind: CheckKind,
    /// Shown to the student when the check fails, instead of the generated message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl From<CheckKind> for Check {
    fn from(kind: CheckKind) -> Self {
        Check { kind, message: None }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CheckKind {
    Tool {
        tool: String,
    },
    Brush {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        size: Option<Range>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hardness: Option<Range>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        opacity: Option<Range>,
    },
    Zoom {
        zoom: Range,
    },
    Layers {
        count: Range,
    },
    Layer {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        visible: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        opacity: Option<Range>,
        /// PhotoCraft layer kind: `Pixel`, `Group`, `Type`, `Shape`, …
        #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
        layer_type: Option<String>,
    },
    Selection {
        #[serde(default = "yes")]
        present: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        width: Option<Range>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        height: Option<Range>,
    },
    History {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entries: Option<Range>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        can_redo: Option<bool>,
    },
    Foreground {
        color: String,
        #[serde(default = "default_foreground_tolerance")]
        tolerance: f64,
    },
    Coverage {
        region: Region,
        changed: Range,
    },
    Luminance {
        region: Region,
        value: Range,
    },
    Darker {
        region: Region,
        than: Region,
        #[serde(default = "default_darker_by")]
        by: f64,
    },
    Color {
        region: Region,
        color: String,
        #[serde(default = "default_color_tolerance")]
        tolerance: f64,
    },
    Similar {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reference: Option<String>,
        score: Range,
    },
}

fn yes() -> bool {
    true
}

fn default_foreground_tolerance() -> f64 {
    0.1
}

fn default_darker_by() -> f64 {
    0.05
}

fn default_color_tolerance() -> f64 {
    0.12
}

impl CheckKind {
    /// Whether the check reads the canvas snapshot rather than the editor state.
    pub fn needs_canvas(&self) -> bool {
        matches!(
            self,
            CheckKind::Coverage { .. } | CheckKind::Luminance { .. } | CheckKind::Darker { .. } | CheckKind::Color { .. } | CheckKind::Similar { .. }
        )
    }

    pub fn name(&self) -> &'static str {
        match self {
            CheckKind::Tool { .. } => "tool",
            CheckKind::Brush { .. } => "brush",
            CheckKind::Zoom { .. } => "zoom",
            CheckKind::Layers { .. } => "layers",
            CheckKind::Layer { .. } => "layer",
            CheckKind::Selection { .. } => "selection",
            CheckKind::History { .. } => "history",
            CheckKind::Foreground { .. } => "foreground",
            CheckKind::Coverage { .. } => "coverage",
            CheckKind::Luminance { .. } => "luminance",
            CheckKind::Darker { .. } => "darker",
            CheckKind::Color { .. } => "color",
            CheckKind::Similar { .. } => "similar",
        }
    }
}

/// Parses `#rrggbb` (or `rrggbb`) into 0–1 RGB.
pub fn parse_hex(color: &str) -> Option<[f64; 3]> {
    let hex = color.strip_prefix('#').unwrap_or(color);
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok().map(|v| f64::from(v) / 255.0);
    Some([channel(0)?, channel(2)?, channel(4)?])
}

pub fn parse_category(text: &str, path: &Path) -> Result<Category> {
    toml::from_str(text).map_err(|e| Error::Toml { path: path.to_path_buf(), message: toml_message(&e, text) })
}

pub fn parse_lesson(text: &str, path: &Path) -> Result<Lesson> {
    toml::from_str(text).map_err(|e| Error::Toml { path: path.to_path_buf(), message: toml_message(&e, text) })
}

/// The TOML error with its line number, which the plain message lacks.
fn toml_message(e: &toml::de::Error, text: &str) -> String {
    match e.span() {
        Some(span) => {
            let end = span.start.min(text.len());
            let line = text.get(..end).map_or(0, |head| head.matches('\n').count()) + 1;
            format!("строка {line}: {}", e.message())
        }
        None => e.message().to_string(),
    }
}

/// Reads `category.toml` from a category folder.
pub fn load_category(dir: &Path) -> Result<Category> {
    let path = dir.join(CATEGORY_FILE);
    let text = std::fs::read_to_string(&path).map_err(error::io(&path))?;
    parse_category(&text, &path)
}

/// Reads `lesson.toml` from a lesson folder.
pub fn load_lesson(dir: &Path) -> Result<Lesson> {
    let path = dir.join(LESSON_FILE);
    let text = std::fs::read_to_string(&path).map_err(error::io(&path))?;
    parse_lesson(&text, &path)
}

pub fn category_to_toml(category: &Category) -> Result<String> {
    toml::to_string_pretty(category).map_err(|e| Error::Serialize(e.to_string()))
}

pub fn lesson_to_toml(lesson: &Lesson) -> Result<String> {
    toml::to_string_pretty(lesson).map_err(|e| Error::Serialize(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LESSON: &str = r##"
format = 1
id = "light.sphere-shadow"
title = "Падающая тень"
difficulty = 3
minutes = 20
skills = ["light", "form"]
requires = ["light.core-shadow"]

[guide]
script = "guide.json"

[[quiz]]
question = "Где тень темнее?"
options = ["У края", "У касания"]
answer = 1

[task]
brief = "Нарисуйте тень."
open = "assets/task.png"

[[task.steps]]
id = "layer"
text = "Новый слой"
  [task.steps.hint]
  text = "Панель «Слои»"
  target = "panel:Layers"
  [[task.steps.hint.shapes]]
  kind = "ellipse"
  rect = [0.4, 0.7, 0.3, 0.1]
  [[task.steps.checks]]
  kind = "layers"
  count = { min = 2 }
  [[task.steps.checks]]
  kind = "darker"
  region = [0.4, 0.7, 0.3, 0.1]
  than = [0, 0.9, 1, 0.1]
  message = "Тень слишком светлая"
"##;

    #[test]
    fn a_lesson_reads_with_defaults() {
        let lesson = parse_lesson(LESSON, Path::new("lesson.toml")).unwrap();
        assert_eq!(lesson.slug(), "sphere-shadow");
        assert_eq!(lesson.category_id(), "light");
        assert_eq!(lesson.theory.file, "theory.md");
        assert_eq!(lesson.difficulty_label(), "Средний");
        assert_eq!(lesson.task.steps[0].checks.len(), 2);
        assert_eq!(lesson.task.steps[0].checks[0].kind, CheckKind::Layers { count: Range::at_least(2.0) });
        match &lesson.task.steps[0].checks[1] {
            Check { kind: CheckKind::Darker { by, .. }, message } => {
                assert_eq!(*by, 0.05);
                assert_eq!(message.as_deref(), Some("Тень слишком светлая"));
            }
            other => panic!("unexpected {other:?}"),
        }
        let hint = lesson.task.steps[0].hint.as_ref().unwrap();
        assert_eq!(hint.shapes, vec![Shape::Ellipse { rect: [0.4, 0.7, 0.3, 0.1] }]);
    }

    #[test]
    fn a_lesson_round_trips_through_toml() {
        let lesson = parse_lesson(LESSON, Path::new("lesson.toml")).unwrap();
        let text = lesson_to_toml(&lesson).unwrap();
        let again = parse_lesson(&text, Path::new("lesson.toml")).unwrap();
        assert_eq!(lesson, again);
    }

    #[test]
    fn unknown_fields_are_errors_with_a_line_number() {
        let text = LESSON.replace("minutes = 20", "minutes = 20\ncolour = \"red\"");
        let err = parse_lesson(&text, Path::new("x/lesson.toml")).unwrap_err().to_string();
        assert!(err.contains("colour"), "{err}");
        assert!(err.contains("строка"), "{err}");
    }

    #[test]
    fn unknown_check_kinds_are_errors() {
        let text = LESSON.replace("kind = \"layers\"", "kind = \"teleport\"");
        assert!(parse_lesson(&text, Path::new("lesson.toml")).is_err());
    }

    #[test]
    fn a_category_reads_and_round_trips() {
        let text = "id = \"light\"\ntitle = \"Свет и тень\"\nkanji = \"影\"\naccent = \"shu\"\norder = 2\n";
        let category = parse_category(text, Path::new("category.toml")).unwrap();
        assert_eq!(category.accent, Accent::Shu);
        assert_eq!(category.description, "");
        let again = parse_category(&category_to_toml(&category).unwrap(), Path::new("c")).unwrap();
        assert_eq!(category, again);
    }

    #[test]
    fn ranges_contain_and_read_in_russian() {
        assert!(Range::between(1.0, 2.0).contains(1.0));
        assert!(Range::between(1.0, 2.0).contains(2.0));
        assert!(!Range::between(1.0, 2.0).contains(2.01));
        assert!(Range::at_least(3.0).contains(1e9));
        assert!(Range::at_most(3.0).contains(-1e9));
        assert!(Range::default().contains(f64::MAX));
        assert_eq!(Range::between(30.0, 80.0).to_string(), "от 30 до 80");
        assert_eq!(Range::at_least(0.25).to_string(), "не меньше 0,25");
        assert_eq!(Range::at_most(2.0).to_string(), "не больше 2");
        assert_eq!(Range::between(2.0, 2.0).to_string(), "ровно 2");
        assert_eq!(Range::at_least(0.3).scaled(100.0).to_string(), "не меньше 30");
    }

    #[test]
    fn hex_colours_parse() {
        assert_eq!(parse_hex("#FFFFFF"), Some([1.0, 1.0, 1.0]));
        assert_eq!(parse_hex("000000"), Some([0.0, 0.0, 0.0]));
        assert!(parse_hex("#BC3823").is_some_and(|c| (c[0] - 188.0 / 255.0).abs() < 1e-9));
        for bad in ["#FFF", "#GGGGGG", "", "#FFFFFFF", "#ффф"] {
            assert_eq!(parse_hex(bad), None, "{bad}");
        }
    }

    #[test]
    fn accents_and_skills_have_labels() {
        for accent in Accent::ALL {
            assert!(parse_hex(accent.hex()).is_some());
        }
        let labels: std::collections::HashSet<_> = Skill::ALL.iter().map(|s| s.label()).collect();
        assert_eq!(labels.len(), Skill::ALL.len());
    }

    #[test]
    fn canvas_checks_are_told_apart() {
        assert!(CheckKind::Coverage { region: [0.0, 0.0, 1.0, 1.0], changed: Range::default() }.needs_canvas());
        assert!(!CheckKind::Tool { tool: "Brush".into() }.needs_canvas());
    }
}
