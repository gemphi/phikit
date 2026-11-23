//! Drag and drop event routing helpers.
use super::super::NSApplication;
use crate::event::{NSDragOperation, NSDraggingInfo};
use crate::graphics::NSPoint;
use crate::view::NSView;

impl NSApplication {
    /// Find the deepest view that contains the point and dispatch drag hover/update events.
    pub fn dispatch_drag_update(
        view: &mut NSView,
        drag_info: &NSDraggingInfo,
        window_pos: NSPoint,
    ) -> bool {
        if view.is_hidden() {
            return false;
        }

        for child in view.subviews_mut().iter_mut().rev() {
            let child_frame = child.frame();
            if child_frame.contains_point(&window_pos) {
                let local_pos = NSPoint::new(
                    window_pos.x - child_frame.origin.x,
                    window_pos.y - child_frame.origin.y,
                );
                let mut local_info = drag_info.clone();
                local_info.dragging_location = local_pos;
                if Self::dispatch_drag_update(child, &local_info, local_pos) {
                    return true;
                }
            }
        }

        let target_id = view.id;
        let mut hovered_changed = false;

        {
            let shared = Self::shared();
            let mut app = shared.lock().unwrap();
            let old = app.drag_hovered_view_id;
            if old != Some(target_id) {
                app.drag_hovered_view_id = Some(target_id);
                hovered_changed = true;
            }
        };

        let mut handled = false;
        if let Some(delegate_arc) = view.delegate().clone() {
            if let Ok(mut d) = delegate_arc.lock() {
                let op = if hovered_changed {
                    d.dragging_entered(view, drag_info)
                } else {
                    d.dragging_updated(view, drag_info)
                };
                handled = op != NSDragOperation::None;
            }
        }

        handled
    }

    /// Dispatch the final dropped file action recursively.
    pub fn dispatch_drop(
        view: &mut NSView,
        drag_info: &NSDraggingInfo,
        window_pos: NSPoint,
    ) -> bool {
        if view.is_hidden() {
            return false;
        }

        for child in view.subviews_mut().iter_mut().rev() {
            let child_frame = child.frame();
            if child_frame.contains_point(&window_pos) {
                let local_pos = NSPoint::new(
                    window_pos.x - child_frame.origin.x,
                    window_pos.y - child_frame.origin.y,
                );
                let mut local_info = drag_info.clone();
                local_info.dragging_location = local_pos;
                if Self::dispatch_drop(child, &local_info, local_pos) {
                    return true;
                }
            }
        }

        if let Some(delegate_arc) = view.delegate().clone() {
            if let Ok(mut d) = delegate_arc.lock() {
                return d.perform_drag_operation(view, drag_info);
            }
        }

        false
    }
}
