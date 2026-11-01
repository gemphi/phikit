//! # NSControl — Base class for all interactive UI controls.
//!
//! Mirrors Apple's NSControl. Provides:
//! - `NSAction` — type-safe action closure (replaces raw `Arc<dyn Fn() + Send + Sync>`)
//! - `NSControlTarget` — target-action pattern for tagged dispatch
//! - `NSControl` — base struct with view, action, enabled state
//!
//! ## Apple Documentation
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)
//! - [NSView](https://developer.apple.com/documentation/appkit/nsview)
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Create a control with an action
//! let mut control = NSControl::new(frame);
//! control.set_action(|| println!("Control activated"));
//! control.set_enabled(true);
//! control.perform_action();
//!
//! // Using target-action pattern
//! struct MyController;
//! impl NSControlTarget for MyController {
//!     fn action_performed(&self, tag: i32) {
//!         println!("Control {} activated", tag);
//!     }
//! }
//!
//! let mut control = NSControl::new(frame);
//! control.set_tag(42);
//! control.set_target(Arc::new(MyController));
//! control.perform_action();
//! ```
//!
//! ## See Also
//! - [NSButton](https://developer.apple.com/documentation/appkit/nsbutton)
//! - [NSSlider](https://developer.apple.com/documentation/appkit/nsslider)

use crate::NSView;
use std::sync::Arc;

// ── Type Aliases ─────────────────────────────────────────────────────────────

/// A thread-safe action closure. Use this instead of `Arc<dyn Fn() + Send + Sync>`.
///
/// ```rust,ignore
/// button.set_action(|| println!("clicked"));
/// // Stored internally as NSAction
/// ```
pub type NSAction = Arc<dyn Fn() + Send + Sync>;

/// A thread-safe action closure that receives a value parameter.
pub type NSActionWith<T> = Arc<dyn Fn(T) + Send + Sync>;

/// A thread-safe predicate (returns bool).
pub type NSPredicate = Arc<dyn Fn() -> bool + Send + Sync>;

// ── Target-Action Pattern ────────────────────────────────────────────────────

/// Target-action pattern for controls that dispatch by tag.
/// Implement this on your controller to receive actions from multiple controls.
pub trait NSControlTarget: Send + Sync {
    fn action_performed(&self, tag: i32);
}

// ── NSControl ────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct NSControl {
    pub(crate) view: NSView,
    pub(crate) action: Option<NSAction>,
    pub(crate) target: Option<Arc<dyn NSControlTarget>>,
    pub(crate) is_enabled: bool,
    pub(crate) tag: i32,
}

impl NSControl {
    pub fn new(frame: crate::graphics::NSRect) -> Self {
        Self {
            view: NSView::new(frame),
            action: None,
            target: None,
            is_enabled: true,
            tag: 0,
        }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }
    
    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }

    pub fn set_action(&mut self, action: impl Fn() + Send + Sync + 'static) {
        self.action = Some(Arc::new(action));
    }

    pub fn set_target(&mut self, target: Arc<dyn NSControlTarget>) {
        self.target = Some(target);
    }

    pub fn set_tag(&mut self, tag: i32) {
        self.tag = tag;
    }

    pub fn tag(&self) -> i32 {
        self.tag
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn perform_action(&self) {
        if !self.is_enabled {
            return;
        }
        if let Some(ref action) = self.action {
            action();
        }
        if let Some(ref target) = self.target {
            target.action_performed(self.tag);
        }
    }
}
