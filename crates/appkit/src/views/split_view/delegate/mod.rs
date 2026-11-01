//! # NSSplitViewDelegate
//!
//! Internal delegate that drives `NSSplitView` rendering and layout.
//!
//! Mirrors `NSSplitViewDelegate` from Apple's AppKit framework.  The delegate
//! is responsible for three things:
//!
//! 1. **Drawing** — renders 1 px divider lines at each `divider_position`,
//!    styled via the current theme's `CLR_SEPARATOR` token.
//! 2. **Layout** — distributes child views across the available axis, skipping
//!    any subview where `is_hidden() == true` so that toggling panel visibility
//!    genuinely reclaims space (matching `NSSplitViewItem.isCollapsed` in
//!    Swift's AppKit).
//! 3. **Interaction** — handles mouse down/drag/up to allow users to resize
//!    panels by dragging divider lines, matching native `NSSplitView` behaviour.

mod drawing;
mod event_handling;

use crate::split_view::NSSplitViewState;
use crate::{NSView, NSViewDelegate};
use std::sync::{Arc, Mutex};

/// Divider hit width comes from theme at runtime — no bare constants.
fn divider_hit_width() -> f32 {
    crate::theme::f32_or(crate::theme::keys::SPACING_MD, 8.0)
}

/// Internal delegate that drives `NSSplitView` rendering and layout.
///
/// # Thread Safety
///
/// The shared `NSSplitViewState` is protected by a `Mutex`, allowing the
/// editor's action system to adjust `divider_positions` from any thread.
pub struct NSSplitViewDelegate {
    /// Shared mutable state describing divider positions and orientation.
    pub state: Arc<Mutex<NSSplitViewState>>,
}

impl NSSplitViewDelegate {
    /// Test whether a pixel coordinate is within the hit region of a divider.
    /// Returns the divider index if so.
    fn divider_at_point(&self, x: f32, y: f32) -> Option<usize> {
        event_handling::divider_at_point_impl(&self.state, x, y)
    }
}

impl NSViewDelegate for NSSplitViewDelegate {
    /// Claims mouse events near divider lines before children see them.
    fn wants_mouse_down(&self, point: crate::graphics::NSPoint) -> bool {
        self.divider_at_point(point.x, point.y).is_some()
    }

    /// Draws thin divider lines at each divider position.
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        drawing::draw_dividers(&self.state, view, context);
    }

    /// Distributes visible subviews across the split view's bounds.
    fn layout(&mut self, view: &mut NSView) {
        drawing::layout_subviews(&self.state, view);
    }

    // --- Mouse interaction — drag dividers to resize panels ---

    fn mouse_down(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let local_point = crate::graphics::NSPoint::new(
            event.location_in_window.x,
            event.location_in_window.y,
        );
        event_handling::handle_mouse_down(&self.state, view, event, local_point);
    }

    fn mouse_dragged(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let local_point = crate::graphics::NSPoint::new(
            event.location_in_window.x,
            event.location_in_window.y,
        );
        event_handling::handle_mouse_dragged(&self.state, view, event, local_point);
    }

    fn mouse_up(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        event_handling::handle_mouse_up(&self.state, view, event);
    }

    fn mouse_moved(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let local_point = crate::graphics::NSPoint::new(
            event.location_in_window.x,
            event.location_in_window.y,
        );
        event_handling::handle_mouse_moved(&self.state, view, event, local_point);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
