//! # NSSlider Delegate
//!
//! Delegate protocol for slider value changes.
//!
//! ## Apple Documentation
//! - [NSSlider](https://developer.apple.com/documentation/appkit/nsslider)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)

use crate::prelude::*;
use std::sync::{Arc, Mutex};

/// Delegate protocol for NSSlider events.
///
/// Apple: NSSlider uses target-action pattern, but we provide delegate for consistency
pub trait NSSliderDelegate: Send + Sync {
    /// Called when the slider value changes.
    ///
    /// Apple: Sent via action method
    fn value_did_change(&mut self, value: f64) {
        let _ = value;
    }
    
    /// Called when user starts dragging the slider.
    fn did_start_dragging(&mut self, value: f64) {
        let _ = value;
    }
    
    /// Called when user stops dragging the slider.
    fn did_stop_dragging(&mut self, value: f64) {
        let _ = value;
    }
    
    /// Called continuously while dragging (if continuous).
    fn value_is_changing(&mut self, value: f64) {
        let _ = value;
    }
}

/// Internal state for slider
pub(crate) struct SliderState {
    pub(crate) value: f64,
    pub(crate) min_value: f64,
    pub(crate) max_value: f64,
    pub(crate) is_dragging: bool,
    pub(crate) is_continuous: bool,
    pub(crate) is_vertical: bool,
}

impl Default for SliderState {
    fn default() -> Self {
        Self {
            value: 50.0,
            min_value: 0.0,
            max_value: 100.0,
            is_dragging: false,
            is_continuous: true,
            is_vertical: false,
        }
    }
}

/// Internal view delegate for NSSlider rendering and events.
pub(crate) struct NSSliderViewDelegate {
    pub(crate) state: Arc<Mutex<SliderState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSSliderDelegate>>>>,
}

impl NSSliderViewDelegate {
    fn value_from_position(&self, state: &SliderState, position: f32, frame: NSRect) -> f64 {
        let (track_start, track_length) = if state.is_vertical {
            (frame.min_y(), frame.height())
        } else {
            (frame.min_x(), frame.width())
        };
        
        let normalized = ((position - track_start) / track_length).clamp(0.0, 1.0) as f64;
        state.min_value + normalized * (state.max_value - state.min_value)
    }
}

impl NSViewDelegate for NSSliderViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let frame = view.bounds();
        
        // Calculate knob position
        let normalized = ((state.value - state.min_value) / (state.max_value - state.min_value))
            .clamp(0.0, 1.0) as f32;
        
        let knob_size = 16.0;
        let track_height = 4.0;
        
        if state.is_vertical {
            // Vertical slider
            let track_x = frame.mid_x() - track_height / 2.0;
            let track_rect = NSRect::new(track_x, frame.min_y(), track_height, frame.height());
            
            // Draw track
            context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_CONTROL));
            context.fill_rounded_rect(track_rect, track_height / 2.0);
            
            // Draw filled portion
            let fill_height = frame.height() * normalized;
            let fill_rect = NSRect::new(track_x, frame.min_y(), track_height, fill_height);
            context.set_color(crate::theme::color(crate::theme::keys::CLR_ACCENT));
            context.fill_rounded_rect(fill_rect, track_height / 2.0);
            
            // Draw knob
            let knob_y = frame.min_y() + fill_height - knob_size / 2.0;
            let knob_rect = NSRect::new(frame.mid_x() - knob_size / 2.0, knob_y, knob_size, knob_size);
            context.set_color(NSColor::white());
            context.fill_rounded_rect(knob_rect, knob_size / 2.0);
        } else {
            // Horizontal slider
            let track_y = frame.mid_y() - track_height / 2.0;
            let track_rect = NSRect::new(frame.min_x(), track_y, frame.width(), track_height);
            
            // Draw track
            context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_CONTROL));
            context.fill_rounded_rect(track_rect, track_height / 2.0);
            
            // Draw filled portion
            let fill_width = frame.width() * normalized;
            let fill_rect = NSRect::new(frame.min_x(), track_y, fill_width, track_height);
            context.set_color(crate::theme::color(crate::theme::keys::CLR_ACCENT));
            context.fill_rounded_rect(fill_rect, track_height / 2.0);
            
            // Draw knob
            let knob_x = frame.min_x() + fill_width - knob_size / 2.0;
            let knob_rect = NSRect::new(knob_x, frame.mid_y() - knob_size / 2.0, knob_size, knob_size);
            context.set_color(NSColor::white());
            context.fill_rounded_rect(knob_rect, knob_size / 2.0);
        }
    }
    
    fn mouse_down(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        state.is_dragging = true;
        
        // Calculate new value from click position
        let position = if state.is_vertical {
            event.location_in_window().y
        } else {
            event.location_in_window().x
        };
        
        let new_value = self.value_from_position(&state, position, view.frame());
        state.value = new_value;
        drop(state);
        
        // Notify delegate
        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.did_start_dragging(new_value);
            if self.state.lock().unwrap().is_continuous {
                delegate.value_is_changing(new_value);
            }
        }
    }
    
    fn mouse_dragged(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        
        if !state.is_dragging {
            return;
        }
        
        // Calculate new value from drag position
        let position = if state.is_vertical {
            event.location_in_window().y
        } else {
            event.location_in_window().x
        };
        
        let new_value = self.value_from_position(&state, position, view.frame());
        state.value = new_value;
        let is_continuous = state.is_continuous;
        drop(state);
        
        // Notify delegate
        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            if is_continuous {
                delegate.value_is_changing(new_value);
            }
        }
    }
    
    fn mouse_up(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        state.is_dragging = false;
        let value = state.value;
        drop(state);
        
        // Notify delegate
        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.did_stop_dragging(value);
            delegate.value_did_change(value);
        }
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
