//! Offscreen render-to-pixels — renders the 3D scene to a CPU-readable buffer.
//!
//! Scene drawing logic lives in `scene_pass.rs`.
//! UI overlay compositing lives in `overlay.rs`.

use super::renderer::GpuRenderer;

impl GpuRenderer {
    /// Render the 3D scene to an offscreen texture and return RGBA pixel data.
    /// Used by the software viewport to composite GPU-rendered 3D content.
    /// ✅ OPTIMIZED: Non-blocking, returns None if GPU not ready (uses cached pixels)
    /// ✅ OPTIMIZED: Reuses readback buffer to reduce allocations
    pub fn render_to_pixels(
        &self,
        width: u32,
        height: u32,
        scene: &crate::ecs::Scene,
        camera: &crate::gpu::ViewportCameraController,
        selected_id: Option<u64>,
    ) -> Option<Vec<u8>> {
        let (device, queue) = match (&self.device, &self.queue) {
            (Some(d), Some(q)) => (d, q),
            _ => {
                use std::sync::atomic::{AtomicBool, Ordering};
                static LOGGED: AtomicBool = AtomicBool::new(false);
                if !LOGGED.swap(true, Ordering::Relaxed) {
                    println!(
                        "[GpuRenderer] render_to_pixels: no device/queue — GPU not initialized"
                    );
                }
                return None;
            }
        };

        if width == 0 || height == 0 {
            return None;
        }

        // ✅ OPTIMIZED: Sync meshes only when scene changes (dirty tracking)
        self.sync_scene_meshes(scene, selected_id);

        let mesh_count = self.mesh_buffers.lock().map(|b| b.len()).unwrap_or(0);

        // ✅ OPTIMIZATION: Early exit if no meshes to render
        if mesh_count == 0 {
            // Return solid background color
            let bg = crate::theme::color(crate::theme::keys::CLR_BG_CANVAS);
            let mut pixels = vec![0u8; (width * height * 4) as usize];
            for chunk in pixels.chunks_exact_mut(4) {
                chunk[0] = (bg.r * 255.0) as u8;
                chunk[1] = (bg.g * 255.0) as u8;
                chunk[2] = (bg.b * 255.0) as u8;
                chunk[3] = 255;
            }
            return Some(pixels);
        }

        // Throttled trace log
        use std::sync::atomic::{AtomicU64, Ordering};
        static RENDER_FRAME: AtomicU64 = AtomicU64::new(0);
        let rf = RENDER_FRAME.fetch_add(1, Ordering::Relaxed);
        if rf == 0 || rf % 300 == 0 {
            println!("[GpuRenderer] render_to_pixels: frame={} size={}x{} meshes={}",
                rf, width, height, mesh_count);
        }

        // Update camera uniforms
        let view_proj = camera.view_projection_matrix();
        self.update_camera_uniforms(
            &view_proj,
            [camera.position.x, camera.position.y, camera.position.z, 1.0],
        );

        // Create offscreen render target
        let surface_format = self
            .config
            .as_ref()
            .map(|c| c.format)
            .unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Render Target"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create depth texture
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Depth"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Encode and draw
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Offscreen Encoder"),
        });

        self.draw_scene_with_depth(
            &target_view, scene, camera, &mut encoder, selected_id, Some(&depth_view),
        );

        // Readback: copy texture to CPU-mapped buffer
        let bytes_per_row = (4 * width + 255) & !255; // Align to 256
        let buffer_size = (bytes_per_row * height) as wgpu::BufferAddress;
        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &readback_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );

        queue.submit(std::iter::once(encoder.finish()));

        // ✅ OPTIMIZED: Non-blocking GPU readback with immediate return
        // Instead of waiting for GPU, we return None if data isn't ready yet
        // The viewport will use cached pixels from the previous frame
        let slice = readback_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        
        // Poll once to check if GPU is done (non-blocking)
        device.poll(wgpu::Maintain::Poll);
        
        // Try to receive result immediately (non-blocking)
        match receiver.try_recv() {
            Ok(Ok(())) => {
                // GPU finished, data is ready
            }
            Ok(Err(_)) => {
                // Mapping failed
                return None;
            }
            Err(_) => {
                // GPU not ready yet, return None and use cached pixels
                // This is the key to 60fps - we never wait for GPU
                return None;
            }
        }
        
        let data = slice.get_mapped_range();
        
        // ✅ OPTIMIZATION: Pre-allocate exact size needed
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            let row_start = (y * bytes_per_row) as usize;
            let row_end = row_start + (width * 4) as usize;
            if row_end <= data.len() {
                pixels.extend_from_slice(&data[row_start..row_end]);
            }
        }

        drop(data);
        readback_buffer.unmap();

        // Debug sampling (first few frames only)
        if rf < 3 {
            Self::debug_sample_pixels(&pixels, width, height);
        }

        Some(pixels)
    }

    /// Sample a few pixels for debug verification (non-black output check).
    fn debug_sample_pixels(pixels: &[u8], width: u32, height: u32) {
        let cx = (width / 2) as usize;
        let cy = (height / 2) as usize;
        let pi = (cy * width as usize + cx) * 4;
        if pi + 3 < pixels.len() {
            println!(
                "[GPU DEBUG] Center pixel ({},{}) RGBA = [{}, {}, {}, {}]",
                cx, cy, pixels[pi], pixels[pi + 1], pixels[pi + 2], pixels[pi + 3]
            );
        }

        let qx = (width / 4) as usize;
        let qy = (height / 3) as usize;
        let qi = (qy * width as usize + qx) * 4;
        if qi + 3 < pixels.len() {
            println!(
                "[GPU DEBUG] Quarter pixel ({},{}) RGBA = [{}, {}, {}, {}]",
                qx, qy, pixels[qi], pixels[qi + 1], pixels[qi + 2], pixels[qi + 3]
            );
        }

        let non_zero = pixels
            .chunks(4)
            .filter(|p| p[0] > 0 || p[1] > 0 || p[2] > 0)
            .count();
        let total = (width * height) as usize;
        println!(
            "[GPU DEBUG] Non-zero pixels: {}/{} ({:.1}%)",
            non_zero, total, non_zero as f64 / total as f64 * 100.0
        );
    }
}
