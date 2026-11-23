//! Split View Layout — all divider positioning and subview layout calculations.
//!
//! This module centralizes:
//! - Divider hit testing
//! - Visible subview frame calculations
//! - Divider position clamping with constraints
//! - Proportional divider scaling on resize
//!
//! All layout values come from theme — delegates contain no layout math.
//! Drag bounds calculation lives in `drag_bounds.rs`.

use super::NSSplitViewState;
use crate::{NSRect, NSSize, NSView};

/// Divider hit width from theme at runtime.
pub fn divider_hit_width() -> f32 {
    crate::theme::f32_or(crate::theme::keys::SPACING_MD, 8.0)
}

/// Divider gap from theme at runtime.
pub fn divider_gap() -> f32 {
    crate::theme::f32_or(crate::theme::keys::SPACING_MD, 8.0)
}

/// Panel radius from theme.
pub fn panel_radius() -> f32 {
    crate::theme::f32_or(crate::theme::keys::RADIUS_PANEL, 10.0)
}

/// Hit-test a point against divider positions.
pub fn divider_at_point(state: &NSSplitViewState, x: f32, y: f32) -> Option<usize> {
    let half = divider_hit_width() / 2.0;
    let test_coord = if state.vertical { x } else { y };

    for (i, &pos) in state.divider_positions.iter().enumerate() {
        if (test_coord - pos).abs() <= half {
            return Some(i);
        }
    }
    None
}

/// Collect indices of visible subviews.
pub fn visible_indices(subviews: &[NSView]) -> Vec<usize> {
    subviews
        .iter()
        .enumerate()
        .filter(|(_, v)| !v.is_hidden())
        .map(|(i, _)| i)
        .collect()
}

/// Scale divider positions proportionally when view size changes.
pub fn scale_dividers_on_resize(
    divider_positions: &mut [f32],
    last_size: Option<NSSize>,
    current_size: NSSize,
    vertical: bool,
) {
    let current_dim = if vertical {
        current_size.width
    } else {
        current_size.height
    };

    let last_dim = last_size.map(|s| if vertical { s.width } else { s.height });

    if let Some(last) = last_dim {
        if last > 0.0 && (current_dim - last).abs() > 0.5 {
            let scale = current_dim / last;
            let gap = divider_gap();
            let num_dividers = divider_positions.len();
            let mut prev_pos = 0.0;

            for (i, pos) in divider_positions.iter_mut().enumerate() {
                let mut val = *pos * scale;
                let min_pos = prev_pos + gap;
                let max_pos = current_dim - (num_dividers - i) as f32 * gap;
                val = val.clamp(min_pos, max_pos.max(min_pos));
                *pos = val;
                prev_pos = val;
            }
        }
    }
}

/// Calculate min and max bounds for a divider during drag.
pub fn calculate_drag_bounds(
    divider_idx: usize,
    visible_indices: &[usize],
    divider_positions: &[f32],
    subviews: &[NSView],
    vertical: bool,
    total_dim: f32,
) -> (f32, f32) {
    super::drag_bounds::calculate_drag_bounds(
        divider_idx, visible_indices, divider_positions, subviews, vertical, total_dim,
    )
}

/// Perform the full layout pass — scale dividers on resize, then position subviews.
pub fn layout_subviews(
    view: &mut crate::NSView,
    state_arc: &std::sync::Arc<std::sync::Mutex<super::NSSplitViewState>>,
) {
    let frame = view.bounds();
    let mut state = state_arc.lock().unwrap();
    let subviews = view.subviews_mut();
    if subviews.is_empty() { return; }

    let gap = divider_gap();

    let last_size = state.last_layout_size;
    let vertical = state.vertical;

    scale_dividers_on_resize(
        &mut state.divider_positions,
        last_size,
        frame.size,
        vertical,
    );
    state.last_layout_size = Some(frame.size);

    let vis = visible_indices(subviews);
    if vis.is_empty() { return; }

    let is_vert = state.vertical;
    let positions = state.divider_positions.clone();
    drop(state);

    if is_vert {
        let mut last_pos = 0.0;
        for (idx, &orig_idx) in vis.iter().enumerate() {
            let next_pos = positions.get(idx).copied().unwrap_or(frame.width());
            let x = last_pos + if idx > 0 { gap / 2.0 } else { 0.0 };
            let w = (next_pos - x - if idx < vis.len() - 1 { gap / 2.0 } else { 0.0 }).max(0.0);
            if let Some(v) = subviews.get_mut(orig_idx) {
                v.set_frame(crate::NSRect::new(x, 0.0, w, frame.height()));
                v.set_clips_to_bounds(true);
                v.layout_subviews();
            }
            last_pos = next_pos;
        }
    } else {
        let mut last_pos = 0.0;
        for (idx, &orig_idx) in vis.iter().enumerate() {
            let next_pos = positions.get(idx).copied().unwrap_or(frame.height());
            let y = last_pos + if idx > 0 { gap / 2.0 } else { 0.0 };
            let h = (next_pos - y - if idx < vis.len() - 1 { gap / 2.0 } else { 0.0 }).max(0.0);
            if let Some(v) = subviews.get_mut(orig_idx) {
                v.set_frame(crate::NSRect::new(0.0, y, frame.width(), h));
                v.set_clips_to_bounds(true);
                v.layout_subviews();
            }
            last_pos = next_pos;
        }
    }
}
