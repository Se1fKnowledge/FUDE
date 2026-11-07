//! Format checks for categories and lessons.
//!
//! Errors keep a lesson out of the catalog; warnings are shown to the author in the lesson
//! editor but the lesson still works. Messages are in Russian, fields name the TOML key.

use std::collections::HashSet;
use std::fmt;
use std::path::Path;

use serde::Serialize;

use crate::guide;
#[cfg(test)]
use crate::model::Style;
use crate::model::{Category, Check, CheckKind, FORMAT, Hint, Lesson, Range, Region, Shape, parse_hex};
use crate::paths::{self, IMAGE_EXTENSIONS, VIDEO_EXTENSIONS};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issue {
    pub severity: Severity,
    /// TOML key path, e.g. `task.steps[2].checks[0].region`.
    pub field: String,
    pub message: String,
}

impl Issue {
    pub fn error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Issue { severity: Severity::Error, field: field.into(), message: message.into() }
    }

    pub fn warning(field: impl Into<String>, message: impl Into<String>) -> Self {
        Issue { severity: Severity::Warning, field: field.into(), message: message.into() }
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = match self.severity {
            Severity::Error => crate::i18n::pick("ошибка", "error"),
            Severity::Warning => crate::i18n::pick("предупреждение", "warning"),
        };
        if self.field.is_empty() { write!(f, "{level}: {}", self.message) } else { write!(f, "{level}: {}: {}", self.field, self.message) }
    }
}

pub fn has_errors(issues: &[Issue]) -> bool {
    issues.iter().any(Issue::is_error)
}

/// PhotoCraft tools (the `Tool` enum of `photocraft/crates/ui-egui/src/state.rs`).
pub const TOOLS: &[&str] = &[
    "Move",
    "Artboard",
    "RectMarquee",
    "EllipseMarquee",
    "SingleRowMarquee",
    "SingleColumnMarquee",
    "Lasso",
    "PolygonLasso",
    "MagneticLasso",
    "MagicWand",
    "Crop",
    "PerspectiveCrop",
    "Eyedropper",
    "ColorSampler",
    "Ruler",
    "Note",
    "Count",
    "Brush",
    "Pencil",
    "MixerBrush",
    "ColorReplacement",
    "Eraser",
    "BackgroundEraser",
    "MagicEraser",
    "Gradient",
    "PaintBucket",
    "Type",
    "VerticalType",
    "HorizontalTypeMask",
    "VerticalTypeMask",
    "Hand",
    "RotateView",
    "Zoom",
    "Remove",
    "SpotHealing",
    "Healing",
    "Patch",
    "ContentAwareMove",
    "RedEye",
    "CloneStamp",
    "PatternStamp",
    "HistoryBrush",
    "Blur",
    "Sharpen",
    "Smudge",
    "Dodge",
    "Burn",
    "Sponge",
    "QuickSelection",
    "ObjectSelection",
    "Pen",
    "FreeformPen",
    "CurvaturePen",
    "AddAnchorPoint",
    "DeleteAnchorPoint",
    "ConvertPoint",
    "PathSelection",
    "DirectSelection",
    "Rectangle",
    "EllipseShape",
    "Triangle",
    "Polygon",
    "Line",
    "CustomShape",
    "Slice",
    "SliceSelect",
    "Frame",
];

/// PhotoCraft's spelling of a tool name, matched as PhotoCraft does: case-insensitively and
/// with or without `Marquee`.
pub fn canonical_tool(name: &str) -> Option<&'static str> {
    let wanted = name.to_ascii_lowercase();
    TOOLS.iter().copied().find(|tool| {
        let lower = tool.to_ascii_lowercase();
        lower == wanted || lower.strip_suffix("marquee") == Some(wanted.as_str())
    })
}

/// Panels a hint may point at.
pub const PANELS: &[&str] = &["Layers", "Color", "Brushes", "BrushSettings", "History", "Properties", "Navigator", "Toolbar", "OptionsBar"];
/// Tool options a hint may point at.
pub const OPTIONS: &[&str] = &["brushSize", "hardness", "opacity", "flow", "zoom", "blend", "layerOpacity"];

