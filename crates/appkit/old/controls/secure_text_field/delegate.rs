//! # NSSecureTextField Delegate
//!
//! Delegate protocol for secure text field events.
//!
//! ## Apple Documentation
//! - [NSTextFieldDelegate](https://developer.apple.com/documentation/appkit/nstextfielddelegate)

use crate::prelude::*;
use std::sync::{Arc, Mutex};

/// Delegate protocol for NSSecureTextField events.
///
/// Apple: Uses NSTextFieldDelegate
pub trait NSSecureTextFieldDelegate: Send + Sync {
    /// Called when the password text changes.
    ///
    /// Apple: `controlTextDidChange:`
    fn text_did_change(&mut self, length: usize) {
        let _ = length; // Don't pass actual password text
    }
    
    /// Called when editing begins.
    ///
    /// Apple: `controlTextDidBeginEditing:`
    fn text_did_begin_editing(&mut self) {}
    
    /// Called when editing ends (e.g., Enter pressed).
    ///
    /// Apple: `controlTextDidEndEditing:`
    fn text_did_end_editing(&mut self, password: &str) {
        let _ = password;
    }
}

/// Internal state for secure text field
pub(crate) struct SecureTextFieldState {
    pub(crate) text: String,
    pub(crate) placeholder: String,
    pub(crate) is_focused: bool,
}

impl Default for SecureTextFieldState {
    fn default() -> Self {
        Self {
            text: String::new(),
            placeholder: String::from("Password"),
            is_focused: false,
        }
    }
}

/// Internal view delegate for NSSecureTextField rendering and events.
pub(crate) struct NSSecureTextFieldViewDelegate {
    pub(crate) state: Arc<Mutex<SecureTextFieldState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSSecureTextFieldDelegate>>>>,
}

impl NSViewDelegate for NSSecureTextFieldViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let frame = view.bounds();
        
        // Background
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
        
        // Display bullets instead of actual text
        let display_text = if state.text.is_empty() {
            &state.placeholder
        } else {
            // Show bullets (•) for each character
            &"•".repeat(state.text.len())
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
        state.is_focused = true;
        drop(state);
        
        // Notify delegate
        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.text_did_begin_editing();
        }
    }
    
    fn key_down(&mut self, _view: &mut NSView, event: &crate::event::NSEvent) {
        let key_code = event.key_code();
        
        // Handle Enter/Return - end editing
        if key_code == 13 {
            let password = self.state.lock().unwrap().text.clone();
            self.state.lock().unwrap().is_focused = false;
            
            // Notify delegate with actual password
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.text_did_end_editing(&password);
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
                let length = state.text.len();
                drop(state);
                
                // Notify delegate (only pass length, not actual text)
                if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                    delegate.text_did_change(length);
                }
            }
            return;
        }
        
        // Handle text input
        if let Some(character) = event.characters() {
            let mut state = self.state.lock().unwrap();
            state.text.push_str(&character);
            let length = state.text.len();
            drop(state);
            
            // Notify delegate (only pass length, not actual text)
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.text_did_change(length);
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
