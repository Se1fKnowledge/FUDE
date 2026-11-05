//! Hint ghost shapes drawn on the canvas.
//!
//! The shapes go onto their own translucent layer, so the student paints next to them, not
//! over them, and removing the layer removes the hint.

use serde_json::{Value, json};

use crate::control::Call;
use crate::model::{Hint, Point, Shape};

/// Name of the layer the hint draws on.
pub const LAYER_NAME: &str = "FUDE · подсказка";
/// Ghost colour: shu/500.
pub const COLOR: &str = "#BC3823";
/// Opacity of a ghost shape that sets none.
pub const OPACITY: f64 = 0.35;

const ELLIPSE_SEGMENTS: usize = 64;

/// Calls that draw the hint's shapes on a `width`×`height` canvas, then make `return_to`
/// (the layer the student was on) active again. Empty when the hint has no shapes.
///
/// The layer takes the opacity of the most opaque shape; each stroke carries the rest, so a
/// faint shape next to a strong one keeps its own look.
pub fn show(hint: &Hint, width: u32, height: u32, return_to: Option<u64>) -> Vec<Call> {
    if hint.shapes.is_empty() {
        return vec![];
    }
    let (w, h) = (f64::from(width.max(1)), f64::from(height.max(1)));
    let default_size = (w.min(h) * 0.006).round().max(2.0);
    let opacity = |s: &Shape| s.style().opacity.unwrap_or(OPACITY).clamp(0.05, 1.0);
    let layer_opacity = hint.shapes.iter().map(opacity).fold(0.0, f64::max);
    let mut calls = vec![Call::engine("layer.new.layer", json!({ "name": LAYER_NAME, "opacity": round2(layer_opacity), "color": "red" }))];
    for shape in &hint.shapes {
        let style = shape.style();
        let size = style.width.map_or(default_size, |v| v.clamp(0.5, 200.0));
        let color = style.color.as_deref().unwrap_or(COLOR);
        let stroke_opacity = round2((opacity(shape) / layer_opacity).min(1.0));
        let px = |[x, y]: Point| [x * w, y * h];
        let mut lines: Vec<Vec<[f64; 2]>> = vec![outline(shape).into_iter().map(px).collect()];
        if let Shape::Arrow { from, to, .. } = shape {
            lines.extend(arrow_head(px(*from), px(*to), size));
        }
        if style.dashed {
            lines = lines.iter().flat_map(|l| dashes(l, size * 4.0, size * 3.0)).collect();
        }
        for line in lines {
            let points: Vec<Value> = line.into_iter().map(|[x, y]| json!([round(x), round(y), 1.0])).collect();
            calls.push(Call::engine(
                "paint.stroke",
                json!({ "points": points, "size": size, "hardness": 1.0, "opacity": stroke_opacity, "flow": 1.0, "color": color }),
            ));
        }
    }
    if let Some(layer) = return_to {
        calls.push(Call::engine("layer.select", json!({ "layer": layer })));
    }
    calls
}

/// The two short lines of an arrow head at `to`, in pixels.
fn arrow_head(from: [f64; 2], to: [f64; 2], size: f64) -> Vec<Vec<[f64; 2]>> {
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let len = dx.hypot(dy);
    if len < 1e-6 {
        return vec![];
    }
    let head = (size * 4.0).max(len * 0.12).min(len * 0.5);
    let (ux, uy) = (dx / len, dy / len);
    let side = |sign: f64| {
        let (c, s) = (0.5_f64.cos(), sign * 0.5_f64.sin());
        let (rx, ry) = (ux * c - uy * s, ux * s + uy * c);
        vec![to, [to[0] - rx * head, to[1] - ry * head]]
    };
    vec![side(1.0), side(-1.0)]
}

