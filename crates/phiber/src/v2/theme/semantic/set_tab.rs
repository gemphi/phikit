//! The strip, tab, viewport and camera surfaces — the sweep and the overlays, split from the host-background cluster.
//!
//! Split from `set.rs` so each half sits under the 240-line
//! read cap (`tests/level_structure.rs`).

use super::{BGradientColors, BSurfaceTone};
use crate::v2::host::BHostType;
use crate::v2::theme::{BColor, BResolvedUiStyle};

impl BResolvedUiStyle {
/// A header strip inside a host: the header shade.
pub fn strip_gradient(&self, host: BHostType) -> BGradientColors {
    if Self::is_chrome(host) {
        return Self::flat(self.host_background(host));
    }
    self.sheen(self.colors.bg_header)
}

/// `TH_CAMERA_PASSEPARTOUT` at the camera's `passepartalpha`: the
/// ladder's darkest shade (Blender's is black), half transparent.
pub fn camera_passepartout(&self) -> BColor {
    self.colors.ladder.at(1, crate::v2::scene::camera_view::PASSEPARTOUT_ALPHA)
}

/// `TH_VIEW_OVERLAY`: the camera frame's dashed line. Blender's is black;
/// here the darkest shade, so it reads on the passepartout in any theme.
pub fn view_overlay(&self) -> BColor {
    self.colors.ladder.color(1)
}

pub fn tab_hover(&self, host: BHostType) -> BColor {
    self.host_item_hover(host)
}

/// A Properties rail tab hands its shade to the panel beside it: the
/// sweep starts one step over the panel at the rail's outer edge and
/// lands on the panel's own shade at the inner edge, so the selected tab
/// and the content read as one surface (the caller turns it onto the
/// rail-to-panel axis with `along`).
/// The active tab in a rail. In Properties it is exactly the surface of
/// the region beside it (`host_body_surface`) — flat, not a sheen: the
/// tab is a hole cut in the strip onto the panel it selects, and a
/// gradient on one side of that join makes the two read as two things.
/// The selected tab's fill. A Properties tab is the accordion heading
/// beside it — the same `heading_surface(Card)` the panels' heads are
/// painted in, lit edge and all — so the selected tab and the first
/// accordion, which sit level (`rail_lead`), read as one row of the same
/// material: the tab names the panel it opens by wearing its colour. It
/// is not the body (`host_body_surface`) any more: the strip is the body
/// now, and a tab in the strip's own colour would mark nothing.
pub fn tab_active_gradient(&self, host: BHostType) -> BGradientColors {
    if host == BHostType::Buts {
        return self.heading_surface(BSurfaceTone::Card);
    }
    self.host_item_active_gradient(host)
}

pub fn rail_tab_hover(&self, _host: BHostType) -> BColor {
    self.colors.bg_hover
}

/// The canvas: the theme's sweep when it has one — a step lighter at
/// the left edge, the canvas shade by mid-width — else flat. The 3D
/// viewport's tracer paints the same sweep into its sky
/// (`viewport_sweep_lift`); the node, image and time canvases paint it
/// as a background.
pub fn viewport_gradient(&self) -> BGradientColors {
    self.sheen(self.viewport_surface())
}

/// How much lighter the canvas sweep's left edge is than the canvas, as
/// a fraction of full white (0 when the theme has no sweep) — what the
/// tracer adds to its sky at the left edge (`Camera::sky.a`).
pub fn viewport_sweep_lift(&self) -> f32 {
    self.viewport_gradient().lift() / 100.0
}}
