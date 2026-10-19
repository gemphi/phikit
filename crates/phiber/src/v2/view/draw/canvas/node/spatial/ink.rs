//! The spatial view's colours, resolved by the caller, and how a face is lit.
//!
//! No design literal lives in the painter: the floor is the canvas shade, a
//! solid's top is its class hue at the header lightness (`theme::class_color`,
//! the same colour the flat card's header wears), a plate is the panel shade
//! a flat frame is filled with, a tube is the wire ink. What is the spatial
//! view's own is the LIGHTING — a lambert term over `set::LIGHT` with
//! `set::AMBIENT` under it — and it only ever scales a lightness the theme
//! chose.

use super::set::{AMBIENT, LIGHT};
use crate::v2::theme::palette::BColor;
use crate::v2::theme::{BNodeClass, class_color};

#[derive(Clone, Copy, Debug)]
pub struct BSpatialInk {
    /// The floor — the canvas shade.
    pub floor: BColor,
    /// The floor grid's lines.
    pub grid: BColor,
    pub grid_width: f32,
    /// The lightness and alpha a class hue is read at — the card header's.
    pub header: BColor,
    /// A frame's plate — the panel shade.
    pub plate: BColor,
    /// A muted solid dims toward this, as a muted header does.
    pub muted_toward: BColor,
    /// A tube's core and its outline — `TH_WIRE_INNER` and `TH_WIRE`.
    pub tube: BColor,
    pub tube_shadow: BColor,
    /// A selected solid's rim; the active one's is drawn wider.
    pub selected: BColor,
    pub rim_width: f32,
    /// A title on a top face — the text ink, faded on a muted node by
    /// `title_muted_alpha` (`BUT_INACTIVE`, as the flat header's title is).
    pub title: BColor,
    pub title_muted_alpha: f32,
}

impl BSpatialInk {
    /// The colour a solid's faces start from: its class hue, dimmed when
    /// muted the way `node::header_color` dims a flat header (Blender's
    /// `get_color_blend_4f(TH_BACK, color_id, 0.4)`).
    pub fn base_of(&self, class: BNodeClass, muted: bool) -> BColor {
        let hue = class_color(class, self.header);
        if muted { toward(hue, self.muted_toward, 0.4) } else { hue }
    }

    /// `base` lit by the fixed light for a face with `normal`: the lightness
    /// scaled between `AMBIENT` of itself and all of it.
    pub fn lit(base: BColor, normal: [f32; 3]) -> BColor {
        let len = (LIGHT[0] * LIGHT[0] + LIGHT[1] * LIGHT[1] + LIGHT[2] * LIGHT[2]).sqrt();
        let lambert = ((normal[0] * LIGHT[0] + normal[1] * LIGHT[1] + normal[2] * LIGHT[2]) / len).max(0.0);
        let factor = AMBIENT + (1.0 - AMBIENT) * lambert;
        (base.0, base.1, (base.2 * factor).clamp(0.0, 100.0), base.3)
    }
}

/// `a` blended toward `b`, keeping fraction `t` of `a`; alpha is `a`'s.
fn toward(a: BColor, b: BColor, t: f32) -> BColor {
    let mix = |x: f32, y: f32| x + (y - x) * (1.0 - t);
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2), a.3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_top_is_lit_brighter_than_a_side_turned_from_the_light() {
        let base: BColor = (120.0, 40.0, 50.0, 1.0);
        let top = BSpatialInk::lit(base, [0.0, 0.0, 1.0]);
        let away = BSpatialInk::lit(base, [1.0, 0.0, 0.0]);
        assert!(top.2 > away.2, "{} > {}", top.2, away.2);
        assert!((away.2 - base.2 * AMBIENT).abs() < 1e-3, "a face the light misses keeps the ambient share");
        assert_eq!((top.0, top.1, top.3), (base.0, base.1, base.3), "only the lightness moves");
    }
}
