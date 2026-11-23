//! GPU Renderer — resource creation, resizing, and draw commands.

use super::GpuRenderer;

impl GpuRenderer {
    // ─── Pipeline & Resource Setup ──────────────────────────────────

    pub(in crate::gpu) fn setup_pipelines(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) {
        let camera_layout = crate::gpu::PipelineFactory::create_camera_bind_group_layout(device);
        let material_layout =
            crate::gpu::PipelineFactory::create_material_bind_group_layout(device);
        let texture_layout =
            crate::gpu::PipelineFactory::create_texture_bind_group_layout(device);

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: std::mem::size_of::<crate::gpu::CameraUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });
        self.camera_uniform_buffer = Some(camera_buffer);
        self.camera_bind_group = Some(camera_bind_group);
        self.material_bind_group_layout = Some(material_layout);

        // Create render pipelines
        let mesh_pipeline = crate::gpu::PipelineFactory::create_render_pipeline(
            device,
            &camera_layout,
            self.material_bind_group_layout.as_ref().unwrap(),
            format,
        );
        let grid_pipeline =
            crate::gpu::PipelineFactory::create_grid_pipeline(device, &camera_layout, format);
        let overlay_pipeline = crate::gpu::PipelineFactory::create_overlay_pipeline(device, &texture_layout, format);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        self.mesh_pipeline = Some(mesh_pipeline);
        self.grid_pipeline = Some(grid_pipeline);
        self.overlay_pipeline = Some(overlay_pipeline);
        self.overlay_sampler = Some(sampler);
    }

    // ─── Resource Helpers ───────────────────────────────────────────

    pub(crate) fn create_grid_buffer(&mut self) {
        use wgpu::util::DeviceExt;
        let device = self.device.as_ref().unwrap();

        let mut vertices = Vec::new();
        let size = 50.0;
        let step = 1.0;
        // Subtle purple-tinted grid lines (matches dark purple viewport bg)
        let color = [0.12, 0.10, 0.16, 0.6];
        let axis_color = [0.20, 0.16, 0.28, 0.8];

        let mut i: f32 = -size;
        while i <= size {
            let c = if i.abs() < 0.1 { axis_color } else { color };
            // X lines
            vertices.push(crate::gpu::GridVertex {
                position: [i, 0.0, -size],
                color: c,
            });
            vertices.push(crate::gpu::GridVertex {
                position: [i, 0.0, size],
                color: c,
            });
            // Z lines
            vertices.push(crate::gpu::GridVertex {
                position: [-size, 0.0, i],
                color: c,
            });
            vertices.push(crate::gpu::GridVertex {
                position: [size, 0.0, i],
                color: c,
            });
            i += step;
        }

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        *self.grid_vertex_buffer.lock().unwrap() = Some(buffer);
    }

    pub(crate) fn create_depth_texture(&mut self, width: u32, height: u32) {
        let device = self.device.as_ref().unwrap();
        let size = wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        self.depth_texture = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));
    }

    /// Resize surface
    pub fn resize(&mut self, width: u32, height: u32) {
        let (w, h) = if let (Some(config), Some(surface), Some(device)) = (
            self.config.as_mut(),
            self.surface.as_ref(),
            self.device.as_ref(),
        ) {
            config.width = width.max(1);
            config.height = height.max(1);
            surface.configure(device, config);
            (config.width, config.height)
        } else {
            return;
        };

        self.create_depth_texture(w, h);
    }

    /// Update camera uniforms
    pub fn update_camera_uniforms(
        &self,
        view_proj: &crate::math::SCNMatrix4,
        camera_pos: [f32; 4],
    ) {
        if let (Some(queue), Some(buffer)) =
            (self.queue.as_ref(), self.camera_uniform_buffer.as_ref())
        {
            let uniforms = crate::gpu::CameraUniforms {
                view_proj: view_proj.to_glam().to_cols_array_2d(),
                camera_position: camera_pos,
            };
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[uniforms]));
        }
    }

}
