# Shaders — GPU Shader Programs

## Overview

`shaders` module contains shader programs written in WGSL (WebGPU Shading Language). Compiled at runtime by wgpu for GPU execution.

## Shader Types

| Type | Stage | Purpose |
|------|-------|---------|
| **Vertex Shader** | Vertex | Transform geometry, compute positions |
| **Fragment Shader** | Fragment | Color pixels, lighting, materials |
| **Compute Shader** | Compute | General GPU computation |

## WGSL Basics

```wgsl
// Vertex shader
@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 1.0);
}

// Fragment shader
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Red
}
```

## Shader Files

| File | Purpose |
|------|---------|
| `mesh.wgsl` | Standard mesh rendering |
| `wireframe.wgsl` | Wireframe overlay |
| `ui.wgsl` | UI element rendering |
| `text.wgsl` | SDF text rendering |
| `skybox.wgsl` | Environment background |
| `grid.wgsl` | Editor grid floor |

## Connection to Other Modules

```
Shaders
├── Loaded by: GPU module (pipeline creation)
├── Used by: 3D viewport (mesh rendering)
├── Used by: UI (accelerated text)
└── Edited by: Shader graph view
```

## Usage Examples

### Mesh Shader
```wgsl
// mesh.wgsl
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> transform: mat4x4<f32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = transform * vec4<f32>(input.position, 1.0);
    output.world_normal = input.normal;
    output.uv = input.uv;
    return output;
}

@group(0) @binding(1)
var diffuse_texture: texture_2d<f32>;
@group(0) @binding(2)
var diffuse_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(diffuse_texture, diffuse_sampler, input.uv);
}
```

### Load Shader
```rust
use rust_appkit::gpu::*;

let shader_code = include_str!("shaders/mesh.wgsl");
let shader = device.create_shader_module(&ShaderModuleDescriptor {
    label: Some("mesh_shader"),
    source: ShaderSource::Wgsl(shader_code),
});
```

## Shader Pipeline

```
WGSL Source Code
        │
        ▼
wgpu Shader Module
        │
        ▼
Compiled to GPU bytecode
        │
        ▼
Linked into RenderPipeline
        │
        ▼
Executed on GPU per-vertex/per-pixel
```

## Testing & Verification

```rust
// Shader should compile without errors
let shader = device.create_shader_module(&ShaderModuleDescriptor {
    label: Some("test"),
    source: ShaderSource::Wgsl(r#"
        @vertex
        fn vs_main() -> @builtin(position) vec4<f32> {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
        
        @fragment
        fn fs_main() -> @location(0) vec4<f32> {
            return vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }
    "#),
});

// Compilation happens lazily, but we can validate
```

## File Structure

```
src/shaders/
├── mesh.wgsl       # Standard PBR mesh shader
├── wireframe.wgsl  # Line rendering
├── ui.wgsl         # 2D UI elements
├── text.wgsl       # Signed distance field text
├── skybox.wgsl     # Cubemap background
├── grid.wgsl       # Infinite floor grid
└── README.md       # This file
```

## WGSL Reference

| Feature | Syntax |
|---------|--------|
| Vector | `vec3<f32>` |
| Matrix | `mat4x4<f32>` |
| Uniform | `@group(0) @binding(0) var<uniform>` |
| Texture | `@group(0) @binding(0) var tex: texture_2d<f32>` |
| Sampler | `@group(0) @binding(1) var samp: sampler` |

## Missing Shader Features

- **Tessellation**: Hull/domain shaders
- **Geometry shaders**: Generate geometry
- **Ray tracing**: RT shaders
- **Mesh shaders**: GPU mesh generation

These are stubbed for current use cases.

## Related

- **GPU module**: Loads and executes shaders
- **Shader graph view**: Visual shader editor
- **wgpu docs**: https://gpuweb.github.io/gpuweb/wgsl/
