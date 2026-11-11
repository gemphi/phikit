//! Point clouds: the `PointCloud` type, statistics and layout adapters,
//! serde, and the mesh → cloud sampling configuration.
//! Status and next steps: `jobs.md` in this folder.

pub mod config;
mod method;
mod serde_impl;
mod data;

pub use data::{PointCloud, PointCloudMetadata, PointCloudStat};
