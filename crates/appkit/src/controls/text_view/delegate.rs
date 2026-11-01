//! # NSTextView Delegate
//!
//! Delegate protocol for multi-line text view events.
//!
//! ## Apple Documentation
//! - [NSTextViewDelegate](https://developer.apple.com/documentation/appkit/nstextviewdelegate)

use crate::prelude::*;
use std::sync::{Arc, Mutex};

/// Delegate protocol for NSTextView events.
///
/// Apple: [NSTextViewDelegate](https://developer.apple.com/documentation/appkit/nstextviewdelegate)
pub trait NSTextViewDelegate: Send + Sync {
    /// Called when the text changes.
    ///
    /// Apple: `textDidChange:`
    fn text_did_change(&mut self, text: &str) {
        let _ = text;
    }
    
    /// Called when editing begins.
    ///
    /// Apple: `textDidBeginEditing:`
    fn text_did_begin_editing(&mut self) {}
    
    /// Called when editing ends.
    ///
    /// Apple: `textDidEndEditing:`
    fn text_did_end_editing(&mut self, text: &str) {
        let _ = text;
    }
    
    /// Called to validate whether editing should begin.
    ///
    /// Apple: `textShouldBeginEditing:`
    fn should_begin_editing(&mut self) -> bool {
        true
    }
    
    /// Called to validate whether editing should end.
    ///
    /// Apple: `textShouldEndEditing:`
    fn should_end_editing(&mut self, text: &str) -> bool {
        let _ = text;
        true
    }
}

/// Internal state for text view
pub(crate) struct TextViewState {
    pub(crate) text: String,
    pub(crate) is_focused: bool,
    pub(crate) is_editable: bool,
    pub(crate) is_selectable: bool,
    pub(crate) is_bordered: bool,
    pub(crate) cursor_position: usize,
}

impl Default for TextViewState {
    fn default() -> Self {
        Self {
            text: String::new(),
            is_focused: false,
            is_editable: true,
            is_selectable: true,
            is_bordered: true,
            cursor_position: 0,
        }
    }
}

/// Internal view delegate for NSTextView rendering and events.
pub(crate) struct NSTextViewViewDelegate {
    pub(crate) state: Arc<Mutex<TextViewState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSTextViewDelegate>>>>,
}

impl NSViewDelegate for NSTextViewViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let frame = view.bounds();
        
        // Background
        if state.is_bordered {
            let bg_color = if state.is_focused {
                crate::theme::color(crate::theme::keys::CLR_BG_SELECTED)
            } else {
                crate::theme::color(crate::theme::keys::CLR_BG_CONTROL)
            };
            
            context.set_color(bg_color);
            context.fill_rounded_rect(frame, 4.0);
            
            // Border
            let border_color = if state.is_focused {
                crate::theme::color(crate::theme::keys::CLR_ACCENT)
            } else {
                crate::theme::color(crate::theme::keys::CLR_SEPARATOR)
            };
            context.set_color(border_color);
            context.stroke_rounded_rect(frame, 4.0, 1.0);
        }
        
        // Text (multi-line)
        let text_color = crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL);
        context.set_color(text_color);
        
        let padding = 8.0;
        let line_height = 18.0;
        let mut y = frame.min_y() + padding;
        
        for line in state.text.lines() {
            if y + line_height > frame.max_y() - padding {
                break; // Stop if we exceed the frame
            }
            
            context.draw_text(line, NSPoint::new(frame.min_x() + padding, y), 13.0);
            y += line_height;
        }
        
        // Cursor (if focused)
        if state.is_focused && state.is_editable {
            // Simple cursor at end of text
            let cursor_x = frame.min_x() + padding + (state.text.len() as f32 * 7.0); // Rough estimate
            let cursor_y = frame.min_y() + padding;
            context.set_color(crate::theme::color(crate::theme::keys::CLR_ACCENT));
            context.fill_rect(NSRect::new(cursor_x, cursor_y, 1.0, 16.0));
        }
    }
    
    fn mouse_down(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        
        if !state.is_editable && !state.is_selectable {
            return;
        }
        
        // Check if should begin editing
        let should_begin = if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.should_begin_editing()
        } else {
            true
        };
        
        if should_begin {
            state.is_focused = true;
            drop(state);
            
            // Notify delegate
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.text_did_begin_editing();
            }
        }
    }
    
    fn key_down(&mut self, _view: &mut NSView, event: &crate::event::NSEvent) {
        // Handle Escape - end editing
        if let Some(chars) = event.characters() {
            if chars == "\u{1b}" { // Escape key
                let text = self.state.lock().unwrap().text.clone();
                
                // Check if should end editing
                let should_end = if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                    delegate.should_end_editing(&text)
                } else {
                    true
                };
                
                if should_end {
                    self.state.lock().unwrap().is_focused = false;
                    
                    // Notify delegate
                    if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                        delegate.text_did_end_editing(&text);
                    }
                }
                return;
            }
            
            // Handle Backspace
            if chars == "\u{8}" || chars == "\u{7f}" { // Backspace or Delete
                let mut state = self.state.lock().unwrap();
                if !state.text.is_empty() {
                    state.text.pop();
                    let text = state.text.clone();
                    drop(state);
                    
                    // Notify delegate
                    if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                        delegate.text_did_change(&text);
                    }
                }
                return;
            }
            
            // Handle Enter - insert newline
            if chars == "\r" || chars == "\n" {
                let mut state = self.state.lock().unwrap();
                state.text.push('\n');
                let text = state.text.clone();
                drop(state);
                
                // Notify delegate
                if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                    delegate.text_did_change(&text);
                }
                return;
            }
            
            // Handle text input
            let mut state = self.state.lock().unwrap();
            state.text.push_str(&chars);
            let text = state.text.clone();
            drop(state);
            
            // Notify delegate
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.text_did_change(&text);
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
