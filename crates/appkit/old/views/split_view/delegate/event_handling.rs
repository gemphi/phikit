//! NSSplitViewDelegate — Mouse interaction for divider resizing.

use crate::split_view::NSSplitViewState;
use crate::NSView;
use std::sync::{Arc, Mutex};

/// Handles a mouse-down event on a divider.
pub(super) fn handle_mouse_down(
    state: &Arc<Mutex<NSSplitViewState>>,
    view: &mut NSView,
    event: &crate::event::NSEvent,
    local_point: crate::graphics::NSPoint,
) {
    if let Some(idx) = divider_at_point_impl(state, local_point.x, local_point.y) {
        state.lock().unwrap().dragging_divider = Some(idx);
        view.set_needs_display();
    }
}

/// Handles a mouse-dragged event to update divider position.
pub(super) fn handle_mouse_dragged(
    state: &Arc<Mutex<NSSplitViewState>>,
    view: &mut NSView,
    event: &crate::event::NSEvent,
    local_point: crate::graphics::NSPoint,
) {
    let mut state_guard = state.lock().unwrap();
    let idx = match state_guard.dragging_divider {
        Some(idx) => idx,
        None => return,
    };

    let bounds = view.bounds();
    let new_pos = match state_guard.vertical {
        true => local_point.x,
        false => local_point.y,
    };

    let subviews = view.subviews();
    let mut visible_indices = Vec::new();
    for (i, v) in subviews.iter().enumerate() {
        if !v.is_hidden() {
            visible_indices.push(i);
        }
    }

    let view_before_idx = visible_indices.get(idx).copied();
    let view_after_idx = visible_indices.get(idx + 1).copied();

    let min_before = match view_before_idx {
        Some(i) => {
            let ms = match state_guard.vertical {
                true => subviews[i].min_size().width,
                false => subviews[i].min_size().height,
            };
            if ms > 0.0 { ms } else { 0.0 }
        }
        None => 0.0,
    };
    let min_after = match view_after_idx {
        Some(i) => {
            let ms = match state_guard.vertical {
                true => subviews[i].min_size().width,
                false => subviews[i].min_size().height,
            };
            if ms > 0.0 { ms } else { 0.0 }
        }
        None => 0.0,
    };
    let max_before = match view_before_idx {
        Some(i) => match state_guard.vertical {
            true => subviews[i].max_size().width,
            false => subviews[i].max_size().height,
        },
        None => f32::INFINITY,
    };
    let max_after = match view_after_idx {
        Some(i) => match state_guard.vertical {
            true => subviews[i].max_size().width,
            false => subviews[i].max_size().height,
        },
        None => f32::INFINITY,
    };

    // Clamp: don't overlap with previous divider or go past next one
    let mut min = match idx > 0 {
        true => state_guard.divider_positions[idx - 1] + min_before,
        false => min_before,
    };
    let max_bound = match state_guard.vertical {
        true => bounds.width(),
        false => bounds.height(),
    };
    let mut max = match idx + 1 < state_guard.divider_positions.len() {
        true => state_guard.divider_positions[idx + 1] - min_after,
        false => max_bound - min_after,
    };

    // Further constrain based on subview max sizes
    if idx > 0 {
        let upper_bound = state_guard.divider_positions[idx - 1] + max_before;
        if upper_bound < max { max = upper_bound; }
    } else if max_before < max {
        max = max_before;
    }
    if idx + 1 < state_guard.divider_positions.len() {
        let lower_bound = state_guard.divider_positions[idx + 1] - max_after;
        if lower_bound > min { min = lower_bound; }
    } else {
        let lower_bound = max_bound - max_after;
        if lower_bound > min { min = lower_bound; }
    }

    state_guard.divider_positions[idx] = new_pos.clamp(min, max);

    // Notify dependent panels in real-time during drag
    let callback = state_guard.on_divider_changed.clone();
    let positions = state_guard.divider_positions.clone();
    let total_width = match state_guard.vertical {
        true => bounds.width(),
        false => bounds.height(),
    };
    drop(state_guard);
    if let Some(cb) = callback {
        cb(positions, total_width);
    }

    view.layout_subviews();
    view.set_needs_display();
}

/// Handles a mouse-up event to end a divider drag.
pub(super) fn handle_mouse_up(
    state: &Arc<Mutex<NSSplitViewState>>,
    view: &mut NSView,
    _event: &crate::event::NSEvent,
) {
    let mut state_guard = state.lock().unwrap();
    if state_guard.dragging_divider.is_some() {
        state_guard.dragging_divider = None;
        let callback = state_guard.on_divider_changed.clone();
        let positions = state_guard.divider_positions.clone();
        let total_width = view.bounds().width();
        drop(state_guard);
        if let Some(cb) = callback {
            cb(positions, total_width);
        }
        crate::window::set_cursor(winit::window::CursorIcon::Default);
        view.set_needs_display();
    }
}

/// Changes cursor when hovering over a divider.
pub(super) fn handle_mouse_moved(
    state: &Arc<Mutex<NSSplitViewState>>,
    _view: &mut NSView,
    event: &crate::event::NSEvent,
    local_point: crate::graphics::NSPoint,
) {
    let over_divider = divider_at_point_impl(state, local_point.x, local_point.y).is_some();
    let state_guard = state.lock().unwrap();
    let cursor = match over_divider {
        true => match state_guard.vertical {
            true => winit::window::CursorIcon::ColResize,
            false => winit::window::CursorIcon::RowResize,
        },
        false => winit::window::CursorIcon::Default,
    };
    drop(state_guard);
    crate::window::set_cursor(cursor);
}

/// Tests whether a pixel coordinate is within the hit region of a divider.
pub(super) fn divider_at_point_impl(
    state: &Arc<Mutex<NSSplitViewState>>,
    x: f32,
    y: f32,
) -> Option<usize> {
    let state = state.lock().unwrap();
    let half: f32 = super::divider_hit_width() / 2.0;
    for (i, &pos) in state.divider_positions.iter().enumerate() {
        let coord: f32 = if state.vertical { x } else { y };
        if (coord - pos).abs() <= half {
            return Some(i);
        }
    }
    None
}
