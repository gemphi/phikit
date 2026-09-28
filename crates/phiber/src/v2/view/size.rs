//! Resizing a region by its edge — standalone, as `structure.md` §3 asks:
//! the rule, the keys it writes, and nothing about how an edge is drawn.
//!
//! Blender ref: `SCREEN_OT_region_scale` (`editors/screen/screen_ops.cc`,
//! `region_scale_modal`). The new size is the size at press plus the drag.
//! Dragged below one widget unit (`UI_UNIT_X`) the region hides and keeps
//! the size it had (`region_scale_toggle_hidden`); dragged back out past
//! that, it shows again. Otherwise the size is clamped to the region's range
//! (Blender clamps to `ED_area_max_regionsize`).

use crate::v2::state::{BPath, BState};
use crate::v2::state::val::V_TRUE;

/// Where a drag leaves a region.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BScaled {
    /// The size to keep — the size at press while hidden.
    pub size: f32,
    pub hidden: bool,
}

/// Blender's `region_scale_modal` for one axis. `collapse` is the size below
/// which the region hides (`UI_UNIT_X`).
pub fn scale(start: f32, delta: f32, min: f32, max: f32, collapse: f32) -> BScaled {
    let raw = start + delta;
    if raw < collapse {
        return BScaled { size: start, hidden: true };
    }
    BScaled { size: raw.clamp(min, max.max(min)), hidden: false }
}

/// `ctx.sidebar.width` — the width every sidebar shares.
pub fn k_sidebar_width() -> BPath {
    BPath::owned(vec!["sidebar", "width"])
}

/// The width a sidebar column was dragged to, or `default` until it is —
/// never under `least`. The file browser's source list and the strip over
/// it read the same width, so the strip's columns land on the list's edge.
pub fn sidebar_column_width(s: &BState, default: f32, least: f32) -> f32 {
    s.get(&k_sidebar_width()).and_then(|w| w.parse::<f32>().ok()).unwrap_or(default).max(least)
}

/// `ctx.sidebar.hidden` — dragged shut (`RGN_FLAG_HIDDEN`).
pub fn k_sidebar_hidden() -> BPath {
    BPath::owned(vec!["sidebar", "hidden"])
}

pub fn sidebar_hidden(s: &BState) -> bool {
    s.get(&k_sidebar_hidden()) == Some(V_TRUE)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: f32 = 160.0;
    const MAX: f32 = 720.0;
    const UNIT: f32 = 26.0;

    #[test]
    fn a_drag_resizes_within_the_range() {
        assert_eq!(scale(240.0, 60.0, MIN, MAX, UNIT), BScaled { size: 300.0, hidden: false });
        assert_eq!(scale(240.0, 900.0, MIN, MAX, UNIT).size, MAX, "clamped to the maximum");
        assert_eq!(scale(240.0, -150.0, MIN, MAX, UNIT).size, MIN, "above a unit, held at the minimum");
    }

    #[test]
    fn dragged_below_a_unit_the_region_hides_and_keeps_its_size() {
        assert_eq!(scale(240.0, -230.0, MIN, MAX, UNIT), BScaled { size: 240.0, hidden: true });
    }

    #[test]
    fn dragged_back_out_it_shows_again() {
        let back = scale(240.0, -200.0, MIN, MAX, UNIT);
        assert!(!back.hidden);
        assert_eq!(back.size, MIN);
    }
}
