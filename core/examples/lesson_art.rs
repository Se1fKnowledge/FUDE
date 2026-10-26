//! Draws the pictures and writes the guide scripts of the built-in lessons.
//!
//! ```bash
//! cargo run -p fude-core --example lesson_art -- lessons
//! ```
//!
//! Each lesson gets `assets/task.png` (what the student starts from, unless the task is a
//! blank canvas), `assets/reference.png` (the finished result, shown as the model answer
//! and used by the tests to prove the checks can pass) and `guide.json`. The same geometry
//! feeds the pictures, the guide strokes and the check regions in `lesson.toml`, so they
//! agree; the tests in `core/tests/lessons.rs` hold them to it.

use std::f32::consts::{PI, TAU};
use std::path::{Path, PathBuf};

use image::{Rgba, RgbaImage};
use serde_json::{Value, json};

const W: u32 = 1200;
const H: u32 = 800;

type Rgb = [u8; 3];
const WHITE: Rgb = [0xFF, 0xFF, 0xFF];
const PAPER: Rgb = [0xFB, 0xF8, 0xF2];
const PAPER_LINE: Rgb = [0xDD, 0xD4, 0xC3];
const INK: Rgb = [0x16, 0x16, 0x1A];
const INK_500: Rgb = [0x62, 0x5E, 0x57];
const SHU: Rgb = [0xBC, 0x38, 0x23];
const AI: Rgb = [0x2F, 0x5D, 0x8C];
const AI_900: Rgb = [0x16, 0x30, 0x4F];
const KAKI: Rgb = [0xE8, 0x89, 0x3B];
const KAKI_100: Rgb = [0xF7, 0xE1, 0xCB];
const KIN_100: Rgb = [0xF1, 0xE6, 0xCC];
const SUMI: Rgb = [0x11, 0x12, 0x16];

/// The sphere of the light-and-shadow lessons.
const CX: f32 = 600.0;
const CY: f32 = 360.0;
const R: f32 = 200.0;
const GROUND: f32 = 560.0;

fn hex(c: Rgb) -> String {
    format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    [0, 1, 2].map(|i| (f32::from(a[i]) + (f32::from(b[i]) - f32::from(a[i])) * t).round() as u8)
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

struct Canvas(RgbaImage);

impl Canvas {
    fn new(color: Rgb) -> Self {
        Canvas(RgbaImage::from_pixel(W, H, Rgba([color[0], color[1], color[2], 255])))
    }

    fn blend(&mut self, x: i32, y: i32, color: Rgb, alpha: f32) {
        if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 || alpha <= 0.0 {
            return;
        }
        let p = self.0.get_pixel_mut(x as u32, y as u32);
        let a = alpha.min(1.0);
        for (channel, c) in p.0.iter_mut().zip(color) {
            *channel = (f32::from(*channel) * (1.0 - a) + f32::from(c) * a).round() as u8;
        }
    }

    /// Paints every pixel of the box with `f(x, y) -> Option<(colour, alpha)>`.
    fn paint(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, f: impl Fn(f32, f32) -> Option<(Rgb, f32)>) {
        let (xa, ya) = (x0.floor().max(0.0) as i32, y0.floor().max(0.0) as i32);
        let (xb, yb) = (x1.ceil().min(W as f32) as i32, y1.ceil().min(H as f32) as i32);
        for y in ya..yb {
            for x in xa..xb {
                if let Some((c, a)) = f(x as f32 + 0.5, y as f32 + 0.5) {
                    self.blend(x, y, c, a);
                }
            }
        }
    }

    /// A brush stroke along a polyline: one continuous mark, like a real stroke at full flow.
    fn stroke(&mut self, points: &[(f32, f32)], radius: f32, hardness: f32, color: Rgb, opacity: f32) {
        if points.is_empty() {
            return;
        }
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for &(x, y) in points {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
        let segments: Vec<((f32, f32), (f32, f32))> =
            if points.len() == 1 { vec![(points[0], points[0])] } else { points.windows(2).map(|w| (w[0], w[1])).collect() };
        let inner = radius * hardness.clamp(0.0, 0.99);
        self.paint(x0 - radius, y0 - radius, x1 + radius, y1 + radius, |x, y| {
            let d = segments.iter().map(|&(a, b)| segment_distance((x, y), a, b)).fold(f32::MAX, f32::min);
            (d < radius).then(|| (color, opacity * (1.0 - smoothstep(inner, radius, d))))
        });
    }

    fn fill_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, color: Rgb, opacity: f32) {
        self.paint(cx - rx - 1.0, cy - ry - 1.0, cx + rx + 1.0, cy + ry + 1.0, |x, y| {
            let d = (((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2)).sqrt();
            let edge = 1.0 / rx.min(ry);
            (d < 1.0 + edge).then(|| (color, opacity * (1.0 - smoothstep(1.0 - edge, 1.0 + edge, d))))
        });
    }

    fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Rgb) {
        self.paint(x, y, x + w, y + h, |_, _| Some((color, 1.0)));
    }

    fn save(&self, path: &Path) {
        if let Some(parent) = path.parent()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            eprintln!("{}: {e}", parent.display());
            std::process::exit(1);
        }
        if let Err(e) = self.0.save(path) {
            eprintln!("{}: {e}", path.display());
            std::process::exit(1);
        }
    }
}

