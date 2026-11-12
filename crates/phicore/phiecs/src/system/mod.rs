//! Logic that runs on the world each step, in stage order.
mod stage;
pub use stage::{FnSystem, Stage, System};
