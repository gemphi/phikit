/*
**
** Backdrops
**
*/

cbuffer BackdropParams: register(b1) {
    float4 backdrop_rect;
    float2 backdrop_target_size;
    float2 backdrop_pixel_to_uv;
    float4 backdrop_uv_limit;
    float2 backdrop_step;
    float backdrop_sigma;
    uint backdrop_pass;
    float4 backdrop_bounds;
    float4 backdrop_radii;
    float4 backdrop_tint;
    float4 backdrop_edge;
    float backdrop_saturation;
    float backdrop_lens;
    float backdrop_opacity;
    float backdrop_rim_band;
};

Texture2D<float4> t_backdrop: register(t0);
SamplerState s_backdrop: register(s0);

float4 backdrop_vertex(uint vertex_id: SV_VertexID): SV_Position {
    float2 unit_vertex = float2(float(vertex_id & 1u), 0.5 * float(vertex_id & 2u));
    float2 position = backdrop_rect.xy + unit_vertex * backdrop_rect.zw;
    float2 device_position = position / backdrop_target_size * 2.0 - 1.0;
    return float4(device_position.x, -device_position.y, 0.0, 1.0);
}

// Held at the edges of what was captured.
float4 backdrop_read(float2 uv) {
    return t_backdrop.SampleLevel(s_backdrop, clamp(uv, backdrop_uv_limit.xy, backdrop_uv_limit.zw), 0.0);
}

float backdrop_distance(float2 position) {
    float2 half_size = backdrop_bounds.zw * 0.5;
    float2 from_centre = position - (backdrop_bounds.xy + half_size);
    float radius = from_centre.x < 0.0
        ? (from_centre.y < 0.0 ? backdrop_radii.x : backdrop_radii.w)
        : (from_centre.y < 0.0 ? backdrop_radii.y : backdrop_radii.z);
    float2 corner = abs(from_centre) - half_size + radius;
    return length(max(corner, 0.0)) + min(max(corner.x, corner.y), 0.0) - radius;
}

float4 backdrop_shrink(float2 position) {
    float2 uv = position * backdrop_pixel_to_uv;
    float2 s = backdrop_step;
    return 0.25 * (backdrop_read(uv + float2(-s.x, -s.y)) + backdrop_read(uv + float2(s.x, -s.y))
        + backdrop_read(uv + float2(-s.x, s.y)) + backdrop_read(uv + float2(s.x, s.y)));
}

float4 backdrop_blur(float2 position) {
    float2 uv = position * backdrop_pixel_to_uv;
    int reach = min(int(ceil(backdrop_sigma * 3.0)), 32);
    float4 sum = float4(0.0, 0.0, 0.0, 0.0);
    float total = 0.0;
    [loop]
    for (int i = -reach; i <= reach; i++) {
        float weight = exp(-float(i * i) / (2.0 * backdrop_sigma * backdrop_sigma + 0.0001));
        sum += backdrop_read(uv + backdrop_step * float(i)) * weight;
        total += weight;
    }
    return sum / total;
}

float4 backdrop_glass(float2 position) {
    float d = backdrop_distance(position);
    float cover = saturate((0.75 - d) / 1.5);
    float band = saturate(1.0 + d / backdrop_rim_band);
    float2 gradient = float2(
        backdrop_distance(position + float2(1.0, 0.0)) - backdrop_distance(position - float2(1.0, 0.0)),
        backdrop_distance(position + float2(0.0, 1.0)) - backdrop_distance(position - float2(0.0, 1.0)));
    float gradient_length = length(gradient);
    float2 normal = gradient_length > 0.0001 ? gradient / gradient_length : float2(0.0, 0.0);

    // The lens: near the rim, read from further inside.
    float pull = pow(band, 2.5) * backdrop_lens;
    float3 color = backdrop_read((position - normal * pull) * backdrop_pixel_to_uv).rgb;
    // Saturation, about the pixel's own luminance.
    float luma = dot(color, float3(0.2126, 0.7152, 0.0722));
    color = lerp(float3(luma, luma, luma), color, backdrop_saturation);
    // Tint. Clear over what is near the tint's own lightness, and thickening
    // over what stands far from it, bright under dark glass or dark under light
    // glass, so the glass stays see-through and the text on it stays readable.
    // A glass with no tint at all stays untinted.
    float tint_luma = dot(backdrop_tint.rgb, float3(0.2126, 0.7152, 0.0722));
    float thick = backdrop_tint.a > 0.0 ? max(backdrop_tint.a, 0.88 * smoothstep(0.2, 0.6, abs(luma - tint_luma))) : 0.0;
    color = lerp(color, backdrop_tint.rgb, thick);
    // Edge light: a rim that faces the light, and a soft glow inside it.
    float rim = saturate(1.0 - abs(d + 1.0) / 2.2);
    float facing = max(dot(normal, float2(-0.507, -0.862)), 0.0);
    float light = rim * (0.2 + 0.8 * facing * facing) + pow(band, 4.0) * 0.07;
    color += backdrop_edge.rgb * light * backdrop_edge.a;
    return float4(saturate(color), cover * backdrop_opacity);
}

float4 backdrop_fragment(float4 position: SV_Position): SV_Target {
    switch (backdrop_pass) {
        case 0: return backdrop_shrink(position.xy);
        case 1: return backdrop_blur(position.xy);
        default: return backdrop_glass(position.xy);
    }
}
