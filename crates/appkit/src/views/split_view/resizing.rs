//! Mouse drag and release handling for divider resizing in `NSSplitView`.

use super::view_construction::NSSplitView;
use crate::event::NSEvent;

impl NSSplitView {
    /// Handles a mouse-dragged event to update the divider position.
    ///
    /// Calculates the cursor delta from the drag start position, adds it to
    /// the original divider position, then clamps the result to min/max
    /// constraints derived from the adjacent subviews' `min_size` and
    /// `max_size` configurations.
    ///
    /// Calls `layout_subviews()` to immediately resize affected subviews,
    /// providing responsive visual feedback during the drag.
    ///
    /// Returns `true` if a drag is active and the event was consumed.
    pub fn mouse_dragged(&mut self, event: &NSEvent) -> bool {
        let drag = match self.active_drag {
            Some(drag) => drag,
            None => return false,
        };

        let pos = event.location_in_window();
        let state_lock = self.state.lock().unwrap();
        let is_vertical = state_lock.vertical;
        drop(state_lock);

        // Calculate delta from the drag start position
        let delta = if is_vertical {
            pos.x - drag.start_pos.x
        } else {
            pos.y - drag.start_pos.y
        };

        // New position = original divider position + cursor delta
        let new_pos = drag.start_divider_pos + delta;

        // Compute min/max constraints from adjacent subviews
        let bounds = self.view.bounds();
        let subviews = self.view.subviews();

        // Collect visible subview indices (hidden panes are skipped)
        let mut visible_indices = Vec::new();
        for (i, v) in subviews.iter().enumerate() {
            if !v.is_hidden() {
                visible_indices.push(i);
            }
        }

        let view_before_idx = visible_indices.get(drag.index).copied();
        let view_after_idx = visible_indices.get(drag.index + 1).copied();

        // Min size of the view before the divider (constrains how far left/up)
        let min_before = match view_before_idx {
            Some(i) => {
                let ms = if is_vertical {
                    subviews[i].min_size().width
                } else {
                    subviews[i].min_size().height
                };
                if ms > 0.0 { ms } else { 0.0 }
            }
            None => 0.0,
        };

        // Min size of the view after the divider (constrains how far right/down)
        let min_after = match view_after_idx {
            Some(i) => {
                let ms = if is_vertical {
                    subviews[i].min_size().width
                } else {
                    subviews[i].min_size().height
                };
                if ms > 0.0 { ms } else { 0.0 }
            }
            None => 0.0,
        };

        // Max size of the view before the divider
        let max_before = match view_before_idx {
            Some(i) => if is_vertical {
                subviews[i].max_size().width
            } else {
                subviews[i].max_size().height
            },
            None => f32::INFINITY,
        };

        // Max size of the view after the divider
        let max_after = match view_after_idx {
            Some(i) => if is_vertical {
                subviews[i].max_size().width
            } else {
                subviews[i].max_size().height
            },
            None => f32::INFINITY,
        };

        let state_lock = self.state.lock().unwrap();
        let divider_positions = state_lock.divider_positions.clone();
        drop(state_lock);

        // Minimum position: must leave min_before space from the previous divider (or edge)
        let min = if drag.index > 0 {
            divider_positions[drag.index - 1] + min_before
        } else {
            min_before
        };

        // Maximum position: must leave min_after space before the next divider (or edge)
        let max_bound = if is_vertical {
            bounds.width()
        } else {
            bounds.height()
        };
        let mut max = if drag.index + 1 < divider_positions.len() {
            divider_positions[drag.index + 1] - min_after
        } else {
            max_bound - min_after
        };

        // Further constrain based on max_before (view before can't exceed its max)
        if drag.index > 0 {
            let upper_bound = divider_positions[drag.index - 1] + max_before;
            if upper_bound < max {
                max = upper_bound;
            }
        } else if max_before < max {
            max = max_before;
        }

        // Further constrain based on max_after (view after can't exceed its max)
        let mut min = min;
        if drag.index + 1 < divider_positions.len() {
            let lower_bound = divider_positions[drag.index + 1] - max_after;
            if lower_bound > min {
                min = lower_bound;
            }
        } else {
            let lower_bound = max_bound - max_after;
            if lower_bound > min {
                min = lower_bound;
            }
        }

        // Clamp the new position to the computed constraints
        let clamped = new_pos.clamp(min, max);

        // Update the divider position in shared state
        let mut state_lock = self.state.lock().unwrap();
        if drag.index < state_lock.divider_positions.len() {
            state_lock.divider_positions[drag.index] = clamped;
        }
        drop(state_lock);

        // Re-layout subviews immediately for responsive drag feedback
        self.view.layout_subviews();
        self.view.set_needs_display();

        // Notify dependent panels in real-time during drag
        self.notify_divider_changed();

        true
    }

    /// Handles a mouse-up event to commit the final divider position and end
    /// the drag operation.
    ///
    /// Clears `active_drag`, triggers a full layout recalculation, and invokes
    /// [`notify_divider_changed`](Self::notify_divider_changed) to propagate
    /// the new divider positions to dependent panels (e.g., Bottom_Panel).
    ///
    /// Returns `true` if a drag was active and has been committed, `false` if
    /// no drag was in progress.
    pub fn mouse_up(&mut self, _event: &NSEvent) -> bool {
        if self.active_drag.is_none() {
            return false;
        }

        // Clear the active drag state
        self.active_drag = None;

        // Trigger a full layout recalculation
        self.view.layout_subviews();
        self.view.set_needs_display();

        // Propagate the new divider positions to dependent panels
        self.notify_divider_changed();

        true
    }
}
