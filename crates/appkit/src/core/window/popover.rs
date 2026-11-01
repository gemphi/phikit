//! Popover presentation — anchored to a source rect.

use crate::graphics::NSRect;
use crate::view::NSView;

/// Edge to which the popover arrow points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

/// A popover presentation anchored to a source rect.
pub struct PopoverPresentation {
    pub content: NSView,
    pub source_rect: NSRect,
    pub preferred_edge: Edge,
    pub width: f32,
    pub height: f32,
}

impl PopoverPresentation {
    /// Create a new popover presentation.
    pub fn new(content: NSView, source_rect: NSRect, edge: Edge, width: f32, height: f32) -> Self {
        Self {
            content,
            source_rect,
            preferred_edge: edge,
            width,
            height,
        }
    }

    /// Present the popover.
    pub fn present(&self) {
        println!(
            "[stub] PopoverPresentation::present() — edge: {:?}, size: {}x{}",
            self.preferred_edge, self.width, self.height
        );
    }

    /// Dismiss the popover.
    pub fn dismiss(&self) {
        println!("[stub] PopoverPresentation::dismiss()");
    }
}
