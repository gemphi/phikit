//! # NSComboBox Delegate
//!
//! Delegate protocol for combo box text changes and item selection.
//!
//! ## Apple Documentation
//! - [NSComboBoxDelegate](https://developer.apple.com/documentation/appkit/nscomboboxdelegate)
//! - [NSComboBox](https://developer.apple.com/documentation/appkit/nscombobox)

use crate::prelude::*;
use std::sync::{Arc, Mutex};

/// Delegate protocol for NSComboBox events.
///
/// Apple: [NSComboBoxDelegate](https://developer.apple.com/documentation/appkit/nscomboboxdelegate)
pub trait NSComboBoxDelegate: Send + Sync {
    /// Called when the text in the combo box changes.
    ///
    /// Apple: Equivalent to `comboBoxSelectionDidChange:` and text field delegate methods
    fn text_did_change(&mut self, text: &str) {
        let _ = text;
    }

    /// Called when an item is selected from the dropdown list.
    ///
    /// Apple: Equivalent to `comboBoxSelectionDidChange:`
    fn did_select_item(&mut self, index: usize) {
        let _ = index;
    }

    /// Called when the dropdown will pop up.
    ///
    /// Apple: Equivalent to `comboBoxWillPopUp:`
    fn will_pop_up(&mut self) {}

    /// Called when the dropdown will dismiss.
    ///
    /// Apple: Equivalent to `comboBoxWillDismiss:`
    fn will_dismiss(&mut self) {}
}

/// Internal state for combo box
pub(crate) struct ComboBoxState {
    pub(crate) items: Vec<String>,
    pub(crate) text: String,
    pub(crate) placeholder: String,
    pub(crate) selected_index: Option<usize>,
    pub(crate) is_open: bool,
    pub(crate) is_editing: bool,
    pub(crate) cursor_position: usize,
}

impl Default for ComboBoxState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            text: String::new(),
            placeholder: String::new(),
            selected_index: None,
            is_open: false,
            is_editing: false,
            cursor_position: 0,
        }
    }
}

/// Internal view delegate for NSComboBox rendering and events.
pub(crate) struct NSComboBoxViewDelegate {
    pub(crate) state: Arc<Mutex<ComboBoxState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSComboBoxDelegate>>>>,
}

impl NSComboBoxViewDelegate {
    /// Calculate the dropdown button area
    fn dropdown_button_rect(&self, frame: NSRect) -> NSRect {
        let button_width = 20.0;
        NSRect::new(
            frame.max_x() - button_width,
            frame.min_y(),
            button_width,
            frame.height(),
        )
    }

    /// Calculate the text field area
    fn text_field_rect(&self, frame: NSRect) -> NSRect {
        let button_width = 20.0;
        NSRect::new(
            frame.min_x(),
            frame.min_y(),
            frame.width() - button_width,
            frame.height(),
        )
    }

    /// Calculate dropdown list rect
    fn dropdown_list_rect(&self, frame: NSRect, item_count: usize) -> NSRect {
        let max_visible_items = 8;
        let item_height = 20.0;
        let visible_items = item_count.min(max_visible_items);
        let list_height = visible_items as f32 * item_height;

        NSRect::new(
            frame.min_x(),
            frame.min_y() - list_height,
            frame.width(),
            list_height,
        )
    }
}

