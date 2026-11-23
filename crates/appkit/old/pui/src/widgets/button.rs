//! PButton — pressable button. Swift: `Button(_:action:)`.

use crate::traits::{PInteractive, PStyle, PWidget};
use rust_appkit::prelude::*;
use std::sync::Arc;

/// A pressable button. Swift: `Button(_:action:)`.
pub struct PButton {
    title: String,
    width: f32,
    height: f32,
    bezel: NSBezelStyle,
    background: Option<NSColor>,
    corner_radius: Option<f32>,
    border_color: Option<NSColor>,
    border_width: Option<f32>,
    enabled: bool,
    action: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl PButton {
    /// Creates a button. Swift: `Button(_:action:)`
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: 120.0,
            height: 32.0,
            bezel: NSBezelStyle::Rounded,
            background: None,
            corner_radius: None,
            border_color: None,
            border_width: None,
            enabled: true,
            action: None,
        }
    }
    
    /// Primary action button. Swift: `.buttonStyle(.borderedProminent)`
    pub fn primary(title: impl Into<String>) -> Self { Self::new(title) }
    
    /// Destructive button (red tint). Swift: `.tint(.red)`
    pub fn destructive(title: impl Into<String>) -> Self {
        Self::new(title).background(NSColor::new(0.85, 0.2, 0.2, 1.0))
    }
    
    /// Bezel style. Swift: `.buttonStyle(_:)`
    pub fn bezel(mut self, bezel: NSBezelStyle) -> Self { self.bezel = bezel; self }
    
    /// Adds a selection ring. Swift: `.overlay(Circle().stroke())`
    pub fn selected_ring(mut self, color: NSColor, width: f32) -> Self {
        self.border_color = Some(color);
        self.border_width = Some(width);
        self
    }
}

impl PStyle for PButton {
    fn color(self, _color: NSColor) -> Self { self }
    fn background(mut self, color: NSColor) -> Self { self.background = Some(color); self }
    fn corner_radius(mut self, radius: f32) -> Self { self.corner_radius = Some(radius); self }
    fn padding(self, _padding: f32) -> Self { self }
    fn width(mut self, width: f32) -> Self { self.width = width; self }
    fn height(mut self, height: f32) -> Self { self.height = height; self }
}

impl PInteractive for PButton {
    fn on_click<F>(mut self, callback: F) -> Self
    where F: Fn() + Send + Sync + 'static {
        self.action = Some(Arc::new(callback));
        self
    }
    fn enabled(mut self, enabled: bool) -> Self { self.enabled = enabled; self }
}

impl PWidget for PButton {
    fn build(self) -> NSView {
        let action = self.action;
        let mut btn = NSButton::new(NSRect::sized(self.width, self.height))
            .with_title(&self.title)
            .with_bezel_style(self.bezel);
        if let Some(bg) = self.background {
            btn = btn.with_background_color(Some(bg));
        }
        if let Some(radius) = self.corner_radius {
            btn = btn.with_corner_radius(radius);
        }
        if let (Some(c), Some(w)) = (self.border_color, self.border_width) {
            btn = btn.with_border_color(Some(c)).with_border_width(w);
        }
        if let Some(action) = action {
            btn = btn.with_action(move || action());
        }
        let _ = self.enabled;
        btn.view_clone()
    }
}
