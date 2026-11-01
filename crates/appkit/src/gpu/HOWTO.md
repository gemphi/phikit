# GPU — How to Use

## Files
- `mod.rs` — global renderer singleton, re-exports
- `renderer.rs` — `GpuRenderer` main struct (wgpu device, queue, pipelines)
- `viewport.rs` — `NS3DViewportGPU` for 3D scene rendering
- `viewport/` — viewport state management
- `camera.rs` — `ViewportCameraController`, `CameraUniforms`
- `mesh.rs` — `MeshBuffer`, `Vertex`, `GridVertex`
- `pipeline.rs` — `PipelineFactory` for shader pipeline creation
- `render_pass.rs` — render pass configuration
- `scene_pass.rs` — scene rendering pass
- `ssao_pipeline.rs` — screen-space ambient occlusion
- `overlay.rs` — 2D overlay rendering on top of 3D

## Quick Start
```rust
use rust_appkit::gpu::{GpuRenderer, NS3DViewportGPU};

// Set up global renderer (called once at init)
let renderer = Arc::new(Mutex::new(GpuRenderer::new(&device, &queue)));
gpu::set_global_renderer(renderer);

// Get the renderer from anywhere
if let Some(r) = gpu::get_global_renderer() {
    let renderer = r.lock().unwrap();
    // Use renderer...
}

// 3D viewport
let viewport = NS3DViewportGPU::new(frame);
```

## Key APIs
- `gpu::set_global_renderer(arc)` — init global renderer
- `gpu::get_global_renderer()` — access from draw delegates
- `NS3DViewportGPU::new(frame)` — 3D viewport widget
- `ViewportCameraController` — orbit/pan/zoom camera
- `MeshBuffer` — GPU mesh data (vertices + indices)

## Rules
- Global renderer is set once at startup via `set_global_renderer`
- Viewport delegates implement `gpu_draw()` for GPU-accelerated rendering
- All GPU state is behind `Arc<Mutex<...>>` for thread safety
- Use `invalidate_viewport_cache()` to force re-render
