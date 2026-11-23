//! PLabel — text label widget. Swift: `Text(_:)`.

use crate::traits::{PStyle, PWidget};
use rust_appkit::prelude::*;

/// A text label. Swift: `Text(_:)`.
pub struct PLabel {
    text: String,
    font_size: f32,
    color: NSColor,
    background: Option<NSColor>,
    width: Option<f32>,
    height: Option<f32>,
    bold: bool,
    alignment: NSTextAlignment,
}

impl PLabel {
    /// Creates a label. Swift: `Text(_:)`
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_size: 13.0,
            color: NSColor::new(0.9, 0.9, 0.9, 1.0),
            background: None,
            width: None,
            height: None,
            bold: false,
            alignment: NSTextAlignment::Left,
        }
    }
    
    /// Title preset (24pt bold). Swift: `.font(.title)`
    pub fn title(text: impl Into<String>) -> Self {
        Self::new(text).font_size(24.0).bold(true)
    }
    
    /// Section header preset (13pt bold). Swift: `Text(name).bold()`
    pub fn section(text: impl Into<String>) -> Self {
        Self::new(text).font_size(13.0).bold(true)
    }
    
    /// Caption preset (11pt dim). Swift: `.font(.caption)`
    pub fn caption(text: impl Into<String>) -> Self {
        Self::new(text).font_size(11.0).color(NSColor::new(0.6, 0.6, 0.6, 1.0))
    }
    
    /// Sets the font size. Swift: `.font(.system(size:))`
    pub fn font_size(mut self, size: f32) -> Self { self.font_size = size; self }
    
    /// Toggles bold weight. Swift: `.fontWeight(.bold)`
    pub fn bold(mut self, bold: bool) -> Self { self.bold = bold; self }
    
    /// Sets text alignment. Swift: `.multilineTextAlignment(_:)`
    pub fn align(mut self, alignment: NSTextAlignment) -> Self { self.alignment = alignment; self }
}

impl PStyle for PLabel {
    fn color(mut self, color: NSColor) -> Self { self.color = color; self }
    fn background(mut self, color: NSColor) -> Self { self.background = Some(color); self }
    fn corner_radius(self, _radius: f32) -> Self { self }
    fn padding(self, _padding: f32) -> Self { self }
    fn width(mut self, width: f32) -> Self { self.width = Some(width); self }
    fn height(mut self, height: f32) -> Self { self.height = Some(height); self }
}

impl PWidget for PLabel {
    fn build(self) -> NSView {
        let _ = self.bold;
        let _ = self.alignment;
        let mut label = NSTextField::label(&self.text);
        label.set_font_size(self.font_size);
        label.set_text_color(self.color);
        label.set_bordered(false);
        label.view_mut().set_background_color(self.background);
        if let (Some(w), Some(h)) = (self.width, self.height) {
            label.view_mut().set_frame(NSRect::sized(w, h));
        } else if let Some(w) = self.width {
            label.view_mut().set_frame(NSRect::sized(w, self.font_size + 6.0));
        }
        label.view_clone()
    }
}