const ID_MAX: usize = 48;

/// Category ids and lesson slugs: lower-case Latin letters, digits and `-`.
pub fn is_slug(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= ID_MAX
        && text.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !text.starts_with('-')
        && !text.ends_with('-')
}

/// Lesson ids: `<category>.<slug>`.
pub fn is_lesson_id(text: &str) -> bool {
    text.split_once('.').is_some_and(|(category, slug)| is_slug(category) && is_slug(slug))
}

/// Checks a hint target: `tool:…`, `panel:…`, `menu:…` or `option:…`.
pub fn check_target(target: &str) -> Result<(), String> {
    let Some((kind, name)) = target.split_once(':') else {
        return Err(crate::tr!("«{target}»: ожидается tool:…, panel:…, menu:… или option:…", "“{target}”: tool:…, panel:…, menu:… or option:… expected"));
    };
    match kind {
        "tool" if canonical_tool(name).is_some() => Ok(()),
        "tool" => Err(crate::tr!("нет такого инструмента PhotoCraft: «{name}»", "PhotoCraft has no such tool: “{name}”")),
        "panel" if PANELS.contains(&name) => Ok(()),
        "panel" => Err(crate::tr!("неизвестная панель «{name}»; есть: {}", "unknown panel “{name}”; there are: {}", PANELS.join(", "))),
        "option" if OPTIONS.contains(&name) => Ok(()),
        "option" => Err(crate::tr!("неизвестный параметр «{name}»; есть: {}", "unknown option “{name}”; there are: {}", OPTIONS.join(", "))),
        "menu" if !name.is_empty() && name.contains('.') && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.') => Ok(()),
        "menu" => Err(crate::tr!("«{name}»: ожидается id команды меню, например edit.undo", "“{name}”: a menu command id is expected, for example edit.undo")),
        _ => Err(crate::tr!("«{target}»: ожидается tool:…, panel:…, menu:… или option:…", "“{target}”: tool:…, panel:…, menu:… or option:… expected")),
    }
}

pub fn category(category: &Category, dir_name: Option<&str>) -> Vec<Issue> {
    let mut issues = Vec::new();
    if !is_slug(&category.id) {
        issues.push(Issue::error(
            "id",
            crate::tr!(
                "«{}»: только строчные латинские буквы, цифры и «-», до {ID_MAX} символов",
                "“{}”: only lowercase Latin letters, digits and “-”, up to {ID_MAX} characters",
                category.id
            ),
        ));
    }
    if let Some(dir) = dir_name
        && dir != category.id
    {
        issues
            .push(Issue::error("id", crate::tr!("id «{}» не совпадает с именем папки «{dir}»", "id “{}” does not match the folder name “{dir}”", category.id)));
    }
    title(&mut issues, "title", &category.title);
    if let Some(kanji) = &category.kanji
        && (kanji.chars().count() != 1)
    {
        issues.push(Issue::warning("kanji", crate::i18n::pick("ожидается один иероглиф", "one kanji is expected")));
    }
    issues
}

fn title(issues: &mut Vec<Issue>, field: &str, text: &str) {
    if text.trim().is_empty() {
        issues.push(Issue::error(field, crate::i18n::pick("не может быть пустым", "cannot be empty")));
    } else if text.chars().count() > 120 {
        issues.push(Issue::warning(
            field,
            crate::i18n::pick("длиннее 120 символов, не поместится на карточке", "longer than 120 characters, it will not fit on the card"),
        ));
    }
}

