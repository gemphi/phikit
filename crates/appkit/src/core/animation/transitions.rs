//! Window Transitions
//!
//! Handles transitions between app states (welcome → editor).
//! Mirrors: NSWindowController scene transitions in AppKit.

use crate::prelude::*;

/// Transition context for view state changes
pub struct TransitionContext {
    pub from_view: String,
    pub to_view: String,
    pub duration_ms: u32,
}

impl TransitionContext {
    pub fn new(from: &str, to: &str) -> Self {
        Self {
            from_view: from.to_string(),
            to_view: to.to_string(),
            duration_ms: 300,
        }
    }
}

/// Generic view transition trait
pub trait ViewTransition {
    fn transition(&self, root: &mut NSView, build_new_view: impl FnOnce() -> NSView);
}

/// Simple fade transition implementation
pub struct FadeTransition;

impl ViewTransition for FadeTransition {
    fn transition(&self, root: &mut NSView, build_new_view: impl FnOnce() -> NSView) {
        root.remove_all_subviews();
        let new_view = build_new_view();
        root.add_subview(new_view);
    }
}
