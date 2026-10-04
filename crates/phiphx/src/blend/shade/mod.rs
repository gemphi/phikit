//! Materials. Slots, viewport colour and Principled inputs, shader node tree, images (keys and packed bytes).
//! Status and next steps: `jobs.md` in this folder.

pub mod graph;
pub mod image;
pub mod socket;
pub(crate) mod material;
pub(crate) mod nodes;
pub(crate) mod blend;
pub(crate) mod eval;
pub(crate) mod links;
pub(crate) mod maps;
pub(crate) mod principled;
pub(crate) mod scalar;
pub(crate) mod slots;
pub mod surface;
pub(crate) mod world;
