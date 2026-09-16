//! USD PreviewSurface Material System
//!
//! Implements Pixar's UsdPreviewSurface specification for PBR rendering.
//! https://graphics.pixar.com/usd/docs/UsdPreviewSurface-Proposal.html

use std::path::PathBuf;

/// USD PreviewSurface Material
/// Matches the standard PBR material model used in USD/glTF/USDZ
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PreviewSurfaceMaterial {
    /// Material identifier
    pub name: String,

    // === Base Layer ===
    /// Base color (diffuse albedo)
    pub diffuse_color: [f32; 3], // default: [0.18, 0.18, 0.18]
    /// Base color texture path
    pub diffuse_color_texture: Option<PathBuf>,

    /// Emissive color (self-illumination)
    pub emissive_color: [f32; 3], // default: [0.0, 0.0, 0.0]
    /// Emissive texture path
    pub emissive_color_texture: Option<PathBuf>,
    /// Emissive intensity multiplier
    pub emissive_intensity: f32, // default: 1.0

    // === Metallic Workflow ===
    /// Metallic factor (0 = dielectric, 1 = metal)
    pub metallic: f32, // default: 0.0
    /// Metallic texture (single channel)
    pub metallic_texture: Option<PathBuf>,

    /// Roughness factor (0 = smooth, 1 = rough)
    pub roughness: f32, // default: 0.5
    /// Roughness texture (single channel, linear)
    pub roughness_texture: Option<PathBuf>,

    // === Specular Workflow (alternative to metallic) ===
    /// Specular color (F0 reflectance)
    pub specular_color: [f32; 3], // default: [0.0, 0.0, 0.0]
    /// Specular roughness
    pub specular_roughness: f32, // default: 0.5
    /// IOR for Fresnel calculation
    pub ior: f32, // default: 1.5

    // === Clearcoat Layer ===
    /// Clearcoat intensity (0 = none, 1 = full)
    pub clearcoat: f32, // default: 0.0
    /// Clearcoat roughness
    pub clearcoat_roughness: f32, // default: 0.01
    /// Clearcoat normal map
    pub clearcoat_normal_texture: Option<PathBuf>,

    // === Occlusion & Normal ===
    /// Ambient occlusion texture (single channel)
    pub occlusion_texture: Option<PathBuf>,
    /// Normal map texture
    pub normal_texture: Option<PathBuf>,
    /// Normal map scale/factor
    pub normal_scale: f32, // default: 1.0

    // === Transparency ===
    /// Opacity (1 = opaque, 0 = transparent)
    pub opacity: f32, // default: 1.0
    /// Opacity texture (single channel)
    pub opacity_texture: Option<PathBuf>,
    /// Use threshold-based transparency
    pub opacity_threshold: f32, // default: 0.0

    // === Displacement ===
    /// Displacement scale
    pub displacement: f32, // default: 0.0
    /// Displacement texture
    pub displacement_texture: Option<PathBuf>,

    // === Additional Data ===
    /// UV coordinate set index (for multi-UV)
    pub uv_set: u32, // default: 0 (st)

    /// Custom properties (USD metadata)
    pub metadata: std::collections::HashMap<String, String>,
}

