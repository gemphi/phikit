//! Display mode — `SpaceOutliner.display_mode`.
//!
//! Structure and content both live in `set.rs` here: the spec struct is
//! small enough that separating it from its own rows would cost more to
//! follow than it saves.

pub mod set;

pub use set::{BOutlineMode, BOutlineModeSpec, MODES};