/// Checks a lesson. With `dir`, also checks that the files it names exist (and validates its
/// guide script); with `category_id`, that the id belongs to that category.
pub fn lesson(lesson: &Lesson, dir: Option<&Path>, category_id: Option<&str>) -> Vec<Issue> {
    let mut issues = Vec::new();
    if lesson.format != FORMAT {
        issues.push(Issue::error(
            "format",
            crate::tr!("версия формата {} не поддерживается (нужна {FORMAT})", "format version {} is not supported ({FORMAT} is needed)", lesson.format),
        ));
    }
    if !is_lesson_id(&lesson.id) {
        issues.push(Issue::error(
            "id",
            crate::tr!(
                "«{}»: ожидается <категория>.<имя>, латиница, цифры и «-»",
                "“{}”: <category>.<name> is expected, Latin letters, digits and “-”",
                lesson.id
            ),
        ));
    } else if let Some(category) = category_id
        && lesson.category_id() != category
    {
        issues.push(Issue::error(
            "id",
            crate::tr!(
                "урок «{}» лежит в категории «{category}», id должен начинаться с «{category}.»",
                "lesson “{}” is in the “{category}” category, its id must start with “{category}.”",
                lesson.id
            ),
        ));
    }
    title(&mut issues, "title", &lesson.title);
    if !(1..=5).contains(&lesson.difficulty) {
        issues.push(Issue::error("difficulty", crate::i18n::pick("сложность — от 1 до 5", "the difficulty is from 1 to 5")));
    }
    if lesson.minutes > 600 {
        issues.push(Issue::warning(
            "minutes",
            crate::i18n::pick("больше 10 часов — разбейте урок на несколько", "more than 10 hours — split the lesson into several"),
        ));
    }
    if lesson.skills.is_empty() {
        issues.push(Issue::warning(
            "skills",
            crate::i18n::pick("урок не развивает ни одну сторону навыка в статистике", "the lesson develops no skill area in the statistics"),
        ));
    }
    let unique: HashSet<_> = lesson.skills.iter().collect();
    if unique.len() != lesson.skills.len() {
        issues.push(Issue::warning("skills", crate::i18n::pick("навык указан дважды", "a skill is given twice")));
    }
    for (i, required) in lesson.requires.iter().enumerate() {
        if !is_lesson_id(required) {
            issues.push(Issue::error(format!("requires[{i}]"), crate::tr!("«{required}» — не id урока", "“{required}” is not a lesson id")));
        } else if *required == lesson.id {
            issues.push(Issue::error(format!("requires[{i}]"), crate::i18n::pick("урок не может требовать сам себя", "a lesson cannot require itself")));
        }
    }

    file(&mut issues, dir, "theory.file", &lesson.theory.file, &["md"]);
    if lesson.guide.is_empty() {
        issues.push(Issue::warning("guide", crate::i18n::pick("нет ни сценария гайда, ни видео", "there is neither a guide script nor a video")));
    }
    if let Some(script) = &lesson.guide.script
        && file(&mut issues, dir, "guide.script", script, &["json"])
        && let Some(dir) = dir
    {
        match guide::load(&dir.join(script)) {
            Ok(guide) => issues.extend(guide::validate(&guide, Some(dir)).into_iter().map(|mut issue| {
                issue.field = format!("{script}: {}", issue.field);
                issue
            })),
            Err(e) => issues.push(Issue::error("guide.script", e.to_string())),
        }
    }
    if let Some(video) = &lesson.guide.video {
        file(&mut issues, dir, "guide.video", video, VIDEO_EXTENSIONS);
    }
    if let Some(poster) = &lesson.guide.poster {
        file(&mut issues, dir, "guide.poster", poster, IMAGE_EXTENSIONS);
    }

    for (i, q) in lesson.quiz.iter().enumerate() {
        let field = format!("quiz[{i}]");
        if q.question.trim().is_empty() {
            issues.push(Issue::error(format!("{field}.question"), crate::i18n::pick("вопрос пустой", "the question is empty")));
        }
        if !(2..=8).contains(&q.options.len()) {
            issues.push(Issue::error(format!("{field}.options"), crate::i18n::pick("нужно от 2 до 8 вариантов", "2 to 8 options are needed")));
        }
        if q.answer >= q.options.len() {
            issues.push(Issue::error(format!("{field}.answer"), crate::tr!("нет варианта с номером {}", "there is no option number {}", q.answer)));
        }
        if q.options.iter().any(|o| o.trim().is_empty()) {
            issues.push(Issue::error(format!("{field}.options"), crate::i18n::pick("пустой вариант ответа", "an empty answer option")));
        }
    }

    task(&mut issues, lesson, dir);
    issues
}

