//! # NSTextField Delegate
//!
//! Delegate protocol for text field events.
//!
//! ## Apple Documentation
//! - [NSTextFieldDelegate](https://developer.apple.com/documentation/appkit/nstextfielddelegate)
//! - [NSControlTextEditingDelegate](https://developer.apple.com/documentation/appkit/nscontroltexteditingdelegate)

use crate::prelude::*;
use std::sync::{Arc, Mutex};

/// Delegate protocol for NSTextField events.
///
/// Apple: [NSTextFieldDelegate](https://developer.apple.com/documentation/appkit/nstextfielddelegate)
pub trait NSTextFieldDelegate: Send + Sync {
    /// Called when the text changes.
    ///
    /// Apple: `controlTextDidChange:`
    fn text_did_change(&mut self, text: &str) {
        let _ = text;
    }
    
    /// Called when editing begins.
    ///
    /// Apple: `controlTextDidBeginEditing:`
    fn text_did_begin_editing(&mut self) {}
    
    /// Called when editing ends.
    ///
    /// Apple: `controlTextDidEndEditing:`
    fn text_did_end_editing(&mut self, text: &str) {
        let _ = text;
    }
    
    /// Called to validate whether editing should begin.
    ///
    /// Apple: `control:textShouldBeginEditing:`
    fn should_begin_editing(&mut self) -> bool {
        true
    }
    
    /// Called to validate whether editing should end.
    ///
    /// Apple: `control:textShouldEndEditing:`
    fn should_end_editing(&mut self, text: &str) -> bool {
        let _ = text;
        true
    }
}

/// Internal state for text field
pub(crate) struct TextFieldState {
    pub(crate) text: String,
    pub(crate) placeholder: String,
    pub(crate) is_focused: bool,
    pub(crate) is_editable: bool,
    pub(crate) is_bordered: bool,
}

impl Default for TextFieldState {
    fn default() -> Self {
        Self {
            text: String::new(),
            placeholder: String::new(),
            is_focused: false,
            is_editable: true,
            is_bordered: true,
        }
    }
}

/// Internal view delegate for NSTextField rendering and events.
pub(crate) struct NSTextFieldViewDelegate {
    pub(crate) state: Arc<Mutex<TextFieldState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSTextFieldDelegate>>>>,
}

impl NSViewDelegate for NSTextFieldViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let frame = view.bounds();
        
        // Background
        if state.is_bordered {
            let bg_color = if state.is_focused {
                crate::theme::color(crate::theme::keys::CLR_BG_CONTROL_FOCUSED)
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
        
        // Text or placeholder
        let display_text = if state.text.is_empty() && !state.placeholder.is_empty() {
            &state.placeholder
        } else {
            &state.text
        };
        
        let text_color = if state.text.is_empty() {
            crate::theme::color(crate::theme::keys::CLR_TEXT_PLACEHOLDER)
        } else {
            crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL)
        };
        
        context.set_color(text_color);
        let text_x = frame.min_x() + 8.0;
        let text_y = frame.mid_y() - 6.0;
        context.draw_text(display_text, NSPoint::new(text_x, text_y), 13.0);
    }
    
    fn mouse_down(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        
        if !state.is_editable {
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
        let key_code = event.key_code();
        
        // Handle Enter/Return - end editing
        if key_code == 13 {
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
        
        // Handle Escape - cancel editing
        if key_code == 27 {
            self.state.lock().unwrap().is_focused = false;
            return;
        }
        
        // Handle Backspace
        if key_code == 8 {
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
        
        // Handle text input
        if let Some(character) = event.characters() {
            let mut state = self.state.lock().unwrap();
            state.text.push_str(&character);
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
