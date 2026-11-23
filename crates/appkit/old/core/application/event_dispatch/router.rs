//! Event dispatch router for NSApplication.
use super::super::NSApplication;
use crate::event::{NSEvent, NSEventType};
use crate::graphics::NSPoint;
use crate::view::NSView;

impl NSApplication {
    /// Dispatches events down the responder chain or directly to hit-tested views.
    pub fn dispatch_event_to_view(view: &mut NSView, event: &NSEvent) {
        if view.is_hidden() {
            return;
        }

        // Handle active drag sessions
        if event.event_type() == NSEventType::LeftMouseDragged {
            if let Ok(app) = Self::shared().try_lock() {
                if app.active_drag_session.is_some() {
                    Self::dispatch_drag_update(
                        view,
                        app.active_drag_session.as_ref().unwrap(),
                        event.location_in_window(),
                    );
                    return;
                }
            }
        }

        // Forward dragging/mouse interactions if a view has captured mouse input
        if Self::handle_captured_view_event(view, event) {
            return;
        }

        // Let parent view delegate claim before propagating down
        if Self::check_delegate_claim(view, event).is_some() {
            return;
        }

        // Recursively dispatch to intersecting child views
        if Self::dispatch_to_children(view, event).is_some() {
            return;
        }

        // Standard direct delivery
        Self::deliver_to_view_delegate(view, event);
    }

    /// Dispatches mouse dragging or mouse release events to views currently capturing input.
    fn handle_captured_view_event(view: &mut NSView, event: &NSEvent) -> bool {
        let captured_opt = {
            if let Ok(app) = Self::shared().try_lock() {
                app.captured_view_id
            } else {
                None
            }
        };

        if let Some(id) = captured_opt {
            if view.id == id {
                Self::deliver_to_view_delegate(view, event);
                return true;
            }

            for child in view.subviews_mut() {
                if child.is_hidden() {
                    continue;
                }
                let child_frame = child.frame();
                let child_event = NSEvent {
                    event_type: event.event_type(),
                    location_in_window: NSPoint::new(
                        event.location_in_window.x - child_frame.origin.x,
                        event.location_in_window.y - child_frame.origin.y,
                    ),
                    timestamp: event.timestamp,
                    delta_x: event.delta_x,
                    delta_y: event.delta_y,
                    modifier_flags: event.modifier_flags,
                    characters: event.characters.clone(),
                    file_path: event.file_path.clone(),
                };
                if Self::handle_captured_view_event(child, &child_event) {
                    return true;
                }
            }
        }
        false
    }
}
