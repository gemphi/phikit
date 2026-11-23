//! Layout Constants - Token-driven geometry for the entire app.
//!
//! Sub-modules grouped by domain:
//! - `spacing`: spacing tokens, radii, and typography
//! - `sizing`: icon sizes, control heights, window/toolbar/bar dimensions
//! - `welcome`: welcome window layout
//! - `panel_defaults`: panel, overlay, accordion, menu, and preferences defaults
//! - `static_constants`: named `pub const` values

mod spacing;
mod sizing;
mod welcome;
mod panel_defaults;
mod static_constants;

pub use spacing::*;
pub use sizing::*;
pub use welcome::*;
pub use panel_defaults::*;
pub use static_constants::*;