/// Checks a path field; returns whether it is usable (safe, right type, and present if `dir`).
fn file(issues: &mut Vec<Issue>, dir: Option<&Path>, field: &str, path: &str, extensions: &[&str]) -> bool {
    if let Err(e) = paths::check_relative(path) {
        issues.push(Issue::error(field, e));
        return false;
    }
    if !paths::has_extension(path, extensions) {
        issues.push(Issue::error(field, crate::tr!("«{path}»: нужен файл {}", "“{path}”: a {} file is needed", extensions.join(", "))));
        return false;
    }
    if let Some(dir) = dir
        && !paths::resolve(dir, path).is_ok_and(|p| p.is_file())
    {
        issues.push(Issue::error(field, crate::tr!("файл «{path}» не найден", "file “{path}” not found")));
        return false;
    }
    true
}

fn task(issues: &mut Vec<Issue>, lesson: &Lesson, dir: Option<&Path>) {
    let task = &lesson.task;
    if task.brief.trim().is_empty() {
        issues.push(Issue::warning("task.brief", crate::i18n::pick("нет описания задания", "the task has no description")));
    }
    match (&task.open, &task.canvas) {
        (Some(_), Some(_)) => issues.push(Issue::error("task", crate::i18n::pick("укажите что-то одно: open или canvas", "give only one of open or canvas"))),
        (None, None) => issues.push(Issue::error(
            "task",
            crate::i18n::pick("укажите open (картинку задания) или canvas (чистый холст)", "give open (the task picture) or canvas (a blank canvas)"),
        )),
        (Some(open), None) => {
            file(issues, dir, "task.open", open, IMAGE_EXTENSIONS);
        }
        (None, Some(canvas)) => {
            if !(16..=8192).contains(&canvas.width) || !(16..=8192).contains(&canvas.height) {
                issues.push(Issue::error(
                    "task.canvas",
                    crate::i18n::pick("размер холста — от 16 до 8192 пикселей", "the canvas size is from 16 to 8192 pixels"),
                ));
            }
            if parse_hex(&canvas.background).is_none() {
                issues.push(Issue::error(
                    "task.canvas.background",
                    crate::tr!("«{}»: ожидается цвет #rrggbb", "“{}”: a #rrggbb colour is expected", canvas.background),
                ));
            }
        }
    }
    let has_reference = task.reference.as_ref().is_some_and(|r| file(issues, dir, "task.reference", r, IMAGE_EXTENSIONS));

    if task.steps.is_empty() {
        issues.push(Issue::error("task.steps", crate::i18n::pick("в задании нет шагов", "the task has no steps")));
    }
    let mut ids = HashSet::new();
    for (i, step) in task.steps.iter().enumerate() {
        let field = format!("task.steps[{i}]");
        if !is_slug(&step.id) {
            issues.push(Issue::error(format!("{field}.id"), crate::tr!("«{}»: латиница, цифры и «-»", "“{}”: Latin letters, digits and “-”", step.id)));
        } else if !ids.insert(step.id.as_str()) {
            issues.push(Issue::error(format!("{field}.id"), crate::tr!("шаг «{}» уже есть", "step “{}” already exists", step.id)));
        }
        if step.text.trim().is_empty() {
            issues.push(Issue::error(format!("{field}.text"), crate::i18n::pick("текст шага пустой", "the step text is empty")));
        }
        if let Some(h) = &step.hint {
            hint(issues, &format!("{field}.hint"), h);
        }
        if step.checks.is_empty() {
            issues.push(Issue::warning(
                format!("{field}.checks"),
                crate::i18n::pick("без проверок шаг засчитывается кнопкой «Готово»", "without checks the step counts with the “Done” button"),
            ));
        }
        for (j, c) in step.checks.iter().enumerate() {
            check(issues, &format!("{field}.checks[{j}]"), c, dir, has_reference);
        }
    }
}

