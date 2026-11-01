//! # NSColorWell Delegate
//!
//! Delegate protocol for color selection.
//!
//! ## Apple Documentation
//! - [NSColorWell](https://developer.apple.com/documentation/appkit/nscolorwell)

use crate::prelude::*;
use std::sync::{Arc, Mutex};

/// Delegate protocol for NSColorWell events.
///
/// Apple: NSColorWell uses target-action pattern
pub trait NSColorWellDelegate: Send + Sync {
    /// Called when a color is selected.
    ///
    /// Apple: Sent via action method
    fn did_select_color(&mut self, color: NSColor) {
        let _ = color;
    }
    
    /// Called when the color panel is about to be shown.
    fn will_activate(&mut self) {}
    
    /// Called when the color panel is closed.
    fn did_deactivate(&mut self) {}
}

/// Internal state for color well
pub(crate) struct ColorWellState {
    pub(crate) color: NSColor,
    pub(crate) is_active: bool,
    pub(crate) is_bordered: bool,
}

impl Default for ColorWellState {
    fn default() -> Self {
        Self {
            color: NSColor::accent(),
            is_active: false,
            is_bordered: true,
        }
    }
}

/// Internal view delegate for NSColorWell rendering and events.
pub(crate) struct NSColorWellViewDelegate {
    pub(crate) state: Arc<Mutex<ColorWellState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSColorWellDelegate>>>>,
}

impl NSViewDelegate for NSColorWellViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let frame = view.bounds();
        
        // Draw color swatch
        context.set_color(state.color);
        context.fill_rounded_rect(frame, 4.0);
        
        // Draw border
        if state.is_bordered {
            let border_color = if state.is_active {
                crate::theme::color(crate::theme::keys::CLR_ACCENT)
            } else {
                crate::theme::color(crate::theme::keys::CLR_SEPARATOR)
            };
            
            context.set_color(border_color);
            context.stroke_rounded_rect(frame, 4.0, if state.is_active { 2.0 } else { 1.0 });
        }
    }
    
    fn mouse_down(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        state.is_active = true;
        drop(state);
        
        // Notify delegate
        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.will_activate();
        }
        
        // TODO: Show color picker panel
        // For now, just simulate color selection
        // In a full implementation, this would open NSColorPanel
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