/// A polyline cut into dashes `on` long with `off` gaps (pixels).
pub fn dashes(line: &[[f64; 2]], on: f64, off: f64) -> Vec<Vec<[f64; 2]>> {
    let (on, off) = (on.max(1.0), off.max(1.0));
    let mut out = Vec::new();
    let mut current: Vec<[f64; 2]> = Vec::new();
    let (mut drawing, mut left) = (true, on);
    for pair in line.windows(2) {
        let (mut a, b) = (pair[0], pair[1]);
        let mut seg = (b[0] - a[0]).hypot(b[1] - a[1]);
        while seg > 1e-9 {
            if drawing && current.is_empty() {
                current.push(a);
            }
            let step = seg.min(left);
            let t = step / seg;
            let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
            if drawing {
                current.push(p);
            }
            left -= step;
            seg -= step;
            a = p;
            if left <= 1e-9 {
                if drawing && current.len() > 1 {
                    out.push(std::mem::take(&mut current));
                }
                current.clear();
                drawing = !drawing;
                left = if drawing { on } else { off };
            }
        }
    }
    if drawing && current.len() > 1 {
        out.push(current);
    }
    out
}

/// Calls that delete every hint layer in `layers` (the `document.layers` tree of
/// `ui.inspect`, or the reply of `layer.list`).
pub fn hide(layers: &Value) -> Vec<Call> {
    let mut ids = Vec::new();
    collect(layers, &mut ids);
    ids.into_iter().map(|id| Call::engine("layer.delete", json!({ "layer": id }))).collect()
}

fn collect(layers: &Value, ids: &mut Vec<u64>) {
    let Some(list) = layers.as_array() else { return };
    for layer in list {
        if layer.get("name").and_then(Value::as_str) == Some(LAYER_NAME)
            && let Some(id) = layer.get("id").and_then(Value::as_u64)
        {
            ids.push(id);
        }
        if let Some(children) = layer.get("children") {
            collect(children, ids);
        }
    }
}

/// The shape as a polyline in canvas fractions (an arrow's head is added by [`show`]).
pub fn outline(shape: &Shape) -> Vec<Point> {
    match shape {
        Shape::Line { from, to, .. } | Shape::Arrow { from, to, .. } => vec![*from, *to],
        Shape::Path { points, .. } => points.clone(),
        Shape::Rect { rect: [x, y, w, h], .. } => vec![[*x, *y], [x + w, *y], [x + w, y + h], [*x, y + h], [*x, *y]],
        Shape::Ellipse { rect: [x, y, w, h], .. } => {
            let (cx, cy, rx, ry) = (x + w / 2.0, y + h / 2.0, w / 2.0, h / 2.0);
            (0..=ELLIPSE_SEGMENTS)
                .map(|i| {
                    let t = std::f64::consts::TAU * i as f64 / ELLIPSE_SEGMENTS as f64;
                    [cx + rx * t.cos(), cy + ry * t.sin()]
                })
                .collect()
        }
    }
}

