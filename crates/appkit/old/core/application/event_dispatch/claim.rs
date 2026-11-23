//! Delegate event claim and delivery handlers.
use super::super::NSApplication;
use crate::event::{NSEvent, NSEventType};
use crate::view::NSView;

impl NSApplication {
    /// Checks if a view's delegate wants to claim mouse down or scroll events before children.
    pub(in crate::application) fn check_delegate_claim(view: &mut NSView, event: &NSEvent) -> Option<()> {
        let event_type = event.event_type();
        let pos = event.location_in_window();

        let is_action = matches!(
            event_type,
            NSEventType::LeftMouseDown | NSEventType::RightMouseDown | NSEventType::ScrollWheel
        );

        if is_action {
            if let Some(delegate_arc) = view.delegate().clone() {
                if let Ok(d) = delegate_arc.lock() {
                    let claim = if event_type == NSEventType::ScrollWheel {
                        d.wants_scroll_wheel()
                    } else {
                        d.wants_mouse_down(pos)
                    };

                    if claim {
                        drop(d);
                        if let Ok(mut d) = delegate_arc.lock() {
                            if let Ok(mut app) = Self::shared().try_lock() {
                                app.captured_view_id = Some(view.id);
                            }
                            match event_type {
                                NSEventType::LeftMouseDown => d.mouse_down(view, event),
                                NSEventType::RightMouseDown => d.right_mouse_down(view, event),
                                NSEventType::ScrollWheel => d.scroll_wheel(view, event),
                                _ => {}
                            }
                        }
                        return Some(());
                    }
                }
            }
        }
        None
    }

    /// Delivers the event directly to the view's registered delegate.
    pub(in crate::application) fn deliver_to_view_delegate(view: &mut NSView, event: &NSEvent) {
        let event_type = event.event_type();
        if let Some(delegate_arc) = view.delegate().clone() {
            if let Ok(mut d) = delegate_arc.lock() {
                match event_type {
                    NSEventType::LeftMouseDown => {
                        if let Ok(mut app) = Self::shared().try_lock() {
                            app.captured_view_id = Some(view.id);
                        }
                        d.mouse_down(view, event);
                    }
                    NSEventType::RightMouseDown => {
                        if let Ok(mut app) = Self::shared().try_lock() {
                            app.captured_view_id = Some(view.id);
                        }
                        d.right_mouse_down(view, event);
                    }
                    NSEventType::LeftMouseUp | NSEventType::RightMouseUp => {
                        d.mouse_up(view, event);
                        if let Ok(mut app) = Self::shared().try_lock() {
                            app.captured_view_id = None;
                        }
                    }
                    NSEventType::LeftMouseDragged | NSEventType::RightMouseDragged => {
                        d.mouse_dragged(view, event)
                    }
                    NSEventType::MouseMoved => d.mouse_moved(view, event),
                    NSEventType::ScrollWheel => d.scroll_wheel(view, event),
                    NSEventType::KeyDown => d.key_down(view, event),
                    NSEventType::KeyUp => d.key_up(view, event),
                    NSEventType::DroppedFile => d.dropped_file(view, event),
                    _ => {}
                }
            }
        }
    }
}
