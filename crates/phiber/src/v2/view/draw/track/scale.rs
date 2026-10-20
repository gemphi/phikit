//! What a pointer's position along a track means.
//!
//! The arithmetic is separated from the gesture because it is the part worth
//! checking without a window: given bounds and a point, what value does a track
//! read? Every case below is a pure function call.

use super::{BTrack, BTrackWrite};
use crate::v2::view::layout::BAxis;
use gpui::{Bounds, Pixels, Point};

impl BTrack {
    /// Where `pointer` sits inside `bounds`, as a fraction of the track.
    ///
    /// Clamped, so a drag that leaves the element pins to the end it left by
    /// rather than running past the range or snapping back to the middle.
    fn share(&self, bounds: Bounds<Pixels>, pointer: Point<Pixels>) -> f32 {
        let (along, origin, span) = match self.axis {
            BAxis::Row => (
                f32::from(pointer.x),
                f32::from(bounds.origin.x),
                f32::from(bounds.size.width),
            ),
            BAxis::Column => (
                f32::from(pointer.y),
                f32::from(bounds.origin.y),
                f32::from(bounds.size.height),
            ),
        };
        if span <= 0.0 {
            return 0.0;
        }
        let share = ((along - origin) / span).clamp(0.0, 1.0);
        // Down the element is up the range: a vertical track's bottom is its min.
        match self.axis {
            BAxis::Row => share,
            BAxis::Column => 1.0 - share,
        }
    }

    /// The value at `pointer`, written the way the control's own value is.
    pub(super) fn value_at(&self, bounds: Bounds<Pixels>, pointer: Point<Pixels>) -> String {
        let value = self.min + self.share(bounds, pointer) * (self.max - self.min);
        match &self.write {
            BTrackWrite::Value if self.whole => format!("{}", value.round() as i64),
            BTrackWrite::Value => format!("{value}"),
            BTrackWrite::Channel { index, color } => {
                let (h, s, l, a) = *color;
                let (h, s, l, a) = match index {
                    0 => (value, s, l, a),
                    1 => (h, value, l, a),
                    2 => (h, s, value, a),
                    _ => (h, s, l, value),
                };
                // One spelling for a colour, stated in `mod.rs`.
                super::encode_hsla((h, s, l, a))
            }
            BTrackWrite::Rgba { index, rgba } => {
                let mut rgba = *rgba;
                rgba[(*index).min(3)] = value;
                crate::v2::state::BValue::Color(rgba).encode()
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::BPath;
    use crate::v2::view::prim::BAction;
    use gpui::{Size, px};

    fn bounds(x: f32, y: f32, w: f32, h: f32) -> Bounds<Pixels> {
        Bounds { origin: Point { x: px(x), y: px(y) }, size: Size { width: px(w), height: px(h) } }
    }

    fn at(track: &BTrack, b: Bounds<Pixels>, x: f32, y: f32) -> f32 {
        track.value_at(b, Point { x: px(x), y: px(y) }).parse().unwrap()
    }

    #[test]
    fn the_pointers_place_along_the_track_is_the_value() {
        let track = BTrack::across(BPath::owned(vec!["x"]), 0.0, 100.0);
        let b = bounds(200.0, 50.0, 400.0, 20.0);
        assert_eq!(at(&track, b, 200.0, 60.0), 0.0, "the left edge is the minimum");
        assert_eq!(at(&track, b, 600.0, 60.0), 100.0, "the right edge is the maximum");
        assert_eq!(at(&track, b, 400.0, 60.0), 50.0, "the middle is the middle");
        assert_eq!(at(&track, b, 500.0, 60.0), 75.0, "three quarters along is three quarters up");
    }

    #[test]
    fn a_drag_that_leaves_the_track_pins_to_the_end_it_left_by() {
        let track = BTrack::across(BPath::owned(vec!["x"]), 10.0, 20.0);
        let b = bounds(0.0, 0.0, 100.0, 20.0);
        assert_eq!(at(&track, b, -500.0, 5.0), 10.0, "far left stays at the minimum");
        assert_eq!(at(&track, b, 5000.0, 5.0), 20.0, "far right stays at the maximum");
    }

    #[test]
    fn a_vertical_track_reads_from_the_bottom() {
        // More is higher: a column track's bottom edge is its minimum, which is
        // the opposite of the y axis it is measured on.
        let track = BTrack { axis: BAxis::Column, ..BTrack::across(BPath::owned(vec!["x"]), 0.0, 100.0) };
        let b = bounds(0.0, 100.0, 20.0, 200.0);
        assert_eq!(at(&track, b, 10.0, 300.0), 0.0, "the bottom is the minimum");
        assert_eq!(at(&track, b, 10.0, 100.0), 100.0, "the top is the maximum");
        assert_eq!(at(&track, b, 10.0, 200.0), 50.0);
    }

    #[test]
    fn a_whole_number_track_writes_whole_numbers() {
        let track = BTrack::across(BPath::owned(vec!["x"]), 1.0, 10.0).whole();
        let b = bounds(0.0, 0.0, 90.0, 20.0);
        for x in [0.0, 13.0, 27.0, 44.0, 90.0] {
            let written = track.value_at(b, Point { x: px(x), y: px(5.0) });
            assert!(written.parse::<i64>().is_ok(), "{written:?} is not a whole number");
        }
    }

    #[test]
    fn a_track_with_no_width_does_not_divide_by_it() {
        let track = BTrack::across(BPath::owned(vec!["x"]), 0.0, 100.0);
        assert_eq!(at(&track, bounds(0.0, 0.0, 0.0, 20.0), 50.0, 5.0), 0.0);
    }

    #[test]
    fn a_drag_adjusts_rather_than_decides() {
        // A `Set` would dismiss the popover the slider is sitting in on the first
        // pixel of movement.
        let track = BTrack::across(BPath::owned(vec!["x"]), 0.0, 1.0);
        let action = track.adjust_at(bounds(0.0, 0.0, 10.0, 10.0), Point { x: px(5.0), y: px(5.0) });
        assert!(matches!(action, BAction::Adjust(..)), "got {action:?}");
    }
}
