//! # NSProgressIndicator — Progress Display Control
//!
//! Displays determinate or indeterminate progress.
//!
//! ## Apple Documentation
//! - [NSProgressIndicator](https://developer.apple.com/documentation/appkit/nsprogressindicator)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Determinate progress bar
//! let mut progress = NSProgressIndicator::new(frame);
//! progress.set_min_value(0.0);
//! progress.set_max_value(100.0);
//! progress.set_value(50.0);
//!
//! // Indeterminate spinner
//! let mut spinner = NSProgressIndicator::new(frame);
//! spinner.set_style(NSProgressIndicatorStyle::Spinning);
//! spinner.set_indeterminate(true);
//! ```
//!
//! ## See Also
//! - [NSLevelIndicator](https://developer.apple.com/documentation/appkit/nslevelindicator)

use crate::theme::{self, keys};
use crate::view::NSViewDelegate;
use crate::{NSColor, NSRect, NSView};

/// Progress indicator style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSProgressIndicatorStyle {
    Bar,
    Spinning,
}

/// A progress indicator control.
pub struct NSProgressIndicator {
    view: NSView,
    min_value: f64,
    max_value: f64,
    value: f64,
    is_indeterminate: bool,
    style: NSProgressIndicatorStyle,
}

impl NSProgressIndicator {
    pub fn new(frame: NSRect) -> Self {
        let mut view = NSView::new(frame).with_name("NSProgressIndicator");
        let indicator = Self {
            view,
            min_value: 0.0,
            max_value: 100.0,
            value: 0.0,
            is_indeterminate: false,
            style: NSProgressIndicatorStyle::Bar,
        };
        indicator
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = value.clamp(self.min_value, self.max_value);
    }
    pub fn value(&self) -> f64 {
        self.value
    }
    pub fn set_min_value(&mut self, v: f64) {
        self.min_value = v;
    }
    pub fn set_max_value(&mut self, v: f64) {
        self.max_value = v;
    }
    pub fn set_indeterminate(&mut self, v: bool) {
        self.is_indeterminate = v;
    }
    pub fn set_style(&mut self, style: NSProgressIndicatorStyle) {
        self.style = style;
    }
    pub fn increment_by(&mut self, delta: f64) {
        self.set_value(self.value + delta);
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }
    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }
}
