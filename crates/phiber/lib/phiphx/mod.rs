//! PHX - PHI Point Cloud eXtension
//!
//! Converts USD and glTF files to various point cloud formats.
//! Supports PHX, PCD, LAS, and custom JSON formats.

pub mod bounding_box;
pub mod bvh;
pub mod config;
pub mod exporter;
pub mod format;
pub mod importer;
pub mod mesh;
pub mod point_cloud;
pub mod project;
pub mod selector;
pub mod services;
pub mod spatial;
pub mod vertex;

pub mod error;

pub use bounding_box::BoundingBox;
pub use bvh::{BvhNode, SimpleRng};
pub use config::{MeshToCloudConfig, SamplingMethod};
pub use error::{PointCloudError, Result};
pub use exporter::*;
pub use format::*;
pub use importer::{
    AssetImportResult, AssetImporter, ImportedCollection, ImportedMaterial, ImportedObject,
};
#[cfg(feature = "appkit")]
pub use importer::{ProjectImportResult, ProjectImporter};
pub use mesh::{Mesh, SampledPoint};
pub use point_cloud::PointCloud;
pub use project::*;
pub use selector::{MeshSource, SubjectSelector};
pub use spatial::{
    BSpatialButton, BSpatialCard, MeshSpaceConversion, PhiQuiMeshType, SpatialCard,
    SpatialMeshElement, SpatialObject3D, SpatialScene, SpatialSpring, SpatialTransform3D,
};
pub use vertex::{Vertex, VertexAttributes};

// Backward-compatible plural module aliases for downstream crates
pub use exporter as exporters;
pub use format as formats;
pub use importer as importers;
