//! # NSSplitViewDelegate — Drawing & Layout
//!
//! Contains the rendering and layout logic for the split view delegate.
//! Draws divider lines and distributes visible subviews across the split
//! view's bounds.

use crate::split_view::NSSplitViewState;
use crate::{NSRect, NSView};
use std::sync::{Arc, Mutex};

/// Draws thin divider lines at each divider position.
///
/// Vertical split views draw vertical lines; horizontal ones draw
/// horizontal lines.  Colour comes from the active theme.
pub(super) fn draw_dividers(
    state: &Arc<Mutex<NSSplitViewState>>,
    view: &NSView,
    context: &mut crate::render::RenderContext,
) {
    let state = state.lock().unwrap();
    let bounds = view.bounds();

    // Draw a subtle divider grip indicator
    let sep_color = crate::theme::color(crate::theme::keys::CLR_SEPARATOR);
    let active_color = crate::theme::color(crate::theme::keys::CLR_ACCENT);
    let grip_color = crate::theme::color(crate::theme::keys::CLR_TEXT_TERTIARY);
    let is_dragging = state.dragging_divider.is_some();

    match state.vertical {
        true => {
            for (i, &pos) in state.divider_positions.iter().enumerate() {
                let active = state.dragging_divider == Some(i);
                // Main divider line
                context.set_color(match active {
                    true => active_color,
                    false => sep_color,
                });
                context.fill_rect(NSRect::new(pos - 0.5, 0.0, 1.0, bounds.height()));
                // Grip dots (3 small marks in the center)
                let _ = (active || is_dragging).then(|| {
                    context.set_color(grip_color);
                    let cy = bounds.height() / 2.0;
                    for dy in [-12.0, 0.0, 12.0] {
                        context.fill_rect(NSRect::new(pos - 2.0, cy + dy - 2.0, 4.0, 4.0));
                    }
                });
            }
        }
        false => {
            for (i, &pos) in state.divider_positions.iter().enumerate() {
                let active = state.dragging_divider == Some(i);
                context.set_color(match active {
                    true => active_color,
                    false => sep_color,
                });
                context.fill_rect(NSRect::new(0.0, pos - 0.5, bounds.width(), 1.0));
                let _ = (active || is_dragging).then(|| {
                    context.set_color(grip_color);
                    let cx = bounds.width() / 2.0;
                    for dx in [-12.0, 0.0, 12.0] {
                        context.fill_rect(NSRect::new(cx + dx - 2.0, pos - 2.0, 4.0, 4.0));
                    }
                });
            }
        }
    }
}

/// Distributes visible subviews across the split view's bounds.
///
/// Hidden subviews are excluded from the layout pass, and the remaining
/// visible children are sized according to `divider_positions`.  This
/// mirrors Swift's `NSSplitViewItem.isCollapsed` behaviour — when a pane
/// is hidden its space is redistributed to the remaining visible panes.
pub(super) fn layout_subviews(
    state: &Arc<Mutex<NSSplitViewState>>,
    view: &mut NSView,
) {
    let frame = view.bounds();
    let mut state = state.lock().unwrap();
    let subviews = view.subviews_mut();
    if subviews.is_empty() {
        return;
    }

    // Divider gap comes from theme — never hardcoded
    let divider_gap = crate::theme::f32_or(crate::theme::keys::SPACING_MD, 8.0);
    let _panel_radius = crate::theme::f32_or(crate::theme::keys::RADIUS_PANEL, 10.0);

    // Scale dividers proportionally if size changed
    if let Some(last_size) = state.last_layout_size {
        let current_dim = if state.vertical {
            frame.width()
        } else {
            frame.height()
        };
        let last_dim = if state.vertical {
            last_size.width
        } else {
            last_size.height
        };

        if last_dim > 0.0 && (current_dim - last_dim).abs() > 0.5 {
            let scale = current_dim / last_dim;
            let mut prev_pos = 0.0;
            let num_dividers = state.divider_positions.len();
            for (i, pos) in state.divider_positions.iter_mut().enumerate() {
                let mut val = *pos * scale;
                let min_pos = prev_pos + divider_gap;
                let max_pos = current_dim - (num_dividers - i) as f32 * divider_gap;
                val = val.clamp(min_pos, max_pos.max(min_pos));
                *pos = val;
                prev_pos = val;
            }
        }
    }
    state.last_layout_size = Some(frame.size);

    // Collect indices of visible subviews only — hidden panes are
    // skipped so their space is reclaimed by neighbours.
    let mut visible_indices = Vec::new();
    for (i, v) in subviews.iter().enumerate() {
        if !v.is_hidden() {
            visible_indices.push(i);
        }
    }

    let count = visible_indices.len();
    if count == 0 {
        return;
    }

    match state.vertical {
        true => {
            let mut last_pos = 0.0;
            for (idx, &orig_idx) in visible_indices.iter().enumerate() {
                let next_pos = match idx < state.divider_positions.len() {
                    true => state.divider_positions[idx],
                    false => frame.width(),
                };
                let x = last_pos
                    + match idx > 0 {
                        true => divider_gap / 2.0,
                        false => 0.0,
                    };
                let w = (next_pos
                    - x
                    - match idx < visible_indices.len() - 1 {
                        true => divider_gap / 2.0,
                        false => 0.0,
                    })
                .max(0.0);
                let _ = subviews.get_mut(orig_idx).map(|v| {
                    v.set_frame(NSRect::new(x, 0.0, w, frame.height()));
                    v.set_clips_to_bounds(true);
                    v.layout_subviews();
                });
                last_pos = next_pos;
            }
        }
        false => {
            let mut last_pos = 0.0;
            for (idx, &orig_idx) in visible_indices.iter().enumerate() {
                let next_pos = match idx < state.divider_positions.len() {
                    true => state.divider_positions[idx],
                    false => frame.height(),
                };
                let y = last_pos
                    + match idx > 0 {
                        true => divider_gap / 2.0,
                        false => 0.0,
                    };
                let h = (next_pos
                    - y
                    - match idx < visible_indices.len() - 1 {
                        true => divider_gap / 2.0,
                        false => 0.0,
                    })
                .max(0.0);
                let _ = subviews.get_mut(orig_idx).map(|v| {
                    v.set_frame(NSRect::new(0.0, y, frame.width(), h));
                    v.set_clips_to_bounds(true);
                    v.layout_subviews();
                });
                last_pos = next_pos;
            }
        }
    }
}
