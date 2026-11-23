//! Where views go: a constraint, a spacing, a metric. So far: breaking
//! text into the lines that fit a width, how far a content longer than
//! its room is scrolled, and a section on a ground of its own, which is
//! how every app groups what belongs together.

mod scroll;
mod section;
mod stack;
mod wrap;

pub use scroll::PScroll;
pub use section::{PDry, ground, section};
pub use stack::{PDrawn, PGrouped, stack, tabbed};
pub use wrap::{fold, wrap};