impl NSViewDelegate for NSComboBoxViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let frame = view.bounds();

        // Draw background
        context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_CONTROL));
        context.fill_rounded_rect(frame, 4.0);

        // Draw border
        context.set_color(crate::theme::color(crate::theme::keys::CLR_BORDER));
        context.stroke_rounded_rect(frame, 4.0, 1.0);

        // Draw text or placeholder
        let text_rect = self.text_field_rect(frame);
        let text_padding = 6.0;
        let text_x = text_rect.min_x() + text_padding;
        let text_y = text_rect.mid_y();

        if state.text.is_empty() && !state.placeholder.is_empty() {
            // Draw placeholder
            context.set_color(crate::theme::color(crate::theme::keys::CLR_TEXT_SECONDARY));
            context.draw_text(&state.placeholder, NSPoint::new(text_x, text_y), 13.0);
        } else {
            // Draw text
            context.set_color(crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL));
            context.draw_text(&state.text, NSPoint::new(text_x, text_y), 13.0);
        }

        // Draw dropdown button
        let button_rect = self.dropdown_button_rect(frame);
        context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_BUTTON_HOVER));
        context.fill_rounded_rect(button_rect, 4.0);

        // Draw dropdown arrow (simple down triangle using text)
        let arrow_x = button_rect.mid_x();
        let arrow_y = button_rect.mid_y();
        context.set_color(crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL));
        context.draw_text("▼", NSPoint::new(arrow_x - 4.0, arrow_y - 6.0), 10.0);

        // Draw dropdown list if open
        if state.is_open && !state.items.is_empty() {
            let list_rect = self.dropdown_list_rect(frame, state.items.len());

            // Draw list background
            context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_PANEL));
            context.fill_rounded_rect(list_rect, 4.0);

            // Draw list border
            context.set_color(crate::theme::color(crate::theme::keys::CLR_BORDER));
            context.stroke_rounded_rect(list_rect, 4.0, 1.0);

            // Draw items
            let item_height = 20.0;
            for (i, item) in state.items.iter().enumerate() {
                let item_y = list_rect.max_y() - (i as f32 + 1.0) * item_height;
                let item_rect = NSRect::new(list_rect.min_x(), item_y, list_rect.width(), item_height);

                // Highlight selected item
                if state.selected_index == Some(i) {
                    context.set_color(crate::theme::color(crate::theme::keys::CLR_ACCENT));
                    context.fill_rect(item_rect);
                    context.set_color(NSColor::white());
                } else {
                    context.set_color(crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL));
                }

                // Draw item text
                let item_text_x = item_rect.min_x() + text_padding;
                let item_text_y = item_rect.mid_y() - 6.0;
                context.draw_text(item, NSPoint::new(item_text_x, item_text_y), 13.0);
            }
        }
    }

    fn mouse_down(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let frame = view.frame();
        let button_rect = self.dropdown_button_rect(frame);
        let click_pos = event.location_in_window();

        // Check if click is on dropdown button
        if button_rect.contains_point(&click_pos) {
            let mut state = self.state.lock().unwrap();
            let was_open = state.is_open;
            state.is_open = !state.is_open;
            drop(state);

            // Notify delegate
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                if was_open {
                    delegate.will_dismiss();
                } else {
                    delegate.will_pop_up();
                }
            }
        } else {
            // Check if click is on dropdown list
            let state = self.state.lock().unwrap();
            if state.is_open && !state.items.is_empty() {
                let list_rect = self.dropdown_list_rect(frame, state.items.len());
                if list_rect.contains_point(&click_pos) {
                    // Calculate which item was clicked
                    let item_height = 20.0;
                    let relative_y = list_rect.max_y() - click_pos.y;
                    let clicked_index = (relative_y / item_height) as usize;

                    if clicked_index < state.items.len() {
                        let selected_text = state.items[clicked_index].clone();
                        drop(state);

                        // Update state
                        let mut state = self.state.lock().unwrap();
                        state.selected_index = Some(clicked_index);
                        state.text = selected_text.clone();
                        state.is_open = false;
                        drop(state);

                        // Notify delegate
                        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                            delegate.did_select_item(clicked_index);
                            delegate.text_did_change(&selected_text);
                            delegate.will_dismiss();
                        }
                    }
                } else {
                    // Click outside dropdown - close it
                    drop(state);
                    let mut state = self.state.lock().unwrap();
                    if state.is_open {
                        state.is_open = false;
                        drop(state);

                        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                            delegate.will_dismiss();
                        }
                    }
                }
            } else {
                // Click on text field - start editing
                drop(state);
                let mut state = self.state.lock().unwrap();
                state.is_editing = true;
            }
        }
    }

    fn key_down(&mut self, _view: &mut NSView, event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();

        if !state.is_editing {
            return;
        }

        // Handle key events using characters
        if let Some(chars) = event.characters() {
            if chars == "\r" || chars == "\n" {
                // Enter key - finish editing
                state.is_editing = false;
                let text = state.text.clone();
                drop(state);

                if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                    delegate.text_did_change(&text);
                }
            } else if chars == "\u{1b}" {
                // Escape key - cancel editing
                state.is_editing = false;
            } else if chars == "\u{8}" || chars == "\u{7f}" {
                // Backspace or Delete
                if !state.text.is_empty() {
                    state.text.pop();
                    let text = state.text.clone();
                    drop(state);

                    if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                        delegate.text_did_change(&text);
                    }
                }
            } else {
                // Regular character input
                for ch in chars.chars() {
                    if ch.is_alphanumeric() || ch.is_whitespace() || ch.is_ascii_punctuation() {
                        state.text.push(ch);
                    }
                }
                let text = state.text.clone();
                drop(state);

                if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                    delegate.text_did_change(&text);
                }
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
