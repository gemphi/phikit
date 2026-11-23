# GPU — Hardware Accelerated Rendering

## Overview

`gpu` module provides GPU-accelerated rendering using `wgpu`. Supports 3D viewport rendering, shaders, and high-performance graphics.

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 GPU Rendering Setup                          │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              wgpu Initialization                             │
│  ├─ Instance: GPU context                                     │
│  ├─ Adapter: Physical GPU selection                         │
│  ├─ Device: Logical GPU interface                           │
│  └─ Queue: Command submission                               │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Render Pipeline                              │
│  ├─ Surface: Window swap chain                                │
│  ├─ Pipeline: Shader + state configuration                  │
│  ├─ Bind Groups: Resources (textures, buffers)              │
│  ├─ Command Encoder: Record draw calls                        │
│  └─ Submit: Send to GPU queue                                 │
└─────────────────────────────────────────────────────────────┘
```

## GPU Components

| Component | Purpose |
|-----------|---------|
| `GPUInstance` | wgpu instance, entry point |
| `GPUAdapter` | Physical device selection |
| `GPUDevice` | Logical device handle |
| `GPUQueue` | Command submission |
| `GPUSurface` | Window output surface |
| `GPURenderPipeline` | Shader + fixed-function state |
| `GPUBuffer` | Vertex/index/uniform data |
| `GPUTexture` | Images, render targets |
| `GPUBindGroup` | Resource binding |

## Connection to Other Modules

```
GPU
├── Used by: 3D viewport (reality-kit)
├── Used by: Shader graph (shader_graph_view)
├── Shaders defined in: src/shaders/
└── Software fallback: RenderContext
```

## Usage Examples

### Initialize GPU
```rust
use rust_appkit::gpu::*;

async fn init_gpu(window: &Window) -> GPUContext {
    let instance = GPUInstance::new();
    let surface = instance.create_surface(window);
    let adapter = instance.request_adapter().await;
    let (device, queue) = adapter.request_device().await;
    
    GPUContext {
        instance,
        surface,
        device,
        queue,
    }
}
```

### Create Render Pipeline
```rust
let shader = device.create_shader_module(&ShaderModuleDescriptor {
    label: Some("main_shader"),
    source: ShaderSource::Wgsl(include_str!("../shaders/mesh.wgsl")),
});

let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
    label: Some("mesh_pipeline"),
    layout: Some(&pipeline_layout),
    vertex: VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[vertex_buffer_layout],
    },
    fragment: Some(FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(ColorTargetState {
            format: surface_format,
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL,
        })],
    }),
    primitive: PrimitiveState::default(),
    depth_stencil: Some(DepthStencilState::default()),
    multisample: MultisampleState::default(),
});
```

### Draw Frame
```rust
fn render(&self, view: &mut View3D) {
    let output = self.surface.get_current_texture();
    let view = output.texture.create_view(&TextureViewDescriptor::default());
    
    let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor::default());
    
    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(...),
        });
        
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
    
    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}
```

## Render Pipeline Architecture

```
┌─────────────────────────────────────────┐
│         3D Viewport                     │
│  ┌─────────────────────────────────┐   │
│  │  wgpu Surface (swap chain)      │   │
│  │  ┌───────────────────────────┐ │   │
│  │  │ Render Pass               │ │   │
│  │  │ ┌─────────────────────┐ │ │   │
│  │  │ │ Pipeline            │ │ │   │
│  │  │ │ ┌───┐ ┌───┐ ┌─────┐ │ │ │   │
│  │  │ │ │VS │→│FS │→│Out  │ │ │ │   │
│  │  │ │ └───┘ └───┘ └─────┘ │ │ │   │
│  │  │ └─────────────────────┘ │ │   │
│  │  └───────────────────────────┘ │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

## Testing & Verification

```rust
let instance = GPUInstance::new();
let adapter = instance.request_adapter().await;

// Check features
assert!(adapter.features().contains(Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES));

// Check limits
assert!(adapter.limits().max_texture_dimension_2d >= 4096);
```

## File Structure

```
src/gpu/
├── mod.rs          # GPUInstance, GPUDevice, GPUQueue
├── adapter.rs      # GPUAdapter, feature detection
├── surface.rs      # GPUSurface, swap chain management
├── pipeline.rs     # GPURenderPipeline, shader compilation
├── buffer.rs       # GPUBuffer, vertex/index/uniform data
├── texture.rs      # GPUTexture, GPUTextureView
└── bind_group.rs   # GPUBindGroup, resource binding
```

## Missing from GPU

- **Compute shaders**: GPGPU operations
- **Ray tracing**: RT pipelines
- **Mesh shaders**: GPU-driven rendering
- **Variable rate shading**: Performance optimization
- **Multi-GPU**: SLI/CrossFire

These are stubbed for current use cases.

## Related

- **src/shaders/**: WGSL shader source files
- **reality-kit**: Uses GPU for 3D rendering
- **shader_graph_view**: Visual shader editor
