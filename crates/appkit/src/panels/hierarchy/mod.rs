//! Hierarchy panel - scene entity tree.
//! 
//! Follows Apple's NSOutlineView pattern with data source binding.
pub mod model;
pub mod view_controller;

pub use model::*;
pub use view_controller::*;