impl Default for PreviewSurfaceMaterial {
    fn default() -> Self {
        Self {
            name: "UsdPreviewSurface".to_string(),

            diffuse_color: [0.18, 0.18, 0.18],
            diffuse_color_texture: None,

            emissive_color: [0.0, 0.0, 0.0],
            emissive_color_texture: None,
            emissive_intensity: 1.0,

            metallic: 0.0,
            metallic_texture: None,

            roughness: 0.5,
            roughness_texture: None,

            specular_color: [0.0, 0.0, 0.0],
            specular_roughness: 0.5,
            ior: 1.5,

            clearcoat: 0.0,
            clearcoat_roughness: 0.01,
            clearcoat_normal_texture: None,

            occlusion_texture: None,
            normal_texture: None,
            normal_scale: 1.0,

            opacity: 1.0,
            opacity_texture: None,
            opacity_threshold: 0.0,

            displacement: 0.0,
            displacement_texture: None,

            uv_set: 0,
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl PreviewSurfaceMaterial {
    /// Create a new material with default values
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Check if this is a metallic workflow material
    pub fn use_metallic_workflow(&self) -> bool {
        self.metallic > 0.001 || self.metallic_texture.is_some()
    }

    /// Check if material has any textures
    pub fn has_textures(&self) -> bool {
        self.diffuse_color_texture.is_some()
            || self.metallic_texture.is_some()
            || self.roughness_texture.is_some()
            || self.normal_texture.is_some()
            || self.occlusion_texture.is_some()
            || self.emissive_color_texture.is_some()
            || self.opacity_texture.is_some()
            || self.clearcoat_normal_texture.is_some()
            || self.displacement_texture.is_some()
    }

    /// Check if material is transparent
    pub fn is_transparent(&self) -> bool {
        self.opacity < 1.0 || self.opacity_texture.is_some()
    }

    /// Check if material is emissive
    pub fn is_emissive(&self) -> bool {
        let intensity = self.emissive_intensity;
        self.emissive_color[0] * intensity > 0.0
            || self.emissive_color[1] * intensity > 0.0
            || self.emissive_color[2] * intensity > 0.0
            || self.emissive_color_texture.is_some()
    }

    /// Get roughness value (considering both metallic and specular workflows)
    pub fn effective_roughness(&self) -> f32 {
        if self.use_metallic_workflow() {
            self.roughness
        } else {
            self.specular_roughness
        }
    }

    /// Convert to a simplified GPU-compatible format
    pub fn to_gpu_material(&self) -> GpuMaterial {
        GpuMaterial {
            base_color: [
                self.diffuse_color[0],
                self.diffuse_color[1],
                self.diffuse_color[2],
                self.opacity,
            ],
            metallic_roughness: [self.metallic, self.roughness, 0.0, 0.0],
            emissive: [
                self.emissive_color[0] * self.emissive_intensity,
                self.emissive_color[1] * self.emissive_intensity,
                self.emissive_color[2] * self.emissive_intensity,
                0.0,
            ],
            has_textures: if self.has_textures() { 1 } else { 0 },
            has_normal_map: if self.normal_texture.is_some() { 1 } else { 0 },
            normal_scale: self.normal_scale,
            clearcoat: self.clearcoat,
            clearcoat_roughness: self.clearcoat_roughness,
            ior: self.ior,
        }
    }

    /// Parse from a USDA material block
    pub fn from_usda(name: &str, block: &str) -> Self {
        let mut mat = Self::new(name);

        for line in block.lines() {
            let trimmed = line.trim();

            // color3f inputs:diffuseColor = (0.18, 0.18, 0.18)
            if let Some(val) = Self::parse_color3f_input(trimmed, "diffuseColor") {
                mat.diffuse_color = val;
            }
            // float inputs:metallic = 0.0
            else if let Some(val) = Self::parse_f32_input(trimmed, "metallic") {
                mat.metallic = val.clamp(0.0, 1.0);
            }
            // float inputs:roughness = 0.5
            else if let Some(val) = Self::parse_f32_input(trimmed, "roughness") {
                mat.roughness = val.clamp(0.0, 1.0);
            }
            // color3f inputs:emissiveColor = (0, 0, 0)
            else if let Some(val) = Self::parse_color3f_input(trimmed, "emissiveColor") {
                mat.emissive_color = val;
            }
            // float inputs:emissiveIntensity = 1.0
            else if let Some(val) = Self::parse_f32_input(trimmed, "emissiveIntensity") {
                mat.emissive_intensity = val;
            }
            // float inputs:opacity = 1.0
            else if let Some(val) = Self::parse_f32_input(trimmed, "opacity") {
                mat.opacity = val.clamp(0.0, 1.0);
            }
            // float inputs:ior = 1.5
            else if let Some(val) = Self::parse_f32_input(trimmed, "ior") {
                mat.ior = val;
            }
            // float inputs:clearcoat = 0.0
            else if let Some(val) = Self::parse_f32_input(trimmed, "clearcoat") {
                mat.clearcoat = val.clamp(0.0, 1.0);
            }
            // float inputs:clearcoatRoughness = 0.01
            else if let Some(val) = Self::parse_f32_input(trimmed, "clearcoatRoughness") {
                mat.clearcoat_roughness = val.clamp(0.0, 1.0);
            }
            // float inputs:displacement = 0.0
            else if let Some(val) = Self::parse_f32_input(trimmed, "displacement") {
                mat.displacement = val;
            }

            // TODO: Parse texture references
            // asset inputs:diffuseColor.connect = </Materials/Texture.diffuse>
        }

        mat
    }

    /// Parse color3f input: `color3f inputs:name = (r, g, b)`
    fn parse_color3f_input(line: &str, name: &str) -> Option<[f32; 3]> {
        let pattern = format!("color3f inputs:{} = ", name);
        if let Some(start) = line.find(&pattern) {
            let rest = &line[start + pattern.len()..];
            if let Some(paren_start) = rest.find('(') {
                if let Some(paren_end) = rest.find(')') {
                    let inner = &rest[paren_start + 1..paren_end];
                    let parts: Vec<f32> = inner
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();
                    if parts.len() >= 3 {
                        return Some([parts[0], parts[1], parts[2]]);
                    }
                }
            }
        }
        None
    }

    /// Parse float input: `float inputs:name = value`
    fn parse_f32_input(line: &str, name: &str) -> Option<f32> {
        let pattern = format!("float inputs:{} = ", name);
        if let Some(start) = line.find(&pattern) {
            let rest = &line[start + pattern.len()..];
            // Extract just the number before any comment or end
            let num_str: String = rest
                .chars()
                .take_while(|c| c.is_digit(10) || *c == '.' || *c == '-')
                .collect();
            num_str.parse().ok()
        } else {
            None
        }
    }
}

/// GPU-compatible material structure (std140 layout)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GpuMaterial {
    pub base_color: [f32; 4],         // RGBA
    pub metallic_roughness: [f32; 4], // metallic, roughness, pad, pad
    pub emissive: [f32; 4],           // RGB, intensity
    pub has_textures: u32,            // 1 if any texture present
    pub has_normal_map: u32,          // 1 if normal map present
    pub normal_scale: f32,
    pub clearcoat: f32,
    pub clearcoat_roughness: f32,
    pub ior: f32,
}

// Safety: GpuMaterial is #[repr(C)] and contains only f32/u32 — no padding issues
unsafe impl bytemuck::Pod for GpuMaterial {}
unsafe impl bytemuck::Zeroable for GpuMaterial {}

/// Material assignment for a mesh/prim
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MaterialBinding {
    /// Name of the material
    pub material_name: String,
    /// Optional material reference path
    pub material_path: Option<String>,
    /// Binding strength (weaker/stronger)
    pub binding_strength: BindingStrength,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BindingStrength {
    /// Can be overridden
    Weaker,
    /// Standard binding
    Fallback,
    /// Overrides all
    Stronger,
}

impl Default for BindingStrength {
    fn default() -> Self {
        BindingStrength::Fallback
    }
}

/// Collection of materials in a scene
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MaterialLibrary {
    materials: std::collections::HashMap<String, PreviewSurfaceMaterial>,
    bindings: std::collections::HashMap<String, MaterialBinding>, // prim_path -> binding
}

impl MaterialLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_material(&mut self, material: PreviewSurfaceMaterial) {
        self.materials.insert(material.name.clone(), material);
    }

    pub fn get_material(&self, name: &str) -> Option<&PreviewSurfaceMaterial> {
        self.materials.get(name)
    }

    pub fn bind_material(&mut self, prim_path: &str, binding: MaterialBinding) {
        self.bindings.insert(prim_path.to_string(), binding);
    }

    pub fn get_binding(&self, prim_path: &str) -> Option<&MaterialBinding> {
        self.bindings.get(prim_path)
    }

    pub fn resolve_material(&self, prim_path: &str) -> Option<&PreviewSurfaceMaterial> {
        self.get_binding(prim_path)
            .and_then(|b| self.get_material(&b.material_name))
    }

    pub fn materials(&self) -> &std::collections::HashMap<String, PreviewSurfaceMaterial> {
        &self.materials
    }

    pub fn materials_mut(
        &mut self,
    ) -> &mut std::collections::HashMap<String, PreviewSurfaceMaterial> {
        &mut self.materials
    }
}