fn segment_distance(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 { 0.0 } else { (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0) };
    let (qx, qy) = (a.0 + t * dx, a.1 + t * dy);
    ((p.0 - qx).powi(2) + (p.1 - qy).powi(2)).sqrt()
}

fn arc(cx: f32, cy: f32, rx: f32, ry: f32, from: f32, to: f32, n: usize) -> Vec<(f32, f32)> {
    (0..=n).map(|i| from + (to - from) * i as f32 / n as f32).map(|t| (cx + rx * t.cos(), cy + ry * t.sin())).collect()
}

fn wave(x0: f32, x1: f32, y: f32, amp: f32, n: usize) -> Vec<(f32, f32)> {
    (0..=n).map(|i| i as f32 / n as f32).map(|t| (x0 + (x1 - x0) * t, y + amp * (t * TAU).sin())).collect()
}

// ------------------------------------------------------------------ guide scripts

fn ui_set(params: Value) -> Value {
    json!({ "method": "ui.set", "params": params })
}

fn engine(command: &str, params: Value) -> Value {
    json!({ "method": "engine.execute", "params": { "command": command, "params": params } })
}

fn stroke_call(points: &[(f32, f32)], size: f32, color: Rgb) -> Value {
    let pts: Vec<Value> = points.iter().map(|&(x, y)| json!([x.round(), y.round(), 1.0])).collect();
    engine("paint.stroke", json!({ "points": pts, "size": size, "color": hex(color) }))
}

fn brush(size: f32, hardness: f32, opacity: f32) -> Value {
    engine("tools.setBrush", json!({ "size": size, "hardness": hardness, "opacity": opacity, "flow": 1.0 }))
}

fn color(c: Rgb) -> Value {
    engine("tools.setColors", json!({ "foreground": hex(c) }))
}

fn step(caption: &str, highlight: Option<&str>, calls: Vec<Value>, pause: u32) -> Value {
    let mut s = json!({ "caption": caption, "calls": calls, "pause": pause });
    if let Some(h) = highlight {
        s["highlight"] = json!(h);
    }
    s
}

fn guide(title: &str, open_task: bool, steps: Vec<Value>) -> Value {
    let mut g = json!({ "format": 1, "title": title, "steps": steps });
    if open_task {
        g["open"] = json!("assets/task.png");
    } else {
        g["canvas"] = json!({ "width": W, "height": H, "background": "#FFFFFF" });
    }
    g
}

struct Out {
    root: PathBuf,
}

impl Out {
    fn dir(&self, lesson: &str) -> PathBuf {
        self.root.join(lesson.replace('/', std::path::MAIN_SEPARATOR_STR))
    }

    fn image(&self, lesson: &str, name: &str, canvas: &Canvas) {
        canvas.save(&self.dir(lesson).join("assets").join(name));
    }

    fn guide(&self, lesson: &str, script: &Value) {
        let path = self.dir(lesson).join("guide.json");
        let text = match serde_json::to_string_pretty(script) {
            Ok(t) => t + "\n",
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        };
        if let Err(e) = std::fs::write(&path, text) {
            eprintln!("{}: {e}", path.display());
            std::process::exit(1);
        }
    }
}

fn main() {
    let root = std::env::args().nth(1).map_or_else(|| PathBuf::from("lessons"), PathBuf::from);
    let out = Out { root };
    basics_canvas(&out);
    basics_brush(&out);
    basics_eraser(&out);
    basics_layers(&out);
    basics_selection(&out);
    basics_opacity(&out);
    light_source(&out);
    light_core_shadow(&out);
    light_cast_shadow(&out);
    light_reflex(&out);
    light_drapery(&out);
    light_backlight(&out);
    println!("Готово: {}", out.root.display());
}

// ------------------------------------------------------------------ basics

fn basics_canvas(out: &Out) {
    let id = "basics/01-canvas";
    let mut c = Canvas::new(PAPER);
    c.stroke(&arc(600.0, 400.0, 250.0, 250.0, 0.35, TAU - 0.15, 120), 16.0, 0.55, INK, 0.9);
    // The seal to find: a tiny vermilion stamp in the corner.
    c.fill_rect(1110.0, 724.0, 40.0, 40.0, SHU);
    c.fill_rect(1117.0, 731.0, 26.0, 26.0, PAPER);
    c.fill_rect(1123.0, 737.0, 14.0, 14.0, SHU);
    out.image(id, "task.png", &c);
    out.guide(
        id,
        &guide(
            "Холст и масштаб",
            true,
            vec![
                step("Инструмент «Масштаб» — лупа на панели инструментов, клавиша Z", Some("tool:Zoom"), vec![ui_set(json!({ "tool": "Zoom" }))], 1400),
                step("Щелчок приближает, Alt+щелчок отдаляет. В углу — печать", None, vec![ui_set(json!({ "zoom": 2.0, "center": [1130, 744] }))], 1600),
                step("Ещё ближе: 400%", None, vec![ui_set(json!({ "zoom": 4.0, "center": [1130, 744] }))], 1600),
                step(
                    "«Рука» (H или зажатый пробел) двигает приближенный холст",
                    Some("tool:Hand"),
                    vec![ui_set(json!({ "tool": "Hand" })), ui_set(json!({ "center": [900, 600] }))],
                    1400,
                ),
                step("Ctrl+0 — показать холст целиком", None, vec![ui_set(json!({ "fit": true }))], 1400),
            ],
        ),
    );
}

