//! Blender's colour wheel: hue round the circle, saturation out from the
//! centre, and a value bar beside it — the picker Blender opens on, since
//! `DNA_userdef_types.h` defaults `color_picker_type` to `USER_CP_CIRCLE_HSV`
//! and `block_colorpicker` falls through to `colorpicker_circle` for it.
//!
//! The geometry is `interface_widgets.cc`'s, transcribed:
//! `hsvcircle_vals_from_pos` reads hue as `atan2(dx, dy) / 2π + 0.5` with y
//! up — cyan at the top, red at the bottom — and saturation as the distance
//! from the centre over the radius; `hsvcircle_pos_from_vals` puts the cursor
//! back where that reading came from. The bar is `GRAD_V_ALT`: black at the
//! bottom, the full colour at the top.
//!
//! The picker had a saturation × lightness square here instead, chosen over
//! the wheel on purpose. It went with the colour controls that never opened it
//! (`BCtrl::read` gave none of them an open key); Blender opens on the wheel,
//! so the wheel is what opens.
//!
//! One wheel serves both kinds of colour a control holds. A theme preference is
//! `"h, s, l, a"` text and a scene property is linear RGBA written `"r g b a"`
//! ([`BPickSpace`]); the wheel works in HSV over the RGB the swatch beside it
//! shows, and writes back in the spelling the key was read in.

mod paint;

pub use paint::view;

use crate::v2::state::BPath;
use crate::v2::theme::BColor;
use crate::v2::view::prim::BAction;
use gpui::{Bounds, Pixels, Point};
use std::f32::consts::TAU;

/// How the colour at a key is spelled, which is how a write must spell it.
#[derive(Clone, Debug, PartialEq)]
pub enum BPickSpace {
    /// A theme preference: `"h, s, l, a"`, hue in degrees, the rest percent.
    Hsl,
    /// A scene property: `"r g b a"` (`BValue::Color`).
    Rgba,
    /// A colour input socket in a node editor. Its value lives in the
    /// editor's overlay of socket values (`canvas/node/edit`), one key for the
    /// whole tree, so a write is that overlay with this one socket's colour
    /// set.
    Socket { node: String, socket: String, edits: crate::v2::view::draw::canvas::node::edit::BValueEdits },
}

/// A colour on the wheel: the key it writes, where it sits in HSV (each
/// 0..1, value allowed above 1 the way an emission colour is), its alpha,
/// and how the key spells it.
#[derive(Clone, Debug, PartialEq)]
pub struct BWheel {
    pub key: BPath,
    pub hsv: [f32; 3],
    pub alpha: f32,
    pub space: BPickSpace,
}

impl BWheel {
    /// A theme colour. HSL goes to HSV directly rather than through RGB, so a
    /// grey keeps the hue it was given — Blender's `rgb_to_hsv_compat` for
    /// the same reason: dragging the value of a grey must not reset its hue.
    pub fn of_hsl(key: BPath, (h, s, l, a): BColor) -> Self {
        let (s, l) = ((s / 100.0).clamp(0.0, 1.0), (l / 100.0).clamp(0.0, 1.0));
        let v = l + s * l.min(1.0 - l);
        let sv = if v > 0.0 { 2.0 * (1.0 - l / v) } else { 0.0 };
        Self { key, hsv: [(h / 360.0).rem_euclid(1.0), sv.clamp(0.0, 1.0), v], alpha: a, space: BPickSpace::Hsl }
    }

    /// A scene colour.
    pub fn of_rgba(key: BPath, [r, g, b, a]: [f32; 4]) -> Self {
        Self { key, hsv: rgb_to_hsv([r, g, b]), alpha: a, space: BPickSpace::Rgba }
    }

    /// A node socket's colour, written into the editor's overlay at `key`.
    pub fn of_socket(key: BPath, rgba: [f32; 4], space: BPickSpace) -> Self {
        Self { space, ..Self::of_rgba(key, rgba) }
    }

    /// The colour as the swatch shows it, value capped at 1 the way Blender
    /// caps it "for display only".
    pub fn shown(&self) -> [f32; 3] {
        let [h, s, v] = self.hsv;
        hsv_to_rgb([h, s, v.clamp(0.0, 1.0)])
    }

    /// `hsv` written the way the key spells a colour.
    fn encode(&self, hsv: [f32; 3]) -> String {
        match &self.space {
            BPickSpace::Hsl => {
                let (s, l) = hsv_to_hsl(hsv[1], hsv[2].clamp(0.0, 1.0));
                super::encode_hsla((hsv[0] * 360.0, s * 100.0, l * 100.0, self.alpha))
            }
            BPickSpace::Rgba => {
                let [r, g, b] = hsv_to_rgb(hsv);
                crate::v2::state::BValue::Color([r, g, b, self.alpha]).encode()
            }
            BPickSpace::Socket { node, socket, edits } => {
                let [r, g, b] = hsv_to_rgb(hsv);
                let mut edits = edits.clone();
                edits.set(node, socket, crate::phiphx::blend::shade::socket::BSocketDefault::Color([r, g, b, self.alpha]));
                crate::v2::view::draw::canvas::node::edit::encode(&edits)
            }
        }
    }

