//! Activity Bar — VS Code-style vertical navigation strip.
//!
//! Sits at the far left of the editor window. Contains icon buttons
//! arranged in three groups:
//! - Top: primary navigation (clock/recent activity)
//! - Middle: workspace sections (home, menu, extensions)
//! - Bottom: user/settings
//!
//! All sizes from layout_constants. All colors from theme.

pub mod model;
pub mod view_controller;

pub use model::*;
pub use view_controller::*;
