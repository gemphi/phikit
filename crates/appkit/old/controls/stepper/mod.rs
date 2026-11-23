//! # NSStepper — Increment/Decrement Control
//!
//! A control with increment/decrement buttons for numeric values.
//!
//! ## Apple Documentation
//! - [NSStepper](https://developer.apple.com/documentation/appkit/nsstepper)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut stepper = NSStepper::new(frame);
//! stepper.set_min_value(0.0);
//! stepper.set_max_value(100.0);
//! stepper.set_increment(1.0);
//! stepper.set_value(50.0);
//! ```
//!
//! ## Delegate Pattern
//!
//! ```rust,ignore
//! struct MyDelegate;
//!
//! impl NSStepperDelegate for MyDelegate {
//!     fn value_did_change(&mut self, value: f64) {
//!         println!("Stepper value changed to: {}", value);
//!     }
//! }
//!
//! let mut stepper = NSStepper::new(frame);
//! stepper.set_delegate(Box::new(MyDelegate));
//! ```
//!
//! ## See Also
//! - [NSSlider](https://developer.apple.com/documentation/appkit/nsslider)
//! - [NSTextField](https://developer.apple.com/documentation/appkit/nstextfield)

pub mod delegate;

pub use delegate::NSStepperDelegate;

use crate::theme::{self, keys};
use crate::{NSColor, NSRect, NSView};
use delegate::{NSStepperViewDelegate, StepperState};
use std::sync::{Arc, Mutex};

/// A stepper control for numeric values.
pub struct NSStepper {
    view: NSView,
    state: Arc<Mutex<StepperState>>,
    delegate: Arc<Mutex<Option<Box<dyn NSStepperDelegate>>>>,
}

impl NSStepper {
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(StepperState::default()));
        let delegate = Arc::new(Mutex::new(None));
        
        let mut view = NSView::new(frame).with_name("NSStepper");
        view.set_delegate(NSStepperViewDelegate {
            state: state.clone(),
            delegate: delegate.clone(),
        });
        
        Self { view, state, delegate }
    }

    pub fn value(&self) -> f64 {
        self.state.lock().unwrap().value
    }
    
    pub fn set_value(&mut self, v: f64) {
        let mut state = self.state.lock().unwrap();
        state.value = v.clamp(state.min_value, state.max_value);
    }
    
    pub fn set_min_value(&mut self, v: f64) {
        self.state.lock().unwrap().min_value = v;
    }
    
    pub fn set_max_value(&mut self, v: f64) {
        self.state.lock().unwrap().max_value = v;
    }
    
    pub fn set_increment(&mut self, v: f64) {
        self.state.lock().unwrap().increment = v;
    }
    
    pub fn set_wraps(&mut self, w: bool) {
        self.state.lock().unwrap().wraps = w;
    }

    pub fn increment_value(&mut self) {
        let mut state = self.state.lock().unwrap();
        let new = state.value + state.increment;
        state.value = if new > state.max_value {
            if state.wraps {
                state.min_value
            } else {
                state.max_value
            }
        } else {
            new
        };
        
        let new_value = state.value;
        drop(state);
        
        // Notify delegate
        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.did_increment(new_value);
            delegate.value_did_change(new_value);
        }
    }

    pub fn decrement_value(&mut self) {
        let mut state = self.state.lock().unwrap();
        let new = state.value - state.increment;
        state.value = if new < state.min_value {
            if state.wraps {
                state.max_value
            } else {
                state.min_value
            }
        } else {
            new
        };
        
        let new_value = state.value;
        drop(state);
        
        // Notify delegate
        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.did_decrement(new_value);
            delegate.value_did_change(new_value);
        }
    }

    /// Set the delegate for stepper events.
    ///
    /// Apple: `setTarget:action:` (we use delegate pattern)
    pub fn set_delegate(&mut self, delegate: Box<dyn NSStepperDelegate>) {
        *self.delegate.lock().unwrap() = Some(delegate);
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }
    
    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }
}
