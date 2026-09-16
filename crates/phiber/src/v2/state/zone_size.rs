//! Zone resize math — pure `BState` transforms for zone-boundary seams.
//!
//! Area resize (module `area_resize`) resizes areas STACKED inside a zone;
//! this module resizes the ZONES themselves against their sibling zones
//! (Left width vs Center, Center vs Right, and the Bottom strip height) —
//! Blender lets you drag every area/zone border.
//!
//! Sizes are absolute pixels at `ctx.zone_size.<space_slug>.<zone_slug>`.
//! Only `Left`, `Right` (width) and `Bottom` (height) are resizable; `Top`
//! (`HEADERY`) and `Center`/`Full` (filler) are not. An absent key means the
//! zone sizes to content like before — seams then anchor off the measured
//! layout extent at drag start.

use super::BState;
use crate::v2::space::BSpaceType;
use crate::v2::theme::METRICS;
use crate::v2::zone::BZoneType;

/// Encoded state key for a zone's explicit pixel size along its seam axis.
pub fn zone_size_key(space: BSpaceType, zone: BZoneType) -> String {
    format!("ctx.zone_size.{}.{}", space.slug(), zone.slug())
}

/// Whether a zone's outer boundary is draggable at all.
pub fn is_resizable(zone: BZoneType) -> bool {
    matches!(zone, BZoneType::Left | BZoneType::Right | BZoneType::Bottom)
}

/// Explicit pixel size for `(space, zone)`, or `None` when sized to content.
pub fn zone_size(state: &BState, space: BSpaceType, zone: BZoneType) -> Option<f32> {
    state
        .get_encoded(&zone_size_key(space, zone))
        .and_then(|s| s.parse::<f32>().ok())
        .filter(|v| *v > 0.0)
}

/// Apply an absolute zone-seam drag: `new = anchor + sign * total`, clamped
/// to `[min_area_extent, 0.9 * extent]` where `extent` is the container size
/// along the seam axis (window width for column seams, height for the bottom
/// strip). Writes only the one zone key. Idempotent in the total delta like
/// `area_resize::apply_drag`. Returns `false` for non-resizable zones.
pub fn apply_zone_drag(
    state: &mut BState,
    space: BSpaceType,
    zone: BZoneType,
    anchor: f32,
    total: f32,
    sign: f32,
    extent: f32,
) -> bool {
    apply_zone_drag_with_min_extent(
        state,
        space,
        zone,
        anchor,
        total,
        sign,
        extent,
        METRICS.min_area_extent,
    )
}

/// Resolved-style variant used by rendered seams. The active root drag path
/// supplies the minimum extent captured with the seam's render-pass payload.
#[allow(clippy::too_many_arguments)]
pub fn apply_zone_drag_with_min_extent(
    state: &mut BState,
    space: BSpaceType,
    zone: BZoneType,
    anchor: f32,
    total: f32,
    sign: f32,
    extent: f32,
    min_extent: f32,
) -> bool {
    if !is_resizable(zone) || extent <= 0.0 {
        return false;
    }
    let min = min_extent;
    let max = (extent * 0.9).max(min + 1.0);
    let new = (anchor + sign * total).clamp(min, max);
    state.set_encoded(&zone_size_key(space, zone), format!("{new}"));
    true
}

/// Clear a zone's explicit size so content sizing applies again (double-click
/// reset — same gesture as `area_resize::reset_equal`).
pub fn reset_zone_size(state: &mut BState, space: BSpaceType, zone: BZoneType) {
    state.set_encoded(&zone_size_key(space, zone), super::V_EMPTY);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_seam_grows_rightward() {
        let mut s = BState::default();
        assert!(apply_zone_drag(
            &mut s,
            BSpaceType::Layout,
            BZoneType::Left,
            200.0,
            50.0,
            1.0,
            1000.0
        ));
        assert_eq!(
            zone_size(&s, BSpaceType::Layout, BZoneType::Left),
            Some(250.0)
        );
    }

    #[test]
    fn right_seam_grows_leftward() {
        let mut s = BState::default();
        assert!(apply_zone_drag(
            &mut s,
            BSpaceType::Layout,
            BZoneType::Right,
            320.0,
            -40.0,
            -1.0,
            1000.0
        ));
        assert_eq!(
            zone_size(&s, BSpaceType::Layout, BZoneType::Right),
            Some(360.0)
        );
    }

    #[test]
    fn clamps_to_min_and_max() {
        let mut s = BState::default();
        apply_zone_drag(
            &mut s,
            BSpaceType::Layout,
            BZoneType::Left,
            200.0,
            -10_000.0,
            1.0,
            1000.0,
        );
        assert_eq!(
            zone_size(&s, BSpaceType::Layout, BZoneType::Left),
            Some(METRICS.min_area_extent)
        );
        apply_zone_drag(
            &mut s,
            BSpaceType::Layout,
            BZoneType::Left,
            200.0,
            10_000.0,
            1.0,
            1000.0,
        );
        assert_eq!(
            zone_size(&s, BSpaceType::Layout, BZoneType::Left),
            Some(900.0)
        );
    }

    #[test]
    fn center_and_top_are_not_resizable() {
        let mut s = BState::default();
        assert!(!apply_zone_drag(
            &mut s,
            BSpaceType::Layout,
            BZoneType::Center,
            500.0,
            100.0,
            1.0,
            1000.0
        ));
        assert!(!apply_zone_drag(
            &mut s,
            BSpaceType::Layout,
            BZoneType::Top,
            26.0,
            100.0,
            1.0,
            1000.0
        ));
        assert!(zone_size(&s, BSpaceType::Layout, BZoneType::Center).is_none());
    }

    #[test]
    fn reset_returns_to_content_sizing() {
        let mut s = BState::default();
        apply_zone_drag(
            &mut s,
            BSpaceType::Layout,
            BZoneType::Bottom,
            64.0,
            40.0,
            -1.0,
            800.0,
        );
        assert!(zone_size(&s, BSpaceType::Layout, BZoneType::Bottom).is_some());
        reset_zone_size(&mut s, BSpaceType::Layout, BZoneType::Bottom);
        assert!(zone_size(&s, BSpaceType::Layout, BZoneType::Bottom).is_none());
    }

    #[test]
    fn rendered_drag_uses_the_resolved_minimum_extent() {
        let mut state = BState::default();
        apply_zone_drag_with_min_extent(
            &mut state,
            BSpaceType::Layout,
            BZoneType::Left,
            200.0,
            -10_000.0,
            1.0,
            1_000.0,
            120.0,
        );

        assert_eq!(
            zone_size(&state, BSpaceType::Layout, BZoneType::Left),
            Some(120.0)
        );
    }
}
