//! Split View Drag Bounds — min/max constraint calculations during divider drag.
//!
//! Extracted from `layout.rs` to keep each file focused on a single concern.

use crate::NSView;

/// Calculate min and max bounds for a divider during drag.
///
/// Takes into account:
/// - Minimum sizes of adjacent subviews
/// - Maximum sizes of adjacent subviews
/// - Positions of neighbouring dividers
pub fn calculate_drag_bounds(
    divider_idx: usize,
    visible_indices: &[usize],
    divider_positions: &[f32],
    subviews: &[NSView],
    vertical: bool,
    total_dim: f32,
) -> (f32, f32) {
    let view_before_idx = visible_indices.get(divider_idx).copied();
    let view_after_idx = visible_indices.get(divider_idx + 1).copied();

    let min_before = view_before_idx
        .map(|i| {
            let ms = if vertical {
                subviews[i].min_size().width
            } else {
                subviews[i].min_size().height
            };
            if ms > 0.0 { ms } else { 0.0 }
        })
        .unwrap_or(0.0);

    let min_after = view_after_idx
        .map(|i| {
            let ms = if vertical {
                subviews[i].min_size().width
            } else {
                subviews[i].min_size().height
            };
            if ms > 0.0 { ms } else { 0.0 }
        })
        .unwrap_or(0.0);

    let max_before = view_before_idx
        .map(|i| {
            if vertical {
                subviews[i].max_size().width
            } else {
                subviews[i].max_size().height
            }
        })
        .unwrap_or(f32::INFINITY);

    let max_after = view_after_idx
        .map(|i| {
            if vertical {
                subviews[i].max_size().width
            } else {
                subviews[i].max_size().height
            }
        })
        .unwrap_or(f32::INFINITY);

    // Calculate min bound
    let mut min = if divider_idx > 0 {
        divider_positions[divider_idx - 1] + min_before
    } else {
        min_before
    };

    // Calculate max bound
    let mut max = if divider_idx + 1 < divider_positions.len() {
        divider_positions[divider_idx + 1] - min_after
    } else {
        total_dim - min_after
    };

    // Apply max_before constraint
    let upper_bound = if divider_idx > 0 {
        divider_positions[divider_idx - 1] + max_before
    } else {
        max_before
    };
    if upper_bound < max {
        max = upper_bound;
    }

    // Apply max_after constraint
    let lower_bound = if divider_idx + 1 < divider_positions.len() {
        divider_positions[divider_idx + 1] - max_after
    } else {
        total_dim - max_after
    };
    if lower_bound > min {
        min = lower_bound;
    }

    (min, max)
}
