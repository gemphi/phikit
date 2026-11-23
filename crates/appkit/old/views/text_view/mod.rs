//! # NSTextView
//!
//! Mirrors Apple's [`NSTextView`](https://developer.apple.com/documentation/appkit/nstextview).
//! A multiline, editable, scrollable text view. Used for script editors,
//! console output, and any rich text display.
//!
//! Distinct from `NSTextField` which is single-line.

use crate::theme::{self, keys};
use crate::{NSColor, NSRect, NSView};

/// A multiline text view.
pub struct NSTextView {
    view: NSView,
    text: String,
    is_editable: bool,
    is_selectable: bool,
    is_rich_text: bool,
    font_size: f32,
    insertion_point: usize,
}

impl NSTextView {
    pub fn new(frame: NSRect) -> Self {
        let view = NSView::new(frame).with_name("NSTextView");
        Self {
            view,
            text: String::new(),
            is_editable: true,
            is_selectable: true,
            is_rich_text: false,
            font_size: theme::f32_or(keys::SIZE_FONT_UI, 13.0),
            insertion_point: 0,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.view.set_needs_display();
    }
    pub fn set_editable(&mut self, editable: bool) {
        self.is_editable = editable;
    }
    pub fn set_selectable(&mut self, selectable: bool) {
        self.is_selectable = selectable;
    }
    pub fn set_rich_text(&mut self, rich: bool) {
        self.is_rich_text = rich;
    }
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
    }
    pub fn insert_text(&mut self, text: &str) {
        self.text.insert_str(self.insertion_point, text);
        self.insertion_point += text.len();
        self.view.set_needs_display();
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }
    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }
}
