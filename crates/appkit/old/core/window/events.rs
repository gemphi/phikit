//! # NSWindow Event Processing
use super::window::NSWindow;
use crate::event::{NSEvent, NSEventType};

impl NSWindow {
    pub fn process_event(&mut self, event: &NSEvent) -> bool {
        // Handle shortcuts first
        if event.event_type() == NSEventType::KeyDown {
            if let Some(shortcut) = event.shortcut_string() {
                let handler_arc = self.shortcut_handler.clone();
                if let Some(handler) = handler_arc {
                    if handler(&shortcut) {
                        return true; // Event handled by shortcut
                    }
                }
            }
        }

        // High-level window event processing (moving, resizing, focus)
        false
    }
}
