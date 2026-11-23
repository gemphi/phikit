//! `NSSplitView` struct definition, construction, and configuration methods.

use super::delegate;
use super::divider::{
    DividerChangedCallback, DividerDrag, NSSplitViewDividerStyle, NSSplitViewState,
    DIVIDER_HIT_WIDTH,
};
use crate::event::NSEvent;
use crate::{NSPoint, NSRect, NSView};
use std::sync::{Arc, Mutex};

/// A container view that arranges child panes along a single axis, separated
/// by movable dividers.
///
/// Mirrors Swift's [`NSSplitView`](https://developer.apple.com/documentation/appkit/nssplitview).
/// Each child is added via [`add_arranged_subview`](Self::add_arranged_subview)
/// and can be collapsed by calling `set_hidden(true)` on the child view.
#[derive(Clone)]
pub struct NSSplitView {
    pub(crate) view: NSView,
    pub(crate) state: Arc<Mutex<NSSplitViewState>>,
    /// Active divider drag operation, if any. Set on mouse_down within a
    /// divider hit region, cleared on mouse_up.
    pub active_drag: Option<DividerDrag>,
    /// Optional callback invoked when divider positions change (e.g., after drag).
    pub(crate) on_divider_changed: Option<DividerChangedCallback>,
}

impl NSSplitView {
    /// Creates a new split view with the given frame and default horizontal
    /// (top-to-bottom) orientation.
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(NSSplitViewState {
            divider_style: NSSplitViewDividerStyle::Thin,
            vertical: false,
            divider_positions: Vec::new(),
            dragging_divider: None,
            last_layout_size: None,
            on_divider_changed: None,
        }));
        let mut view = NSView::new(frame);
        view.set_delegate(delegate::NSSplitViewDelegate {
            state: state.clone(),
        });
        Self {
            view,
            state,
            active_drag: None,
            on_divider_changed: None,
        }
    }

    /// Convenience: creates a vertical split view (children arranged left-to-right).
    pub fn vertical() -> Self {
        let mut split = Self::new(NSRect::zero());
        split.set_vertical(true);
        split
    }

    /// Convenience: creates a horizontal split view (children arranged top-to-bottom).
    pub fn horizontal() -> Self {
        let mut split = Self::new(NSRect::zero());
        split.set_vertical(false);
        split
    }

    /// Sets the split axis. `true` → vertical dividers (side-by-side),
    /// `false` → horizontal dividers (stacked). Triggers an immediate layout pass.
    pub fn set_vertical(&mut self, vertical: bool) {
        self.state.lock().unwrap().vertical = vertical;
        self.view.layout_subviews();
    }

    /// Appends a child view as an arranged subview and re-layouts.
    pub fn add_arranged_subview(&mut self, view: NSView) {
        self.view.add_subview(view);
        self.view.layout_subviews();
    }

    /// Sets the divider rendering style.
    pub fn set_divider_style(&mut self, style: NSSplitViewDividerStyle) {
        self.state.lock().unwrap().divider_style = style;
        self.view.layout_subviews();
    }

    /// Programmatically sets a divider's position along the split axis.
    /// Triggers an immediate layout pass.
    pub fn set_divider_at_index(&mut self, index: usize, position: f64) {
        let mut state = self.state.lock().unwrap();
        if index >= state.divider_positions.len() {
            state.divider_positions.resize(index + 1, 0.0);
        }
        state.divider_positions[index] = position as f32;
        drop(state);
        self.view.layout_subviews();
    }

    /// Returns the pixel position of the divider at `index`. Returns `0.0` if out of range.
    pub fn divider_position(&self, index: usize) -> f64 {
        let state = self.state.lock().unwrap();
        state.divider_positions.get(index).copied().unwrap_or(0.0) as f64
    }

    /// Returns a clone of all divider positions.
    pub fn divider_positions(&self) -> Vec<f32> {
        let state = self.state.lock().unwrap();
        state.divider_positions.clone()
    }

    /// Notifies dependent panels that divider positions have changed.
    /// Invokes the registered `on_divider_changed` callback if one has been set.
    pub fn notify_divider_changed(&self) {
        if let Some(ref callback) = self.on_divider_changed {
            let positions = self.divider_positions();
            let total_width = self.view.frame().width();
            callback(positions, total_width);
        }
    }

    /// Registers a callback to be invoked when divider positions change.
    ///
    /// The callback receives the current divider positions and the total width
    /// of the split view. Used by the bottom panel to stay aligned with the
    /// main split's column boundaries.
    pub fn set_on_divider_changed<F>(&mut self, callback: F)
    where
        F: Fn(Vec<f32>, f32) + Send + Sync + 'static,
    {
        let cb = Arc::new(callback);
        self.on_divider_changed = Some(cb.clone());
        // Also store in shared state so the delegate can invoke it on mouse_up
        if let Ok(mut state) = self.state.lock() {
            state.on_divider_changed = Some(cb);
        }
    }

    /// Returns a shared reference to the underlying `NSView`.
    pub fn view(&self) -> &NSView {
        &self.view
    }

    /// Returns a mutable reference to the underlying `NSView`.
    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }

    /// Tests whether `point` (in the split view's local coordinate space)
    /// falls within the hit region of any divider.
    ///
    /// The hit region is ±4px from the divider center along the split axis
    /// (total width = [`DIVIDER_HIT_WIDTH`] = 8px).
    ///
    /// Returns `Some(index)` for the first divider whose hit region contains
    /// the point, or `None` if no divider is hit.
    pub fn divider_at_point(&self, point: NSPoint) -> Option<usize> {
        let state = self.state.lock().unwrap();
        let half = DIVIDER_HIT_WIDTH / 2.0;
        let test_coord = if state.vertical { point.x } else { point.y };

        for (i, &pos) in state.divider_positions.iter().enumerate() {
            if (test_coord - pos).abs() <= half {
                return Some(i);
            }
        }
        None
    }

    /// Handles a mouse-down event for divider drag initiation.
    ///
    /// If the event location falls within a divider's hit region, begins a
    /// drag operation by storing a [`DividerDrag`] in `self.active_drag` and
    /// returns `true` (event consumed). Otherwise returns `false`.
    pub fn mouse_down(&mut self, event: &NSEvent) -> bool {
        let point = event.location_in_window();
        if let Some(index) = self.divider_at_point(point) {
            let state = self.state.lock().unwrap();
            let start_divider_pos = state
                .divider_positions
                .get(index)
                .copied()
                .unwrap_or(0.0);
            drop(state);

            self.active_drag = Some(DividerDrag {
                index,
                start_pos: point,
                start_divider_pos,
            });
            true
        } else {
            false
        }
    }
}
