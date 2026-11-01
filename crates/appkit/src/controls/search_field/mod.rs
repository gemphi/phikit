//! # NSSearchField — Search Text Field
//!
//! A text field optimized for performing text-based searches.
//!
//! ## Apple Documentation
//! - [NSSearchField](https://developer.apple.com/documentation/appkit/nssearchfield)
//! - [NSSearchFieldDelegate](https://developer.apple.com/documentation/appkit/nssearchfielddelegate)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut search = NSSearchField::new(frame);
//! search.set_placeholder_string("Search...");
//! search.set_delegate(Box::new(MySearchDelegate));
//! ```
//!
//! ## Search Flow
//!
//! 1. User types → `text_did_change()` called (live search)
//! 2. User presses Enter → `search_did_start()` called (full search)
//! 3. User presses Escape or clears → `search_did_end()` called
//!
//! ## See Also
//! - [NSSearchField](https://developer.apple.com/documentation/appkit/nssearchfield)
//! - [NSTextField](https://developer.apple.com/documentation/appkit/nstextfield)

pub mod delegate;

pub use delegate::NSSearchFieldDelegate;

use crate::control::NSControl;
use crate::graphics::NSRect;
use crate::view::NSView;
use delegate::{NSSearchFieldViewDelegate, SearchFieldState};
use std::sync::{Arc, Mutex};

/// Search field control.
///
/// Apple: [NSSearchField](https://developer.apple.com/documentation/appkit/nssearchfield)
#[derive(Clone)]
pub struct NSSearchField {
    control: NSControl,
    state: Arc<Mutex<SearchFieldState>>,
    delegate: Arc<Mutex<Option<Box<dyn NSSearchFieldDelegate>>>>,
}

impl NSSearchField {
    /// Create a new search field.
    ///
    /// Apple: `[[NSSearchField alloc] initWithFrame:]`
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(SearchFieldState::default()));
        let delegate = Arc::new(Mutex::new(None));
        
        let mut control = NSControl::new(frame);
        control.view_mut().set_delegate(NSSearchFieldViewDelegate {
            state: state.clone(),
            delegate: delegate.clone(),
        });
        
        Self { control, state, delegate }
    }

    /// Set the placeholder text.
    ///
    /// Apple: `setPlaceholderString:`
    pub fn set_placeholder_string(&mut self, placeholder: &str) {
        self.state.lock().unwrap().placeholder = placeholder.to_string();
    }

    /// Set the delegate for search events.
    ///
    /// Apple: `setDelegate:`
    pub fn set_delegate(&mut self, delegate: Box<dyn NSSearchFieldDelegate>) {
        *self.delegate.lock().unwrap() = Some(delegate);
    }

    /// Get the current search text.
    ///
    /// Apple: `stringValue`
    pub fn text(&self) -> String {
        self.state.lock().unwrap().text.clone()
    }

    /// Set the search text programmatically.
    ///
    /// Apple: `setStringValue:`
    pub fn set_text(&mut self, text: &str) {
        self.state.lock().unwrap().text = text.to_string();
    }

    /// Clear the search field.
    pub fn clear(&mut self) {
        self.state.lock().unwrap().text.clear();
        
        // Notify delegate
        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.search_did_end();
        }
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
