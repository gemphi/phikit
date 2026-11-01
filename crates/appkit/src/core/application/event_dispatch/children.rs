//! Recursive subview event dispatching and hit testing.
use super::super::NSApplication;
use crate::event::{NSEvent, NSEventType};
use crate::graphics::NSPoint;
use crate::view::NSView;

impl NSApplication {
    /// Dispatches an event to the appropriate child view if it intersects.
    pub(in crate::application) fn dispatch_to_children(view: &mut NSView, event: &NSEvent) -> Option<()> {
        let event_type = event.event_type();
        let pos = event.location_in_window();

        for child in view.subviews_mut().iter_mut().rev() {
            let child_frame = child.frame();

            if !child.is_hidden() && child_frame.contains_point(&pos) {
                if event_type == NSEventType::MouseMoved && child.hover_color.is_some() {
                    child.is_hovered = true;
                }

                let child_pos =
                    NSPoint::new(pos.x - child_frame.origin.x, pos.y - child_frame.origin.y);

                if let Some(delegate_arc) = child.delegate().clone() {
                    if let Ok(d) = delegate_arc.lock() {
                        if !d.hit_test(child, child_pos) {
                            continue;
                        }
                    }
                }

                let child_event = NSEvent {
                    event_type,
                    location_in_window: child_pos,
                    timestamp: event.timestamp,
                    delta_x: event.delta_x,
                    delta_y: event.delta_y,
                    modifier_flags: event.modifier_flags,
                    characters: event.characters.clone(),
                    file_path: event.file_path.clone(),
                };
                Self::dispatch_event_to_view(child, &child_event);
                return Some(());
            } else if event_type == NSEventType::MouseMoved && child.hover_color.is_some() {
                child.is_hovered = false;
            }
        }
        None
    }
}
