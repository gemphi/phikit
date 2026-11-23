//! Project Module — Reality Composer Pro project management
//!
//! Mirrors Apple's .rcproject bundle architecture with these interfaces:
//! - ProjectManifest: Top-level project.json
//! - SceneData: Scene serialization
//! - AssetCatalog: Asset indexing
//! - ProjectManager: Main project interface (like NSDocument)

pub mod asset_catalog;
pub mod manifest;
pub mod project_manager;
pub mod scene_data;

pub use asset_catalog::{AssetCatalog, AssetEntry, AssetType};
pub use manifest::ProjectManifest;
pub use project_manager::ProjectManager;
pub use scene_data::{ComponentData, EntityData, EnvironmentData, SceneData, TransformData};
