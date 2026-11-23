// Bounding box wireframe shader (green lines)

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model_transform: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let world_pos = model_transform * vec4<f32>(in.position, 1.0);
    let clip_pos = camera.view_proj * world_pos;
    
    var out: VertexOutput;
    out.clip_position = clip_pos;
    // Bright green for bounding box edges
    out.color = vec4<f32>(0.0, 1.0, 0.0, 1.0);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
