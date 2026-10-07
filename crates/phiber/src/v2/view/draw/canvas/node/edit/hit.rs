//! The clickable targets that edit a socket's value, laid over the widgets the
//! painter draws (`paint/dot.rs`): a checkbox for each boolean input, and a pair
//! of step regions — down on the left, up on the right — for each numeric input.
//!
//! The card body is empty on purpose (`set.rs`): a socket and its value widget
//! are PAINTED over the canvas, not laid out as divs, because a socket sits on
//! the card's border. So a hit target cannot be a child of the card; it is a
//! canvas-level overlay, placed in the same canvas FRACTIONS a reroute's handle
//! uses (`reroute.rs`) and derived from the view the painter draws with, so the
//! two agree to the sub-pixel the fraction/zoom split already tolerates
//! (`plan.rs`).
//!
//! Click only, no text entry or drag-scrub yet: a boolean toggles, a number
//! steps. A field you type into wants `EntityInputHandler` (an app-level job),
//! and a drag-scrub wants the pointer gesture a card drag uses; both are later.

use super::super::plan::BCardSize;
use super::super::socket::value::BSocketWidget;
use super::super::socket::{self, BSocketSide, DOT};
use super::super::view::BNodeView;
use crate::phiphx::blend::shade::graph::BlendNode;
use crate::phiphx::blend::shade::socket::BSocketDefault;

/// A float socket steps by a tenth, an int by one. A simplification: Blender
/// takes the step from the socket's own `step` and subtype, which phiber does
/// not parse yet, so every float field steps the same amount.
const FLOAT_STEP: f32 = 0.1;

/// One clickable value target: where to place a div (canvas fractions) and the
/// value a click there writes into the edit overlay. A checkbox is one target
/// (the toggled value); a number is two (stepped down, stepped up).
#[derive(Clone, Debug, PartialEq)]
pub struct BValueHit {
    pub socket: String,
    pub set: BSocketDefault,
    pub at: (f32, f32),
    pub size: (f32, f32),
}

/// Every clickable value target on `node`, in canvas fractions. `at` is the
/// card's top-left fraction, `size` the card in view units, `linked` the wired
/// inputs (a linked socket shows no value, so no target). A reroute or a
/// collapsed node draws no values and yields nothing.
pub fn value_targets(
    node: &BlendNode,
    at: (f32, f32),
    size: BCardSize,
    view: &BNodeView,
    linked: &[&str],
    picking: Option<&str>,
) -> Vec<BValueHit> {
    if node.reroute() || node.collapsed() {
        return Vec::new();
    }
    // A view-unit rect (offsets from the card's top-left) to the top-left
    // fraction and size fraction a div is placed with.
    let frac = |x0: f32, y0: f32, w: f32, h: f32| {
        ((at.0 + x0 / view.width(), at.1 + y0 / view.height()), (w / view.width(), h / view.height()))
    };
    // The value column's right edge — a dot's width in from the card's right,
    // where the painter right-aligns the widget (`paint/dot.rs`).
    let inner_right = size.width - 2.0 * DOT * size.unit;
    let mut out = Vec::new();
    for row in socket::rows(node, linked) {
        if row.side != BSocketSide::Input {
            continue;
        }
        let ycenter = size.header + row.y * size.unit;
        match socket::value::widget(row.side, row.linked, row.default.as_ref()) {
            // A checkbox: one box `0.5 * unit` on a side at the row's inner-right,
            // toggling the stored bool.
            Some(BSocketWidget::Check(on)) => {
                let s = 0.5 * size.unit;
                let (a, z) = frac(inner_right - s, ycenter - s / 2.0, s, s);
                out.push(BValueHit { socket: row.identifier, set: BSocketDefault::Bool(!on), at: a, size: z });
            }
            // A number: a field the right half of the card wide, split into a
            // step-down half and a step-up half — Blender's drag direction, as
            // two clicks (left lowers, right raises).
            Some(BSocketWidget::Text(_)) => {
                let Some((down, up)) = stepped(row.default.as_ref()) else { continue };
                let fieldw = 0.5 * size.width;
                let half = fieldw / 2.0;
                let left = inner_right - fieldw;
                let (ad, zd) = frac(left, ycenter - 0.5 * size.unit, half, size.unit);
                out.push(BValueHit { socket: row.identifier.clone(), set: down, at: ad, size: zd });
                let (au, zu) = frac(left + half, ycenter - 0.5 * size.unit, half, size.unit);
                out.push(BValueHit { socket: row.identifier, set: up, at: au, size: zu });
            }
            // A vector: a field per component, each a row below the name
            // (`paint/dot.rs` draws them at `y + (i+1)*unit`), each its own
            // down/up pair stepping that one component.
            Some(BSocketWidget::Fields(_)) => {
                let Some(BSocketDefault::Vector(v)) = row.default else { continue };
                let fieldw = 0.5 * size.width;
                let half = fieldw / 2.0;
                let left = inner_right - fieldw;
                for i in 0..v.len() {
                    let cy = ycenter + (i as f32 + 1.0) * size.unit;
                    let (ad, zd) = frac(left, cy - 0.5 * size.unit, half, size.unit);
                    out.push(BValueHit { socket: row.identifier.clone(), set: vec_step(v, i, -FLOAT_STEP), at: ad, size: zd });
                    let (au, zu) = frac(left + half, cy - 0.5 * size.unit, half, size.unit);
                    out.push(BValueHit { socket: row.identifier.clone(), set: vec_step(v, i, FLOAT_STEP), at: au, size: zu });
                }
            }
            // A colour: nothing inline until its swatch is clicked (`swatch_hits`
            // opens it); while open, a channel per row below the swatch — R, G,
            // B, A — each its own down/up pair, like a vector's components.
            Some(BSocketWidget::Swatch(rgba)) => {
                let open = picking.is_some_and(|p| p == format!("{}/{}", node.name, row.identifier));
                if !open {
                    continue;
                }
                let fieldw = 0.5 * size.width;
                let half = fieldw / 2.0;
                let left = inner_right - fieldw;
                for i in 0..rgba.len() {
                    let cy = ycenter + (i as f32 + 1.0) * size.unit;
                    let (ad, zd) = frac(left, cy - 0.5 * size.unit, half, size.unit);
                    out.push(BValueHit { socket: row.identifier.clone(), set: color_step(rgba, i, -FLOAT_STEP), at: ad, size: zd });
                    let (au, zu) = frac(left + half, cy - 0.5 * size.unit, half, size.unit);
                    out.push(BValueHit { socket: row.identifier.clone(), set: color_step(rgba, i, FLOAT_STEP), at: au, size: zu });
                }
            }
            _ => {}
        }
    }
    out
}

