// PBR Mesh Shader (Disney-style BRDF with metallic material support)

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_position: vec4<f32>,
};

struct MaterialUniform {
    model_matrix: mat4x4<f32>,
    base_color: vec4<f32>,
    roughness: f32,
    metallic: f32,
    reflectance: f32,
    is_selected: f32,
    env_intensity: f32,
    anisotropy: f32,
    _padding0: f32,
    _padding1: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> material: MaterialUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Apply model matrix to vertex position
    let world_pos = material.model_matrix * vec4<f32>(in.position, 1.0);
    out.world_position = world_pos.xyz;
    out.clip_position = camera.view_proj * world_pos;
    
    // Apply model matrix rotation/scale to normal
    let normal_matrix = mat3x3<f32>(
        material.model_matrix[0].xyz,
        material.model_matrix[1].xyz,
        material.model_matrix[2].xyz
    );
    let raw_normal = normal_matrix * in.normal;
    let normal_len = length(raw_normal);
    if (normal_len > 0.001) {
        out.normal = raw_normal / normal_len;
    } else {
        out.normal = vec3<f32>(0.0, 1.0, 0.0);
    }
    out.color = in.color;
    
    return out;
}

const PI: f32 = 3.14159265359;

fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;
    let nom = a2;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    return nom / (PI * denom * denom + 0.0001);
}

/// Anisotropic GGX distribution for brushed metal appearance.
/// Stretches the specular highlight along the tangent direction.
fn distribution_ggx_anisotropic(
    N: vec3<f32>, H: vec3<f32>, T: vec3<f32>, B: vec3<f32>,
    roughness: f32, anisotropy: f32
) -> f32 {
    // Compute anisotropic roughness along tangent and bitangent
    let aspect = sqrt(1.0 - 0.9 * anisotropy);
    let ax = roughness * roughness / aspect;
    let ay = roughness * roughness * aspect;

    let NdotH = max(dot(N, H), 0.0);
    let TdotH = dot(T, H);
    let BdotH = dot(B, H);

    let d = (TdotH * TdotH) / (ax * ax) + (BdotH * BdotH) / (ay * ay) + NdotH * NdotH;
    return 1.0 / (PI * ax * ay * d * d + 0.0001);
}

fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;
    let nom = NdotV;
    let denom = NdotV * (1.0 - k) + k;
    return nom / (denom + 0.0001);
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = geometry_schlick_ggx(NdotV, roughness);
    let ggx1 = geometry_schlick_ggx(NdotL, roughness);
    return ggx1 * ggx2;
}

fn fresnel_schlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

