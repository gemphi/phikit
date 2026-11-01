//! Panel Defaults — standardized styling for all editor panels.
//!
//! Provides default colors, fonts, spacing, and corner radius for panel types.
//! Apps only specify layout structure; appearance comes from the current theme.

mod label_defaults;
mod panel_styling;
mod styled_panel;
mod widget_defaults;

pub use label_defaults::*;
pub use panel_styling::*;
pub use styled_panel::*;
pub use widget_defaults::*;
