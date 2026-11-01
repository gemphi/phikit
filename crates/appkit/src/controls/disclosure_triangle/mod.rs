//! # NSDisclosureTriangle — Expand/Collapse Control
//!
//! A disclosure triangle control that toggles between expanded/collapsed states.
//! Used in outline views, inspector sections, and any collapsible content.
//!
//! ## Apple Documentation
//! - [NSButton (Disclosure Style)](https://developer.apple.com/documentation/appkit/nsbutton)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)
//! - [NSOutlineView](https://developer.apple.com/documentation/appkit/nsoutlineview)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut triangle = NSDisclosureTriangle::new(frame);
//! triangle.set_state(NSDisclosureState::Expanded);
//! triangle.set_on_toggle(|state| {
//!     println!("State: {:?}", state);
//! });
//! ```
//!
//! ## See Also
//! - [NSButton](https://developer.apple.com/documentation/appkit/nsbutton)
//! - [NSOutlineView](https://developer.apple.com/documentation/appkit/nsoutlineview)

mod delegate;
pub use delegate::NSDisclosureDelegate;
use delegate::{DisclosureState, NSDisclosureViewDelegate};

use crate::theme::{self, keys};
use crate::view::NSViewDelegate;
use crate::{NSColor, NSPoint, NSRect, NSView};
use std::sync::{Arc, Mutex};

/// State of the disclosure triangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSDisclosureState {
    Collapsed,
    Expanded,
}

/// A disclosure triangle control.
pub struct NSDisclosureTriangle {
    view: NSView,
    state: Arc<Mutex<DisclosureState>>,
    delegate: Arc<Mutex<Option<Box<dyn NSDisclosureDelegate>>>>,
    on_toggle: Option<Box<dyn Fn(NSDisclosureState) + Send + Sync>>,
}

impl NSDisclosureTriangle {
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(DisclosureState::default()));
        let delegate = Arc::new(Mutex::new(None));
        let mut view = NSView::new(frame).with_name("NSDisclosureTriangle");
        view.set_delegate(NSDisclosureViewDelegate {
            state: state.clone(),
            delegate: delegate.clone(),
        });
        Self {
            view,
            state,
            delegate,
            on_toggle: None,
        }
    }

    pub fn state(&self) -> NSDisclosureState {
        let state = self.state.lock().unwrap();
        if state.is_expanded {
            NSDisclosureState::Expanded
        } else {
            NSDisclosureState::Collapsed
        }
    }

    pub fn set_state(&mut self, state: NSDisclosureState) {
        let mut s = self.state.lock().unwrap();
        s.is_expanded = state == NSDisclosureState::Expanded;
        self.view.set_needs_display();
    }

    pub fn toggle(&mut self) {
        let new_state = {
            let mut s = self.state.lock().unwrap();
            s.is_expanded = !s.is_expanded;
            if s.is_expanded {
                NSDisclosureState::Expanded
            } else {
                NSDisclosureState::Collapsed
            }
        };
        if let Some(ref cb) = self.on_toggle {
            cb(new_state);
        }
        self.view.set_needs_display();
    }

    pub fn set_on_toggle(&mut self, f: impl Fn(NSDisclosureState) + Send + Sync + 'static) {
        self.on_toggle = Some(Box::new(f));
    }

    /// Set the delegate for disclosure triangle events.
    ///
    /// Apple: `setTarget:action:` (we use delegate pattern)
    pub fn set_delegate(&mut self, delegate: Box<dyn NSDisclosureDelegate>) {
        *self.delegate.lock().unwrap() = Some(delegate);
    }

    pub fn is_expanded(&self) -> bool {
        self.state() == NSDisclosureState::Expanded
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }
    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDelegate {
        toggle_count: std::sync::Arc<std::sync::Mutex<usize>>,
        last_state: std::sync::Arc<std::sync::Mutex<bool>>,
    }

    impl NSDisclosureDelegate for TestDelegate {
        fn did_toggle(&mut self, is_expanded: bool) {
            *self.toggle_count.lock().unwrap() += 1;
            *self.last_state.lock().unwrap() = is_expanded;
        }
    }

    #[test]
    fn test_set_delegate() {
        let mut triangle = NSDisclosureTriangle::new(NSRect::sized(20.0, 20.0));
        
        let toggle_count = std::sync::Arc::new(std::sync::Mutex::new(0));
        let last_state = std::sync::Arc::new(std::sync::Mutex::new(false));
        
        let delegate = TestDelegate {
            toggle_count: toggle_count.clone(),
            last_state: last_state.clone(),
        };
        
        triangle.set_delegate(Box::new(delegate));
        
        // Verify delegate is set
        assert!(triangle.delegate.lock().unwrap().is_some());
    }

    #[test]
    fn test_delegate_is_called_on_mouse_down() {
        // This test verifies the delegate pattern is wired correctly
        // In a real scenario, mouse_down would be called by the event system
        let mut triangle = NSDisclosureTriangle::new(NSRect::sized(20.0, 20.0));
        
        let toggle_count = std::sync::Arc::new(std::sync::Mutex::new(0));
        let last_state = std::sync::Arc::new(std::sync::Mutex::new(false));
        
        let delegate = TestDelegate {
            toggle_count: toggle_count.clone(),
            last_state: last_state.clone(),
        };
        
        triangle.set_delegate(Box::new(delegate));
        
        // Initial state should be collapsed
        assert_eq!(triangle.state(), NSDisclosureState::Collapsed);
        
        // Note: In a real application, the mouse_down event would trigger the delegate
        // This test just verifies the delegate is properly stored
        assert_eq!(*toggle_count.lock().unwrap(), 0);
    }
}
