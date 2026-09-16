//! Phiber library root — declares the phiphx scene/point-cloud library so
//! its internal `crate::phx::*` / `crate::PointCloud` references resolve.
//! Binary entry point lives in `src/main.rs`.

#[path = "../lib/phiphx/mod.rs"]
pub mod phiphx;
pub mod phx {
    pub use crate::phiphx::*;
}
pub use phiphx::{
    AssetImportResult, AssetImporter, ImportedCollection, ImportedMaterial, ImportedObject, Mesh,
    PointCloud, PointCloudError, SampledPoint, Vertex, VertexAttributes,
};
pub use phx::Result as PhxResult;

pub mod app;
pub mod bui;
pub mod state;
pub mod store;
pub mod v2;