const BRUSH_LINES: [f32; 3] = [150.0, 230.0, 310.0];

fn basics_brush(out: &Out) {
    let id = "basics/02-brush";
    let mut c = Canvas::new(WHITE);
    for (i, y) in BRUSH_LINES.iter().enumerate() {
        c.stroke(&wave(200.0, 1000.0, *y, 12.0, 40), 25.0, if i == 0 { 0.9 } else { 0.1 }, INK, 1.0);
    }
    c.stroke(&wave(200.0, 1000.0, 560.0, 18.0, 40), 25.0, 0.1, SHU, 1.0);
    out.image(id, "reference.png", &c);
    let line = |y: f32| wave(200.0, 1000.0, y, 12.0, 24);
    out.guide(
        id,
        &guide(
            "Кисть: размер, жёсткость, цвет",
            false,
            vec![
                step("Инструмент «Кисть» — клавиша B", Some("tool:Brush"), vec![ui_set(json!({ "tool": "Brush" }))], 1200),
                step("Размер 50 пикселей: клавиши [ и ] меняют его на ходу", Some("option:brushSize"), vec![brush(50.0, 1.0, 1.0)], 1200),
                step("Жёсткая кисть: чёткий край", None, vec![stroke_call(&line(BRUSH_LINES[0]), 50.0, INK)], 900),
                step(
                    "Жёсткость 0%: край тает, как тушь на влажной бумаге",
                    Some("option:hardness"),
                    vec![brush(50.0, 0.0, 1.0), stroke_call(&line(BRUSH_LINES[1]), 50.0, INK)],
                    900,
                ),
                step("Ещё один мягкий мазок", None, vec![stroke_call(&line(BRUSH_LINES[2]), 50.0, INK)], 900),
                step("Основной цвет — киноварь #BC3823", Some("panel:Color"), vec![color(SHU)], 1000),
                step("Мазок новым цветом", None, vec![stroke_call(&wave(200.0, 1000.0, 560.0, 18.0, 24), 50.0, SHU)], 1200),
            ],
        ),
    );
}

const BLOT: (f32, f32, f32) = (850.0, 380.0, 70.0);

fn basics_eraser(out: &Out) {
    let id = "basics/03-eraser-undo";
    let mut clean = Canvas::new(PAPER);
    clean.stroke(&arc(400.0, 400.0, 150.0, 150.0, 0.0, TAU, 96), 5.0, 0.8, INK, 1.0);
    out.image(id, "reference.png", &clean);
    let mut c = Canvas::new(PAPER);
    c.stroke(&arc(400.0, 400.0, 150.0, 150.0, 0.0, TAU, 96), 5.0, 0.8, INK, 1.0);
    let (bx, by, br) = BLOT;
    c.fill_ellipse(bx, by, br, br * 0.85, INK, 1.0);
    for (dx, dy, r) in [(-60.0, -50.0, 12.0), (55.0, 48.0, 9.0), (64.0, -30.0, 7.0), (-40.0, 52.0, 6.0)] {
        c.fill_ellipse(bx + dx, by + dy, r, r, INK, 1.0);
    }
    out.image(id, "task.png", &c);
    let scrub: Vec<(f32, f32)> = (0..7).map(|i| (bx - 70.0 + i as f32 * 23.0, by + if i % 2 == 0 { -55.0 } else { 55.0 })).collect();
    out.guide(
        id,
        &guide(
            "Ластик и отмена",
            true,
            vec![
                step("Инструмент «Ластик» — клавиша E", Some("tool:Eraser"), vec![ui_set(json!({ "tool": "Eraser" }))], 1200),
                step("Крупный ластик: размер 90", Some("option:brushSize"), vec![brush(90.0, 0.6, 1.0)], 1000),
                step("Стираем кляксу зигзагом", None, vec![json!({ "method": "engine.execute", "params": { "command": "paint.stroke", "params": { "points": scrub.iter().map(|&(x, y)| json!([x.round(), y.round(), 1.0])).collect::<Vec<_>>(), "size": 90, "erase": true } } })], 1200),
                step("Случайный штрих кистью…", Some("tool:Brush"), vec![ui_set(json!({ "tool": "Brush" })), brush(12.0, 1.0, 1.0), stroke_call(&[(300.0, 300.0), (520.0, 520.0)], 12.0, INK)], 1200),
                step("…и отмена: Ctrl+Z. Ctrl+Shift+Z возвращает отменённое", Some("menu:edit.undo"), vec![engine("edit.undo", json!({}))], 1600),
            ],
        ),
    );
}

