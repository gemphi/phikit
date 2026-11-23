//! # NSDisclosureTriangle Delegate
//!
//! Delegate protocol for disclosure triangle (expand/collapse).
//!
//! ## Apple Documentation
//! - [NSButton (disclosure)](https://developer.apple.com/documentation/appkit/nsbutton)

use crate::prelude::*;
use std::sync::{Arc, Mutex};

/// Delegate protocol for disclosure triangle events.
///
/// Apple: Part of NSButton with disclosure bezel style
pub trait NSDisclosureDelegate: Send + Sync {
    /// Called when the disclosure state toggles.
    ///
    /// Apple: Sent via action method
    fn did_toggle(&mut self, is_expanded: bool) {
        let _ = is_expanded;
    }
}

/// Internal state for disclosure triangle
pub(crate) struct DisclosureState {
    pub(crate) is_expanded: bool,
}

impl Default for DisclosureState {
    fn default() -> Self {
        Self {
            is_expanded: false,
        }
    }
}

/// Internal view delegate for disclosure triangle rendering and events.
pub(crate) struct NSDisclosureViewDelegate {
    pub(crate) state: Arc<Mutex<DisclosureState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSDisclosureDelegate>>>>,
}

impl NSViewDelegate for NSDisclosureViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let frame = view.bounds();
        
        // Draw triangle
        let triangle_size = 8.0;
        let center_x = frame.mid_x();
        let center_y = frame.mid_y();
        
        context.set_color(crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL));
        
        if state.is_expanded {
            // Down-pointing triangle (▼)
            context.draw_text("▼", NSPoint::new(center_x - 4.0, center_y - 4.0), 10.0);
        } else {
            // Right-pointing triangle (▶)
            context.draw_text("▶", NSPoint::new(center_x - 4.0, center_y - 4.0), 10.0);
        }
    }
    
    fn mouse_down(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        state.is_expanded = !state.is_expanded;
        let new_state = state.is_expanded;
        drop(state);
        
        // Notify delegate
        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.did_toggle(new_state);
        }
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