fn hint(issues: &mut Vec<Issue>, field: &str, hint: &Hint) {
    if hint.text.trim().is_empty() {
        issues.push(Issue::error(format!("{field}.text"), crate::i18n::pick("текст подсказки пустой", "the hint text is empty")));
    }
    if let Some(target) = &hint.target
        && let Err(e) = check_target(target)
    {
        issues.push(Issue::error(format!("{field}.target"), e));
    }
    for (i, shape) in hint.shapes.iter().enumerate() {
        let field = format!("{field}.shapes[{i}]");
        let points: Vec<[f64; 2]> = match shape {
            Shape::Ellipse { rect, .. } | Shape::Rect { rect, .. } => {
                region(issues, &field, rect);
                vec![]
            }
            Shape::Line { from, to, .. } | Shape::Arrow { from, to, .. } => vec![*from, *to],
            Shape::Path { points, .. } => {
                if points.len() < 2 {
                    issues.push(Issue::error(&field, crate::i18n::pick("в линии нужно хотя бы две точки", "a line needs at least two points")));
                }
                if points.len() > 2000 {
                    issues.push(Issue::error(&field, crate::i18n::pick("больше 2000 точек", "more than 2000 points")));
                }
                points.clone()
            }
        };
        if points.iter().flatten().any(|v| !(0.0..=1.0).contains(v)) {
            issues.push(Issue::error(&field, crate::i18n::pick("координаты — доли холста от 0 до 1", "coordinates are fractions of the canvas from 0 to 1")));
        }
        let style = shape.style();
        if let Some(color) = &style.color
            && parse_hex(color).is_none()
        {
            issues.push(Issue::error(format!("{field}.style.color"), crate::i18n::pick("цвет — в виде #rrggbb", "the colour is #rrggbb")));
        }
        if style.width.is_some_and(|w| !(0.5..=200.0).contains(&w)) {
            issues.push(Issue::error(
                format!("{field}.style.width"),
                crate::i18n::pick("толщина — от 0,5 до 200 пикселей", "the width is from 0.5 to 200 pixels"),
            ));
        }
        if style.opacity.is_some_and(|o| !(0.05..=1.0).contains(&o)) {
            issues.push(Issue::error(format!("{field}.style.opacity"), crate::i18n::pick("непрозрачность — от 0,05 до 1", "the opacity is from 0.05 to 1")));
        }
    }
}

fn region(issues: &mut Vec<Issue>, field: &str, r: &Region) {
    let [x, y, w, h] = *r;
    let ok = r.iter().all(|v| v.is_finite()) && x >= 0.0 && y >= 0.0 && w > 0.0 && h > 0.0 && x + w <= 1.0 + 1e-9 && y + h <= 1.0 + 1e-9;
    if !ok {
        issues.push(Issue::error(
            field,
            crate::i18n::pick(
                "область [x, y, ширина, высота] — доли холста от 0 до 1, внутри холста",
                "the area [x, y, width, height] is fractions of the canvas from 0 to 1, inside the canvas",
            ),
        ));
    }
}

fn range(issues: &mut Vec<Issue>, field: &str, r: &Range, bounds: Option<(f64, f64)>) {
    if let (Some(a), Some(b)) = (r.min, r.max)
        && a > b
    {
        issues.push(Issue::error(field, crate::i18n::pick("min больше max", "min is greater than max")));
    }
    if r.min.is_none() && r.max.is_none() {
        issues.push(Issue::warning(
            field,
            crate::i18n::pick("диапазон без границ — проверка всегда проходит", "a range without bounds — the check always passes"),
        ));
    }
    if let Some((lo, hi)) = bounds
        && [r.min, r.max].into_iter().flatten().any(|v| !(lo..=hi).contains(&v))
    {
        issues.push(Issue::error(field, crate::tr!("значения — от {lo} до {hi}", "values are from {lo} to {hi}")));
    }
}

