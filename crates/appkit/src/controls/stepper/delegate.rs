//! # NSStepper Delegate
//!
//! Delegate protocol for stepper value changes.
//!
//! ## Apple Documentation
//! - [NSStepper](https://developer.apple.com/documentation/appkit/nsstepper)

use crate::prelude::*;
use std::sync::{Arc, Mutex};

/// Delegate protocol for NSStepper events.
///
/// Apple: NSStepper uses target-action pattern
pub trait NSStepperDelegate: Send + Sync {
    /// Called when the stepper value changes.
    ///
    /// Apple: Sent via action method
    fn value_did_change(&mut self, value: f64) {
        let _ = value;
    }
    
    /// Called when the increment button is clicked.
    fn did_increment(&mut self, new_value: f64) {
        let _ = new_value;
    }
    
    /// Called when the decrement button is clicked.
    fn did_decrement(&mut self, new_value: f64) {
        let _ = new_value;
    }
}

/// Internal state for stepper
pub(crate) struct StepperState {
    pub(crate) value: f64,
    pub(crate) min_value: f64,
    pub(crate) max_value: f64,
    pub(crate) increment: f64,
    pub(crate) wraps: bool,
    pub(crate) autorepeat: bool,
}

impl Default for StepperState {
    fn default() -> Self {
        Self {
            value: 0.0,
            min_value: 0.0,
            max_value: 100.0,
            increment: 1.0,
            wraps: false,
            autorepeat: true,
        }
    }
}

/// Internal view delegate for NSStepper rendering and events.
pub(crate) struct NSStepperViewDelegate {
    pub(crate) state: Arc<Mutex<StepperState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSStepperDelegate>>>>,
}

impl NSViewDelegate for NSStepperViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let frame = view.bounds();
        
        // Split into two buttons (up/down or left/right)
        let button_height = frame.height() / 2.0;
        
        // Up/Increment button
        let up_rect = NSRect::new(frame.min_x(), frame.min_y() + button_height, frame.width(), button_height);
        context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_CONTROL));
        context.fill_rounded_rect(up_rect, 4.0);
        
        // Draw up arrow
        context.set_color(crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL));
        let arrow_x = frame.mid_x();
        let arrow_y = up_rect.mid_y();
        context.draw_text("▲", NSPoint::new(arrow_x - 4.0, arrow_y - 6.0), 12.0);
        
        // Down/Decrement button
        let down_rect = NSRect::new(frame.min_x(), frame.min_y(), frame.width(), button_height);
        context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_CONTROL));
        context.fill_rounded_rect(down_rect, 4.0);
        
        // Draw down arrow
        context.set_color(crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL));
        let arrow_x = frame.mid_x();
        let arrow_y = down_rect.mid_y();
        context.draw_text("▼", NSPoint::new(arrow_x - 4.0, arrow_y - 6.0), 12.0);
        
        // Separator
        context.set_color(crate::theme::color(crate::theme::keys::CLR_SEPARATOR));
        context.fill_rect(NSRect::new(frame.min_x(), frame.mid_y() - 0.5, frame.width(), 1.0));
    }
    
    fn mouse_down(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let frame = view.frame();
        let y = event.location_in_window().y;
        let mid_y = frame.mid_y();
        
        let mut state = self.state.lock().unwrap();
        let old_value = state.value;
        
        if y > mid_y {
            // Increment
            let new_value = if state.value + state.increment > state.max_value {
                if state.wraps {
                    state.min_value
                } else {
                    state.max_value
                }
            } else {
                state.value + state.increment
            };
            
            state.value = new_value;
            drop(state);
            
            // Notify delegate
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.did_increment(new_value);
                delegate.value_did_change(new_value);
            }
        } else {
            // Decrement
            let new_value = if state.value - state.increment < state.min_value {
                if state.wraps {
                    state.max_value
                } else {
                    state.min_value
                }
            } else {
                state.value - state.increment
            };
            
            state.value = new_value;
            drop(state);
            
            // Notify delegate
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.did_decrement(new_value);
                delegate.value_did_change(new_value);
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
