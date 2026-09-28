//! The Outliner's sets, by subject.
//!
//! Each is a folder of its own — display modes, restriction columns, row
//! icons, header filters — and this re-exports them so a caller that wants
//! one row type does not have to know which folder it came from.

pub use super::mode::{BOutlineMode, BOutlineModeSpec, MODES};
pub use super::toggle::{BOutlineToggle, BOutlineToggleSpec, TOGGLES};
pub use super::row_type::{BOutlineRowSpec, BOutlineRowType, ROW_TYPES};
pub use super::filter::{BOutlineFilterSpec, FILTERS};
