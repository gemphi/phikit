//! Area- and zone-resize drags from the interactive dividers — a legacy seam
//! kept for the `BResizeHandler` callback, so the ladder of area trees in
//! `area/size.rs` can be driven from here with the window in hand.
//!
//! The absolute variants (`handle_resize_drag`, `handle_zone_drag`) are what
//! the dividers use: a drag is idempotent in its total pixel delta, so
//! re-dispatching the same move converges to the same fractions regardless of
//! how many move events GPUI emits. The incremental `handle_resize` stayed
//! for callers that still send a delta per frame.

use super::super::BPhiberApp;
use crate::v2::area::size::{apply_delta, apply_drag_with_min_extent, normalize};
use crate::v2::space::BSpaceType;
use crate::v2::zone::BZoneType;
use crate::v2::zone::size::apply_zone_drag_with_min_extent;
use gpui::Context;

impl BPhiberApp {
    pub fn handle_resize(
        &mut self,
        space: BSpaceType,
        zone: BZoneType,
        index: usize,
        dpx: f32,
        extent: f32,
        cx: &mut Context<Self>,
    ) {
        if apply_delta(&mut self.state, space, zone, index, dpx, extent).is_ok() {
            normalize(&mut self.state, space, zone);
            cx.notify();
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn handle_resize_drag(
        &mut self,
        space: BSpaceType,
        zone: BZoneType,
        index: usize,
        anchor_i: f32,
        anchor_j: f32,
        total_dpx: f32,
        extent: f32,
        min_extent: f32,
        cx: &mut Context<Self>,
    ) {
        if apply_drag_with_min_extent(&mut self.state, space, zone, index, anchor_i, anchor_j, total_dpx, extent, min_extent).is_ok()
        {
            normalize(&mut self.state, space, zone);
            cx.notify();
        }
    }

    /// An absolute zone-seam drag (whole Left/Right column width or Bottom
    /// strip height). `anchor` is the zone size at drag start, `total` the
    /// cursor travel since then, `sign` the direction mapping.
    pub fn handle_zone_drag(
        &mut self,
        space: BSpaceType,
        zone: BZoneType,
        anchor: f32,
        total: f32,
        sign: f32,
        extent: f32,
        min_extent: f32,
        cx: &mut Context<Self>,
    ) {
        if apply_zone_drag_with_min_extent(&mut self.state, space, zone, anchor, total, sign, extent, min_extent) {
            cx.notify();
        }
    }
}
