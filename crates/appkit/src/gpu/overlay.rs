//! UI Overlay rendering — texture upload and fullscreen-triangle composite pass.
//!
//! Split from `render_pass.rs` to keep each file under ~200 lines.

use super::renderer::GpuRenderer;

impl GpuRenderer {
    /// Update UI overlay texture from CPU buffer
    pub fn update_ui_overlay(&mut self, width: u32, height: u32, pixels: &[u32]) {
        let (device, queue) = if let (Some(d), Some(q)) = (&self.device, &self.queue) {
            (d, q)
        } else {
            return;
        };

        // Create or resize texture if needed
        let mut needs_bind_group = false;
        if self.overlay_texture.is_none()
            || self.overlay_texture.as_ref().unwrap().width() != width
            || self.overlay_texture.as_ref().unwrap().height() != height
        {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("UI Overlay Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.overlay_texture = Some(texture);
            needs_bind_group = true;
        }

        // Upload pixels
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: self.overlay_texture.as_ref().unwrap(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(pixels),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        if needs_bind_group {
            let view = self
                .overlay_texture
                .as_ref()
                .unwrap()
                .create_view(&wgpu::TextureViewDescriptor::default());
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Overlay Bind Group"),
                layout: &self
                    .overlay_pipeline
                    .as_ref()
                    .unwrap()
                    .get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            self.overlay_sampler.as_ref().unwrap(),
                        ),
                    },
                ],
            });
            self.overlay_bind_group = Some(bind_group);
        }
    }

    /// Composite the UI overlay onto the final surface using a fullscreen triangle.
    pub fn render_overlay(&self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let (pipeline, bind_group) =
            if let (Some(p), Some(bg)) = (&self.overlay_pipeline, &self.overlay_bind_group) {
                (p, bg)
            } else {
                return;
            };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Overlay Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
