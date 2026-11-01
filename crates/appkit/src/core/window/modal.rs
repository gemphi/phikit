//! Modal presentation — blocks interaction with the parent window.

use crate::view::NSView;

/// A modal window presentation.
pub struct ModalPresentation {
    pub title: String,
    pub content: NSView,
    pub width: f32,
    pub height: f32,
    pub is_closable: bool,
}

impl ModalPresentation {
    /// Create a new modal presentation.
    pub fn new(title: &str, content: NSView, width: f32, height: f32) -> Self {
        Self {
            title: title.to_string(),
            content,
            width,
            height,
            is_closable: true,
        }
    }

    /// Present the modal.
    pub fn present(&self) {
        println!(
            "[stub] ModalPresentation::present() — title: \"{}\", size: {}x{}",
            self.title, self.width, self.height
        );
    }

    /// Dismiss the modal.
    pub fn dismiss(&self) {
        println!("[stub] ModalPresentation::dismiss() — \"{}\"", self.title);
    }

    /// Set whether the modal can be closed by the user.
    pub fn set_closable(&mut self, closable: bool) {
        self.is_closable = closable;
    }
}
