//! Mesh synchronization — bridges ECS scene state to GPU buffers.
//!
//! Split from `renderer.rs` to keep each file under ~200 lines.
//! Swift equivalent: `MeshSynchronizer.swift`

mod buffer_sync;
mod mesh_upload;
mod vertex_management;
