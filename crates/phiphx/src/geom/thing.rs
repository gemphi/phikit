//! Things built of parts, as a collection's tables say: each row of a
//! thing is a part (a box, a ball, a pill, a can, a cone, a ring, a
//! dome, a letter of a font, an outline of a drawing, or another thing)
//! with its size, where it stands, how it is turned, how round its
//! edges are and the look it wears; a row may repeat, each copy moved
//! and turned a step on. A thing inside a thing keeps its own place, so
//! a scene is a thing of things, as deep as its tables nest them.
//! `export::usd` writes a built thing as USDA (`ThingTree::to_usda`).

mod build;
mod form;
mod outline;
mod read;
mod svg;
#[cfg(test)]
mod svg_tests;
#[cfg(test)]
mod tests;

pub use build::{ThingNode, ThingTree};
pub use read::{Thing, ThingForm, ThingLook, ThingRow, ThingSet};
