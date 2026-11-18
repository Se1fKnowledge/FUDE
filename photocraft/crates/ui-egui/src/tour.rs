//! FUDE build: a guided tour driven by the host app over the control channel.
//!
//! The host dims the window, leaves one place lit (a tool, a zone of the window, the canvas) and
//! shows a card next to it with a short text, a small task and a few buttons. While `block` is
//! on, presses and keys outside the lit place and the card are dropped, so the student can only
//! do what the step asks for. The host reads button presses and what the pointer did with
//! `tour.poll`; it decides itself when a step is done (it sees the editor's state already).
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use egui::{Align2, Color32, Context, FontId, Id, LayerId, Order, Pos2, Rect, RichText, Stroke, StrokeKind, Vec2, pos2, vec2};
use serde_json::{Value, json};

use crate::PhotocraftApp;

/// Whether a tour is on: `mark` is called for every tool button each frame, so it must be cheap
/// when no tour runs.
static ACTIVE: AtomicBool = AtomicBool::new(false);

const INDIGO: Color32 = Color32::from_rgb(0x1F, 0x44, 0x70);
const INDIGO_DEEP: Color32 = Color32::from_rgb(0x16, 0x30, 0x4F);
const INDIGO_PALE: Color32 = Color32::from_rgb(0xDC, 0xE7, 0xF1);
const INDIGO_LIGHT: Color32 = Color32::from_rgb(0x8F, 0xB2, 0xD6);
const VERMILION: Color32 = Color32::from_rgb(0xBC, 0x38, 0x23);
const PERSIMMON: Color32 = Color32::from_rgb(0xE8, 0x89, 0x3B);
const CARD_WIDTH: f32 = 324.0;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Button {
    pub id: String,
    pub label: String,
    pub primary: bool,
}

/// One step of the tour as the host set it, and what happened since the last `tour.poll`.
#[derive(Clone, Debug, Default)]
pub struct Tour {
    /// `tool:Brush`, `menu`, `options`, `toolbar`, `colors`, `canvas`, `dock`, `status`, or empty
    /// for no lit place (the whole window is dim).
    pub target: String,
    pub title: String,
    pub body: String,
    pub task: String,
    pub status: String,
    pub done: bool,
    pub buttons: Vec<Button>,
    /// Drop input outside the lit place and the card.
    pub block: bool,
    /// Keys that still work while blocked (`B`, `[`, `]`…).
    pub keys: Vec<String>,
    /// Shown for a moment after a press outside the lit place.
    pub toast: String,
    events: Vec<String>,
    outside: u32,
    inside_s: f32,
    hole: Option<Rect>,
    card: Option<Rect>,
    pointer: Option<Pos2>,
    toast_until: f64,
    /// When the previous frame was drawn, to count real time over the lit place.
    last_frame: f64,
}

#[derive(Clone, Default)]
struct Marks(HashMap<String, Rect>);

fn marks_id() -> Id {
    Id::new("fude-tour-marks")
}

/// Remember where a place of the window is on screen this frame. Cheap when no tour runs.
pub fn mark(ctx: &Context, id: &str, rect: Rect) {
    if ACTIVE.load(Ordering::Relaxed) {
        ctx.data_mut(|d| d.get_temp_mut_or_default::<Marks>(marks_id()).0.insert(id.to_string(), rect));
    }
}

