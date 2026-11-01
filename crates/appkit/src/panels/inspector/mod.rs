//! Inspector panel — property editor for selected entities.
//! 
//! Follows Apple's NSViewController pattern with automatic layout.
pub mod model;
pub mod view_controller;

pub use model::*;
pub use view_controller::*;