fn round(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guide::check_call;
    use crate::model::Style;

    fn hint(shapes: Vec<Shape>) -> Hint {
        Hint { text: "x".into(), target: None, shapes }
    }

    fn line(from: Point, to: Point) -> Shape {
        Shape::Line { from, to, style: Style::default() }
    }

    #[test]
    fn a_hint_without_shapes_draws_nothing() {
        assert!(show(&hint(vec![]), 100, 100, Some(3)).is_empty());
    }

    #[test]
    fn shapes_go_on_a_translucent_layer_and_return_to_the_students_layer() {
        let calls = show(&hint(vec![line([0.0, 0.5], [1.0, 0.5]), Shape::Rect { rect: [0.1, 0.1, 0.2, 0.2], style: Style::default() }]), 1000, 500, Some(7));
        assert_eq!(calls.len(), 4);
        assert_eq!(calls[0].command(), Some("layer.new.layer"));
        assert_eq!(calls[0].params["params"]["name"], LAYER_NAME);
        assert_eq!(calls[0].params["params"]["opacity"], OPACITY);
        assert_eq!(calls[1].params["params"]["points"], json!([[0.0, 250.0, 1.0], [1000.0, 250.0, 1.0]]));
        assert_eq!(calls[1].params["params"]["color"], COLOR);
        assert_eq!(calls[1].params["params"]["size"], 3.0);
        assert_eq!(calls[1].params["params"]["opacity"], 1.0);
        assert_eq!(calls[2].params["params"]["points"].as_array().map(Vec::len), Some(5));
        assert_eq!(calls[3], Call::engine("layer.select", json!({"layer": 7})));
        for call in &calls {
            assert_eq!(check_call(call), Ok(()), "hint calls are guide-safe: {call:?}");
        }
    }

    #[test]
    fn each_shape_keeps_its_own_colour_width_and_opacity() {
        let blue = Style { color: Some("#2F5D8C".into()), width: Some(12.0), opacity: Some(0.7), dashed: false };
        let calls = show(&hint(vec![line([0.0, 0.0], [1.0, 0.0]), Shape::Line { from: [0.0, 1.0], to: [1.0, 1.0], style: blue }]), 100, 100, None);
        assert_eq!(calls[0].params["params"]["opacity"], 0.7, "the layer takes the most opaque shape");
        assert_eq!(calls[1].params["params"]["opacity"], 0.5, "35% of 70%: the default shape stays as faint as before");
        assert_eq!(calls[2].params["params"]["color"], "#2F5D8C");
        assert_eq!(calls[2].params["params"]["size"], 12.0);
        assert_eq!(calls[2].params["params"]["opacity"], 1.0);
    }

    #[test]
    fn an_arrow_gets_a_head_and_a_dashed_line_breaks_into_strokes() {
        let arrow = show(&hint(vec![Shape::Arrow { from: [0.0, 0.5], to: [1.0, 0.5], style: Style::default() }]), 1000, 1000, None);
        assert_eq!(arrow.len(), 4, "layer, shaft and two head strokes");
        for head in &arrow[2..] {
            let p = &head.params["params"]["points"];
            assert_eq!(p[0], json!([1000.0, 500.0, 1.0]), "the head starts at the tip");
            assert!(p[1][0].as_f64().is_some_and(|x| x < 1000.0), "and points back along the shaft");
        }
        let dashed = Style { width: Some(10.0), dashed: true, ..Style::default() };
        let calls = show(&hint(vec![Shape::Line { from: [0.0, 0.5], to: [1.0, 0.5], style: dashed }]), 1000, 1000, None);
        // 1000 px with 40 px dashes and 30 px gaps: 15 dashes, the last one cut short.
        assert_eq!(calls.len(), 1 + 15);
    }

    #[test]
    fn dashes_follow_a_bent_line() {
        let d = dashes(&[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]], 4.0, 2.0);
        assert_eq!(d.len(), 4);
        assert_eq!(d[0], vec![[0.0, 0.0], [4.0, 0.0]]);
        assert_eq!(d[1], vec![[6.0, 0.0], [10.0, 0.0]], "a dash ends exactly at the corner");
        assert_eq!(d[2], vec![[10.0, 2.0], [10.0, 6.0]]);
        assert_eq!(d[3], vec![[10.0, 8.0], [10.0, 10.0]]);
        assert!(dashes(&[[0.0, 0.0]], 4.0, 2.0).is_empty());
    }

    #[test]
    fn an_ellipse_is_a_closed_ring_inside_its_box() {
        let points = outline(&Shape::Ellipse { rect: [0.2, 0.4, 0.4, 0.2], style: Style::default() });
        assert_eq!(points.len(), ELLIPSE_SEGMENTS + 1);
        let (first, last) = (points[0], points[ELLIPSE_SEGMENTS]);
        assert!((first[0] - last[0]).abs() < 1e-9 && (first[1] - last[1]).abs() < 1e-9);
        for [x, y] in points {
            assert!((0.2 - 1e-9..=0.6 + 1e-9).contains(&x) && (0.4 - 1e-9..=0.6 + 1e-9).contains(&y));
        }
    }

    #[test]
    fn hiding_finds_hint_layers_in_groups() {
        let layers = json!([
            {"id": 1, "name": "Фон"},
            {"id": 4, "name": LAYER_NAME},
            {"id": 2, "name": "Группа", "children": [{"id": 9, "name": LAYER_NAME}, {"id": 3, "name": "Тень"}]}
        ]);
        let calls = hide(&layers);
        assert_eq!(calls, vec![Call::engine("layer.delete", json!({"layer": 4})), Call::engine("layer.delete", json!({"layer": 9}))]);
        assert!(hide(&json!(null)).is_empty());
    }
}