    /// Hue and saturation at `(across, down)` of the wheel's square, each
    /// 0..1 — `hsvcircle_vals_from_pos` with the y axis turned to the screen's.
    /// A point outside the circle is on its rim.
    pub fn hs_at(across: f32, down: f32) -> (f32, f32) {
        let (dx, dy) = (across - 0.5, 0.5 - down);
        let s = ((dx * dx + dy * dy).sqrt() / 0.5).min(1.0);
        ((dx.atan2(dy) / TAU + 0.5).rem_euclid(1.0), s)
    }

    /// Where the cursor sits, as `(across, down)` — `hsvcircle_pos_from_vals`.
    pub fn marker(&self) -> (f32, f32) {
        let [h, s, _] = self.hsv;
        let ang = TAU * h + std::f32::consts::FRAC_PI_2;
        let r = s.clamp(0.0, 1.0) * 0.5;
        (0.5 + (-ang).cos() * r, 0.5 - (-ang).sin() * r)
    }

    /// The write for the pointer at `pointer` over a wheel drawn in `bounds`:
    /// hue and saturation from the circle, value and alpha as they were.
    pub fn adjust_at(&self, bounds: Bounds<Pixels>, pointer: Point<Pixels>) -> BAction {
        let (across, down) = square_share(bounds, pointer);
        let (h, s) = Self::hs_at(across, down);
        BAction::Adjust(self.key.clone(), self.encode([h, s, self.hsv[2]]))
    }

    /// The write for the pointer at `pointer` over the value bar in `bounds`:
    /// the bottom is black, the top the full colour.
    pub fn value_at(&self, bounds: Bounds<Pixels>, pointer: Point<Pixels>) -> BAction {
        let span = f32::from(bounds.size.height);
        let down = if span > 0.0 { ((f32::from(pointer.y) - f32::from(bounds.origin.y)) / span).clamp(0.0, 1.0) } else { 0.0 };
        BAction::Adjust(self.key.clone(), self.encode([self.hsv[0], self.hsv[1], 1.0 - down]))
    }
}

/// `pointer` inside the square the circle is drawn in — the largest centred
/// in `bounds`, as `hsvcircle_vals_from_pos` takes the smaller side — as
/// fractions of its side.
fn square_share(bounds: Bounds<Pixels>, pointer: Point<Pixels>) -> (f32, f32) {
    let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
    let side = w.min(h);
    if side <= 0.0 {
        return (0.5, 0.5);
    }
    let x0 = f32::from(bounds.origin.x) + (w - side) / 2.0;
    let y0 = f32::from(bounds.origin.y) + (h - side) / 2.0;
    ((f32::from(pointer.x) - x0) / side, (f32::from(pointer.y) - y0) / side)
}

/// HSV (each 0..1, value unbounded) to RGB.
pub fn hsv_to_rgb([h, s, v]: [f32; 3]) -> [f32; 3] {
    let h6 = h.rem_euclid(1.0) * 6.0;
    let k = |n: f32| {
        let k = (n + h6) % 6.0;
        v - v * s * k.min(4.0 - k).clamp(0.0, 1.0)
    };
    [k(5.0), k(3.0), k(1.0)]
}

/// RGB to HSV, hue 0 for a grey.
pub fn rgb_to_hsv([r, g, b]: [f32; 3]) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    // Which channel leads decides the sextant (`rgb_to_hsv` in `BLI_math_color`).
    let h = match (d > 0.0, max == r, max == g) {
        (false, _, _) => 0.0,
        (true, true, _) => ((g - b) / d).rem_euclid(6.0) / 6.0,
        (true, false, true) => ((b - r) / d + 2.0) / 6.0,
        (true, false, false) => ((r - g) / d + 4.0) / 6.0,
    };
    [h, if max > 0.0 { d / max } else { 0.0 }, max]
}

/// HSV's saturation and value as HSL's saturation and lightness, all 0..1.
fn hsv_to_hsl(s: f32, v: f32) -> (f32, f32) {
    let l = v * (1.0 - s / 2.0);
    let sl = if l <= 0.0 || l >= 1.0 { 0.0 } else { (v - l) / l.min(1.0 - l) };
    (sl.clamp(0.0, 1.0), l.clamp(0.0, 1.0))
}

/// RGB (0..1) as the `BColor` a swatch button shows: degrees and percent.
pub fn rgb_to_hsla([r, g, b]: [f32; 3], a: f32) -> BColor {
    let [h, s, v] = rgb_to_hsv([r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)]);
    let (s, l) = hsv_to_hsl(s, v);
    (h * 360.0, s * 100.0, l * 100.0, a)
}

#[cfg(test)]
mod tests;
