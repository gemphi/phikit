//! GPU rendering module (< 200 lines each)
//!
//! Modular GPU viewport implementation

pub mod assets;
pub mod camera;
pub mod mesh;
pub mod mesh_sync;
pub mod overlay;
pub mod pipeline;
pub mod render_pass;
pub mod renderer;
pub mod scene_pass;
// pub mod ssao_pipeline;  // TODO: Missing file
pub mod viewport;
pub mod viewport_types;

use std::sync::{Arc, Mutex, OnceLock};

static GLOBAL_RENDERER: OnceLock<Arc<Mutex<GpuRenderer>>> = OnceLock::new();

/// Set the global GPU renderer (called once during init).
pub fn set_global_renderer(renderer: Arc<Mutex<GpuRenderer>>) {
    let _ = GLOBAL_RENDERER.set(renderer);
}

/// Get the global GPU renderer (safe to call from draw delegates).
pub fn get_global_renderer() -> Option<Arc<Mutex<GpuRenderer>>> {
    GLOBAL_RENDERER.get().cloned()
}

pub use assets::AssetLoader;
pub use camera::{CameraUniforms, MouseButton, ViewportCameraController};
pub use mesh::{GridVertex, MeshBuffer, Vertex};
pub use pipeline::PipelineFactory;
pub use renderer::GpuRenderer;
pub use viewport::{invalidate_viewport_cache, NS3DViewportGPU};