/// `tour.set`: a missing field keeps nothing from the previous step (each call is a whole step),
/// but what the student did so far (button presses not read yet) is kept while the target stays.
pub fn set(app: &mut PhotocraftApp, p: &Value) -> Result<(), String> {
    const FIELDS: [&str; 10] = ["target", "title", "body", "task", "status", "done", "buttons", "block", "keys", "toast"];
    let Some(obj) = p.as_object() else { return Err("tour.set takes an object".into()) };
    if let Some(k) = obj.keys().find(|k| !FIELDS.contains(&k.as_str())) {
        return Err(format!("unknown field `{k}` (fields: {})", FIELDS.join(", ")));
    }
    let text = |k: &str| match obj.get(k) {
        None | Some(Value::Null) => Ok(String::new()),
        Some(Value::String(s)) => Ok(s.clone()),
        Some(_) => Err(format!("`{k}` must be a string")),
    };
    let flag = |k: &str| match obj.get(k) {
        None | Some(Value::Null) => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(format!("`{k}` must be true or false")),
    };
    let mut buttons = Vec::new();
    if let Some(list) = obj.get("buttons").filter(|v| !v.is_null()) {
        let Some(list) = list.as_array() else { return Err("`buttons` must be a list".into()) };
        for b in list {
            let (Some(id), Some(label)) = (b.get("id").and_then(Value::as_str), b.get("label").and_then(Value::as_str)) else {
                return Err("a button needs `id` and `label`".into());
            };
            buttons.push(Button { id: id.into(), label: label.into(), primary: b.get("primary").and_then(Value::as_bool).unwrap_or(false) });
        }
    }
    let mut keys = Vec::new();
    if let Some(list) = obj.get("keys").filter(|v| !v.is_null()) {
        let Some(list) = list.as_array() else { return Err("`keys` must be a list of key names".into()) };
        for k in list {
            let Some(k) = k.as_str() else { return Err("`keys` must be a list of key names".into()) };
            keys.push(k.to_string());
        }
    }
    let next = Tour {
        target: text("target")?,
        title: text("title")?,
        body: text("body")?,
        task: text("task")?,
        status: text("status")?,
        done: flag("done")?,
        buttons,
        block: flag("block")?,
        keys,
        toast: text("toast")?,
        ..Tour::default()
    };
    let prev = app.tour.take();
    app.tour = Some(match prev {
        Some(prev) if prev.target == next.target => Tour {
            events: prev.events,
            outside: prev.outside,
            inside_s: prev.inside_s,
            hole: prev.hole,
            card: prev.card,
            pointer: prev.pointer,
            last_frame: prev.last_frame,
            ..next
        },
        Some(prev) => Tour { card: prev.card, pointer: prev.pointer, last_frame: prev.last_frame, ..next },
        None => next,
    });
    ACTIVE.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn clear(app: &mut PhotocraftApp) {
    app.tour = None;
    ACTIVE.store(false, Ordering::Relaxed);
}

/// `tour.poll`: what happened since the last call. Button presses and the count of dropped
/// presses are handed over once; the time over the lit place keeps growing while the target stays.
pub fn poll(app: &mut PhotocraftApp) -> Value {
    match app.tour.as_mut() {
        None => json!({"active": false}),
        Some(t) => json!({
            "active": true,
            "target": t.target,
            "found": t.hole.is_some(),
            "events": std::mem::take(&mut t.events),
            "outside": std::mem::take(&mut t.outside),
            "inside": t.inside_s,
        }),
    }
}

/// The key cap behind a key: with Shift held `[` arrives as `{`, and a step that allows the
/// cap allows it with Shift too (Shift+[ softens the brush).
fn cap(key: egui::Key) -> egui::Key {
    match key {
        egui::Key::OpenCurlyBracket => egui::Key::OpenBracket,
        egui::Key::CloseCurlyBracket => egui::Key::CloseBracket,
        other => other,
    }
}

fn key_allowed(t: &Tour, key: egui::Key) -> bool {
    let key = cap(key);
    t.keys.iter().any(|k| k.eq_ignore_ascii_case(key.name()) || k.eq_ignore_ascii_case(key.symbol_or_name()))
}

/// Drop what the step doesn't allow, before any widget sees it. A popup opened from the lit
/// control (a dropdown, a flyout) and a focused text field stay usable.
pub fn filter_input(app: &mut PhotocraftApp, ctx: &Context, raw: &mut egui::RawInput) {
    let Some(t) = app.tour.as_mut() else { return };
    for e in &raw.events {
        match e {
            egui::Event::PointerMoved(p) | egui::Event::PointerButton { pos: p, .. } => t.pointer = Some(*p),
            egui::Event::PointerGone => t.pointer = None,
            _ => {}
        }
    }
    if !t.block {
        return;
    }
    let free = egui::Popup::is_any_open(ctx) || ctx.text_edit_focused();
    let allowed = |p: Pos2, t: &Tour| t.hole.is_some_and(|h| h.contains(p)) || t.card.is_some_and(|c| c.expand(4.0).contains(p));
    let now = ctx.input(|i| i.time);
    let mut pointer = t.pointer;
    let mut dropped = 0;
    raw.events.retain(|e| match e {
        egui::Event::PointerMoved(p) => {
            pointer = Some(*p);
            true
        }
        egui::Event::PointerButton { pos, pressed, .. } => {
            let keep = !*pressed || free || allowed(*pos, t);
            if !keep {
                dropped += 1;
            }
            keep
        }
        egui::Event::MouseWheel { .. } | egui::Event::Zoom(_) => free || pointer.is_some_and(|p| allowed(p, t)),
        egui::Event::Key { key, .. } => free || key_allowed(t, *key),
        egui::Event::Text(_) | egui::Event::Paste(_) => free,
        _ => true,
    });
    if dropped > 0 {
        t.outside += dropped;
        t.toast_until = now + 2.2;
    }
}

fn dim(painter: &egui::Painter, screen: Rect, hole: Option<Rect>) {
    let shade = Color32::from_rgba_unmultiplied(6, 7, 11, 178);
    let Some(h) = hole else {
        painter.rect_filled(screen, 0.0, shade);
        return;
    };
    for r in [
        Rect::from_min_max(screen.min, pos2(screen.right(), h.top())),
        Rect::from_min_max(pos2(screen.left(), h.bottom()), screen.max),
        Rect::from_min_max(pos2(screen.left(), h.top()), pos2(h.left(), h.bottom())),
        Rect::from_min_max(pos2(h.right(), h.top()), pos2(screen.right(), h.bottom())),
    ] {
        if r.is_positive() {
            painter.rect_filled(r, 0.0, shade);
        }
    }
    painter.rect_stroke(h.expand(6.0), 9.0, Stroke::new(1.5, INDIGO_LIGHT.gamma_multiply(0.4)), StrokeKind::Outside);
    painter.rect_stroke(h, 4.0, Stroke::new(2.0, INDIGO_LIGHT), StrokeKind::Outside);
}

/// Next to the lit place without covering it: to the right, to the left, below, above; over the
/// right edge when nothing fits (a canvas that fills the window).
fn card_pos(screen: Rect, hole: Option<Rect>, size: Vec2) -> Pos2 {
    let gap = 26.0;
    let fit = |p: Pos2| screen.shrink(8.0).contains_rect(Rect::from_min_size(p, size));
    let clamp_y = |y: f32| y.clamp(screen.top() + 12.0, (screen.bottom() - size.y - 12.0).max(screen.top() + 12.0));
    let clamp_x = |x: f32| x.clamp(screen.left() + 12.0, (screen.right() - size.x - 12.0).max(screen.left() + 12.0));
    let Some(h) = hole else { return pos2(screen.center().x - size.x / 2.0, screen.center().y - size.y / 2.0) };
    let y = clamp_y(h.center().y - size.y / 2.0);
    let x = clamp_x(h.center().x - size.x / 2.0);
    for p in [pos2(h.right() + gap, y), pos2(h.left() - gap - size.x, y), pos2(x, h.bottom() + gap), pos2(x, h.top() - gap - size.y)] {
        if fit(p) {
            return p;
        }
    }
    pos2(screen.right() - size.x - 12.0, y)
}

/// Draw the tour over the finished frame. Called last in the frame, after every panel marked
/// its place.
pub fn show(app: &mut PhotocraftApp, ctx: &Context) {
    ACTIVE.store(app.tour.is_some(), Ordering::Relaxed);
    let Some(t) = app.tour.as_mut() else { return };
    let screen = ctx.content_rect();
    let marked = ctx.data(|d| d.get_temp::<Marks>(marks_id())).and_then(|m| m.0.get(&t.target).copied());
    let hole = marked.map(|r| r.expand(3.0).intersect(screen)).filter(|r| r.is_positive());
    t.hole = hole;
    // Real time between frames: egui's own `stable_dt` is a guess while the app only repaints on
    // input, and a resting pointer must count too.
    let now = ctx.input(|i| i.time);
    let dt = (now - t.last_frame).clamp(0.0, 0.25) as f32;
    t.last_frame = now;
    if let (Some(h), Some(p)) = (hole, t.pointer)
        && h.contains(p)
    {
        t.inside_s += dt;
    }
    dim(&ctx.layer_painter(LayerId::new(Order::Middle, Id::new("fude-tour-shade"))), screen, hole);

    let size = t.card.map_or(vec2(CARD_WIDTH, 220.0), |c| c.size());
    let pos = card_pos(screen, hole, size);
    let mut clicked = None;
    let area = egui::Area::new(Id::new("fude-tour-card")).order(Order::Tooltip).fixed_pos(pos).interactable(true).show(ctx, |ui| {
        egui::Frame::new()
            .fill(INDIGO)
            .corner_radius(14)
            .inner_margin(egui::Margin::same(18))
            .shadow(egui::Shadow { offset: [0, 10], blur: 28, spread: 0, color: Color32::from_black_alpha(110) })
            .show(ui, |ui| {
                let inner = CARD_WIDTH - 36.0;
                ui.set_width(inner);
                ui.spacing_mut().item_spacing = vec2(8.0, 10.0);
                ui.label(RichText::new(&t.title).size(19.0).strong().color(Color32::WHITE));
                if !t.body.is_empty() {
                    ui.label(RichText::new(&t.body).size(13.0).color(INDIGO_PALE));
                }
                if !t.task.is_empty() {
                    egui::Frame::new().fill(INDIGO_DEEP).corner_radius(10).inner_margin(egui::Margin::symmetric(12, 10)).show(ui, |ui| {
                        ui.set_width(inner - 24.0);
                        ui.horizontal_top(|ui| {
                            let (dot, _) = ui.allocate_exact_size(vec2(16.0, 18.0), egui::Sense::hover());
                            let c = dot.center();
                            if t.done {
                                ui.painter().circle_filled(c, 8.0, INDIGO_LIGHT);
                                let s = Stroke::new(2.0, INDIGO_DEEP);
                                ui.painter().line_segment([c + vec2(-4.0, 0.5), c + vec2(-1.2, 3.2)], s);
                                ui.painter().line_segment([c + vec2(-1.2, 3.2), c + vec2(4.2, -3.0)], s);
                            } else {
                                ui.painter().circle_stroke(c, 7.0, Stroke::new(2.0, PERSIMMON));
                                ui.painter().circle_filled(c, 2.6, PERSIMMON);
                            }
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 2.0;
                                ui.set_width(inner - 24.0 - 26.0);
                                ui.label(RichText::new(&t.task).size(13.0).strong().color(Color32::WHITE));
                                if !t.status.is_empty() {
                                    ui.label(RichText::new(&t.status).size(12.0).color(if t.done { INDIGO_LIGHT } else { PERSIMMON }));
                                }
                            });
                        });
                    });
                }
                if !t.buttons.is_empty() {
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 14.0;
                        let (plain, primary): (Vec<&Button>, Vec<&Button>) = t.buttons.iter().partition(|b| !b.primary);
                        for b in plain {
                            let label = RichText::new(&b.label).size(13.0).color(INDIGO_PALE);
                            if ui.add(egui::Button::new(label).frame(false)).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                clicked = Some(b.id.clone());
                            }
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            for b in primary {
                                let label = RichText::new(&b.label).size(13.0).strong().color(Color32::WHITE);
                                let button = egui::Button::new(label).fill(VERMILION).corner_radius(10).min_size(vec2(96.0, 36.0));
                                if ui.add(button).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                    clicked = Some(b.id.clone());
                                }
                            }
                        });
                    });
                }
            });
    });
    t.card = Some(area.response.rect);
    if let Some(id) = clicked {
        t.events.push(id);
    }
    if !t.toast.is_empty() && now < t.toast_until {
        let painter = ctx.layer_painter(LayerId::new(Order::Tooltip, Id::new("fude-tour-toast")));
        let galley = painter.layout_no_wrap(t.toast.clone(), FontId::proportional(13.0), Color32::from_rgb(0xE8, 0xE4, 0xDC));
        let center = pos2(screen.center().x, screen.bottom() - 54.0);
        let pill = Rect::from_center_size(center, galley.size() + vec2(28.0, 16.0));
        painter.rect_filled(pill, pill.height() / 2.0, Color32::from_rgb(0x23, 0x26, 0x2D));
        painter.rect_stroke(pill, pill.height() / 2.0, Stroke::new(1.0, Color32::from_rgb(0x3A, 0x3E, 0x48)), StrokeKind::Inside);
        painter.galley(Align2::CENTER_CENTER.pos_in_rect(&pill) - galley.size() / 2.0, galley, Color32::WHITE);
        ctx.request_repaint();
    }
    // The card's size is known a frame late and the time over the lit place grows while the
    // pointer rests there, so keep frames coming while a tour is on.
    ctx.request_repaint_after(std::time::Duration::from_millis(120));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> PhotocraftApp {
        PhotocraftApp::new(photocraft_engine::Session::new(), Default::default())
    }

    fn press(x: f32, y: f32) -> egui::Event {
        egui::Event::PointerButton { pos: pos2(x, y), button: egui::PointerButton::Primary, pressed: true, modifiers: egui::Modifiers::NONE }
    }

    fn key(key: egui::Key) -> egui::Event {
        egui::Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE }
    }

    #[test]
    fn a_step_replaces_the_previous_one_and_keeps_unread_presses_on_the_same_target() {
        let mut app = app();
        set(&mut app, &json!({"target": "tool:Brush", "title": "Brush", "buttons": [{"id": "skip", "label": "Skip"}], "block": true, "keys": ["B"]})).unwrap();
        app.tour.as_mut().unwrap().events.push("skip".into());
        set(&mut app, &json!({"target": "tool:Brush", "title": "Brush", "done": true})).unwrap();
        let t = app.tour.as_ref().unwrap();
        assert!(t.done && !t.block && t.buttons.is_empty(), "each call is a whole step");
        assert_eq!(poll(&mut app)["events"], json!(["skip"]));
        assert_eq!(poll(&mut app)["events"], json!([]), "handed over once");
        set(&mut app, &json!({"target": "canvas"})).unwrap();
        app.tour.as_mut().unwrap().events.push("next".into());
        set(&mut app, &json!({"target": "toolbar"})).unwrap();
        assert_eq!(poll(&mut app)["events"], json!([]), "another target starts clean");
        clear(&mut app);
        assert_eq!(poll(&mut app), json!({"active": false}));
    }

    #[test]
    fn bad_steps_are_refused_whole() {
        let mut app = app();
        assert!(set(&mut app, &json!({"target": "canvas", "colour": "red"})).unwrap_err().contains("unknown field"));
        assert!(set(&mut app, &json!({"block": "yes"})).is_err());
        assert!(set(&mut app, &json!({"buttons": [{"id": "next"}]})).is_err());
        assert!(app.tour.is_none());
    }

    #[test]
    fn a_blocked_step_drops_presses_and_keys_outside_the_lit_place() {
        let ctx = Context::default();
        let mut app = app();
        set(&mut app, &json!({"target": "tool:Brush", "block": true, "keys": ["B", "["], "toast": "only the lit place works"})).unwrap();
        let t = app.tour.as_mut().unwrap();
        t.hole = Some(Rect::from_min_size(pos2(0.0, 100.0), vec2(40.0, 40.0)));
        t.card = Some(Rect::from_min_size(pos2(80.0, 80.0), vec2(300.0, 200.0)));
        let mut raw = egui::RawInput {
            events: vec![
                press(20.0, 120.0),
                press(500.0, 500.0),
                press(100.0, 100.0),
                key(egui::Key::B),
                key(egui::Key::E),
                key(egui::Key::OpenBracket),
                // Shift+[ arrives as `{`: the same key cap, so it passes too.
                key(egui::Key::OpenCurlyBracket),
                key(egui::Key::CloseCurlyBracket),
                egui::Event::Text("e".into()),
            ],
            ..Default::default()
        };
        filter_input(&mut app, &ctx, &mut raw);
        let kept: Vec<String> = raw
            .events
            .iter()
            .map(|e| match e {
                egui::Event::PointerButton { pos, .. } => format!("press {}", pos.x),
                egui::Event::Key { key, .. } => format!("key {}", key.name()),
                other => format!("{other:?}"),
            })
            .collect();
        assert_eq!(kept, ["press 20", "press 100", "key B", "key OpenBracket", "key OpenCurlyBracket"]);
        assert_eq!(poll(&mut app)["outside"], 1);
        // Not blocked: everything passes.
        set(&mut app, &json!({"target": "tool:Brush"})).unwrap();
        let mut raw = egui::RawInput { events: vec![press(500.0, 500.0), key(egui::Key::E)], ..Default::default() };
        filter_input(&mut app, &ctx, &mut raw);
        assert_eq!(raw.events.len(), 2);
    }

    #[test]
    fn the_card_sits_beside_the_lit_place_and_never_over_it() {
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(1440.0, 900.0));
        let size = vec2(324.0, 220.0);
        let tool = Rect::from_min_size(pos2(4.0, 360.0), vec2(34.0, 34.0));
        let menu = Rect::from_min_size(pos2(0.0, 0.0), vec2(1440.0, 30.0));
        let status = Rect::from_min_size(pos2(0.0, 876.0), vec2(1440.0, 24.0));
        let dock = Rect::from_min_size(pos2(1150.0, 70.0), vec2(290.0, 800.0));
        for hole in [tool, menu, status, dock] {
            let card = Rect::from_min_size(card_pos(screen, Some(hole), size), size);
            assert!(!card.intersects(hole), "{hole:?} is covered by {card:?}");
            assert!(screen.contains_rect(card));
        }
        assert_eq!(card_pos(screen, Some(tool), size).x, tool.right() + 26.0, "a tool's card is to its right");
        assert!(screen.contains_rect(Rect::from_min_size(card_pos(screen, None, size), size)));
    }
}
