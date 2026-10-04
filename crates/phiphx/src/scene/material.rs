//! Material slot data shared by the viewport and the Properties panel.

use crate::blend::BlendSceneObject;

/// A single material slot on a [`SceneObject`], mirrored from Blender's
/// Principled BSDF: an RGBA base colour plus the two PBR scalars and an
/// optional base-colour texture path. Kept kit-agnostic (pure data) so both
/// the viewport and the Properties material panel can read it.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneMaterial {
    pub name: String,
    /// Linear RGBA base / diffuse colour (Principled "Base Color").
    pub base_color: [f32; 4],
    /// `Material.r/g/b/a` — Solid-mode display colour (Blender default
    /// `color_type = MATERIAL`). Falls back to `base_color` when unset.
    pub viewport_color: [f32; 4],
    /// `Material.roughness` / `metallic`: Solid mode's (0.4 / 0 by default).
    pub viewport_pbr: [f32; 2],
    pub roughness: f32,
    pub metallic: f32,
    /// The material's node tree as the Shader editor draws it
    /// (`blend::shade::graph`). Empty for a material with no tree, and for a
    /// `.phx` import, which carries no graph.
    pub graph: crate::blend::shade::graph::BlendNodeGraph,
    /// Base-colour image texture path (Blender `//`-relative or absolute), if
    /// the material's shader graph links an Image Texture node. `None` when
    /// the material is a flat colour.
    pub base_color_texture: Option<String>,
    /// Decoded `base_color_texture` (packed or found on disk), if any.
    pub texture: Option<super::texture::SceneTexture>,
    /// Principled BSDF inputs (emission, alpha, transmission, IOR …).
    pub bsdf: SceneBsdf,
    /// A Mix Shader's two lobes, when the surface is one (`BlendMix`).
    pub mix: Option<crate::blend::shade::surface::BlendMix>,
    /// Normal map image key and strength (Normal Map node), if linked.
    pub normal_texture: Option<(String, f32)>,
    /// Decoded normal map (raw, non-colour bytes).
    pub normal_map: Option<super::texture::SceneTexture>,
    /// Roughness / Metallic / Emission Color images (keys, then decoded).
    pub maps: SceneMaps,
}

/// One Principled input driven by an image: its key and, once attached, the
/// decoded image. A linked image replaces the material's scalar value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneMap {
    pub key: String,
    pub image: Option<super::texture::SceneTexture>,
}

/// The mapped inputs of one material (see `blend::shade::maps`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneMaps {
    pub rough: Option<SceneMap>,
    pub metal: Option<SceneMap>,
    pub emit: Option<SceneMap>,
}

impl SceneMaps {
    /// Decoded image per input, in field order.
    pub fn images(&self) -> [Option<&super::texture::SceneTexture>; 3] {
        [&self.rough, &self.metal, &self.emit].map(|m| m.as_ref()?.image.as_ref())
    }

    /// Each input slot, for attaching decoded images.
    pub(super) fn each_mut_pub(&mut self) -> [&mut Option<SceneMap>; 3] {
        [&mut self.rough, &mut self.metal, &mut self.emit]
    }

    /// From the file's image keys.
    fn of(m: &crate::blend::BlendMaps) -> Self {
        let mut out = Self::default();
        for (slot, key) in out.each_mut_pub().into_iter().zip(m.each()) {
            *slot = key.clone().map(|key| SceneMap { key, image: None });
        }
        out
    }
}

/// Principled BSDF inputs as read from the file.
pub type SceneBsdf = crate::blend::BlendBsdf;

impl SceneMaterial {
    /// A neutral opaque-grey material used when a source object declares no
    /// material at all but a slot is required.
    pub fn neutral() -> Self {
        Self {
            name: String::from("Material"),
            base_color: [0.8, 0.8, 0.8, 1.0],
            viewport_color: [0.8, 0.8, 0.8, 1.0],
            viewport_pbr: crate::blend::VIEWPORT_PBR,
            roughness: 0.5,
            metallic: 0.0,
            // A stand-in material has no tree.
            graph: Default::default(),
            base_color_texture: None,
            texture: None,
            bsdf: SceneBsdf::default(),
            mix: None,
            normal_texture: None,
            normal_map: None,
            maps: SceneMaps::default(),
        }
    }
}

/// Map the object's material slots into kit-agnostic [`SceneMaterial`]s.
///
/// Prefers the explicit `materials` list (multi-slot). Falls back to the
/// object's flat `base_color` / `roughness` / `metallic` when only the
/// single-material fields are populated, and returns an empty vec when the
/// object declared no material at all (renderer then uses its neutral colour).
pub(super) fn materials_from_blend(obj: &BlendSceneObject) -> Vec<SceneMaterial> {
    if !obj.materials.is_empty() {
        return obj
            .materials
            .iter()
            .map(|m| SceneMaterial {
                name: m.name.clone(),
                base_color: m.base_color,
                viewport_color: m.viewport_color,
                viewport_pbr: m.viewport_pbr,
                roughness: m.roughness,
                metallic: m.metallic,
                graph: m.graph.clone(),
                base_color_texture: m.base_color_texture.clone(),
                texture: None,
                bsdf: m.bsdf,
                mix: m.mix.clone(),
                normal_texture: m.normal_map.as_ref().map(|n| (n.image.clone(), n.strength)),
                normal_map: None,
                maps: SceneMaps::of(&m.maps),
            })
            .collect();
    }
    // A fully-default material record (black, rough 0, metal 0) means the
    // importer found no material; don't fabricate a slot for it.
    let is_default = obj.base_color == [0.0, 0.0, 0.0, 0.0]
        && obj.roughness == 0.0
        && obj.metallic == 0.0;
    if is_default {
        return Vec::new();
    }
    vec![SceneMaterial {
        name: String::from("Material"),
        base_color: obj.base_color,
        viewport_color: obj.base_color,
        viewport_pbr: crate::blend::VIEWPORT_PBR,
        roughness: obj.roughness,
        metallic: obj.metallic,
        // The object's flat colour fields, not a tree.
        graph: Default::default(),
        base_color_texture: obj.texture_paths.first().cloned(),
        texture: None,
        bsdf: SceneBsdf::default(),
        mix: None,
        normal_texture: None,
        normal_map: None,
        maps: SceneMaps::default(),
    }]
}
