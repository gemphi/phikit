//! Logic that runs on the world each step, in stage order.
pub mod event;
mod schedule;
pub mod space;
mod stage;
pub use event::{DidAdd, EventQueue, WillRemove};
pub use stage::{FnSystem, Stage, System};
