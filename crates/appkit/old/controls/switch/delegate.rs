//! # NSSwitch Delegate
//!
//! Delegate protocol for switch state changes.
//!
//! ## Apple Documentation
//! - [NSSwitch](https://developer.apple.com/documentation/appkit/nsswitch)

use crate::prelude::*;
use std::sync::{Arc, Mutex};

/// Delegate protocol for NSSwitch events.
///
/// Apple: NSSwitch uses target-action pattern
pub trait NSSwitchDelegate: Send + Sync {
    /// Called when the switch state changes.
    ///
    /// Apple: Sent via action method
    fn state_did_change(&mut self, is_on: bool) {
        let _ = is_on;
    }
    
    /// Called to validate whether the state should change.
    fn should_change_state(&mut self, new_state: bool) -> bool {
        let _ = new_state;
        true
    }
}

/// Internal state for switch
pub(crate) struct SwitchState {
    pub(crate) is_on: bool,
    pub(crate) is_enabled: bool,
}

impl Default for SwitchState {
    fn default() -> Self {
        Self {
            is_on: false,
            is_enabled: true,
        }
    }
}

/// Internal view delegate for NSSwitch rendering and events.
pub(crate) struct NSSwitchViewDelegate {
    pub(crate) state: Arc<Mutex<SwitchState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSSwitchDelegate>>>>,
}

impl NSViewDelegate for NSSwitchViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let frame = view.bounds();
        
        // Switch track (pill shape)
        let track_width = 44.0;
        let track_height = 24.0;
        let track_x = frame.min_x();
        let track_y = frame.mid_y() - track_height / 2.0;
        let track_rect = NSRect::new(track_x, track_y, track_width, track_height);
        
        // Track background
        let track_color = if state.is_on {
            crate::theme::color(crate::theme::keys::CLR_ACCENT)
        } else {
            crate::theme::color(crate::theme::keys::CLR_BG_CONTROL)
        };
        
        context.set_color(track_color);
        context.fill_rounded_rect(track_rect, track_height / 2.0);
        
        // Knob (circle)
        let knob_size = 20.0;
        let knob_x = if state.is_on {
            track_x + track_width - knob_size - 2.0
        } else {
            track_x + 2.0
        };
        let knob_y = track_y + (track_height - knob_size) / 2.0;
        let knob_rect = NSRect::new(knob_x, knob_y, knob_size, knob_size);
        
        context.set_color(NSColor::white());
        context.fill_rounded_rect(knob_rect, knob_size / 2.0);
        
        // Disabled overlay
        if !state.is_enabled {
            context.set_color(NSColor::new(0.5, 0.5, 0.5, 0.3));
            context.fill_rounded_rect(track_rect, track_height / 2.0);
        }
    }
    
    fn mouse_down(&mut self, view: &mut NSView, _event: &crate::event::NSEvent) {
        let state = self.state.lock().unwrap();
        
        if !state.is_enabled {
            return;
        }
        
        let current_state = state.is_on;
        let new_state = !current_state;
        drop(state);
        
        // Check if should change
        let should_change = if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.should_change_state(new_state)
        } else {
            true
        };
        
        if !should_change {
            return;
        }
        
        // Toggle state
        self.state.lock().unwrap().is_on = new_state;
        
        // Notify delegate
        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.state_did_change(new_state);
        }
        
        // Request redraw
        view.set_needs_display();
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
