//! PTextField — single-line text input. Swift: `TextField(_:text:)`.

use crate::traits::{PStyle, PWidget};
use rust_appkit::prelude::*;
use std::sync::Arc;

/// Single-line text input. Swift: `TextField(_:text:)`.
pub struct PTextField {
    placeholder: String,
    initial_value: String,
    width: f32,
    height: f32,
    on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

impl PTextField {
    /// Creates a text field. Swift: `TextField(_:text:)`
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            initial_value: String::new(),
            width: 200.0,
            height: 24.0,
            on_change: None,
        }
    }
    
    /// Initial text value. Swift: binding initial state
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.initial_value = value.into(); self
    }
    
    /// Change callback. Swift: binding write-back on edit
    pub fn on_change<F>(mut self, callback: F) -> Self
    where F: Fn(String) + Send + Sync + 'static {
        self.on_change = Some(Arc::new(callback));
        self
    }
}

impl PStyle for PTextField {
    fn color(self, _color: NSColor) -> Self { self }
    fn background(self, _color: NSColor) -> Self { self }
    fn corner_radius(self, _radius: f32) -> Self { self }
    fn padding(self, _padding: f32) -> Self { self }
    fn width(mut self, width: f32) -> Self { self.width = width; self }
    fn height(mut self, height: f32) -> Self { self.height = height; self }
}

impl PWidget for PTextField {
    fn build(self) -> NSView {
        let mut field = NSTextField::editable(&self.initial_value);
        field.set_placeholder(&self.placeholder);
        field.view_mut().set_frame(NSRect::sized(self.width, self.height));
        let _ = self.on_change;
        field.view_clone()
    }
}
