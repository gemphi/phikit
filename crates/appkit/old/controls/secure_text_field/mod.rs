//! # NSSecureTextField — Password Input Field
//!
//! A text field that obscures the entered text for password entry.
//!
//! ## Apple Documentation
//! - [NSSecureTextField](https://developer.apple.com/documentation/appkit/nssecuretextfield)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut password_field = NSSecureTextField::new(frame);
//! password_field.set_placeholder_string("Enter password");
//! password_field.set_delegate(Box::new(MyPasswordDelegate));
//! ```
//!
//! ## See Also
//! - [NSSecureTextField](https://developer.apple.com/documentation/appkit/nssecuretextfield)
//! - [NSTextField](https://developer.apple.com/documentation/appkit/nstextfield)

pub mod delegate;

pub use delegate::NSSecureTextFieldDelegate;

use crate::control::NSControl;
use crate::graphics::NSRect;
use crate::view::NSView;
use delegate::{NSSecureTextFieldViewDelegate, SecureTextFieldState};
use std::sync::{Arc, Mutex};

/// Secure text field control for password entry.
///
/// Apple: [NSSecureTextField](https://developer.apple.com/documentation/appkit/nssecuretextfield)
#[derive(Clone)]
pub struct NSSecureTextField {
    control: NSControl,
    state: Arc<Mutex<SecureTextFieldState>>,
    delegate: Arc<Mutex<Option<Box<dyn NSSecureTextFieldDelegate>>>>,
}

impl NSSecureTextField {
    /// Create a new secure text field.
    ///
    /// Apple: `[[NSSecureTextField alloc] initWithFrame:]`
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(SecureTextFieldState::default()));
        let delegate = Arc::new(Mutex::new(None));
        
        let mut control = NSControl::new(frame);
        control.view_mut().set_delegate(NSSecureTextFieldViewDelegate {
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

    /// Get the password text.
    ///
    /// Apple: `stringValue`
    pub fn string_value(&self) -> String {
        self.state.lock().unwrap().text.clone()
    }

    /// Set the password text programmatically.
    ///
    /// Apple: `setStringValue:`
    pub fn set_string_value(&mut self, text: &str) {
        self.state.lock().unwrap().text = text.to_string();
    }

    /// Clear the password field.
    pub fn clear(&mut self) {
        self.state.lock().unwrap().text.clear();
    }

    /// Set the delegate for password events.
    ///
    /// Apple: `setDelegate:`
    pub fn set_delegate(&mut self, delegate: Box<dyn NSSecureTextFieldDelegate>) {
        *self.delegate.lock().unwrap() = Some(delegate);
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
