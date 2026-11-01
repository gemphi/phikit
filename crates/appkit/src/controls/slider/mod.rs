//! # NSSlider — Slider Control
//!
//! Slider control for selecting values from a continuous or discrete range.
//!
//! ## Apple Documentation
//! - [NSSlider](https://developer.apple.com/documentation/appkit/nsslider)
//! - [NSSliderCell](https://developer.apple.com/documentation/appkit/nsslidercell)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut slider = NSSlider::new(frame);
//! slider.set_min_value(0.0);
//! slider.set_max_value(100.0);
//! slider.set_value(50.0);
//! slider.set_delegate(Box::new(MySliderDelegate));
//! ```
//!
//! ## See Also
//! - [NSSlider](https://developer.apple.com/documentation/appkit/nsslider)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)

pub mod delegate;

pub use delegate::NSSliderDelegate;

use crate::control::NSControl;
use crate::graphics::NSRect;
use crate::view::NSView;
use delegate::{NSSliderViewDelegate, SliderState};
use std::sync::{Arc, Mutex};

/// Slider control.
///
/// Apple: [NSSlider](https://developer.apple.com/documentation/appkit/nsslider)
#[derive(Clone)]
pub struct NSSlider {
    control: NSControl,
    state: Arc<Mutex<SliderState>>,
    delegate: Arc<Mutex<Option<Box<dyn NSSliderDelegate>>>>,
}

impl NSSlider {
    /// Create a new slider.
    ///
    /// Apple: `[[NSSlider alloc] initWithFrame:]`
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(SliderState::default()));
        let delegate = Arc::new(Mutex::new(None));
        
        let mut control = NSControl::new(frame);
        control.view_mut().set_delegate(NSSliderViewDelegate {
            state: state.clone(),
            delegate: delegate.clone(),
        });
        
        Self { control, state, delegate }
    }

    /// Create horizontal slider with range.
    pub fn horizontal(min: f64, max: f64, value: f64) -> Self {
        let mut slider = Self::new(NSRect::sized(200.0, 24.0));
        slider.set_min_value(min);
        slider.set_max_value(max);
        slider.set_value(value);
        slider
    }

    /// Create vertical slider with range.
    pub fn vertical(min: f64, max: f64, value: f64) -> Self {
        let mut slider = Self::new(NSRect::sized(24.0, 200.0));
        slider.set_min_value(min);
        slider.set_max_value(max);
        slider.set_value(value);
        slider.set_vertical(true);
        slider
    }

    // ── Setters (Apple API) ──────────────────────────────────────────

    /// Set the current value.
    ///
    /// Apple: `setDoubleValue:`
    pub fn set_value(&mut self, value: f64) {
        self.state.lock().unwrap().value = value;
    }

    /// Get the current value.
    ///
    /// Apple: `doubleValue`
    pub fn value(&self) -> f64 {
        self.state.lock().unwrap().value
    }

    /// Set the minimum value.
    ///
    /// Apple: `setMinValue:`
    pub fn set_min_value(&mut self, min: f64) {
        self.state.lock().unwrap().min_value = min;
    }

    /// Set the maximum value.
    ///
    /// Apple: `setMaxValue:`
    pub fn set_max_value(&mut self, max: f64) {
        self.state.lock().unwrap().max_value = max;
    }

    /// Set whether the slider is vertical.
    ///
    /// Apple: `setVertical:`
    pub fn set_vertical(&mut self, vertical: bool) {
        self.state.lock().unwrap().is_vertical = vertical;
    }

    /// Set whether the slider is continuous.
    ///
    /// Apple: `setContinuous:`
    pub fn set_continuous(&mut self, continuous: bool) {
        self.state.lock().unwrap().is_continuous = continuous;
    }

    /// Set the delegate for slider events.
    ///
    /// Apple: `setTarget:action:` (we use delegate pattern)
    pub fn set_delegate(&mut self, delegate: Box<dyn NSSliderDelegate>) {
        *self.delegate.lock().unwrap() = Some(delegate);
    }

    // ── View Access ──────────────────────────────────────────────────

    pub fn view(&self) -> NSView {
        self.control.view().clone()
    }

    pub fn view_mut(&mut self) -> &mut NSView {
        self.control.view_mut()
    }

    pub fn view_clone(&self) -> NSView {
        self.control.view().clone()
    }
}
