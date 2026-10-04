//! A plane you drag in: two channels at once, from one pointer.
//!
//! The last thing the colour picker was missing. A `BTrack` reads one axis, so
//! saturation and lightness needed two passes of it; Blender's picker gives you
//! both in one square and that is what people reach for first.
//!
//! The square is drawn as a grid of cells, each filled with the actual colour at
//! its own point, rather than as stacked gradients. Every picker approximates the
//! shading that way because a 2-stop linear gradient cannot express a hue sweep
//! crossed with a lightness sweep — and an approximation here would mean the
//! square showed one colour and set another. A grid is exact: what a cell looks
//! like is what dropping the pointer on it writes. The cost is visible banding at
//! `CELLS` steps, which is a fair trade for a square that does not lie.

mod grid;

pub use grid::view;

use super::encode_hsla;
use crate::v2::state::BPath;
use crate::v2::theme::BColor;
use crate::v2::view::prim::BAction;
use gpui::{Bounds, Pixels, Point};


/// Two channels of one colour, driven by one pointer.
///
/// `x` and `y` each name a channel and the range it spans. `y` reads from the top,
/// because a colour square puts its lightest row there and a reader expects down
/// to mean less.
#[derive(Clone, Debug, PartialEq)]
pub struct BPlane {
    pub key: BPath,
    pub color: BColor,
    pub x: BPlaneAxis,
    pub y: BPlaneAxis,
}

/// One axis of a plane: which channel it moves, between which bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPlaneAxis {
    pub channel: usize,
    pub min: f32,
    pub max: f32,
}

impl BPlane {
    /// Saturation across, lightness down — Blender's `GRAD_SV`, with hue on the
    /// bar below.
    ///
    /// Transcribed from Blender 5.2.0 in `crates/phiber/refs/blender`, which
    /// pins down three things that are easy to get wrong by reasoning:
    ///
    /// `editors/include/UI_interface_c.hh:665` numbers the gradients
    /// `GRAD_SV = 0, GRAD_HV = 1, GRAD_HS = 2, GRAD_H = 3, GRAD_S = 4,
    /// GRAD_V = 5`, then `GRAD_V_ALT = 9, GRAD_L_ALT = 10` apart from the run.
    /// `colorpicker_square` (`interface_region_color_picker.cc:622`) draws the
    /// square and gives the bar under it `gradient_type = type + 3` — so a
    /// `GRAD_SV` square's companion is `GRAD_H`, a **hue** bar, and the pairing
    /// is fixed by that arithmetic rather than chosen per square. And
    /// `GRAD_L_ALT` is reachable only from `colorpicker_circle` (line 617,
    /// `U.color_picker_type == USER_CP_CIRCLE_HSL ? GRAD_L_ALT : GRAD_V_ALT`):
    /// no square ever gets a lightness bar.
    ///
    /// Which is to say the pairing this briefly held — hue × saturation with a
    /// lightness bar — is one Blender does not ship. `GRAD_HS`'s bar is
    /// `GRAD_V`. Saturation × lightness, which this was before and is again, is
    /// `GRAD_SV` read in HSL: the zeroth gradient, the square Blender lists
    /// first.
    ///
    /// The deliberate divergence is the *shape*, not the axes.
    /// `DNA_userdef_types.h:1118` defaults `color_picker_type` to
    /// `USER_CP_CIRCLE_HSV`, and `block_colorpicker` falls through to
    /// `colorpicker_circle` for it, so Blender opens on a hue/saturation ring
    /// with a value bar. This picks `USER_CP_SQUARE_SV` instead — one of the
    /// three squares Blender itself offers, so a mode rather than an invention —
    /// because a square has no polar dead zone to miss, because hue gets a full
    /// bar of its own instead of sharing an axis with saturation, and because it
    /// is the arrangement every other picker a reader has used puts in front of
    /// them.
    pub fn saturation_lightness(key: BPath, color: BColor) -> Self {
        Self {
            key,
            color,
            x: BPlaneAxis { channel: 1, min: 0.0, max: 100.0 },
            y: BPlaneAxis { channel: 2, min: 0.0, max: 100.0 },
        }
    }

    /// `color` with both of this plane's channels set from the fractions
    /// `(across, down)`, each 0..1. `down` is inverted on the way in, so the top
    /// row is the maximum.
    pub fn color_at(&self, across: f32, down: f32) -> BColor {
        let value = |axis: BPlaneAxis, share: f32| axis.min + share.clamp(0.0, 1.0) * (axis.max - axis.min);
        let mut parts = [self.color.0, self.color.1, self.color.2, self.color.3];
        parts[self.x.channel.min(3)] = value(self.x, across);
        parts[self.y.channel.min(3)] = value(self.y, 1.0 - down);
        (parts[0], parts[1], parts[2], parts[3])
    }

    /// Where the colour currently sits in the square, as `(across, down)`.
    pub fn marker(&self) -> (f32, f32) {
        let share = |axis: BPlaneAxis| {
            let parts = [self.color.0, self.color.1, self.color.2, self.color.3];
            let value = parts[axis.channel.min(3)];
            if axis.max > axis.min { ((value - axis.min) / (axis.max - axis.min)).clamp(0.0, 1.0) } else { 0.0 }
        };
        (share(self.x), 1.0 - share(self.y))
    }