const SUN: (f32, f32, f32) = (840.0, 250.0, 90.0);

fn landscape() -> Canvas {
    let mut c = Canvas::new(PAPER);
    c.stroke(&wave(60.0, 1140.0, 520.0, 6.0, 60), 4.0, 0.7, INK, 1.0);
    for i in 0..9 {
        let x = 120.0 + i as f32 * 115.0;
        c.stroke(&[(x, 600.0), (x + 18.0, 560.0)], 2.5, 0.7, INK_500, 1.0);
        c.stroke(&[(x + 30.0, 640.0), (x + 40.0, 610.0)], 2.5, 0.7, INK_500, 1.0);
    }
    c
}

fn basics_layers(out: &Out) {
    let id = "basics/04-layers";
    out.image(id, "task.png", &landscape());
    let mut r = landscape();
    let (sx, sy, sr) = SUN;
    r.fill_ellipse(sx, sy, sr, sr, KAKI, 1.0);
    out.image(id, "reference.png", &r);
    let rings: Vec<Value> = [70.0, 40.0, 12.0].iter().map(|rr| stroke_call(&arc(sx, sy, *rr, *rr, 0.0, TAU, 48), 46.0, KAKI)).collect();
    out.guide(
        id,
        &guide(
            "Слои",
            true,
            vec![
                step(
                    "Новый слой: кнопка внизу панели «Слои» или Ctrl+Shift+N",
                    Some("panel:Layers"),
                    vec![engine("layer.new.layer", json!({ "name": "Солнце" }))],
                    1600,
                ),
                step("Двойной щелчок по имени слоя — переименовать. Назовём его «Солнце»", Some("panel:Layers"), vec![], 1600),
                step(
                    "Рисуем солнце на своём слое: пейзаж под ним не задет",
                    Some("tool:Brush"),
                    [vec![ui_set(json!({ "tool": "Brush" })), brush(46.0, 0.8, 1.0), color(KAKI)], rings].concat(),
                    1400,
                ),
                step("Глаз слева от слоя прячет и показывает его", Some("panel:Layers"), vec![engine("layer.setProps", json!({ "visible": false }))], 1200),
                step("Слой снова виден", Some("panel:Layers"), vec![engine("layer.setProps", json!({ "visible": true }))], 1200),
            ],
        ),
    );
}

/// `[x, y, w, h]` in pixels.
const RECT: [f32; 4] = [120.0, 160.0, 420.0, 480.0];
const CIRCLE: [f32; 4] = [696.0, 200.0, 360.0, 360.0];

fn basics_selection(out: &Out) {
    let id = "basics/05-selection-fill";
    let mut r = Canvas::new(WHITE);
    let [x, y, w, h] = RECT;
    r.fill_rect(x, y, w, h, AI);
    let [ex, ey, ew, eh] = CIRCLE;
    r.fill_ellipse(ex + ew / 2.0, ey + eh / 2.0, ew / 2.0, eh / 2.0, SHU, 1.0);
    out.image(id, "reference.png", &r);
    out.guide(
        id,
        &guide(
            "Выделение и заливка",
            false,
            vec![
                step("«Прямоугольная область» — клавиша M", Some("tool:RectMarquee"), vec![ui_set(json!({ "tool": "RectMarquee" }))], 1200),
                step(
                    "Тянем рамку: бегущие муравьи показывают границу",
                    None,
                    vec![engine("select.rect", json!({ "x": x, "y": y, "width": w, "height": h }))],
                    1400,
                ),
                step(
                    "Редактирование › Выполнить заливку: индиго #2F5D8C",
                    Some("menu:edit.fill"),
                    vec![engine("edit.fill", json!({ "contents": "color", "color": hex(AI) }))],
                    1400,
                ),
                step(
                    "«Овальная область»: Shift+M или долгий щелчок по значку",
                    Some("tool:EllipseMarquee"),
                    vec![ui_set(json!({ "tool": "EllipseMarquee" }))],
                    1200,
                ),
                step(
                    "Shift при выделении делает круг",
                    None,
                    vec![engine("select.rect", json!({ "x": ex, "y": ey, "width": ew, "height": eh, "ellipse": true }))],
                    1400,
                ),
                step("Заливка киноварью #BC3823", Some("menu:edit.fill"), vec![engine("edit.fill", json!({ "contents": "color", "color": hex(SHU) }))], 1400),
                step("Ctrl+D — снять выделение", Some("menu:select.deselect"), vec![engine("select.deselect", json!({}))], 1200),
            ],
        ),
    );
}

const STONE: (f32, f32, f32) = (450.0, 400.0, 160.0);

fn stone() -> Canvas {
    let mut c = Canvas::new(PAPER);
    let (sx, sy, sr) = STONE;
    c.fill_ellipse(sx, sy, sr, sr, INK, 1.0);
    c
}