fn check(issues: &mut Vec<Issue>, field: &str, check: &Check, dir: Option<&Path>, has_reference: bool) {
    let unit = Some((0.0, 1.0));
    match &check.kind {
        CheckKind::Tool { tool } => {
            if canonical_tool(tool).is_none() {
                issues.push(Issue::error(
                    format!("{field}.tool"),
                    crate::tr!("нет такого инструмента PhotoCraft: «{tool}»", "PhotoCraft has no such tool: “{tool}”"),
                ));
            }
        }
        CheckKind::Brush { size, hardness, opacity } => {
            if size.is_none() && hardness.is_none() && opacity.is_none() {
                issues.push(Issue::warning(field, crate::i18n::pick("не задан ни один параметр кисти", "no brush setting is given")));
            }
            if let Some(r) = size {
                range(issues, &format!("{field}.size"), r, Some((0.5, 5000.0)));
            }
            if let Some(r) = hardness {
                range(issues, &format!("{field}.hardness"), r, unit);
            }
            if let Some(r) = opacity {
                range(issues, &format!("{field}.opacity"), r, unit);
            }
        }
        CheckKind::Zoom { zoom } => range(issues, &format!("{field}.zoom"), zoom, Some((0.01, 128.0))),
        CheckKind::Layers { count } => range(issues, &format!("{field}.count"), count, Some((0.0, 10000.0))),
        CheckKind::Layer { name, visible, opacity, layer_type } => {
            if name.is_none() && visible.is_none() && opacity.is_none() && layer_type.is_none() {
                issues.push(Issue::warning(field, crate::i18n::pick("без условий подойдёт любой слой", "without conditions any layer fits")));
            }
            if let Some(r) = opacity {
                range(issues, &format!("{field}.opacity"), r, unit);
            }
        }
        CheckKind::Selection { width, height, .. } => {
            for (name, r) in [("width", width), ("height", height)] {
                if let Some(r) = r {
                    range(issues, &format!("{field}.{name}"), r, unit);
                }
            }
        }
        CheckKind::History { entries, can_redo } => {
            if entries.is_none() && can_redo.is_none() {
                issues.push(Issue::warning(field, crate::i18n::pick("не задано ни entries, ни can_redo", "neither entries nor can_redo is given")));
            }
            if let Some(r) = entries {
                range(issues, &format!("{field}.entries"), r, Some((0.0, 100000.0)));
            }
        }
        CheckKind::Foreground { color, tolerance } => {
            colour(issues, &format!("{field}.color"), color);
            tolerance_ok(issues, field, *tolerance);
        }
        CheckKind::Coverage { region: r, changed } => {
            region(issues, &format!("{field}.region"), r);
            range(issues, &format!("{field}.changed"), changed, unit);
        }
        CheckKind::Luminance { region: r, value } => {
            region(issues, &format!("{field}.region"), r);
            range(issues, &format!("{field}.value"), value, unit);
        }
        CheckKind::Darker { region: r, than, by } => {
            region(issues, &format!("{field}.region"), r);
            region(issues, &format!("{field}.than"), than);
            if !(0.0..=1.0).contains(by) {
                issues.push(Issue::error(format!("{field}.by"), crate::i18n::pick("от 0 до 1", "from 0 to 1")));
            }
        }
        CheckKind::Color { region: r, color, tolerance } => {
            region(issues, &format!("{field}.region"), r);
            colour(issues, &format!("{field}.color"), color);
            tolerance_ok(issues, field, *tolerance);
        }
        CheckKind::Similar { reference, score } => {
            range(issues, &format!("{field}.score"), score, unit);
            match reference {
                Some(r) => {
                    file(issues, dir, &format!("{field}.reference"), r, IMAGE_EXTENSIONS);
                }
                None if !has_reference => issues.push(Issue::error(
                    format!("{field}.reference"),
                    crate::i18n::pick("нет эталона: задайте reference здесь или в task.reference", "no reference: set reference here or in task.reference"),
                )),
                None => {}
            }
        }
    }
}

fn colour(issues: &mut Vec<Issue>, field: &str, color: &str) {
    if parse_hex(color).is_none() {
        issues.push(Issue::error(field, crate::tr!("«{color}»: ожидается цвет #rrggbb", "“{color}”: a #rrggbb colour is expected")));
    }
}

