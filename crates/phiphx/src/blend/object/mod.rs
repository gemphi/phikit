//! Objects. DNA fields, transforms and parent chain, rotation modes, kind, camera/light/empty data, collections, view layer, world.
//! Status and next steps: `jobs.md` in this folder.

pub(crate) mod collection;
pub mod data;
pub mod light_driver;
pub(crate) mod light_driver_read;
pub mod dof;
pub(crate) mod dna;
pub(crate) mod kind;
pub mod layer;
pub(crate) mod meta;
pub(crate) mod read;
pub(crate) mod rotation;
pub(crate) mod transform;
pub mod visibility;
pub(crate) mod world;
