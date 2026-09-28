//! Row icons per Blender ID type (the common ones).
//!
//! Structure and content both live in `set.rs` here: the spec struct is
//! small enough that separating it from its own rows would cost more to
//! follow than it saves.

pub mod set;

pub use set::{BOutlineRowSpec, BOutlineRowType, ROW_TYPES};