/// A numeric socket's value stepped down and up, or `None` when the widget's
/// default is not a number (a Text widget today is only ever a number).
fn stepped(default: Option<&BSocketDefault>) -> Option<(BSocketDefault, BSocketDefault)> {
    match default {
        Some(BSocketDefault::Float(x)) => Some((BSocketDefault::Float(x - FLOAT_STEP), BSocketDefault::Float(x + FLOAT_STEP))),
        Some(BSocketDefault::Int(n)) => Some((BSocketDefault::Int(n - 1), BSocketDefault::Int(n + 1))),
        _ => None,
    }
}

/// A vector with component `i` moved by `delta`, the rest untouched — one
/// component of a vector socket stepping on its own.
fn vec_step(mut v: [f32; 3], i: usize, delta: f32) -> BSocketDefault {
    v[i] += delta;
    BSocketDefault::Vector(v)
}

/// An RGBA colour with channel `i` moved by `delta`, the rest untouched.
fn color_step(mut c: [f32; 4], i: usize, delta: f32) -> BSocketDefault {
    c[i] += delta;
    BSocketDefault::Color(c)
}

/// One colour socket's swatch, as the rectangle a click opens its picker on —
/// the swatch the painter draws at the row's inner-right (`paint/dot.rs`).
#[derive(Clone, Debug, PartialEq)]
pub struct BSwatchHit {
    pub socket: String,
    pub at: (f32, f32),
    pub size: (f32, f32),
}

/// Every colour swatch on `node`, in canvas fractions — the click targets that
/// open and close the inline channel picker. Empty for a reroute, a collapsed
/// node, or a linked colour input (which shows the wire, not a swatch).
pub fn swatch_hits(node: &BlendNode, at: (f32, f32), size: BCardSize, view: &BNodeView, linked: &[&str]) -> Vec<BSwatchHit> {
    if node.reroute() || node.collapsed() {
        return Vec::new();
    }
    let inner_right = size.width - 2.0 * DOT * size.unit;
    let mut out = Vec::new();
    for row in socket::rows(node, linked) {
        if row.side != BSocketSide::Input {
            continue;
        }
        if let Some(BSocketWidget::Swatch(_)) = socket::value::widget(row.side, row.linked, row.default.as_ref()) {
            // The painter's swatch: `0.9 * unit` wide, `0.42 * unit` tall, its
            // right edge at the row's inner-right, centred on the row.
            let (sw, sh) = (0.9 * size.unit, 0.42 * size.unit);
            let ycenter = size.header + row.y * size.unit;
            let a = (at.0 + (inner_right - sw) / view.width(), at.1 + (ycenter - sh / 2.0) / view.height());
            let z = (sw / view.width(), sh / view.height());
            out.push(BSwatchHit { socket: row.identifier, at: a, size: z });
        }
    }
    out
}