fn tolerance_ok(issues: &mut Vec<Issue>, field: &str, tolerance: f64) {
    if !(0.0..=1.0).contains(&tolerance) {
        issues.push(Issue::error(format!("{field}.tolerance"), crate::i18n::pick("от 0 до 1", "from 0 to 1")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Canvas, Guide, Question, Skill, Step, Task, Theory};

    pub(crate) fn sample() -> Lesson {
        Lesson {
            format: FORMAT,
            id: "basics.brush".into(),
            title: "Кисть".into(),
            summary: String::new(),
            difficulty: 1,
            minutes: 10,
            order: 1,
            skills: vec![Skill::Tools],
            requires: vec![],
            author: None,
            theory: Theory::default(),
            guide: Guide { script: Some("guide.json".into()), video: None, poster: None },
            quiz: vec![Question { question: "?".into(), options: vec!["a".into(), "b".into()], answer: 0, explain: None }],
            task: Task {
                brief: "Нарисуйте".into(),
                open: None,
                reference: None,
                canvas: Some(Canvas::default()),
                steps: vec![Step {
                    id: "tool".into(),
                    text: "Возьмите кисть".into(),
                    hint: Some(Hint { text: "Слева".into(), target: Some("tool:Brush".into()), shapes: vec![] }),
                    checks: vec![CheckKind::Tool { tool: "Brush".into() }.into()],
                }],
            },
        }
    }

    fn errors(issues: &[Issue]) -> Vec<String> {
        issues.iter().filter(|i| i.is_error()).map(|i| i.field.clone()).collect()
    }

    #[test]
    fn the_sample_lesson_is_valid() {
        let issues = lesson(&sample(), None, Some("basics"));
        assert!(!has_errors(&issues), "{issues:?}");
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn ids_follow_the_slug_rules() {
        for ok in ["basics", "light-2", "a"] {
            assert!(is_slug(ok), "{ok}");
        }
        for bad in ["", "Basics", "свет", "a_b", "-a", "a-", "a.b", &"a".repeat(49)] {
            assert!(!is_slug(bad), "{bad}");
        }
        assert!(is_lesson_id("light.sphere-shadow"));
        assert!(!is_lesson_id("light"));
        assert!(!is_lesson_id("light.a.b"));
        assert!(!is_lesson_id(".a"));
    }

    #[test]
    fn a_lesson_must_live_in_its_category() {
        let issues = lesson(&sample(), None, Some("light"));
        assert_eq!(errors(&issues), vec!["id"]);
    }

    #[test]
    fn bad_fields_are_reported_by_key() {
        let mut l = sample();
        l.format = 2;
        l.difficulty = 6;
        l.title = "  ".into();
        l.requires = vec!["basics.brush".into(), "nope".into()];
        l.quiz[0].answer = 5;
        l.theory.file = "../theory.md".into();
        let got = errors(&lesson(&l, None, None));
        for field in ["format", "difficulty", "title", "requires[0]", "requires[1]", "quiz[0].answer", "theory.file"] {
            assert!(got.contains(&field.to_string()), "{field} missing in {got:?}");
        }
    }

    #[test]
    fn a_task_needs_exactly_one_start_and_some_steps() {
        let mut l = sample();
        l.task.open = Some("assets/task.png".into());
        assert_eq!(errors(&lesson(&l, None, None)), vec!["task"]);
        l.task.canvas = None;
        l.task.open = None;
        assert_eq!(errors(&lesson(&l, None, None)), vec!["task"]);
        let mut l = sample();
        l.task.steps.clear();
        assert_eq!(errors(&lesson(&l, None, None)), vec!["task.steps"]);
        let mut l = sample();
        l.task.canvas = Some(Canvas { width: 10, height: 10, background: "white".into() });
        assert_eq!(errors(&lesson(&l, None, None)), vec!["task.canvas", "task.canvas.background"]);
    }

    #[test]
    fn duplicate_step_ids_are_errors() {
        let mut l = sample();
        let step = l.task.steps[0].clone();
        l.task.steps.push(step);
        assert_eq!(errors(&lesson(&l, None, None)), vec!["task.steps[1].id"]);
    }

    #[test]
    fn check_values_are_range_checked() {
        let mut l = sample();
        l.task.steps[0].checks = vec![
            CheckKind::Tool { tool: "Teleport".into() }.into(),
            CheckKind::Brush { size: None, hardness: Some(Range::at_least(2.0)), opacity: None }.into(),
            CheckKind::Coverage { region: [0.5, 0.5, 0.6, 0.1], changed: Range::at_least(0.2) }.into(),
            CheckKind::Luminance { region: [0.0, 0.0, 1.0, 1.0], value: Range::between(0.8, 0.2) }.into(),
            CheckKind::Color { region: [0.0, 0.0, 1.0, 1.0], color: "red".into(), tolerance: 0.1 }.into(),
            CheckKind::Similar { reference: None, score: Range::at_least(0.8) }.into(),
        ];
        let got = errors(&lesson(&l, None, None));
        assert_eq!(
            got,
            vec![
                "task.steps[0].checks[0].tool",
                "task.steps[0].checks[1].hardness",
                "task.steps[0].checks[2].region",
                "task.steps[0].checks[3].value",
                "task.steps[0].checks[4].color",
                "task.steps[0].checks[5].reference",
            ]
        );
    }

    #[test]
    fn hint_targets_and_shapes_are_checked() {
        assert_eq!(check_target("tool:brush"), Ok(()));
        assert_eq!(check_target("tool:Rect"), Ok(()));
        assert_eq!(check_target("panel:Layers"), Ok(()));
        assert_eq!(check_target("menu:edit.undo"), Ok(()));
        assert_eq!(check_target("option:brushSize"), Ok(()));
        for bad in ["Brush", "tool:Laser", "panel:Nope", "menu:undo", "menu:", "button:x", "option:size"] {
            assert!(check_target(bad).is_err(), "{bad}");
        }
        let mut l = sample();
        l.task.steps[0].hint = Some(Hint {
            text: "x".into(),
            target: None,
            shapes: vec![
                Shape::Path { points: vec![[0.1, 0.1]], style: Style::default() },
                Shape::Line { from: [0.0, 0.0], to: [1.2, 0.5], style: Style::default() },
            ],
        });
        assert_eq!(errors(&lesson(&l, None, None)), vec!["task.steps[0].hint.shapes[0]", "task.steps[0].hint.shapes[1]"]);
    }

    #[test]
    fn ghost_styles_are_checked() {
        let mut l = sample();
        let bad = Style { color: Some("red".into()), width: Some(0.1), opacity: Some(1.5), dashed: true, reveal: false };
        let good = Style { color: Some("#2F5D8C".into()), width: Some(6.0), opacity: Some(0.35), dashed: true, reveal: true };
        l.task.steps[0].hint = Some(Hint {
            text: "x".into(),
            target: None,
            shapes: vec![Shape::Arrow { from: [0.1, 0.1], to: [0.5, 0.5], style: bad }, Shape::Arrow { from: [0.1, 0.1], to: [0.5, 0.5], style: good }],
        });
        assert_eq!(
            errors(&lesson(&l, None, None)),
            vec!["task.steps[0].hint.shapes[0].style.color", "task.steps[0].hint.shapes[0].style.width", "task.steps[0].hint.shapes[0].style.opacity"]
        );
    }

    #[test]
    fn tools_match_photocraft_spelling() {
        assert_eq!(canonical_tool("brush"), Some("Brush"));
        assert_eq!(canonical_tool("RECTMARQUEE"), Some("RectMarquee"));
        assert_eq!(canonical_tool("ellipse"), Some("EllipseMarquee"));
        assert_eq!(canonical_tool("paintbucket"), Some("PaintBucket"));
        assert_eq!(canonical_tool("laser"), None);
    }

    #[test]
    fn categories_are_checked() {
        let c = Category {
            id: "Light".into(),
            title: String::new(),
            kanji: Some("影影".into()),
            description: String::new(),
            order: 0,
            accent: Default::default(),
            author: None,
        };
        let issues = category(&c, Some("light"));
        assert_eq!(issues.iter().filter(|i| i.is_error()).count(), 3, "{issues:?}");
        assert!(issues.iter().any(|i| i.field == "kanji" && !i.is_error()));
    }

    #[test]
    fn issues_print_in_russian() {
        assert_eq!(Issue::error("title", "пусто").to_string(), "ошибка: title: пусто");
        assert_eq!(Issue::warning("", "внимание").to_string(), "предупреждение: внимание");
    }
}