fn fresnel_schlick_roughness(cosTheta: f32, F0: vec3<f32>, roughness: f32) -> vec3<f32> {
    let max_reflect = max(vec3<f32>(1.0 - roughness), F0);
    return F0 + (max_reflect - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

/// Compute a tangent vector from the normal (for anisotropy when no explicit tangent is available)
fn compute_tangent(N: vec3<f32>) -> vec3<f32> {
    // Choose a vector not parallel to N
    var up = vec3<f32>(0.0, 1.0, 0.0);
    if (abs(dot(N, up)) > 0.99) {
        up = vec3<f32>(1.0, 0.0, 0.0);
    }
    return normalize(cross(N, up));
}

/// Sample environment reflection (approximation without cubemap — uses analytical sky model)
fn sample_environment_reflection(R: vec3<f32>, roughness: f32) -> vec3<f32> {
    // Approximate sky/environment: blend between ground color and sky color based on reflection direction
    let sky_color = vec3<f32>(0.4, 0.6, 0.9);    // Blue sky
    let horizon_color = vec3<f32>(0.7, 0.75, 0.8); // Hazy horizon
    let ground_color = vec3<f32>(0.2, 0.2, 0.2);   // Dark ground

    // Vertical blend factor based on reflection Y component
    let y = R.y;
    var env_color: vec3<f32>;
    if (y > 0.0) {
        // Above horizon: blend from horizon to sky
        env_color = mix(horizon_color, sky_color, clamp(y, 0.0, 1.0));
    } else {
        // Below horizon: blend from horizon to ground
        env_color = mix(horizon_color, ground_color, clamp(-y, 0.0, 1.0));
    }

    // Rougher surfaces see a more blurred (averaged) environment
    let mip_blend = clamp(roughness * roughness, 0.0, 1.0);
    let avg_env = (sky_color + horizon_color + ground_color) / 3.0;
    env_color = mix(env_color, avg_env, mip_blend);

    return env_color;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Normal safety
    var N = normalize(in.normal);
    if (length(in.normal) < 0.001) {
        N = vec3<f32>(0.0, 1.0, 0.0);
    }
    
    // Use vertex color directly as base color (OBJ loader bakes tint into vertex colors)
    var base_color = in.color.rgb;
    
    // Fallback: if vertex color is black, use material uniform
    let vert_sum = base_color.r + base_color.g + base_color.b;
    if (vert_sum < 0.01) {
        base_color = material.base_color.rgb;
    }
    
    // Final safety net — never output pure black
    if (base_color.r + base_color.g + base_color.b < 0.01) {
        base_color = vec3<f32>(0.6, 0.6, 0.6);
    }
    
    // PBR Material parameters — clamp all to valid ranges to prevent NaN
    let roughness = clamp(material.roughness, 0.04, 1.0);
    let metallic = clamp(material.metallic, 0.0, 1.0);
    let reflectance = clamp(material.reflectance, 0.0, 1.0);
    let env_intensity = clamp(material.env_intensity, 0.0, 1.0);
    let anisotropy = clamp(material.anisotropy, 0.0, 1.0);
    
    // View direction
    let V = normalize(camera.camera_position.xyz - in.world_position);
    let NdotV = max(dot(N, V), 0.001);
    
    // Calculate F0 (specular reflectance at normal incidence)
    // Non-metallic: F0 is reflectance * 0.16, metallic: F0 is base_color
    let dielectric_f0 = 0.16 * reflectance * reflectance;
    let F0 = mix(vec3<f32>(dielectric_f0), base_color, metallic);
    
    // Compute tangent and bitangent for anisotropy
    let T = compute_tangent(N);
    let B = cross(N, T);
    
    // Multiple light sources for studio lighting
    let key_light = normalize(vec3<f32>(0.7, 0.8, 0.4));
    let fill_light = normalize(vec3<f32>(-0.4, 0.5, -0.6));
    let rim_light = normalize(vec3<f32>(-0.3, 0.2, 0.8));
    
    let key_color = vec3<f32>(1.0, 0.98, 0.95);   // Warm key
    let fill_color = vec3<f32>(0.85, 0.9, 1.0);  // Cool fill
    let rim_color = vec3<f32>(1.0, 1.0, 1.0);    // White rim
    
    // Ambient term (IBL approximation)
    let ambient = base_color * 0.08;
    
    var total_diffuse = vec3<f32>(0.0);
    var total_specular = vec3<f32>(0.0);
    
    // Process each light with full PBR (unrolled for WGSL compatibility)
    // Key light
    {
        let L = key_light;
        let light_color = key_color;
        let intensity = 1.2;
        
        let NdotL = max(dot(N, L), 0.0);
        
        if (NdotL > 0.0) {
            let H = normalize(V + L);
            
            // Choose isotropic or anisotropic distribution based on anisotropy parameter
            var D: f32;
            if (anisotropy > 0.001) {
                D = distribution_ggx_anisotropic(N, H, T, B, roughness, anisotropy);
            } else {
                D = distribution_ggx(N, H, roughness);
            }
            let G = geometry_smith(N, V, L, roughness);
            let F = fresnel_schlick(max(dot(H, V), 0.0), F0);
            
            let numerator = D * G * F;
            let denominator = 4.0 * NdotV * NdotL + 0.001;
            let specular = numerator / denominator;
            
            // Energy conservation: for metallic == 1.0, diffuse contribution is zero
            let kD = (vec3<f32>(1.0) - F) * (1.0 - metallic);
            let diffuse = kD * base_color / PI;
            
            total_diffuse += diffuse * light_color * NdotL * intensity;
            total_specular += specular * light_color * NdotL * intensity;
        }
    }
    
    // Fill light
    {
        let L = fill_light;
        let light_color = fill_color;
        let intensity = 0.6;
        
        let NdotL = max(dot(N, L), 0.0);
        
        if (NdotL > 0.0) {
            let H = normalize(V + L);
            
            var D: f32;
            if (anisotropy > 0.001) {
                D = distribution_ggx_anisotropic(N, H, T, B, roughness, anisotropy);
            } else {
                D = distribution_ggx(N, H, roughness);
            }
            let G = geometry_smith(N, V, L, roughness);
            let F = fresnel_schlick(max(dot(H, V), 0.0), F0);
            
            let numerator = D * G * F;
            let denominator = 4.0 * NdotV * NdotL + 0.001;
            let specular = numerator / denominator;
            
            let kD = (vec3<f32>(1.0) - F) * (1.0 - metallic);
            let diffuse = kD * base_color / PI;
            
            total_diffuse += diffuse * light_color * NdotL * intensity;
            total_specular += specular * light_color * NdotL * intensity;
        }
    }
    
    // Rim light
    {
        let L = rim_light;
        let light_color = rim_color;
        let intensity = 0.4;
        
        let NdotL = max(dot(N, L), 0.0);
        
        if (NdotL > 0.0) {
            let H = normalize(V + L);
            
            var D: f32;
            if (anisotropy > 0.001) {
                D = distribution_ggx_anisotropic(N, H, T, B, roughness, anisotropy);
            } else {
                D = distribution_ggx(N, H, roughness);
            }
            let G = geometry_smith(N, V, L, roughness);
            let F = fresnel_schlick(max(dot(H, V), 0.0), F0);
            
            let numerator = D * G * F;
            let denominator = 4.0 * NdotV * NdotL + 0.001;
            let specular = numerator / denominator;
            
            let kD = (vec3<f32>(1.0) - F) * (1.0 - metallic);
            let diffuse = kD * base_color / PI;
            
            total_diffuse += diffuse * light_color * NdotL * intensity;
            total_specular += specular * light_color * NdotL * intensity;
        }
    }
    
    // Environment reflection sampling (for metallic/reflective surfaces)
    var env_contribution = vec3<f32>(0.0);
    if (env_intensity > 0.001) {
        let R = reflect(-V, N);
        let env_color = sample_environment_reflection(R, roughness);
        let F_env = fresnel_schlick_roughness(NdotV, F0, roughness);
        // Metals reflect environment with base_color tint; dielectrics reflect neutrally
        env_contribution = env_color * F_env * env_intensity;
    }
    
    // Combine lighting
    var final_color = ambient + total_diffuse + total_specular + env_contribution;
    
    // Selection highlight (blue glow)
    if (material.is_selected > 0.5) {
        final_color = mix(final_color, vec3<f32>(0.3, 0.6, 1.0), 0.25);
    }
    
    // Tone mapping (ACES-inspired approximation)
    final_color = final_color * (2.51 * final_color + 0.03) / (final_color * (2.43 * final_color + 0.59) + 0.14);
    
    // Final NaN safety — replace any NaN with gray
    if (any(final_color != final_color)) {
        final_color = vec3<f32>(0.5, 0.5, 0.5);
    }
    
    // Linear → sRGB conversion (manual gamma since render target is Rgba8Unorm)
    let srgb = pow(clamp(final_color, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(srgb, 1.0);
}
