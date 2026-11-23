// Glass for wgpu and the web: the stages of `glass.hlsl`, body for body, over
// the same block (`GlassParam` in `src/plan.rs`). One draw runs one stage:
// shrink the copy, blur it one way, or draw the shape from the result.

struct GlassParam {
    rect: vec4<f32>,
    target_size: vec2<f32>,
    pixel_to_uv: vec2<f32>,
    uv_limit: vec4<f32>,
    step: vec2<f32>,
    sigma: f32,
    stage: u32,
    bounds: vec4<f32>,
    radii: vec4<f32>,
    tint: vec4<f32>,
    edge: vec4<f32>,
    saturation: f32,
    lens: f32,
    opacity: f32,
    rim_band: f32,
}

@group(0) @binding(0) var<uniform> glass: GlassParam;
@group(0) @binding(1) var t_backdrop: texture_2d<f32>;
@group(0) @binding(2) var s_backdrop: sampler;

@vertex
fn glass_vertex(@builtin(vertex_index) vertex_id: u32) -> @builtin(position) vec4<f32> {
    let unit_vertex = vec2<f32>(f32(vertex_id & 1u), 0.5 * f32(vertex_id & 2u));
    let position = glass.rect.xy + unit_vertex * glass.rect.zw;
    let device_position = position / glass.target_size * 2.0 - 1.0;
    return vec4<f32>(device_position.x, -device_position.y, 0.0, 1.0);
}

// Held at the edges of what was captured.
fn glass_read(uv: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(t_backdrop, s_backdrop, clamp(uv, glass.uv_limit.xy, glass.uv_limit.zw), 0.0);
}

fn glass_distance(position: vec2<f32>) -> f32 {
    let half_size = glass.bounds.zw * 0.5;
    let from_centre = position - (glass.bounds.xy + half_size);
    let top = from_centre.y < 0.0;
    let on_left = select(glass.radii.w, glass.radii.x, top);
    let on_right = select(glass.radii.z, glass.radii.y, top);
    let radius = select(on_right, on_left, from_centre.x < 0.0);
    let corner = abs(from_centre) - half_size + radius;
    return length(max(corner, vec2<f32>(0.0))) + min(max(corner.x, corner.y), 0.0) - radius;
}

fn glass_shrink(position: vec2<f32>) -> vec4<f32> {
    let uv = position * glass.pixel_to_uv;
    let s = glass.step;
    return 0.25 * (glass_read(uv + vec2<f32>(-s.x, -s.y)) + glass_read(uv + vec2<f32>(s.x, -s.y))
        + glass_read(uv + vec2<f32>(-s.x, s.y)) + glass_read(uv + vec2<f32>(s.x, s.y)));
}

fn glass_blur(position: vec2<f32>) -> vec4<f32> {
    let uv = position * glass.pixel_to_uv;
    let reach = min(i32(ceil(glass.sigma * 3.0)), 32);
    var sum = vec4<f32>(0.0);
    var total = 0.0;
    for (var i = -reach; i <= reach; i++) {
        let weight = exp(-f32(i * i) / (2.0 * glass.sigma * glass.sigma + 0.0001));
        sum += glass_read(uv + glass.step * f32(i)) * weight;
        total += weight;
    }
    return sum / total;
}

fn glass_shape(position: vec2<f32>) -> vec4<f32> {
    let d = glass_distance(position);
    let cover = clamp((0.75 - d) / 1.5, 0.0, 1.0);
    let band = clamp(1.0 + d / glass.rim_band, 0.0, 1.0);
    let gradient = vec2<f32>(
        glass_distance(position + vec2<f32>(1.0, 0.0)) - glass_distance(position - vec2<f32>(1.0, 0.0)),
        glass_distance(position + vec2<f32>(0.0, 1.0)) - glass_distance(position - vec2<f32>(0.0, 1.0)));
    let gradient_length = length(gradient);
    let normal = select(vec2<f32>(0.0), gradient / max(gradient_length, 0.0001), gradient_length > 0.0001);

    // The lens: near the rim, read from further inside.
    let pull = pow(band, 2.5) * glass.lens;
    var color = glass_read((position - normal * pull) * glass.pixel_to_uv).rgb;
    // Saturation, about the pixel's own luminance.
    let luma = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
    color = mix(vec3<f32>(luma), color, glass.saturation);
    // Tint. Clear over what is near the tint's own lightness, and thickening
    // over what stands far from it, bright under dark glass or dark under light
    // glass, so the glass stays see-through and the text on it stays readable.
    // A glass with no tint at all stays untinted.
    let tint_luma = dot(glass.tint.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let thickened = max(glass.tint.a, 0.88 * smoothstep(0.2, 0.6, abs(luma - tint_luma)));
    let thick = select(0.0, thickened, glass.tint.a > 0.0);
    color = mix(color, glass.tint.rgb, thick);
    // Edge light: a rim that faces the light, and a soft glow inside it.
    let rim = clamp(1.0 - abs(d + 1.0) / 2.2, 0.0, 1.0);
    let facing = max(dot(normal, vec2<f32>(-0.507, -0.862)), 0.0);
    let light = rim * (0.2 + 0.8 * facing * facing) + pow(band, 4.0) * 0.07;
    color += glass.edge.rgb * light * glass.edge.a;
    return vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), cover * glass.opacity);
}

@fragment
fn glass_fragment(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    switch glass.stage {
        case 0u: { return glass_shrink(position.xy); }
        case 1u: { return glass_blur(position.xy); }
        default: { return glass_shape(position.xy); }
    }
}
