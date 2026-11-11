//! Mesh geometry. Positions, faces, UVs, normals, face attributes, CustomData (≤ 4.x) and attribute storage (5.x), vertex groups.
//! Status and next steps: `jobs.md` in this folder.

pub(crate) mod attr;
pub(crate) mod block;
pub(crate) mod crease;
pub(crate) mod custom;
pub(crate) mod face;
pub(crate) mod group;
pub(crate) mod legacy;
pub mod list;
pub(crate) mod normal;
pub(crate) mod offset;
pub(crate) mod position;
pub(crate) mod read;
pub(crate) mod shading;
pub(crate) mod uv;
