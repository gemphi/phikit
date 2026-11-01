//! PToggle — boolean switch.
//!
//! Swift: `Toggle("Label", isOn: $value)`.

use crate::traits::{PInteractive, PStyle, PWidget};
use rust_appkit::prelude::*;
use std::sync::Arc;

/// A boolean toggle. Swift: `Toggle(_:isOn:)`.
pub struct PToggle {
    label: String,
    is_on: bool,
    width: f32,
    height: f32,
    on_change: Option<Arc<dyn Fn(bool) + Send + Sync>>,
}

impl PToggle {
    /// Creates a toggle with label. Swift: `Toggle(_:isOn:)`
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            is_on: false,
            width: 200.0,
            height: 24.0,
            on_change: None,
        }
    }
    
    /// Sets the initial on/off state. Swift: `isOn:` binding initial value
    pub fn on(mut self, on: bool) -> Self {
        self.is_on = on;
        self
    }
    
    /// Toggle callback. Swift: binding write-back
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_change = Some(Arc::new(callback));
        self
    }
}

impl PStyle for PToggle {
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

impl PInteractive for PToggle {
    fn on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        let cb = Arc::new(callback);
        self.on_change = Some(Arc::new(move |_v| cb()));
        self
    }
    
    fn enabled(self, _enabled: bool) -> Self { self }
}

impl PWidget for PToggle {
    fn build(self) -> NSView {
        // Composite: checkbox + label in horizontal stack
        let mut btn = NSButton::new(NSRect::sized(self.height, self.height))
            .with_title(if self.is_on { "✓" } else { "" })
            .with_bezel_style(NSBezelStyle::Rounded);
        if let Some(cb) = self.on_change {
            let state = std::sync::Arc::new(std::sync::Mutex::new(self.is_on));
            btn = btn.with_action(move || {
                let mut guard = state.lock().unwrap();
                *guard = !*guard;
                cb(*guard);
            });
        }
        
        let mut label = NSTextField::label(&self.label);
        label.set_bordered(false);
        label.view_mut().set_background_color(None);
        label.view_mut().set_frame(NSRect::sized(
            self.width - self.height - 8.0,
            self.height,
        ));
        
        let mut row = NSStackView::horizontal();
        row.set_spacing(8.0);
        row.add_arranged_subview(btn.view_clone());
        row.add_arranged_subview(label.view_clone());
        row.with_frame(NSRect::sized(self.width, self.height)).view_clone()
    }
}
