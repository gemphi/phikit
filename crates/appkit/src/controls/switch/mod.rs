//! # NSSwitch — Modern Toggle Switch
//!
//! A modern on/off toggle switch control (macOS 10.15+).
//!
//! ## Apple Documentation
//! - [NSSwitch](https://developer.apple.com/documentation/appkit/nsswitch)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut switch = NSSwitch::new(frame);
//! switch.set_on(true);
//! switch.set_delegate(Box::new(MySwitchDelegate));
//! ```
//!
//! ## Delegate Pattern
//!
//! ```rust,ignore
//! struct MySwitchDelegate;
//!
//! impl NSSwitchDelegate for MySwitchDelegate {
//!     fn state_did_change(&mut self, is_on: bool) {
//!         println!("Switch is now: {}", if is_on { "ON" } else { "OFF" });
//!     }
//! }
//! ```
//!
//! ## See Also
//! - [NSButton](https://developer.apple.com/documentation/appkit/nsbutton)
//! - [NSCheckBox](https://developer.apple.com/documentation/appkit/nsbutton)

mod delegate;
pub use delegate::NSSwitchDelegate;

use crate::{NSRect, NSView};
use delegate::{NSSwitchViewDelegate, SwitchState};
use std::sync::{Arc, Mutex};

/// **NSSwitch** — A modern on/off switch
#[derive(Clone)]
pub struct NSSwitch {
    view: NSView,
    state: Arc<Mutex<SwitchState>>,
    delegate: Arc<Mutex<Option<Box<dyn NSSwitchDelegate>>>>,
}

impl NSSwitch {
    /// Create a new switch.
    ///
    /// Apple: `[[NSSwitch alloc] initWithFrame:]`
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(SwitchState::default()));
        let delegate = Arc::new(Mutex::new(None));

        let mut view = NSView::new(frame);
        view.set_delegate(NSSwitchViewDelegate {
            state: state.clone(),
            delegate: delegate.clone(),
        });

        Self { view, state, delegate }
    }

    /// Set the switch state (on/off).
    ///
    /// Apple: `setState:`
    pub fn set_on(&mut self, on: bool) {
        self.state.lock().unwrap().is_on = on;
        self.view.set_needs_display();
    }

    /// Get the switch state.
    ///
    /// Apple: `state`
    pub fn is_on(&self) -> bool {
        self.state.lock().unwrap().is_on
    }

    /// Set whether the switch is enabled.
    ///
    /// Apple: `setEnabled:`
    pub fn set_enabled(&mut self, enabled: bool) {
        self.state.lock().unwrap().is_enabled = enabled;
        self.view.set_needs_display();
    }

    /// Get whether the switch is enabled.
    ///
    /// Apple: `isEnabled`
    pub fn is_enabled(&self) -> bool {
        self.state.lock().unwrap().is_enabled
    }

    /// Set the delegate for switch events.
    ///
    /// Apple: `setTarget:action:` (we use delegate pattern)
    pub fn set_delegate(&mut self, delegate: Box<dyn NSSwitchDelegate>) {
        *self.delegate.lock().unwrap() = Some(delegate);
    }

    // ── View Access ──────────────────────────────────────────────────

    pub fn view(&self) -> NSView {
        self.view.clone()
    }

    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }

    pub fn view_clone(&self) -> NSView {
        self.view.clone()
    }
}
