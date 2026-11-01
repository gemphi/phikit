//! # NSTextView — Multi-line Text Editor
//!
//! A view for displaying and editing multi-line text with rich formatting support.
//!
//! ## Apple Documentation
//! - [NSTextView](https://developer.apple.com/documentation/appkit/nstextview)
//! - [NSText](https://developer.apple.com/documentation/appkit/nstext)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut text_view = NSTextView::new(frame);
//! text_view.set_string("Multi-line\ntext content");
//! text_view.set_editable(true);
//! text_view.set_delegate(Box::new(MyTextDelegate));
//! ```
//!
//! ## See Also
//! - [NSTextView](https://developer.apple.com/documentation/appkit/nstextview)
//! - [NSTextViewDelegate](https://developer.apple.com/documentation/appkit/nstextviewdelegate)

pub mod delegate;

pub use delegate::NSTextViewDelegate;

use crate::control::NSControl;
use crate::graphics::NSRect;
use crate::view::NSView;
use delegate::{NSTextViewViewDelegate, TextViewState};
use std::sync::{Arc, Mutex};

/// Multi-line text view control.
///
/// Apple: [NSTextView](https://developer.apple.com/documentation/appkit/nstextview)
#[derive(Clone)]
pub struct NSTextView {
    control: NSControl,
    state: Arc<Mutex<TextViewState>>,
    delegate: Arc<Mutex<Option<Box<dyn NSTextViewDelegate>>>>,
}

impl NSTextView {
    /// Create a new text view.
    ///
    /// Apple: `[[NSTextView alloc] initWithFrame:]`
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(TextViewState::default()));
        let delegate = Arc::new(Mutex::new(None));
        
        let mut control = NSControl::new(frame);
        control.view_mut().set_delegate(NSTextViewViewDelegate {
            state: state.clone(),
            delegate: delegate.clone(),
        });
        
        Self { control, state, delegate }
    }

    /// Set the text content.
    ///
    /// Apple: `setString:`
    pub fn set_string(&mut self, text: &str) {
        self.state.lock().unwrap().text = text.to_string();
    }

    /// Get the text content.
    ///
    /// Apple: `string`
    pub fn string(&self) -> String {
        self.state.lock().unwrap().text.clone()
    }

    /// Set whether the text is editable.
    ///
    /// Apple: `setEditable:`
    pub fn set_editable(&mut self, editable: bool) {
        self.state.lock().unwrap().is_editable = editable;
    }

    /// Set whether the text is selectable.
    ///
    /// Apple: `setSelectable:`
    pub fn set_selectable(&mut self, selectable: bool) {
        self.state.lock().unwrap().is_selectable = selectable;
    }

    /// Set whether the text view has a border.
    pub fn set_bordered(&mut self, bordered: bool) {
        self.state.lock().unwrap().is_bordered = bordered;
    }

    /// Set the delegate for text events.
    ///
    /// Apple: `setDelegate:`
    pub fn set_delegate(&mut self, delegate: Box<dyn NSTextViewDelegate>) {
        *self.delegate.lock().unwrap() = Some(delegate);
    }

    /// Insert text at the current cursor position.
    ///
    /// Apple: `insertText:`
    pub fn insert_text(&mut self, text: &str) {
        let mut state = self.state.lock().unwrap();
        state.text.push_str(text);
    }

    /// Clear all text.
    pub fn clear(&mut self) {
        self.state.lock().unwrap().text.clear();
    }

    /// Set the text content (alias for set_string).
    pub fn set_text(&mut self, text: &str) {
        self.set_string(text);
    }

    /// Get the text content (alias for string).
    pub fn get_text(&self) -> String {
        self.string()
    }

    /// Append text to the end of the current content.
    pub fn append_text(&mut self, text: &str) {
        let mut state = self.state.lock().unwrap();
        state.text.push_str(text);
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
