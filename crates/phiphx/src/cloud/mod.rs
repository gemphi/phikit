//! Point clouds: the `PointCloud` type, statistics and layout adapters,
//! serde, and the mesh → cloud sampling configuration.
//! Status and next steps: `jobs.md` in this folder.

pub mod config;
mod methods;
mod serde_impl;
mod types;

pub use types::{PointCloud, PointCloudMetadata, PointCloudStats};
