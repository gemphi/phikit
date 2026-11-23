use super::NSView;
use crate::event::{NSEvent, NSEventType};
use crate::graphics::NSPoint;

impl NSView {
    pub fn dispatch_mouse_event(&mut self, event: &NSEvent, window_height: f32) {
        if self.is_hidden || !self.is_enabled {
            return;
        }

        let window_pos = event.location_in_window();
        let local_pos = NSPoint::new(
            window_pos.x - self.frame.origin.x,
            window_pos.y - self.frame.origin.y,
        );

        let is_continuation = matches!(
            event.event_type(),
            NSEventType::LeftMouseDragged
                | NSEventType::LeftMouseUp
                | NSEventType::RightMouseDragged
                | NSEventType::RightMouseUp
        );

        // If this view's delegate is currently handling a drag (started via
        // wants_mouse_down), route continuation events directly to it instead
        // of dispatching to children. This ensures split view divider drags
        // aren't lost when the cursor moves over child panels.
        if is_continuation && self.delegate_is_dragging {
            let delegate_arc = self.delegate.clone();
            if let Some(delegate) = delegate_arc {
                if let Ok(mut d) = delegate.lock() {
                    match event.event_type() {
                        NSEventType::LeftMouseDragged | NSEventType::RightMouseDragged => {
                            d.mouse_dragged(self, event);
                        }
                        NSEventType::LeftMouseUp | NSEventType::RightMouseUp => {
                            d.mouse_up(self, event);
                            self.delegate_is_dragging = false;
                        }
                        _ => {}
                    }
                }
            }
            return;
        }

        // On mouse down, check if this view's delegate wants to intercept the
        // event BEFORE dispatching to children. This is how NSSplitView claims
        // clicks on divider regions that overlap with child panel frames.
        if matches!(event.event_type(), NSEventType::LeftMouseDown) {
            if let Some(delegate) = &self.delegate {
                let wants = delegate
                    .lock()
                    .map(|d| d.wants_mouse_down(local_pos))
                    .unwrap_or(false);
                if wants {
                    let delegate_arc = self.delegate.clone();
                    if let Some(delegate) = delegate_arc {
                        if let Ok(mut d) = delegate.lock() {
                            d.mouse_down(self, event);
                            self.delegate_is_dragging = true;
                        }
                    }
                    return;
                }
            }
        }

        // For MouseMoved, also check if the delegate wants to handle it at
        // divider regions (for cursor changes). Route to delegate if near a
        // divider, otherwise reset cursor and fall through to children.
        if matches!(event.event_type(), NSEventType::MouseMoved) {
            if let Some(delegate) = &self.delegate {
                let wants = delegate
                    .lock()
                    .map(|d| d.wants_mouse_down(local_pos))
                    .unwrap_or(false);
                if wants {
                    let delegate_arc = self.delegate.clone();
                    if let Some(delegate) = delegate_arc {
                        if let Ok(mut d) = delegate.lock() {
                            d.mouse_moved(self, event);
                        }
                    }
                    return;
                } else {
                    // Not near a divider — reset cursor in case it was previously
                    // set to a resize cursor by this delegate.
                    crate::window::set_cursor(winit::window::CursorIcon::Default);
                }
            }
        }

        // Dispatch to the deepest child that contains the point.
        for subview in self.subviews.iter_mut().rev() {
            if subview.frame.contains_point(&local_pos) {
                let mut local_event = event.clone();
                local_event.location_in_window = NSPoint::new(
                    local_pos.x - subview.frame.origin.x,
                    local_pos.y - subview.frame.origin.y,
                );
                subview.dispatch_mouse_event(&local_event, window_height);
                return;
            }
        }

        // No child handled it — dispatch to this view's delegate
        let delegate_arc = self.delegate.clone();
        if let Some(delegate) = delegate_arc {
            if let Ok(mut d) = delegate.lock() {
                match event.event_type() {
                    NSEventType::LeftMouseDown => d.mouse_down(self, event),
                    NSEventType::LeftMouseUp => d.mouse_up(self, event),
                    NSEventType::LeftMouseDragged => d.mouse_dragged(self, event),
                    NSEventType::RightMouseDown => d.right_mouse_down(self, event),
                    NSEventType::RightMouseUp => d.mouse_up(self, event),
                    NSEventType::RightMouseDragged => d.mouse_dragged(self, event),
                    NSEventType::MouseMoved => d.mouse_moved(self, event),
                    NSEventType::ScrollWheel => d.scroll_wheel(self, event),
                    _ => {}
                }
            }
        }
    }
}
