//! Blender Native Exporter — Full Scene Export with Instancing
//!
//! This module provides comprehensive Blender scene export capabilities:
//! - Mesh geometry with materials
//! - Light sources (point, sun, spot, area)
//! - Camera settings
//! - Object instancing (linked duplicates, collection instances)
//! - Scene hierarchy (parent-child relationships)
//! - Collection export
//! - Point cloud generation via area-weighted sampling
//!
//! # Supported Export Formats
//! - PHX Scene (.phx.ply) - Extended PHX with scene metadata
//! - glTF 2.0 (.gltf/.glb) - With PHX point cloud extensions
//! - USDA (.usda) - USD ASCII with PointInstancer for instances
//!
//! # Instancing Support
//! Blender's instancing is exported via:
//! - **USD**: `PointInstancer` with prototypes for particle/dupli systems
//! - **glTF**: `EXT_mesh_gpu_instancing` extension
//! - **PHX Scene**: Instance transforms with prototype references

pub mod chunks;
mod config;
mod helpers;
mod script_gltf;
mod script_phx;
mod script_usd;
mod types;

pub use config::{PointCloudConfig, SceneExportConfig};
pub use chunks::PhxChunks;
pub use types::{
    BlenderLightType, BlenderObjectType, BlenderScene, BlenderSceneFormat, CameraData, CameraType,
    CollectionData, InstanceData, LightData, MaterialData, MeshObjectData,
};

#[cfg(test)]
mod tests;
