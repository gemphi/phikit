//! Materials. Slots, viewport colour and Principled inputs, shader node tree, images (keys and packed bytes).
//! Status and next steps: `jobs.md` in this folder.

pub mod graph;
pub(crate) mod group;
pub(crate) mod fold;
pub(crate) mod sepcomb;
pub(crate) mod vector;
pub mod image;
pub mod socket;
pub(crate) mod material;
pub(crate) mod node;
pub mod bake;
pub(crate) mod blend;
pub mod driver;
pub(crate) mod driver_read;
pub(crate) mod emit;
pub(crate) mod eval;
pub(crate) mod link;
pub(crate) mod map;
pub(crate) mod principled;
pub mod procedural;
pub(crate) mod procscalar;
pub(crate) mod texnode;
pub(crate) mod scalar;
pub(crate) mod slot;
pub mod surface;
pub(crate) mod world;
pub mod world_driver;
