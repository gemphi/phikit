//! Geometry primitives shared by every reader and exporter: vertices,
//! bounds, triangle meshes with surface sampling, the sampling BVH and the
//! primary-subject selector.
//! Status and next steps: `jobs.md` in this folder.

pub mod bounds;
pub mod bvh;
pub mod mesh;
pub mod selector;
pub mod vertex;
