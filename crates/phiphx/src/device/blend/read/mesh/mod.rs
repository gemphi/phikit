//! Mesh geometry. Positions, faces, UVs, normals, face attributes, CustomData (≤ 4.x) and attribute storage (5.x), vertex groups.
//! Status and next steps: `jobs.md` in this folder.

pub(crate) mod attrs;
pub(crate) mod block;
pub(crate) mod crease;
pub(crate) mod custom;
pub(crate) mod faces;
pub(crate) mod groups;
pub(crate) mod legacy;
pub mod list;
pub(crate) mod normals;
pub(crate) mod offsets;
pub(crate) mod positions;
pub(crate) mod read;
pub(crate) mod shading;
pub(crate) mod uv;