fn basics_opacity(out: &Out) {
    let id = "basics/06-opacity";
    out.image(id, "task.png", &stone());
    let mut r = stone();
    let band: Vec<(f32, f32)> = vec![(130.0, 400.0), (1070.0, 400.0)];
    r.stroke(&band, 60.0, 0.7, AI, 0.45);
    out.image(id, "reference.png", &r);
    out.guide(
        id,
        &guide(
            "Непрозрачность и видимость слоя",
            true,
            vec![
                step("Новый слой для мазка", Some("panel:Layers"), vec![engine("layer.new.layer", json!({ "name": "Мазок" }))], 1200),
                step(
                    "Широкая кисть индиго поверх камня",
                    Some("tool:Brush"),
                    vec![ui_set(json!({ "tool": "Brush" })), brush(120.0, 0.7, 1.0), color(AI), stroke_call(&band, 120.0, AI)],
                    1400,
                ),
                step(
                    "Непрозрачность слоя 45%: камень проступает сквозь мазок",
                    Some("option:layerOpacity"),
                    vec![engine("layer.setProps", json!({ "opacity": 0.45 }))],
                    1800,
                ),
                step("Глаз выключает слой целиком, не стирая его", Some("panel:Layers"), vec![engine("layer.setProps", json!({ "visible": false }))], 1400),
            ],
        ),
    );
}

// ------------------------------------------------------------------ light and shadow

/// Light from the upper left, a little towards the viewer.
const LIGHT: [f32; 3] = [-0.62, -0.62, 0.48];

fn normal(x: f32, y: f32) -> Option<[f32; 3]> {
    let (nx, ny) = ((x - CX) / R, (y - CY) / R);
    let rr = nx * nx + ny * ny;
    (rr <= 1.0).then(|| [nx, ny, (1.0 - rr).sqrt()])
}

fn lambert(n: [f32; 3]) -> f32 {
    let l = LIGHT;
    let len = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
    (n[0] * l[0] + n[1] * l[1] + n[2] * l[2]) / len
}

/// Tone (0 dark – 1 light) of the sphere at a normal.
fn sphere_tone(n: [f32; 3], reflex: bool) -> f32 {
    let d = lambert(n);
    let lit = if d > 0.0 { 0.3 + 0.6 * d.powf(0.8) } else { 0.3 };
    let core = 0.3 - 0.08 * (1.0 - (d + 0.25).abs() / 0.25).max(0.0);
    let mut t = if d > 0.0 { lit } else { core.min(lit) };
    if reflex && d < 0.0 {
        let rim = smoothstep(0.55, 0.95, (n[0] * n[0] + n[1] * n[1]).sqrt());
        let towards_ground = (0.4 * n[0] + 0.9 * n[1]).max(0.0);
        t += 0.22 * rim * towards_ground;
    }
    t
}

fn tone_color(t: f32) -> Rgb {
    mix([0x2B, 0x2A, 0x30], [0xF3, 0xEE, 0xE4], t)
}

fn sun_marker(c: &mut Canvas, x: f32, y: f32) {
    c.fill_ellipse(x, y, 20.0, 20.0, KAKI, 1.0);
    for i in 0..8 {
        let a = i as f32 * TAU / 8.0;
        c.stroke(&[(x + 30.0 * a.cos(), y + 30.0 * a.sin()), (x + 44.0 * a.cos(), y + 44.0 * a.sin())], 2.5, 0.8, KAKI, 1.0);
    }
}

