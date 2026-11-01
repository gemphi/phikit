//! Asset Loading System
//!
//! Loads 3D meshes and textures from disk.
//! Supports: OBJ (via tobj), USD/USDA/USDZ (via phx::importers::usd),
//! PHX (via phx::PointCloud), and raw vertex/index buffers.

mod asset_loading;
mod mesh_assets;
mod texture_management;

pub use asset_loading::AssetLoader;
