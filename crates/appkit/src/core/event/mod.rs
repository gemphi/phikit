//! # NSEvent Modular Implementation
use crate::graphics::NSPoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSEventType {
    LeftMouseDown,
    LeftMouseUp,
    RightMouseDown,
    RightMouseUp,
    MouseMoved,
    LeftMouseDragged,
    RightMouseDragged,
    KeyDown,
    KeyUp,
    ScrollWheel,
    MouseEntered,
    MouseExited,
    DroppedFile,
}

#[derive(Debug, Clone)]
pub struct NSEvent {
    pub event_type: NSEventType,
    pub location_in_window: NSPoint,
    pub timestamp: f64,
    pub delta_x: f32,
    pub delta_y: f32,
    pub modifier_flags: u32,
    pub characters: Option<String>,
    pub file_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSDragOperation {
    None,
    Copy,
    Link,
    Generic,
    Move,
    Delete,
}

#[derive(Debug, Clone)]
pub struct NSDraggingInfo {
    pub dragging_location: NSPoint,
    pub dragging_pasteboard_string: String,
    pub source_operation_mask: NSDragOperation,
}

impl NSEvent {
    pub fn event_type(&self) -> NSEventType {
        self.event_type
    }
    pub fn location_in_window(&self) -> NSPoint {
        self.location_in_window
    }
    pub fn scrolling_delta(&self) -> (f32, f32, f32) {
        (self.delta_x, self.delta_y, 0.0)
    }
    pub fn is_option_key_down(&self) -> bool {
        (self.modifier_flags & 1) != 0
    }
    pub fn is_shift_key_down(&self) -> bool {
        (self.modifier_flags & 2) != 0
    }
    pub fn is_command_key_down(&self) -> bool {
        (self.modifier_flags & 4) != 0
    }
    pub fn is_control_key_down(&self) -> bool {
        (self.modifier_flags & 8) != 0
    }

    pub fn characters(&self) -> Option<String> {
        self.characters.clone()
    }

    /// Returns a string representation of the shortcut (e.g., "Cmd+S", "Shift+Cmd+Z")
    pub fn shortcut_string(&self) -> Option<String> {
        if self.event_type != NSEventType::KeyDown {
            return None;
        }

        let chars = self.characters.as_ref()?;
        if chars.is_empty() {
            return None;
        }

        let mut parts = Vec::new();
        if self.is_control_key_down() {
            parts.push("Ctrl");
        }
        if self.is_option_key_down() {
            parts.push("Opt");
        }
        if self.is_shift_key_down() {
            parts.push("Shift");
        }
        if self.is_command_key_down() {
            parts.push("Cmd");
        }

        parts.push(chars);
        Some(parts.join("+"))
    }
}
