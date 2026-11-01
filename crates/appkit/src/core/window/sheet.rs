//! Sheet presentation — slides from top, attached to a parent window.

use crate::view::NSView;

/// A sheet presentation attached to a parent window.
pub struct SheetPresentation {
    pub parent_window_id: u64,
    pub content: NSView,
    pub width: f32,
    pub height: f32,
}

impl SheetPresentation {
    /// Create a new sheet presentation.
    pub fn new(parent_id: u64, content: NSView, width: f32, height: f32) -> Self {
        Self {
            parent_window_id: parent_id,
            content,
            width,
            height,
        }
    }

    /// Present the sheet.
    pub fn present(&self) {
        println!(
            "[stub] SheetPresentation::present() — parent: {}, size: {}x{}",
            self.parent_window_id, self.width, self.height
        );
    }

    /// Dismiss the sheet.
    pub fn dismiss(&self) {
        println!(
            "[stub] SheetPresentation::dismiss() — parent: {}",
            self.parent_window_id
        );
    }
}
