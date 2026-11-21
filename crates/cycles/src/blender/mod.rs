// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
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
//! | `graph.rs` | the node tree `blender/shader.cpp` reads through RNA |
//! | `node.rs` | `blender/shader.cpp` `add_node` |
//! | `shader.rs` | `blender/shader.cpp` `add_nodes` |
//! | `model.rs` | (none) phiphx's `SceneModel` into a `SceneSource`: the only file that names phiphx |
mod camera;
mod graph;
mod integrator;
mod mesh;
mod model;
mod node;
mod shader;
mod source;
mod sync;
pub use graph::{GraphSource, LinkSource, NodeSource, NodeStorage, SocketClass, SocketSource, SocketValue};
pub use shader::graph_of;
pub use camera::{camera_matrix, sync_camera, transform_of, viewplane};
pub use integrator::{sample_count, setting, sync_film, sync_integrator};
pub use mesh::sync_mesh;
pub use model::{Model, read_blend, read_model, source_of_model};
pub use source::{CameraSource, MaterialSource, MeshSource, NodeSurface, ObjectSource, SceneSource, SensorFit};
pub use sync::{surface_of, sync_scene};
