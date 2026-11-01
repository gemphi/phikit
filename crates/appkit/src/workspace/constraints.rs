//! Layout constraints — derived from NSSplitView divider positions.
//!
//! Keeps the bottom panel columns aligned with the main split above.

use crate::split_view::NSSplitView;
use super::layout as defaults;

/// Shared layout constraints derived from the main NSSplitView divider positions.
///
/// Both the main split and the bottom panel read from this struct to ensure
/// column boundaries stay aligned. Updated whenever dividers move.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutConstraints {
    /// Width of the sidebar (left panel). Equals divider 0 position.
    pub sidebar_w: f32,
    /// Width of the inspector (right panel). Equals total_width - divider 1 position.
    pub inspector_w: f32,
    /// Width of the viewport (center panel). Equals divider 1 - divider 0.
    pub viewport_w: f32,
}

impl LayoutConstraints {
    /// Derive layout constraints from the current NSSplitView divider positions.
    pub fn from_split_view(split: &NSSplitView) -> Self {
        let state = split.state.lock().unwrap_or_else(|e| e.into_inner());
        let total_width = split.view().frame().width();

        let divider_0_pos = state.divider_positions.get(0).copied().unwrap_or(0.0);
        let divider_1_pos = state.divider_positions.get(1).copied().unwrap_or(total_width);

        let sidebar_min = defaults::sidebar_min();
        let inspector_min = defaults::inspector_min();

        let clamped_div0 = divider_0_pos.max(sidebar_min);
        let clamped_div1 = divider_1_pos.min(total_width - inspector_min);
        let clamped_div1 = clamped_div1.max(clamped_div0);

        Self {
            sidebar_w: clamped_div0,
            inspector_w: total_width - clamped_div1,
            viewport_w: clamped_div1 - clamped_div0,
        }
    }

    /// Derive layout constraints from raw divider positions and total width.
    pub fn from_divider_positions(positions: &[f32], total_width: f32) -> Self {
        let divider_0_pos = positions.get(0).copied().unwrap_or(0.0);
        let divider_1_pos = positions.get(1).copied().unwrap_or(total_width);

        let sidebar_min = defaults::sidebar_min();
        let inspector_min = defaults::inspector_min();

        let clamped_div0 = divider_0_pos.max(sidebar_min);
        let clamped_div1 = divider_1_pos.min(total_width - inspector_min);
        let clamped_div1 = clamped_div1.max(clamped_div0);

        Self {
            sidebar_w: clamped_div0,
            inspector_w: total_width - clamped_div1,
            viewport_w: clamped_div1 - clamped_div0,
        }
    }

    /// Total width covered by all three regions.
    pub fn total_width(&self) -> f32 {
        self.sidebar_w + self.viewport_w + self.inspector_w
    }
}

// ─── Bottom panel alignment ──────────────────────────────────────────

/// Free-function wrapper for bottom panel column alignment.
pub fn align_bottom_panel_columns(bottom_panel: &mut crate::NSView, constraints: &LayoutConstraints) {
    NSLayoutConstraintHelper::align_bottom_panel_columns(bottom_panel, constraints);
}

/// Free-function wrapper for alignment from raw divider positions.
pub fn align_bottom_panel_columns_from_positions(
    bottom_panel: &mut crate::NSView,
    divider_positions: &[f32],
    total_width: f32,
) {
    NSLayoutConstraintHelper::align_bottom_panel_columns_from_positions(bottom_panel, divider_positions, total_width);
}

pub struct NSLayoutConstraintHelper;

impl NSLayoutConstraintHelper {

/// Aligns the bottom panel's internal column boundaries to match the main
/// NSSplitView's divider positions.
pub fn align_bottom_panel_columns(bottom_panel: &mut crate::NSView, constraints: &LayoutConstraints) {
    let panel_h = bottom_panel.frame().height();
    let subviews = bottom_panel.subviews_mut();

    if subviews.len() < 3 {
        return;
    }

    let sidebar_w = constraints.sidebar_w;
    let viewport_w = constraints.viewport_w;
    let inspector_w = constraints.inspector_w;

    subviews[0].set_frame(crate::NSRect::sized(sidebar_w, panel_h));
    subviews[1].set_frame(crate::NSRect::new(sidebar_w, 0.0, viewport_w, panel_h));
    subviews[2].set_frame(crate::NSRect::new(
        sidebar_w + viewport_w,
        0.0,
        inspector_w,
        panel_h,
    ));
}

/// Aligns the bottom panel columns using raw divider positions from the main split.
pub fn align_bottom_panel_columns_from_positions(
    bottom_panel: &mut crate::NSView,
    divider_positions: &[f32],
    total_width: f32,
) {
    let constraints = LayoutConstraints::from_divider_positions(divider_positions, total_width);
    align_bottom_panel_columns(bottom_panel, &constraints);
}

} // impl NSLayoutConstraintHelper
