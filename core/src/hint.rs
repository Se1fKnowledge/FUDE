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
/// Opacity of the hint layer.
pub const OPACITY: f64 = 0.35;

const ELLIPSE_SEGMENTS: usize = 64;

/// Calls that draw the hint's shapes on a `width`×`height` canvas, then make `return_to`
/// (the layer the student was on) active again. Empty when the hint has no shapes.
pub fn show(hint: &Hint, width: u32, height: u32, return_to: Option<u64>) -> Vec<Call> {
    if hint.shapes.is_empty() {
        return vec![];
    }
    let (w, h) = (f64::from(width.max(1)), f64::from(height.max(1)));
    let size = (w.min(h) * 0.006).round().max(2.0);
    let mut calls = vec![Call::engine("layer.new.layer", json!({ "name": LAYER_NAME, "opacity": OPACITY, "color": "red" }))];
    for shape in &hint.shapes {
        let points: Vec<Value> = outline(shape).into_iter().map(|[x, y]| json!([round(x * w), round(y * h), 1.0])).collect();
        calls.push(Call::engine("paint.stroke", json!({ "points": points, "size": size, "hardness": 1.0, "opacity": 1.0, "flow": 1.0, "color": COLOR })));
    }
    if let Some(layer) = return_to {
        calls.push(Call::engine("layer.select", json!({ "layer": layer })));
    }
    calls
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

/// The shape as a polyline in canvas fractions.
pub fn outline(shape: &Shape) -> Vec<Point> {
    match shape {
        Shape::Line { from, to } => vec![*from, *to],
        Shape::Path { points } => points.clone(),
        Shape::Rect { rect: [x, y, w, h] } => vec![[*x, *y], [x + w, *y], [x + w, y + h], [*x, y + h], [*x, *y]],
        Shape::Ellipse { rect: [x, y, w, h] } => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guide::check_call;

    fn hint(shapes: Vec<Shape>) -> Hint {
        Hint { text: "x".into(), target: None, shapes }
    }

    #[test]
    fn a_hint_without_shapes_draws_nothing() {
        assert!(show(&hint(vec![]), 100, 100, Some(3)).is_empty());
    }

    #[test]
    fn shapes_go_on_a_translucent_layer_and_return_to_the_students_layer() {
        let calls = show(&hint(vec![Shape::Line { from: [0.0, 0.5], to: [1.0, 0.5] }, Shape::Rect { rect: [0.1, 0.1, 0.2, 0.2] }]), 1000, 500, Some(7));
        assert_eq!(calls.len(), 4);
        assert_eq!(calls[0].command(), Some("layer.new.layer"));
        assert_eq!(calls[0].params["params"]["name"], LAYER_NAME);
        assert_eq!(calls[0].params["params"]["opacity"], OPACITY);
        assert_eq!(calls[1].params["params"]["points"], json!([[0.0, 250.0, 1.0], [1000.0, 250.0, 1.0]]));
        assert_eq!(calls[1].params["params"]["color"], COLOR);
        assert_eq!(calls[1].params["params"]["size"], 3.0);
        assert_eq!(calls[2].params["params"]["points"].as_array().map(Vec::len), Some(5));
        assert_eq!(calls[3], Call::engine("layer.select", json!({"layer": 7})));
        for call in &calls {
            assert_eq!(check_call(call), Ok(()), "hint calls are guide-safe: {call:?}");
        }
    }

    #[test]
    fn an_ellipse_is_a_closed_ring_inside_its_box() {
        let points = outline(&Shape::Ellipse { rect: [0.2, 0.4, 0.4, 0.2] });
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
