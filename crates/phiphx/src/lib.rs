//! phiphx — Phiber's asset core.
//!
//! **Entry point: [`phx::Phx`].** Every consumer opens, peeks and converts
//! files through it; the modules below are its parts.
//!
//! | Module | What it holds |
//! |---|---|
//! | [`phx`] | `Phx`: open / peek / detect, progress, errors (`phx.rs`) |
//! | [`device`] | one device per scene-file variant: `Device`, `DeviceList`, `Peek`; `usd`, `blend` (the native `.blend` reader), `phx` |
//! | [`scene`] | `SceneModel`: the runtime scene every view draws |
//! | [`import`] | other readers: glTF, PLY, OBJ; asset router |
//! | [`export`] | writers: glTF, PLY, PCD, LAS, XYZ, JSON, binary |
//! | [`cloud`] | point clouds and mesh → cloud sampling |
//! | [`geom`] | vertices, bounds, meshes, BVH, subject selection |
//! | [`format`] | output formats and material descriptions |
//! | [`spatial`] | spatial UI objects (cards, springs, animator) |
//!
//! Every folder has a `README.md` (what it is) and a `jobs.md` (status).


pub mod cloud;
pub mod comp;
pub mod device;
pub mod error;
pub mod export;
pub mod format;
pub mod geom;
pub mod import;
pub mod phx;
pub mod scene;
pub mod sky;
pub mod spatial;
pub mod uv;

pub use cloud::PointCloud;
pub use cloud::config::{MeshToCloudConfig, SamplingMethod};
pub use error::{PointCloudError, Result};
pub use export::*;
pub use format::*;
pub use geom::bound::BoundingBox;
pub use geom::bvh::{BvhNode, SimpleRng};
pub use geom::mesh::{Mesh, SampledPoint};
pub use geom::selector::{MeshSource, SubjectSelector};
pub use geom::vertex::{Vertex, VertexAttributeList};
pub use import::{
    AssetImportResult, AssetImporter, ImportedCollection, ImportedMaterial, ImportedObject,
};
pub use phx::Phx;
pub use device::phx::project;
pub use device::phx::project::*;
pub use spatial::{
    MeshSpaceConversion, PhiQuiMeshType, SpatialObject3D, SpatialSpring, SpatialTransform3D,
};
#[cfg(feature = "bui")]
pub use spatial::{BSpatialButton, BSpatialCard, SpatialCard, SpatialMeshElement, SpatialScene};
