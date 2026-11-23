//! # NSAlert — Modal Dialogs
//!
//! Direct port of Apple's NSAlert for showing modal alerts to the user.
//!
//! ## Apple Documentation
//! - [NSAlert](https://developer.apple.com/documentation/appkit/nsalert)
//! - [NSAlertStyle](https://developer.apple.com/documentation/appkit/nsalert/style)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut alert = NSAlert::new();
//! alert.set_message_text("Save changes?");
//! alert.set_informative_text("Your changes will be lost if you don't save.");
//! alert.add_button_with_title("Save");
//! alert.add_button_with_title("Cancel");
//! alert.run_modal();
//! ```
//!
//! ## See Also
//! - [NSAlert](https://developer.apple.com/documentation/appkit/nsalert)

use crate::{NSButton, NSColor, NSPoint, NSRect, NSSize, NSTextField, NSView, NSWindow};
use std::sync::{Arc, Mutex};

/// **NSAlertStyle** — The severity level of the alert
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSAlertStyle {
    Warning = 0,
    Informational = 1,
    Critical = 2,
}

/// **NSAlert** — A modal alert dialog
pub struct NSAlert {
    message_text: String,
    informative_text: String,
    alert_style: NSAlertStyle,
    buttons: Vec<String>,
}

impl NSAlert {
    pub fn new() -> Self {
        Self {
            message_text: String::new(),
            informative_text: String::new(),
            alert_style: NSAlertStyle::Informational,
            buttons: Vec::new(),
        }
    }

    pub fn set_message_text(&mut self, text: &str) {
        self.message_text = text.to_string();
    }
    pub fn set_informative_text(&mut self, text: &str) {
        self.informative_text = text.to_string();
    }
    pub fn set_alert_style(&mut self, style: NSAlertStyle) {
        self.alert_style = style;
    }

    /// **addButtonWithTitle** — Add a button to the alert
    pub fn add_button_with_title(&mut self, title: &str) {
        self.buttons.push(title.to_string());
    }

    /// **runModal** — Show the alert modally (stub for now, just logs)
    /// In a full implementation, this would create a window and block.
    pub fn run_modal(&self) -> i64 {
        println!("[NSAlert] Running Modal Alert:");
        println!("  Style: {:?}", self.alert_style);
        println!("  Message: {}", self.message_text);
        println!("  Info: {}", self.informative_text);
        for (i, btn) in self.buttons.iter().enumerate() {
            println!("  Button {}: {}", i + 1, btn);
        }

        // Default to first button (typically "OK" or "Yes")
        1000
    }
}

impl Default for NSAlert {
    fn default() -> Self {
        Self::new()
    }
}
