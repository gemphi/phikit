//! Text field types, state, and core construction/accessors.

use super::super::{NSColor, NSControl, NSPoint, NSRect, NSView};
use std::sync::{Arc, Mutex};

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSTextAlignment {
    Left,
    Right,
    Center,
    Justified,
    Natural,
}

/// Shared text field state
pub(super) struct NSTextFieldState {
    pub(super) text: String,
    pub(super) editable: bool,
    pub(super) bordered: bool,
    pub(super) text_color: NSColor,
    pub(super) alignment: NSTextAlignment,
    pub(super) is_scrubbing: bool,
    pub(super) scrub_start_value: f32,
    pub(super) scrub_start_pos: f32,
    pub(super) action: Option<Arc<dyn Fn(String) + Send + Sync>>,
    // Caret and selection for keyboard input
    pub(super) caret_position: usize,
    pub(super) selection_start: usize,
    pub(super) selection_end: usize,
    pub(super) is_first_responder: bool,
    pub(super) font_size: f32,
}

/// Text field control
#[derive(Clone)]
pub struct NSTextField {
    pub(super) control: NSControl,
    pub(super) inner: Arc<Mutex<NSTextFieldState>>,
}

impl NSTextField {
    pub fn new(frame: NSRect) -> Self {
        let inner = Arc::new(Mutex::new(NSTextFieldState {
            text: String::new(),
            editable: true,
            bordered: true,
            text_color: crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL),
            alignment: NSTextAlignment::Left,
            is_scrubbing: false,
            scrub_start_value: 0.0,
            scrub_start_pos: 0.0,
            action: None,
            caret_position: 0,
            selection_start: 0,
            selection_end: 0,
            is_first_responder: false,
            font_size: crate::theme::f32_or(crate::theme::keys::SIZE_FONT_UI, 13.0),
        }));

        let mut control = NSControl::new(frame);
        control.view_mut().set_delegate(super::rendering::NSTextFieldDelegate {
            state: inner.clone(),
        });

        Self { control, inner }
    }

    pub fn label(text: &str) -> Self {
        let char_w = crate::theme::f32_or(crate::theme::keys::SIZE_CHAR_WIDTH, 7.5);
        let padding = crate::theme::f32_or(crate::theme::keys::SPACING_SM, 4.0) * 2.0;
        let font_sz = crate::theme::f32_or(crate::theme::keys::SIZE_FONT_UI, 13.0);
        let w = (text.chars().count() as f32 * char_w) + padding;
        let h = font_sz + 8.0; // font size + vertical padding
        let mut f = Self::new(NSRect::sized(w.max(20.0), h));
        f.set_text(text);
        f.set_bordered(false);
        f.set_editable(false);
        f
    }

    pub fn set_text(&mut self, text: &str) {
        self.inner.lock().unwrap().text = text.to_string();
    }

    pub fn text(&self) -> String {
        self.inner.lock().unwrap().text.clone()
    }

    pub fn set_action(&mut self, action: impl Fn(String) + Send + Sync + 'static) {
        self.inner.lock().unwrap().action = Some(Arc::new(action));
    }

    pub fn control_mut(&mut self) -> &mut NSControl {
        &mut self.control
    }

    pub fn view_mut(&mut self) -> &mut NSView {
        self.control.view_mut()
    }

    pub fn view_clone(&self) -> NSView {
        self.control.view().clone()
    }

    /// AppKit-compatible alias for set_text
    pub fn set_string_value(&mut self, text: &str) {
        self.set_text(text);
    }

    /// Get the underlying view (AppKit-compatible)
    pub fn view(&self) -> NSView {
        self.view_clone()
    }

    /// Set placeholder text (stub - stores but doesn't display yet)
    pub fn set_placeholder(&mut self, _text: &str) {
        // Placeholder functionality not yet implemented
    }

    /// Set editable state
    pub fn set_editable(&mut self, editable: bool) {
        self.inner.lock().unwrap().editable = editable;
    }

    /// Set bordered state
    pub fn set_bordered(&mut self, bordered: bool) {
        self.inner.lock().unwrap().bordered = bordered;
    }

    /// Set text color
    pub fn set_text_color(&mut self, color: NSColor) {
        self.inner.lock().unwrap().text_color = color;
    }

    /// Set text alignment
    pub fn set_text_alignment(&mut self, alignment: NSTextAlignment) {
        self.inner.lock().unwrap().alignment = alignment;
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.inner.lock().unwrap().font_size = size;
    }

    // ── Builder Methods ──────────────────────────────────────────────

    /// Set the text (builder pattern).
    pub fn with_text(mut self, text: &str) -> Self {
        self.set_text(text);
        self
    }

    /// Set the string value (builder pattern, AppKit-compatible).
    pub fn with_string_value(mut self, text: &str) -> Self {
        self.set_string_value(text);
        self
    }

    /// Set the placeholder (builder pattern).
    pub fn with_placeholder(mut self, text: &str) -> Self {
        self.set_placeholder(text);
        self
    }

    /// Set editable state (builder pattern).
    pub fn with_editable(mut self, editable: bool) -> Self {
        self.set_editable(editable);
        self
    }

    /// Set bordered state (builder pattern).
    pub fn with_bordered(mut self, bordered: bool) -> Self {
        self.set_bordered(bordered);
        self
    }

    /// Set text color (builder pattern).
    pub fn with_text_color(mut self, color: NSColor) -> Self {
        self.set_text_color(color);
        self
    }

    /// Set text alignment (builder pattern).
    pub fn with_text_alignment(mut self, alignment: NSTextAlignment) -> Self {
        self.set_text_alignment(alignment);
        self
    }

    /// Set font size (builder pattern).
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.set_font_size(size);
        self
    }

    /// Set the action (builder pattern).
    pub fn with_action(mut self, action: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.set_action(action);
        self
    }

    /// Set the view name (builder pattern).
    pub fn with_name(mut self, name: &str) -> Self {
        self.control.view_mut().set_name(name);
        self
    }

    /// Set the frame (builder pattern).
    pub fn with_frame(mut self, frame: NSRect) -> Self {
        self.control.view_mut().set_frame(frame);
        self
    }

    /// Set the width (builder pattern).
    pub fn with_width(mut self, width: f32) -> Self {
        let current = self.control.view().frame();
        self.control.view_mut().set_frame(NSRect::new(current.x(), current.y(), width, current.height()));
        self
    }

    /// Set the height (builder pattern).
    pub fn with_height(mut self, height: f32) -> Self {
        let current = self.control.view().frame();
        self.control.view_mut().set_frame(NSRect::new(current.x(), current.y(), current.width(), height));
        self
    }

    /// Set the background color (builder pattern).
    pub fn with_background_color(mut self, color: Option<NSColor>) -> Self {
        self.control.view_mut().set_background_color(color);
        self
    }
}
