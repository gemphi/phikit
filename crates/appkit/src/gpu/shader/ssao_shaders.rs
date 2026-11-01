//! SSAO Shader Library
//!
//! WGSL shaders for deferred rendering pipeline

/// G-Buffer pass - vertex shader
pub const GBUFFER_VERT: &str = r#"
struct Camera {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
};

struct Transform {
    model: mat4x4<f32>,
    normal: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> transform: Transform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = transform.model * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_pos;
    out.world_position = world_pos.xyz;
    out.world_normal = (transform.normal * vec4<f32>(in.normal, 0.0)).xyz;
    out.uv = in.uv;
    out.color = in.color;
    return out;
}
"#;

/// G-Buffer pass - fragment shader (outputs color + normal + linear depth)
pub const GBUFFER_FRAG: &str = r#"
struct Camera {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct FragmentInput {
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct GBufferOutput {
    @location(0) color: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) linear_depth: f32,
};

@fragment
fn fs_main(in: FragmentInput) -> GBufferOutput {
    var out: GBufferOutput;
    
    // Base color
    out.color = in.color;
    
    // World-space normal (encode to RGBA16Float range)
    out.normal = vec4<f32>(normalize(in.world_normal), 1.0);
    
    // Linear depth (distance from camera)
    let view_vec = in.world_position - camera.position;
    out.linear_depth = length(view_vec);
    
    return out;
}
"#;

/// SSAO computation shader
pub const SSAO_COMPUTE: &str = r#"
struct SsaoParams {
    radius: f32,
    bias: f32,
    intensity: f32,
    scale: f32,
    sample_count: u32,
    noise_size: u32,
    screen_width: u32,
    screen_height: u32,
};

struct Camera {
    view_proj: mat4x4<f32>,
    projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
};

@group(0) @binding(0)
var<uniform> params: SsaoParams;

@group(0) @binding(1)
var<uniform> camera: Camera;

@group(1) @binding(0)
var linear_depth_tex: texture_2d<f32>;

@group(1) @binding(1)
var normal_tex: texture_2d<f32>;

@group(1) @binding(2)
var noise_tex: texture_2d<f32>;

@group(1) @binding(3)
var<uniform> samples: array<vec4<f32>, 64>;

@group(1) @binding(4)
var nearest_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Full-screen triangle
    var out: VertexOutput;
    let x = f32(vertex_index % 2u) * 2.0 - 1.0; // 0 -> -1, 1 -> 1
    let y = f32(vertex_index / 2u) * 2.0 - 1.0; // 0 -> -1, 1 -> 1
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    return out;
}

// Reconstruct view-space position from depth
fn position_from_depth(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let clip_xy = vec2<f32>(uv.x * 2.0 - 1.0, (1.0 - uv.y) * 2.0 - 1.0);
    let clip_pos = vec4<f32>(clip_xy, depth, 1.0);
    let view_pos = camera.inverse_projection * clip_pos;
    return view_pos.xyz / view_pos.w;
}

@fragment
fn fs_ssao(in: VertexOutput) -> @location(0) f32 {
    let tex_size = vec2<f32>(f32(params.screen_width), f32(params.screen_height));
    let uv = in.uv;
    
    // Sample linear depth and normal
    let depth = textureSample(linear_depth_tex, nearest_sampler, uv).r;
    let normal = normalize(textureSample(normal_tex, nearest_sampler, uv).xyz);
    
    // Reconstruct view-space position
    let frag_pos = position_from_depth(uv, depth);
    
    // Sample random vector from noise texture
    let noise_uv = uv * tex_size / f32(params.noise_size);
    let random_vec = normalize(textureSample(noise_tex, nearest_sampler, noise_uv).xyz);
    
    // Create TBN matrix
    let tangent = normalize(random_vec - normal * dot(random_vec, normal));
    let bitangent = cross(normal, tangent);
    let tbn = mat3x3<f32>(tangent, bitangent, normal);
    
    // SSAO sampling
    var occlusion = 0.0;
    let sample_count = i32(params.sample_count);
    
    for (var i = 0; i < sample_count; i++) {
        // Get sample position in view space
        let sample_offset = samples[i].xyz;
        let sample_pos = frag_pos + (tbn * sample_offset) * params.radius;
        
        // Project to clip space
        let offset = camera.projection * vec4<f32>(sample_pos, 1.0);
        let offset_xy = offset.xy / offset.w * 0.5 + 0.5;
        let sample_uv = vec2<f32>(offset_xy.x, 1.0 - offset_xy.y);
        
        // Sample depth at offset position
        let sample_depth = textureSample(linear_depth_tex, nearest_sampler, sample_uv).r;
        
        // Range check
        let range_check = smoothstep(0.0, 1.0, params.radius / abs(frag_pos.z - sample_depth));
        
        // Occlusion test
        let bias = params.bias;
        if (sample_depth < sample_pos.z - bias) {
            occlusion += range_check;
        }
    }
    
    occlusion = 1.0 - (occlusion / f32(sample_count));
    return pow(occlusion, params.intensity);
}
"#;

/// Simple blur shader for SSAO noise reduction
pub const SSAO_BLUR: &str = r#"
@group(0) @binding(0)
var ssao_tex: texture_2d<f32>;

@group(0) @binding(1)
var nearest_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(vertex_index % 2u) * 2.0 - 1.0;
    let y = f32(vertex_index / 2u) * 2.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    return out;
}

@fragment
fn fs_blur(in: VertexOutput) -> @location(0) f32 {
    let texel_size = 1.0 / vec2<f32>(textureDimensions(ssao_tex));
    var result = 0.0;
    
    // 4x4 box blur
    for (var x = -2; x < 2; x++) {
        for (var y = -2; y < 2; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            result += textureSample(ssao_tex, nearest_sampler, in.uv + offset).r;
        }
    }
    
    return result / 16.0;
}
"#;

/// Final composition shader (combines color + SSAO)
pub const COMPOSITE: &str = r#"
@group(0) @binding(0)
var color_tex: texture_2d<f32>;

@group(0) @binding(1)
var ssao_tex: texture_2d<f32>;

@group(0) @binding(2)
var depth_tex: texture_2d<f32>;

@group(0) @binding(3)
var linear_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(vertex_index % 2u) * 2.0 - 1.0;
    let y = f32(vertex_index / 2u) * 2.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    return out;
}

@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(color_tex, linear_sampler, in.uv);
    let ao = textureSample(ssao_tex, linear_sampler, in.uv).r;
    
    // Apply ambient occlusion
    // Mix between full brightness (1.0) and AO based on a user factor
    let ao_factor = 0.3; // How much AO affects the scene
    let final_ao = mix(1.0, ao, ao_factor);
    
    return vec4<f32>(color.rgb * final_ao, color.a);
}
"#;

/// Wireframe shader (for debug/editing view)
pub const WIREFRAME: &str = r#"
struct Camera {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
};

struct Transform {
    model: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> transform: Transform;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = transform.model * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_pos;
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.8, 0.8, 0.8, 1.0); // Light gray wireframe
}
"#;
