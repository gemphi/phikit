//! What is true of a thing in space: where it stands relative to its
//! parent ([`pose`]) and the tree it stands in ([`tree`]).

pub mod pose;
pub mod tree;

pub use pose::{Matrix, Pose};
pub use tree::{Parent, Placed, adopt, children, place};
