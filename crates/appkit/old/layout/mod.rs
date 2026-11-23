//! # Unified Layout System
//!
//! Single source of truth for spacing, padding, margin, and visual hierarchy.
//! All values follow an 8pt grid system for perfect alignment.

pub mod spacing;
pub mod hierarchy;
pub mod constants;

pub use spacing::*;
pub use hierarchy::*;
pub use constants::*;

// Re-export for convenience
pub use crate::graphics::NSEdgeInsets;

/// Create edge insets with uniform spacing
pub fn inset_uniform(value: f32) -> NSEdgeInsets {
    NSEdgeInsets::new(value, value, value, value)
}

/// Create edge insets with vertical and horizontal spacing
pub fn inset_vh(vertical: f32, horizontal: f32) -> NSEdgeInsets {
    NSEdgeInsets::new(vertical, horizontal, vertical, horizontal)
}

/// Create edge insets with individual values
pub fn inset_trbl(top: f32, right: f32, bottom: f32, left: f32) -> NSEdgeInsets {
    NSEdgeInsets::new(top, right, bottom, left)
}
