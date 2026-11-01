//! # NSSearchField Delegate
//!
//! Delegate protocol for search field events.
//!
//! ## Apple Documentation
//! - [NSSearchFieldDelegate](https://developer.apple.com/documentation/appkit/nssearchfielddelegate)
//! - [NSTextFieldDelegate](https://developer.apple.com/documentation/appkit/nstextfielddelegate)

use crate::prelude::*;
use std::sync::{Arc, Mutex};

/// Delegate protocol for NSSearchField events.
///
/// Apple: [NSSearchFieldDelegate](https://developer.apple.com/documentation/appkit/nssearchfielddelegate)
pub trait NSSearchFieldDelegate: Send + Sync {
    /// Called when the text in the search field changes.
    ///
    /// Apple: `controlTextDidChange:`
    fn text_did_change(&mut self, text: &str) {
        let _ = text; // Default: do nothing
    }
    
    /// Called when the user presses Enter or starts a search.
    ///
    /// Apple: `searchFieldDidStartSearching:`
    fn search_did_start(&mut self, text: &str) {
        let _ = text; // Default: do nothing
    }
    
    /// Called when the search field loses focus or search ends.
    ///
    /// Apple: `searchFieldDidEndSearching:`
    fn search_did_end(&mut self) {
        // Default: do nothing
    }
    
    /// Called when the user clicks the search button.
    fn search_button_clicked(&mut self, text: &str) {
        // Default: same as search_did_start
        self.search_did_start(text);
    }
    
    /// Called when the user clicks the cancel button.
    fn cancel_button_clicked(&mut self) {
        // Default: same as search_did_end
        self.search_did_end();
    }
}

/// Internal state for search field
pub(crate) struct SearchFieldState {
    pub(crate) text: String,
    pub(crate) placeholder: String,
    pub(crate) is_focused: bool,
    pub(crate) is_searching: bool,
}

impl Default for SearchFieldState {
    fn default() -> Self {
        Self {
            text: String::new(),
            placeholder: String::from("Search"),
            is_focused: false,
            is_searching: false,
        }
    }
}

/// Internal view delegate for NSSearchField rendering and events.
pub(crate) struct NSSearchFieldViewDelegate {
    pub(crate) state: Arc<Mutex<SearchFieldState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSSearchFieldDelegate>>>>,
}

impl NSViewDelegate for NSSearchFieldViewDelegate {
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
        context.fill_rounded_rect(frame, frame.height() / 2.0); // Pill shape
        
        // Text or placeholder
        let display_text = if state.text.is_empty() {
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
        let text_x = frame.min_x() + 12.0; // Padding for search icon
        let text_y = frame.mid_y() - 6.0;
        context.draw_text(display_text, NSPoint::new(text_x, text_y), 13.0);
        
        // TODO: Draw search icon on left
        // TODO: Draw clear button on right (if text not empty)
    }
    
    fn mouse_down(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        state.is_focused = true;
    }
    
    fn key_down(&mut self, _view: &mut NSView, event: &crate::event::NSEvent) {
        // Handle Enter key
        if let Some(chars) = event.characters() {
            if chars == "\r" || chars == "\n" {
                let text = self.state.lock().unwrap().text.clone();
                
                // Call delegate
                if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                    delegate.search_did_start(&text);
                }
                
                self.state.lock().unwrap().is_searching = true;
                return;
            }
            
            // Handle Escape key
            if chars == "\u{001B}" { // ESC character
                self.state.lock().unwrap().text.clear();
                
                // Call delegate
                if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                    delegate.cancel_button_clicked();
                }
                return;
            }
        }
        
        // Handle text input
        if let Some(character) = event.characters() {
            let mut state = self.state.lock().unwrap();
            state.text.push_str(&character);
            let text = state.text.clone();
            drop(state);
            
            // Call delegate
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
