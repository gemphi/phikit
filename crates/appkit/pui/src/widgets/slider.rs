//! PSlider — continuous value slider.
//!
//! Swift: `Slider(value: $value, in: range)`.

use crate::traits::{PStyle, PWidget};
use rust_appkit::prelude::*;
use std::sync::Arc;

/// Continuous value slider. Swift: `Slider(value:in:)`.
pub struct PSlider {
    value: f64,
    min: f64,
    max: f64,
    width: f32,
    height: f32,
    on_change: Option<Arc<dyn Fn(f64) + Send + Sync>>,
}

impl PSlider {
    /// Creates a slider with the given initial value and range. Swift: `Slider(value:in:)`
    pub fn new(value: f64, min: f64, max: f64) -> Self {
        Self {
            value, min, max,
            width: 200.0,
            height: 24.0,
            on_change: None,
        }
    }
    
    /// Normalised slider [0.0, 1.0]. Swift: `Slider(value: $value, in: 0...1)`
    pub fn normalized(value: f64) -> Self {
        Self::new(value.clamp(0.0, 1.0), 0.0, 1.0)
    }
    
    /// Change callback. Swift: binding write-back
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(f64) + Send + Sync + 'static,
    {
        self.on_change = Some(Arc::new(callback));
        self
    }
}

impl PStyle for PSlider {
    fn color(self, _color: NSColor) -> Self { self }
    fn background(self, _color: NSColor) -> Self { self }
    fn corner_radius(self, _radius: f32) -> Self { self }
    fn padding(self, _padding: f32) -> Self { self }
    
    fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    
    fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl PWidget for PSlider {
    fn build(self) -> NSView {
        let mut slider = NSSlider::new(NSRect::sized(self.width, self.height));
        slider.set_min_value(self.min);
        slider.set_max_value(self.max);
        slider.set_double_value(self.value);
        if let Some(cb) = self.on_change {
            slider.set_action(move |v| cb(v));
        }
        slider.view().clone()
    }
}
