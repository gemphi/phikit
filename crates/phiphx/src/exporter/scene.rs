//! PHX Scene Format — Extended PHX with Scene Metadata
//!
//! The PHX Scene format extends the standard PHX point cloud format to include:
//! - Scene metadata (name, frame, resolution)
//! - Camera with FOV, clip planes, and transform
//! - Light sources (point, sun, spot, area) with PBR properties
//! - Mesh geometry with materials and UVs
//! - Object instancing (linked duplicates, collection instances)
//! - Scene hierarchy (collections, parent-child relationships)
//!
//! # File Format
//! ```text
//! PHX_SCENE\x01                    # Magic + version
//! <u32: json_length>              # Length of JSON metadata
//! <json: scene_metadata>           # Scene description
//! <binary: geometry_data>          # Vertices, indices, etc.
//! ```
//!
//! # Example Scene JSON
//! ```json
//! {
//!   "name": "MyScene",
//!   "frame": 1,
//!   "resolution": [1920, 1080],
//!   "format": "phx-scene-v1",
//!   "camera": {
//!     "name": "Camera",
//!     "type": "perspective",
//!     "fov": 0.785,
//!     "matrix_world": [...]
//!   },
//!   "lights": [
//!     { "name": "Sun", "type": "SUN", "energy": 10.0, ... }
//!   ],
//!   "meshes": [
//!     { "name": "Cube", "vertices": [...], "indices": [...], ... }
//!   ],
//!   "instances": [
//!     { "name": "Tree_Instancer", "prototype": "Tree", "transforms": [...] }
//!   ],
//!   "materials": [
//!     { "name": "Material", "base_color": [1, 0, 0, 1], ... }
//!   ]
//! }
//! ```

mod read;
mod scene;
mod types;
mod write;

pub use read::{import_scene_from_bytes, read_scene};
pub use types::{
    PHX_SCENE_MAGIC, PHX_SCENE_VERSION, PhxCamera, PhxCollection, PhxInstance, PhxLight,
    PhxMaterial, PhxMesh, PhxScene,
};
pub use write::{export_phx_scene_to_bytes, phx_scene_to_phx, sample_scene, write_phx_scene};

#[cfg(test)]
mod tests;
