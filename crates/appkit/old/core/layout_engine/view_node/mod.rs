//! # ViewNode Module
//!
//! The declarative view tree representation. Each variant describes
//! a UI element or container that the layout engine resolves into
//! absolute frames.

mod builders;
mod clone_impl;
mod debug_impl;
mod node_types;

pub use node_types::{next_view_id, Action, ViewNode};
