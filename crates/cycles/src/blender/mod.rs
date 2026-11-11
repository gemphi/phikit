// SPDX-License-Identifier: Apache-2.0
//! The bridge from a scene file to a Cycles scene: `intern/cycles/blender/`.
//! Cycles' `blender/` reads Blender's RNA; here the file is read by phiphx
//! and handed over as a [`SceneSource`] (`source.rs`), which `sync.rs` turns
//! into a [`Scene`](crate::scene::Scene).
//!
//! | File | Ported from |
//! |---|---|
//! | `source.rs` | the values `blender/*.cpp` read from RNA |
//! | `camera.rs` | `blender/camera.cpp` |
//! | `mesh.rs` | `blender/mesh.cpp` |
//! | `sync.rs` | `blender/sync.cpp`, `object.cpp`, `shader.cpp` |
//! | `model.rs` | (none) phiphx's `SceneModel` into a `SceneSource`: the only file that names phiphx |
mod camera;
mod mesh;
mod model;
mod source;
mod sync;
pub use camera::{camera_matrix, sync_camera, transform_of, viewplane};
pub use mesh::sync_mesh;
pub use model::{Model, read_blend, read_model, source_of_model};
pub use source::{CameraSource, MaterialSource, MeshSource, ObjectSource, SceneSource, SensorFit};
pub use sync::{surface_of, sync_scene};