/// Dashed ray from the sun towards the sphere.
fn light_ray(c: &mut Canvas, from: (f32, f32), to: (f32, f32)) {
    for i in 0..9 {
        let (t0, t1) = (i as f32 / 9.0, (i as f32 + 0.5) / 9.0);
        let p = |t: f32| (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
        c.stroke(&[p(t0), p(t1)], 1.8, 0.8, KAKI, 0.8);
    }
}

fn ground(c: &mut Canvas) {
    c.stroke(&[(80.0, GROUND), (1120.0, GROUND)], 2.0, 0.7, PAPER_LINE, 1.0);
}

fn sphere_outline(c: &mut Canvas) {
    c.stroke(&arc(CX, CY, R, R, 0.0, TAU, 160), 2.2, 0.8, INK, 1.0);
}

fn sphere_task() -> Canvas {
    let mut c = Canvas::new(PAPER);
    ground(&mut c);
    sun_marker(&mut c, 150.0, 110.0);
    light_ray(&mut c, (190.0, 150.0), (430.0, 230.0));
    c
}

fn shade(c: &mut Canvas, tone: impl Fn([f32; 3]) -> f32) {
    c.paint(CX - R - 1.0, CY - R - 1.0, CX + R + 1.0, CY + R + 1.0, |x, y| {
        let n = normal(x, y)?;
        Some((tone_color(tone(n)), 1.0))
    });
}

fn light_source(out: &Out) {
    let id = "light/01-light-source";
    let mut task = sphere_task();
    sphere_outline(&mut task);
    out.image(id, "task.png", &task);
    let mut r = sphere_task();
    shade(&mut r, |n| if lambert(n) > 0.25 { 0.72 } else { 0.42 });
    sphere_outline(&mut r);
    out.image(id, "reference.png", &r);
    let lit_side: Vec<Value> =
        [(470.0, 260.0), (520.0, 300.0)].iter().map(|&(x, y)| stroke_call(&arc(x, y, 60.0, 40.0, PI, TAU, 24), 110.0, tone_color(0.78))).collect();
    out.guide(
        id,
        &guide(
            "Источник света и форма",
            true,
            vec![
                step("Свет падает слева сверху — там, где солнце", None, vec![], 1800),
                step("Новый слой «Тон» поверх линий", Some("panel:Layers"), vec![engine("layer.new.layer", json!({ "name": "Тон" }))], 1200),
                step(
                    "Овальной областью выделяем шар",
                    Some("tool:EllipseMarquee"),
                    vec![
                        ui_set(json!({ "tool": "EllipseMarquee" })),
                        engine("select.rect", json!({ "x": CX - R, "y": CY - R, "width": 2.0 * R, "height": 2.0 * R, "ellipse": true })),
                    ],
                    1400,
                ),
                step(
                    "Заливаем ровным средним тоном",
                    Some("menu:edit.fill"),
                    vec![engine("edit.fill", json!({ "contents": "color", "color": hex(tone_color(0.45)) }))],
                    1400,
                ),
                step(
                    "Мягкой светлой кистью высветляем сторону к источнику",
                    Some("tool:Brush"),
                    [vec![ui_set(json!({ "tool": "Brush" })), brush(110.0, 0.0, 0.6)], lit_side].concat(),
                    1600,
                ),
                step("Выделение больше не нужно: Ctrl+D", Some("menu:select.deselect"), vec![engine("select.deselect", json!({}))], 1000),
            ],
        ),
    );
}

fn flat_sphere() -> Canvas {
    let mut c = sphere_task();
    shade(&mut c, |_| 0.62);
    sphere_outline(&mut c);
    c
}

fn light_core_shadow(out: &Out) {
    let id = "light/02-core-shadow";
    out.image(id, "task.png", &flat_sphere());
    let mut r = sphere_task();
    shade(&mut r, |n| sphere_tone(n, false));
    sphere_outline(&mut r);
    out.image(id, "reference.png", &r);
    // The terminator: an arc across the sphere, facing away from the light.
    let terminator = arc(CX - 40.0, CY - 40.0, 190.0, 190.0, -0.2, PI * 0.7, 30);
    let halftone = arc(CX - 60.0, CY - 60.0, 120.0, 120.0, -0.1, PI * 0.65, 24);
    out.guide(
        id,
        &guide(
            "Собственная тень и полутон",
            true,
            vec![
                step("Новый слой «Тени»", Some("panel:Layers"), vec![engine("layer.new.layer", json!({ "name": "Тени" }))], 1200),
                step(
                    "Мягкая кисть: жёсткость 0%, непрозрачность 30%",
                    Some("option:opacity"),
                    vec![ui_set(json!({ "tool": "Brush" })), brush(90.0, 0.0, 0.3), color(tone_color(0.15))],
                    1400,
                ),
                step("Граница света и тени — линия терминатора — проходит через шар дугой", None, vec![stroke_call(&terminator, 90.0, tone_color(0.15))], 1400),
                step(
                    "Слой за слоем темнее: собственная тень",
                    None,
                    vec![stroke_call(&terminator, 70.0, tone_color(0.15)), stroke_call(&terminator, 50.0, tone_color(0.1))],
                    1400,
                ),
                step("Полутон — мягкий переход между светом и тенью", None, vec![brush(120.0, 0.0, 0.2), stroke_call(&halftone, 120.0, tone_color(0.3))], 1600),
            ],
        ),
    );
}

/// The cast shadow on the ground: darker at the contact point, fading to the right.
fn cast_shadow(c: &mut Canvas) {
    let (sx, sy, rx, ry) = (740.0, 568.0, 210.0, 26.0);
    c.paint(sx - rx, sy - ry, sx + rx, sy + ry, |x, y| {
        let d = (((x - sx) / rx).powi(2) + ((y - sy) / ry).powi(2)).sqrt();
        if d >= 1.0 {
            return None;
        }
        let along = ((x - (sx - rx)) / (2.0 * rx)).clamp(0.0, 1.0);
        Some((tone_color(0.12), (0.85 - 0.55 * along) * (1.0 - smoothstep(0.75, 1.0, d))))
    });
}

fn shaded_sphere(reflex: bool, cast: bool, warm_ground: bool) -> Canvas {
    let mut c = sphere_task();
    if warm_ground {
        c.paint(0.0, GROUND, W as f32, H as f32, |_, y| Some((KAKI_100, 0.8 * (1.0 - smoothstep(GROUND, H as f32, y) * 0.5))));
        ground(&mut c);
    }
    if cast {
        cast_shadow(&mut c);
    }
    shade(&mut c, |n| sphere_tone(n, reflex));
    sphere_outline(&mut c);
    c
}

fn light_cast_shadow(out: &Out) {
    let id = "light/03-cast-shadow";
    out.image(id, "task.png", &shaded_sphere(false, false, false));
    out.image(id, "reference.png", &shaded_sphere(false, true, false));
    let shadow_line = |dy: f32| vec![(600.0, 566.0 + dy), (700.0, 568.0 + dy), (850.0, 570.0 + dy), (930.0, 571.0 + dy)];
    out.guide(
        id,
        &guide(
            "Падающая тень от сферы",
            true,
            vec![
                step("Тень ложится в сторону от света: вправо и назад", None, vec![], 1800),
                step(
                    "Новый слой «Тень» — под шаром удобно править отдельно",
                    Some("panel:Layers"),
                    vec![engine("layer.new.layer", json!({ "name": "Тень" }))],
                    1200,
                ),
                step(
                    "Мягкая кисть, тёмный тон, непрозрачность 40%",
                    Some("tool:Brush"),
                    vec![ui_set(json!({ "tool": "Brush" })), brush(60.0, 0.0, 0.4), color(tone_color(0.12))],
                    1200,
                ),
                step("Тень начинается в точке касания…", None, vec![stroke_call(&shadow_line(0.0), 60.0, tone_color(0.12))], 1200),
                step(
                    "…и темнее всего у самого шара: туда свет не попадает совсем",
                    None,
                    vec![brush(36.0, 0.0, 0.6), stroke_call(&[(605.0, 566.0), (690.0, 568.0)], 36.0, tone_color(0.08))],
                    1600,
                ),
            ],
        ),
    );
}

fn light_reflex(out: &Out) {
    let id = "light/04-reflected-light";
    out.image(id, "task.png", &shaded_sphere(false, true, true));
    out.image(id, "reference.png", &shaded_sphere(true, true, true));
    let rim = arc(CX, CY, R - 24.0, R - 24.0, PI * 0.08, PI * 0.62, 24);
    out.guide(
        id,
        &guide(
            "Рефлекс и отражённый свет",
            true,
            vec![
                step("Тёплый пол отражает свет обратно на теневую сторону шара", None, vec![], 1800),
                step("Новый слой «Рефлекс»", Some("panel:Layers"), vec![engine("layer.new.layer", json!({ "name": "Рефлекс" }))], 1200),
                step(
                    "Мягкая кисть, непрозрачность 25%, тон чуть светлее тени",
                    Some("tool:Brush"),
                    vec![ui_set(json!({ "tool": "Brush" })), brush(50.0, 0.0, 0.25), color(mix(tone_color(0.5), KAKI, 0.25))],
                    1400,
                ),
                step("Ведём по краю шара внутри тени, снизу справа", None, vec![stroke_call(&rim, 50.0, mix(tone_color(0.5), KAKI, 0.25))], 1400),
                step(
                    "Рефлекс всегда темнее освещённой стороны — не перестарайтесь",
                    None,
                    vec![stroke_call(&rim, 36.0, mix(tone_color(0.5), KAKI, 0.25))],
                    1600,
                ),
            ],
        ),
    );
}

/// Drapery: vertical folds lit from the left; ridges at 360, 600, 840, valleys at 480, 720.
const CLOTH: [f32; 4] = [240.0, 120.0, 960.0, 700.0];
const VALLEYS: [f32; 2] = [480.0, 720.0];

fn cloth_x(x: f32, y: f32) -> f32 {
    // The folds sway a little from top to bottom.
    x - 10.0 * ((y - CLOTH[1]) / (CLOTH[3] - CLOTH[1]) * PI).sin()
}

fn cloth_tone(x: f32) -> f32 {
    let phase = (x - 360.0) / 240.0 * TAU;
    (0.56 + 0.24 * phase.cos() - 0.08 * phase.sin()).clamp(0.0, 0.8)
}

fn drapery(shaded: bool) -> Canvas {
    let mut c = Canvas::new(PAPER);
    sun_marker(&mut c, 110.0, 400.0);
    let [x0, y0, x1, y1] = CLOTH;
    if shaded {
        c.paint(x0, y0, x1, y1, |x, y| Some((tone_color(cloth_tone(cloth_x(x, y))), 1.0)));
    }
    let edge =
        |x: f32| -> Vec<(f32, f32)> { (0..=40).map(|i| y0 + (y1 - y0) * i as f32 / 40.0).map(|y| (x + 10.0 * ((y - y0) / (y1 - y0) * PI).sin(), y)).collect() };
    for v in VALLEYS {
        c.stroke(&edge(v), 1.6, 0.8, INK, 1.0);
    }
    c.stroke(&edge(x0), 2.0, 0.8, INK, 1.0);
    c.stroke(&edge(x1), 2.0, 0.8, INK, 1.0);
    c.stroke(&[(x0 - 30.0, y0), (x1 + 30.0, y0)], 5.0, 0.8, INK, 1.0);
    c.stroke(&wave(x0 + 10.0, x1 + 10.0, y1, 8.0, 60), 2.0, 0.8, INK, 1.0);
    c
}

fn light_drapery(out: &Out) {
    let id = "light/05-drapery";
    out.image(id, "task.png", &drapery(false));
    out.image(id, "reference.png", &drapery(true));
    let fold =
        |x: f32| -> Vec<(f32, f32)> { (0..=12).map(|i| 140.0 + 540.0 * i as f32 / 12.0).map(|y| (x + 10.0 * ((y - 120.0) / 580.0 * PI).sin(), y)).collect() };
    out.guide(
        id,
        &guide(
            "Свет на сложной форме: драпировка",
            true,
            vec![
                step("Свет слева. Каждая складка — маленький цилиндр со своим светом и тенью", None, vec![], 2000),
                step(
                    "Новый слой «Тон» и общий средний тон ткани",
                    Some("panel:Layers"),
                    vec![
                        engine("layer.new.layer", json!({ "name": "Тон" })),
                        engine("select.rect", json!({ "x": 240, "y": 120, "width": 730, "height": 580 })),
                        engine("edit.fill", json!({ "contents": "color", "color": hex(tone_color(0.56)) })),
                        engine("select.deselect", json!({})),
                    ],
                    1600,
                ),
                step(
                    "Мягкая кисть: в глубине складок тень",
                    Some("tool:Brush"),
                    [
                        vec![ui_set(json!({ "tool": "Brush" })), brush(70.0, 0.0, 0.5)],
                        VALLEYS.iter().map(|v| stroke_call(&fold(*v - 10.0), 70.0, tone_color(0.25))).collect(),
                    ]
                    .concat(),
                    1600,
                ),
                step(
                    "На гребнях — свет",
                    None,
                    [vec![brush(60.0, 0.0, 0.45)], [360.0, 600.0, 840.0].iter().map(|v| stroke_call(&fold(*v - 15.0), 60.0, tone_color(0.85))).collect()]
                        .concat(),
                    1600,
                ),
                step("Правая сторона каждой складки отвёрнута от света — она чуть темнее левой", None, vec![], 1800),
            ],
        ),
    );
}

fn backlight(lit: bool) -> Canvas {
    let mut c = Canvas::new(AI_900);
    c.paint(0.0, GROUND, W as f32, H as f32, |_, _| Some(([0x0F, 0x22, 0x38], 1.0)));
    if lit {
        // Glow of the light hidden behind the sphere.
        c.paint(0.0, 0.0, W as f32, H as f32, |x, y| {
            let d = ((x - CX).powi(2) + (y - CY).powi(2)).sqrt() - R;
            (d > 0.0).then(|| (mix(KAKI, KIN_100, 0.4), 0.85 * (-d / 110.0).exp()))
        });
    }
    c.fill_ellipse(CX, CY, R, R, SUMI, 1.0);
    if lit {
        c.paint(CX - R, CY - R, CX + R, CY + R, |x, y| {
            let d = R - ((x - CX).powi(2) + (y - CY).powi(2)).sqrt();
            if !(0.0..30.0).contains(&d) {
                return None;
            }
            // Strongest on top, where the light behind grazes the edge.
            let top = (0.35 + 0.65 * ((CY - y) / R).max(0.0)).min(1.0);
            Some((KIN_100, top * (1.0 - d / 30.0).powf(1.2)))
        });
    }
    c
}

fn light_backlight(out: &Out) {
    let id = "light/06-backlight";
    out.image(id, "task.png", &backlight(false));
    out.image(id, "reference.png", &backlight(true));
    let glow: Vec<Value> =
        [230.0, 260.0].iter().map(|rr| stroke_call(&arc(CX, CY, *rr, *rr, PI * 1.05, PI * 1.95, 30), 90.0, mix(KAKI, KIN_100, 0.4))).collect();
    let rim = arc(CX, CY, R - 6.0, R - 6.0, PI * 1.1, PI * 1.9, 30);
    out.guide(
        id,
        &guide(
            "Контровой свет и атмосфера",
            true,
            vec![
                step("Источник за шаром: мы видим силуэт и светящийся контур", None, vec![], 2000),
                step("Новый слой «Свечение»", Some("panel:Layers"), vec![engine("layer.new.layer", json!({ "name": "Свечение" }))], 1200),
                step(
                    "Большая мягкая кисть, тёплый свет, непрозрачность 30%: ореол вокруг шара",
                    Some("tool:Brush"),
                    [vec![ui_set(json!({ "tool": "Brush" })), brush(90.0, 0.0, 0.3)], glow].concat(),
                    1600,
                ),
                step("Тонкая кисть — контровой свет по верхнему краю силуэта", None, vec![brush(14.0, 0.3, 0.9), stroke_call(&rim, 14.0, KIN_100)], 1600),
                step("Середина силуэта остаётся тёмной — так читается контражур", None, vec![], 1800),
            ],
        ),
    );
}
