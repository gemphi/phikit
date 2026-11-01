//! GPU Renderer — core struct definition and state queries.

use std::sync::{Arc, Mutex};

/// GPU Renderer - manages WGPU device and executes render passes
pub struct GpuRenderer {
    pub surface: Option<wgpu::Surface<'static>>,
    pub device: Option<Arc<wgpu::Device>>,
    pub queue: Option<Arc<wgpu::Queue>>,
    pub config: Option<wgpu::SurfaceConfiguration>,
    pub camera_uniform_buffer: Option<wgpu::Buffer>,
    pub camera_bind_group: Option<wgpu::BindGroup>,

    // UI Overlay (Software Blit)
    pub overlay_pipeline: Option<wgpu::RenderPipeline>,
    pub overlay_texture: Option<wgpu::Texture>,
    pub overlay_bind_group: Option<wgpu::BindGroup>,
    pub overlay_sampler: Option<wgpu::Sampler>,

    // 3D Rendering
    pub mesh_pipeline: Option<wgpu::RenderPipeline>,
    pub grid_pipeline: Option<wgpu::RenderPipeline>,
    pub material_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub grid_vertex_buffer: Mutex<Option<wgpu::Buffer>>,
    pub gizmo_vertex_buffer: Mutex<Option<wgpu::Buffer>>,
    pub gizmo_vertex_count: Mutex<u32>,
    pub depth_texture: Option<wgpu::TextureView>,
    pub mesh_buffers: Mutex<Vec<crate::gpu::MeshBuffer>>,
}

impl GpuRenderer {
    pub fn new() -> Self {
        Self {
            surface: None,
            device: None,
            queue: None,
            config: None,
            camera_uniform_buffer: None,
            camera_bind_group: None,
            overlay_pipeline: None,
            overlay_texture: None,
            overlay_bind_group: None,
            overlay_sampler: None,
            mesh_pipeline: None,
            grid_pipeline: None,
            material_bind_group_layout: None,
            grid_vertex_buffer: Mutex::new(None),
            gizmo_vertex_buffer: Mutex::new(None),
            gizmo_vertex_count: Mutex::new(0),
            depth_texture: None,
            mesh_buffers: Mutex::new(Vec::new()),
        }
    }

    /// Returns (total_vertices, total_indices, buffer_count) from all loaded GPU mesh buffers.
    pub fn mesh_stats(&self) -> (u32, u32, u32) {
        if let Ok(bufs) = self.mesh_buffers.lock() {
            let mut verts = 0u32;
            let mut indices = 0u32;
            for buf in bufs.iter() {
                verts += buf.vertex_count;
                indices += buf.index_count;
            }
            (verts, indices, bufs.len() as u32)
        } else {
            (0, 0, 0)
        }
    }

    /// Returns (min, max) bounding box of all loaded mesh buffers, if any exist.
    pub fn mesh_bounds(&self) -> Option<([f32; 3], [f32; 3])> {
        if let Ok(bufs) = self.mesh_buffers.lock() {
            if bufs.is_empty() {
                return None;
            }
            let mut global_min = [f32::MAX; 3];
            let mut global_max = [f32::MIN; 3];
            let mut found = false;
            for buf in bufs.iter() {
                if let Some((bmin, bmax)) = buf.bounds {
                    for i in 0..3 {
                        global_min[i] = global_min[i].min(bmin[i]);
                        global_max[i] = global_max[i].max(bmax[i]);
                    }
                    found = true;
                }
            }
            if found {
                Some((global_min, global_max))
            } else {
                None
            }
        } else {
            None
        }
    }
}
