//! Where views go: a constraint, a spacing, a metric. So far: breaking
//! text into the lines that fit a width, how far a content longer than
//! its room is scrolled, and a section on a ground of its own, which is
//! how every app groups what belongs together; and three places in a
//! row or a column, the start, the middle and the end (`three.rs`).

mod scroll;
mod section;
mod stack;
mod three;
mod wrap;

pub use scroll::PScroll;
pub use section::{PDry, ground, section};
pub use stack::{BARE, PDrawn, PGrouped, SPACE, bare, level, stack, tabbed};
pub use three::{PRun, places};
pub use wrap::{fold, wrap};
