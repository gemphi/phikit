//! The channel expander's hit test — which channel a press in the expand slot
//! folds, and the write that folds it.
//!
//! Split from `gesture.rs` to keep it under the file budget, along a real seam:
//! the gesture layer owns the event and this owns the geometry, which is the part
//! worth asserting without standing up a window.

use super::row::{BRowKind, BTimeRow};
use crate::v2::state::BKey as _;
use crate::v2::view::prim::BAction;

/// Which channel's fold a press at `(x, y)` flips, as the write that flips it.
///
/// Pure, so the hit region can be asserted without standing up a window — the
/// region is the whole claim here, and it is easy to get wrong by a few pixels in
/// a way no type checks.
///
/// `ANIM_channel_draw` reserves `ICON_WIDTH` at the head of an expandable channel
/// for its expand widget, so that slot is the hit target — the same region
/// `row::set::expander` draws the triangle in. A press on the NAME is a selection
/// in Blender (`anim.channels_click`) and is not handled yet, so it returns `None`
/// and falls through rather than being swallowed.
pub(crate) fn expander_write(
    rows: &[BTimeRow],
    x: f32,
    y: f32,
    row_height: f32,
    channel_width: f32,
) -> Option<BAction> {
    use crate::v2::state::val::{V_FALSE, V_TRUE};
    if y < super::SCRUB_HEIGHT || x >= channel_width {
        return None;
    }
    let index = BTimeRow::index_at(y - super::SCRUB_HEIGHT, row_height)?;
    let BRowKind::Object { name, expanded } = &rows.get(index)?.kind else { return None };
    let slot = super::WIDGET_UNIT * 0.5;
    if x < slot || x >= slot + 0.85 * super::WIDGET_UNIT {
        return None;
    }
    let next = if *expanded { V_FALSE } else { V_TRUE };
    Some(BAction::Set(
        crate::v2::state::DopeKey::Expanded(crate::v2::state::intern(name)).path(),
        next.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::val::{V_FALSE, V_TRUE};

    fn rows() -> Vec<BTimeRow> {
        vec![
            BTimeRow { kind: BRowKind::Summary { expanded: true }, keys: Vec::new() },
            BTimeRow { kind: BRowKind::Object { name: "Rig".into(), expanded: false }, keys: Vec::new() },
            BTimeRow { kind: BRowKind::Curve { name: "X Location".into(), color: [0.0; 3] }, keys: Vec::new() },
        ]
    }

    /// The vertical middle of row `index`, in the gesture's own coordinates.
    fn mid_of(index: usize, row_height: f32) -> f32 {
        super::super::SCRUB_HEIGHT + BTimeRow::center(index, row_height)
    }

    #[test]
    fn the_slot_the_triangle_is_drawn_in_is_the_slot_that_answers() {
        let (rows, h) = (rows(), super::super::row::ROW_HEIGHT);
        let slot = super::super::WIDGET_UNIT * 0.5;
        let y = mid_of(1, h);
        // Left of the slot: the indent, which is not a target.
        assert!(expander_write(&rows, slot - 1.0, y, h, 500.0).is_none());
        // Inside it: the object's fold flips on.
        let BAction::Set(path, value) = expander_write(&rows, slot + 1.0, y, h, 500.0).expect("in the slot") else {
            panic!("a press writes")
        };
        assert!(path.encode().ends_with(".expanded.Rig"), "{}", path.encode());
        assert_eq!(value, V_TRUE, "a shut channel opens");
        // Past it: the name, which is a selection in Blender and not ours yet.
        assert!(expander_write(&rows, slot + 0.85 * super::super::WIDGET_UNIT + 1.0, y, h, 500.0).is_none());
    }

    #[test]
    fn an_open_channel_shuts() {
        let mut rows = rows();
        rows[1].kind = BRowKind::Object { name: "Rig".into(), expanded: true };
        let h = super::super::row::ROW_HEIGHT;
        let x = super::super::WIDGET_UNIT * 0.5 + 1.0;
        let BAction::Set(_, value) = expander_write(&rows, x, mid_of(1, h), h, 500.0).expect("in the slot") else {
            panic!("a press writes")
        };
        assert_eq!(value, V_FALSE);
    }

    #[test]
    fn only_a_channel_that_can_expand_answers() {
        // `acf_fcurve_setting_valid`: "F-Curves are not containers", so a curve
        // row has no expander and a press in its indent must do nothing — not
        // flip the object above it.
        let (rows, h) = (rows(), super::super::row::ROW_HEIGHT);
        let x = super::super::WIDGET_UNIT * 0.5 + 1.0;
        assert!(expander_write(&rows, x, mid_of(2, h), h, 500.0).is_none(), "an F-curve row");
        assert!(expander_write(&rows, x, mid_of(0, h), h, 500.0).is_none(), "the summary row");
    }

    #[test]
    fn a_press_outside_the_channel_column_is_not_a_fold() {
        // Past the column is the key area, where a press scrubs; above it is the
        // scrub band itself. Neither may flip a channel.
        let (rows, h) = (rows(), super::super::row::ROW_HEIGHT);
        let x = super::super::WIDGET_UNIT * 0.5 + 1.0;
        assert!(expander_write(&rows, 600.0, mid_of(1, h), h, 500.0).is_none(), "right of the column");
        assert!(expander_write(&rows, x, super::super::SCRUB_HEIGHT - 1.0, h, 500.0).is_none(), "the scrub band");
    }
}
