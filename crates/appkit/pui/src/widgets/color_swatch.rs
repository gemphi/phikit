//! PColorSwatch — circular color picker swatch.
//!
//! Swift: `Circle().fill(color)` button used in BOTanist's color selector.

use crate::traits::{PInteractive, PStyle, PWidget};
use rust_appkit::prelude::*;
use std::sync::Arc;

/// A circular color swatch with optional selection ring.
/// Swift: `Circle().fill(color)` inside a `Button`.
pub struct PColorSwatch {
    color: NSColor,
    diameter: f32,
    selected: bool,
    ring_color: NSColor,
    ring_width: f32,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl PColorSwatch {
    /// Creates a swatch with the given color. Swift: `Circle().fill(color)`
    pub fn new(color: NSColor) -> Self {
        Self {
            color,
            diameter: 40.0,
            selected: false,
            ring_color: NSColor::new(1.0, 1.0, 1.0, 1.0),
            ring_width: 3.0,
            on_click: None,
        }
    }
    
    /// Sets selected state (draws ring around the swatch). Swift: `.background(Circle().stroke(...))`
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    
    /// Customises the selection ring color. Swift: `.stroke(_:lineWidth:)`
    pub fn ring(mut self, color: NSColor, width: f32) -> Self {
        self.ring_color = color;
        self.ring_width = width;
        self
    }
    
    /// Diameter (the swatch is square; circle drawn via corner radius). Swift: `.frame(width:height:)`
    pub fn diameter(mut self, diameter: f32) -> Self {
        self.diameter = diameter;
        self
    }
}

impl PStyle for PColorSwatch {
    fn color(mut self, color: NSColor) -> Self {
        self.color = color;
        self
    }
    
    fn background(self, _color: NSColor) -> Self { self }
    fn corner_radius(self, _radius: f32) -> Self { self }
    fn padding(self, _padding: f32) -> Self { self }
    
    fn width(mut self, width: f32) -> Self {
        self.diameter = width;
        self
    }
    
    fn height(mut self, height: f32) -> Self {
        self.diameter = height;
        self
    }
}

impl PInteractive for PColorSwatch {
    fn on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Arc::new(callback));
        self
    }
    
    fn enabled(self, _enabled: bool) -> Self { self }
}

impl PWidget for PColorSwatch {
    fn build(self) -> NSView {
        let mut btn = NSButton::new(NSRect::sized(self.diameter, self.diameter))
            .with_title("")
            .with_bezel_style(NSBezelStyle::Rounded)
            .with_background_color(Some(self.color))
            .with_corner_radius(self.diameter / 2.0);
        
        if self.selected {
            btn = btn
                .with_border_color(Some(self.ring_color))
                .with_border_width(self.ring_width);
        }
        
        if let Some(cb) = self.on_click {
            btn = btn.with_action(move || cb());
        }
        
        btn.view_clone()
    }
}
