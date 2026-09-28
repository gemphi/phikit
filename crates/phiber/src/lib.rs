//! Phiber library root. The `.blend` reader, scene model and asset
//! import/export live in the `phiphx` crate (`crates/phiphx`); it is
//! re-exported here as `crate::phiphx` (and the legacy alias `crate::phx`) so
//! existing `crate::phiphx::scene::…` paths keep resolving.
//! Binary entry point lives in `src/main.rs`.

pub use ::phiphx;
pub mod phx {
    pub use phiphx::*;
}
pub use phiphx::{
    AssetImportResult, AssetImporter, ImportedCollection, ImportedMaterial, ImportedObject, Mesh,
    PointCloud, PointCloudError, SampledPoint, Vertex, VertexAttributes,
};
pub use phiphx::Result as PhxResult;

pub mod app;
pub mod v2;
// The v1 view layer (`bui`, `state`, `store`, `services` and the app shell that
// drove them) was retired to `_unused/phiber-v1/` once v2 drew every view;
// git still has it.