    /// The write for a pointer at `pointer` inside `bounds`.
    pub fn adjust_at(&self, bounds: Bounds<Pixels>, pointer: Point<Pixels>) -> BAction {
        let span = |along: f32, origin: f32, size: f32| if size > 0.0 { ((along - origin) / size).clamp(0.0, 1.0) } else { 0.0 };
        let across = span(f32::from(pointer.x), f32::from(bounds.origin.x), f32::from(bounds.size.width));
        let down = span(f32::from(pointer.y), f32::from(bounds.origin.y), f32::from(bounds.size.height));
        BAction::Adjust(self.key.clone(), encode_hsla(self.color_at(across, down)))
    }
}

#[cfg(test)]
mod tests {
    use super::grid::CELLS;
    use super::*;
    use gpui::{Size, px};

    fn plane() -> BPlane {
        BPlane::saturation_lightness(BPath::owned(vec!["c"]), (210.0, 40.0, 60.0, 1.0))
    }

    #[test]
    fn across_is_saturation_and_down_is_lightness_from_the_top() {
        let p = plane();
        // Top-left: no saturation, full lightness — white, as the top-left of a
        // `GRAD_SV` square is. The hue rides along at the 210 it came in with.
        assert_eq!(p.color_at(0.0, 0.0), (210.0, 0.0, 100.0, 1.0));
        // Top-right: fully saturated, still fully light.
        assert_eq!(p.color_at(1.0, 0.0), (210.0, 100.0, 100.0, 1.0));
        // Bottom-left: no saturation, no lightness — black.
        assert_eq!(p.color_at(0.0, 1.0), (210.0, 0.0, 0.0, 1.0));
        // The middle is half of each.
        assert_eq!(p.color_at(0.5, 0.5), (210.0, 50.0, 50.0, 1.0));
    }

    #[test]
    fn the_untouched_channels_ride_along() {
        // Hue and alpha are not this plane's business — hue is the bar below it,
        // Blender's `GRAD_H` — and must come out as they went in. A square that
        // quietly reset the hue would undo the row under it.
        let p = BPlane::saturation_lightness(BPath::owned(vec!["c"]), (33.0, 10.0, 20.0, 0.25));
        for (across, down) in [(0.0, 0.0), (1.0, 1.0), (0.3, 0.7)] {
            let (h, _, _, a) = p.color_at(across, down);
            assert_eq!((h, a), (33.0, 0.25));
        }
    }

    #[test]
    fn the_marker_is_where_the_colour_already_is() {
        // Saturation 40 of 100 across, lightness 60 of 100 measured DOWN from the
        // top — so 0.40 down, since the top row is the lightest.
        let (across, down) = plane().marker();
        assert!((across - 0.4).abs() < 1e-5, "{across}");
        assert!((down - 0.4).abs() < 1e-5, "{down}");
        // And it round-trips: the colour at the marker is the colour. To within
        // float noise — the trip is two divisions and a multiply, so 60 comes
        // back as 60.000004 and asserting exact equality would be testing the
        // arithmetic rather than the mapping.
        let (h, s, l, a) = plane().color_at(across, down);
        assert_eq!((h, a), (210.0, 1.0));
        assert!((s - 40.0).abs() < 1e-3 && (l - 60.0).abs() < 1e-3, "{s}, {l}");
    }

    #[test]
    fn a_pointer_outside_the_square_pins_to_its_edge() {
        let b = Bounds {
            origin: Point { x: px(100.0), y: px(100.0) },
            size: Size { width: px(200.0), height: px(200.0) },
        };
        let BAction::Adjust(_, far) = plane().adjust_at(b, Point { x: px(-900.0), y: px(-900.0) }) else {
            panic!("a drag adjusts")
        };
        // Up and left of the square is its top-left corner — white at this hue —
        // not a wrapped channel.
        assert_eq!(far, encode_hsla((210.0, 0.0, 100.0, 1.0)));
    }

    #[test]
    fn a_square_with_no_size_does_not_divide_by_it() {
        let b = Bounds { origin: Point { x: px(0.0), y: px(0.0) }, size: Size { width: px(0.0), height: px(0.0) } };
        let BAction::Adjust(_, written) = plane().adjust_at(b, Point { x: px(5.0), y: px(5.0) }) else {
            panic!("a drag adjusts")
        };
        assert!(crate::v2::theme::parse_hsla(&written).is_ok(), "{written:?}");
    }

    #[test]
    fn every_cell_and_every_drag_writes_a_colour_the_descriptor_accepts() {
        let p = plane();
        for row in 0..CELLS {
            for column in 0..CELLS {
                let color = p.color_at((column as f32 + 0.5) / CELLS as f32, (row as f32 + 0.5) / CELLS as f32);
                assert!(
                    crate::v2::theme::parse_hsla(&encode_hsla(color)).is_ok(),
                    "cell {row},{column} is {color:?}"
                );
            }
        }
    }
}
