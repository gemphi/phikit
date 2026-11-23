//! GPU Renderer — core struct, device initialization, and helpers.
//!
//! Render pass logic is in `render_pass.rs`.
//! Mesh synchronization is in `mesh_sync.rs`.

mod draw_commands;
mod frame_state;
mod render_loop;

pub use frame_state::GpuRenderer;
