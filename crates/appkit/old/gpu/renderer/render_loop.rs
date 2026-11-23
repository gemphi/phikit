//! GPU Renderer — initialization and pipeline setup.

use super::GpuRenderer;
use std::sync::Arc;

impl GpuRenderer {
    /// Initialize WGPU surface and device
    pub async fn initialize(&mut self, window: Arc<winit::window::Window>) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::empty(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: Some("Device"),
                },
                None,
            )
            .await
            .unwrap();

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        self.surface = Some(surface);
        self.device = Some(device.clone());
        self.queue = Some(queue);
        self.config = Some(config.clone());

        self.setup_pipelines(&device, config.format);
        self.create_depth_texture(size.width, size.height);
        self.create_grid_buffer();
    }

    /// Initialize GPU device/queue WITHOUT a window surface.
    ///
    /// This is the primary init path for `rust-kit-pro` which uses its own
    /// AppKit-based windowing and composites GPU output via `render_to_pixels`.
    /// Falls back gracefully if no GPU adapter is available on the system.
    pub fn initialize_headless(&mut self) -> Result<(), String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::empty(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        // Request adapter without a surface (headless / offscreen rendering)
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }));

        let adapter = match adapter {
            Some(a) => a,
            None => {
                // Try again with fallback adapter
                let fallback =
                    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::LowPower,
                        compatible_surface: None,
                        force_fallback_adapter: true,
                    }));
                match fallback {
                    Some(a) => a,
                    None => {
                        return Err("No GPU adapter available (tried hardware + fallback)".into())
                    }
                }
            }
        };

        let adapter_info = adapter.get_info();
        println!(
            "[GpuRenderer] Adapter: {} ({:?})",
            adapter_info.name, adapter_info.backend
        );

        let (device, queue) = match pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                label: Some("HeadlessDevice"),
            },
            None,
        )) {
            Ok(pair) => pair,
            Err(e) => return Err(format!("Failed to create device: {}", e)),
        };

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // Use Rgba8Unorm (linear) for offscreen render targets (render_to_pixels).
        // The shader applies a manual linear→sRGB conversion in the fragment output
        // so that blitted pixels look correct on the software surface.
        let offscreen_format = wgpu::TextureFormat::Rgba8Unorm;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: offscreen_format,
            width: 960,
            height: 570,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        self.device = Some(device.clone());
        self.queue = Some(queue);
        self.config = Some(config);
        // No surface needed for offscreen rendering

        self.setup_pipelines(&device, offscreen_format);
        self.create_depth_texture(960, 570);
        self.create_grid_buffer();

        println!(
            "[GpuRenderer] Headless GPU initialized OK — device={} format={:?}",
            adapter_info.name, offscreen_format
        );
        Ok(())
    }


}
