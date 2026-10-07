//! phiphx — Phiber's asset core.
//!
//! **Entry point: [`phx::Phx`].** Every consumer opens, peeks and converts
//! files through it; the modules below are its parts.
//!
//! | Module | What it holds |
//! |---|---|
//! | [`phx`] | `Phx` — open / peek / detect, progress, errors (`phx.rs`) |
//! | [`scene`] | `SceneModel` — the runtime scene every view draws |
//! | [`blend`] | the native `.blend` reader (file, objects, meshes, materials, modifiers) |
//! | [`import`] | other readers: glTF, USD, PLY, PHX, OBJ; asset router |
//! | [`export`] | writers: PHX, glTF, USD, PLY, PCD, LAS, XYZ, JSON, binary, blend scripts |
//! | [`cloud`] | point clouds and mesh → cloud sampling |
//! | [`geom`] | vertices, bounds, meshes, BVH, subject selection |
//! | [`format`] | output formats and material descriptions |
//! | [`spatial`] | spatial UI objects (cards, springs, animator) |
//!
//! Every folder has a `README.md` (what it is) and a `jobs.md` (status).

pub mod blend;
pub mod cloud;
pub mod comp;
pub mod error;
pub mod export;
pub mod format;
pub mod geom;
pub mod import;
pub mod phx;
#[cfg(test)]
mod phx_tests;
pub mod project;
pub mod scene;
pub mod sky;
pub mod spatial;
pub mod uv;

pub use cloud::PointCloud;
pub use cloud::config::{MeshToCloudConfig, SamplingMethod};
pub use error::{PointCloudError, Result};
pub use export::*;
pub use format::*;
pub use geom::bounds::BoundingBox;
pub use geom::bvh::{BvhNode, SimpleRng};
pub use geom::mesh::{Mesh, SampledPoint};
pub use geom::selector::{MeshSource, SubjectSelector};
pub use geom::vertex::{Vertex, VertexAttributes};
pub use import::{
    AssetImportResult, AssetImporter, ImportedCollection, ImportedMaterial, ImportedObject,
};
#[cfg(feature = "appkit")]
pub use import::{ProjectImportResult, ProjectImporter};
pub use phx::Phx;
pub use project::*;
pub use spatial::{
    MeshSpaceConversion, PhiQuiMeshType, SpatialObject3D, SpatialSpring, SpatialTransform3D,
};
#[cfg(feature = "bui")]
pub use spatial::{BSpatialButton, BSpatialCard, SpatialCard, SpatialMeshElement, SpatialScene};
