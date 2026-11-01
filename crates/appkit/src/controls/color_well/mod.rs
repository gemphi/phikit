//! # NSColorWell — Color Picker Control
//!
//! A control that displays a color and allows the user to change it.
//!
//! ## Apple Documentation
//! - [NSColorWell](https://developer.apple.com/documentation/appkit/nscolorwell)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)
//! - [NSColorPanel](https://developer.apple.com/documentation/appkit/nscolorpanel)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut color_well = NSColorWell::new(frame);
//! color_well.set_color(NSColor::red());
//! color_well.set_bordered(true);
//! ```
//!
//! ## See Also
//! - [NSColorPanel](https://developer.apple.com/documentation/appkit/nscolorpanel)
//! - [NSColor](https://developer.apple.com/documentation/appkit/nscolor)

mod delegate;
pub use delegate::NSColorWellDelegate;

use crate::theme::{self, keys};
use crate::view::NSViewDelegate;
use crate::{NSColor, NSRect, NSView};
use delegate::{ColorWellState, NSColorWellViewDelegate};
use std::sync::{Arc, Mutex};

/// A color well control.
pub struct NSColorWell {
    view: NSView,
    state: Arc<Mutex<ColorWellState>>,
    delegate: Arc<Mutex<Option<Box<dyn NSColorWellDelegate>>>>,
}

impl NSColorWell {
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(ColorWellState::default()));
        let delegate = Arc::new(Mutex::new(None));
        
        let mut view = NSView::new(frame).with_name("NSColorWell");
        view.set_delegate(NSColorWellViewDelegate {
            state: state.clone(),
            delegate: delegate.clone(),
        });
        
        Self {
            view,
            state,
            delegate,
        }
    }

    pub fn color(&self) -> NSColor {
        self.state.lock().unwrap().color
    }
    pub fn set_color(&mut self, color: NSColor) {
        self.state.lock().unwrap().color = color;
        self.view.set_needs_display();
    }
    pub fn is_active(&self) -> bool {
        self.state.lock().unwrap().is_active
    }
    pub fn activate(&mut self) {
        self.state.lock().unwrap().is_active = true;
        self.view.set_needs_display();
    }
    pub fn deactivate(&mut self) {
        self.state.lock().unwrap().is_active = false;
        self.view.set_needs_display();
    }
    pub fn set_bordered(&mut self, bordered: bool) {
        self.state.lock().unwrap().is_bordered = bordered;
        self.view.set_needs_display();
    }

    /// Set the delegate for color well events.
    ///
    /// Apple: `setTarget:action:` (we use delegate pattern)
    pub fn set_delegate(&mut self, delegate: Box<dyn NSColorWellDelegate>) {
        *self.delegate.lock().unwrap() = Some(delegate);
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }
    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }
}
