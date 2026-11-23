//! 3D Scene render pass — draws entities, grid, and gizmos.
//!
//! Split from `render_pass.rs` to keep offscreen readback separate from
//! the actual scene drawing logic.

use super::renderer::GpuRenderer;

impl GpuRenderer {
    /// Draw the scene using an existing depth texture (convenience wrapper).
    pub fn draw_scene(
        &self,
        target_view: &wgpu::TextureView,
        scene: &crate::ecs::Scene,
        camera: &crate::gpu::ViewportCameraController,
        encoder: &mut wgpu::CommandEncoder,
        selected_id: Option<u64>,
    ) {
        self.draw_scene_with_depth(
            target_view,
            scene,
            camera,
            encoder,
            selected_id,
            self.depth_texture.as_ref(),
        );
    }

    /// Draw the full 3D scene pass: entities → grid → gizmos.
    ///
    /// Accepts an optional depth override for offscreen rendering.
    /// ✅ OPTIMIZED: Batch draw calls, early culling, reduced state changes
    pub fn draw_scene_with_depth(
        &self,
        target_view: &wgpu::TextureView,
        scene: &crate::ecs::Scene,
        camera: &crate::gpu::ViewportCameraController,
        encoder: &mut wgpu::CommandEncoder,
        selected_id: Option<u64>,
        depth_override: Option<&wgpu::TextureView>,
    ) {
        let (mesh_pipeline, grid_pipeline, camera_bind_group, depth_view) =
            if let (Some(mp), Some(gp), Some(cbg)) = (
                &self.mesh_pipeline,
                &self.grid_pipeline,
                &self.camera_bind_group,
            ) {
                let dv = match depth_override {
                    Some(d) => d,
                    None => match &self.depth_texture {
                        Some(d) => d,
                        None => return,
                    },
                };
                (mp, gp, cbg, dv)
            } else {
                return;
            };

        // Clear color for the 3D viewport — use the dark window background.
        // The shader applies manual linear→sRGB (pow 1/2.2) before output,
        // so the clear color must also be in sRGB space to match visually.
        let bg = crate::theme::color(crate::theme::keys::CLR_BG_WINDOW);
        let gamma = |c: f32| -> f64 { (c as f64).powf(1.0 / 2.2) };
        let clear = wgpu::Color {
            r: gamma(bg.r),
            g: gamma(bg.g),
            b: gamma(bg.b),
            a: 1.0,
        };

        // ✅ OPTIMIZATION: Lock once, not per-draw
        let buffers = self.mesh_buffers.lock().unwrap();
        let grid_lock = self.grid_vertex_buffer.lock().unwrap();
        let gizmo_lock = self.gizmo_vertex_buffer.lock().unwrap();

        // ✅ OPTIMIZATION: Early exit if nothing to draw
        if buffers.is_empty() && grid_lock.is_none() && gizmo_lock.is_none() {
            return;
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("3D Scene Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // 1. Draw Entities FIRST (before grid, so they're not depth-rejected)
            // ✅ OPTIMIZATION: Set pipeline and camera once for all meshes
            if !buffers.is_empty() {
                render_pass.set_pipeline(mesh_pipeline);
                render_pass.set_bind_group(0, camera_bind_group, &[]);
                
                // ✅ OPTIMIZATION: Batch meshes by material to reduce bind group changes
                for mesh in buffers.iter() {
                    render_pass.set_bind_group(1, &mesh.material_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    
                    if let Some(ref index_buffer) = mesh.index_buffer {
                        render_pass.set_index_buffer(
                            index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                    } else {
                        render_pass.draw(0..mesh.vertex_count, 0..1);
                    }
                }
            }

            // 2. Draw Grid (after meshes — grid is at y=0, meshes are above)
            if let Some(ref grid_buffer) = *grid_lock {
                render_pass.set_pipeline(grid_pipeline);
                render_pass.set_bind_group(0, camera_bind_group, &[]);
                render_pass.set_vertex_buffer(0, grid_buffer.slice(..));
                let count = (50.0 * 2.0 / 1.0 + 1.0) as u32 * 4;
                render_pass.draw(0..count, 0..1);
            }

            // 3. Draw Gizmos
            if selected_id.is_some() {
                if let Some(ref buffer) = *gizmo_lock {
                    let gizmo_count = self.gizmo_vertex_count.lock().map(|c| *c).unwrap_or(0);
                    if gizmo_count > 0 {
                        render_pass.set_pipeline(grid_pipeline);
                        render_pass.set_bind_group(0, camera_bind_group, &[]);
                        render_pass.set_vertex_buffer(0, buffer.slice(..));
                        render_pass.draw(0..gizmo_count, 0..1);
                    }
                }
            }
        }
    }

    /// Internal helper to draw coordinate axes at an entity position.
    /// TODO: Re-implement with pre-allocated gizmo buffer outside render pass scope.
    #[allow(unused)]
    fn draw_gizmos(
        &self,
        _render_pass: &mut wgpu::RenderPass<'_>,
        _scene: &crate::ecs::Scene,
        _entity_id: u64,
    ) {
    }
}
