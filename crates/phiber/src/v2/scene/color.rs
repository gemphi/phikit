//! Per-triangle colour of a scene object — material slot resolution and the
//! Solid (viewport colour) vs Material Preview (base colour) choice.

use super::{BObject3D, BShading};
use crate::phiphx::scene::SceneMaterial;

impl BObject3D {
    /// The object's name, primitive or loaded.
    pub fn name(&self) -> &str {
        match self {
            BObject3D::Primitive { name, .. } | BObject3D::Loaded { name, .. } => name,
        }
    }

    /// The base colour for triangle `tri` of a loaded object, resolved from
    /// its material slots (per-face index → slot 0 → `None`). Primitives and
    /// material-less loaded objects return `None` so the renderer uses its
    /// neutral loaded colour.
    pub fn triangle_base_color(&self, tri: usize) -> Option<[f32; 4]> {
        self.triangle_slot(tri).map(|m| m.base_color)
    }

    /// The colour the viewport paints triangle `tri` with in `shading` mode:
    /// Solid → `Material.r/g/b` (viewport display colour, Blender's default
    /// `color_type = MATERIAL`); Material Preview / Rendered → the base
    /// colour texture sampled at the triangle's UV centroid, else Base Color.
    pub fn triangle_color(&self, tri: usize, shading: BShading) -> Option<[f32; 4]> {
        // Solid paints every mesh in the theme's one solid shade (Blender's
        // Solid with `color_type = SINGLE`): no colour of the object's own.
        if shading == BShading::Solid {
            return None;
        }
        let m = self.triangle_slot(tri)?;
        if shading.uses_viewport_color() {
            return Some(m.viewport_color);
        }
        Some(self.texture_color(tri, m).unwrap_or(m.base_color))
    }

    /// Texture colour at the UV centroid of triangle `tri` — one sample for
    /// the whole face.
    ///
    /// Kept for callers that need a single representative colour. The
    /// viewport does not use it any more: it shades per pixel, so it carries
    /// [`Self::triangle_uvs`] through to the rasteriser and samples there.
    fn texture_color(&self, tri: usize, m: &SceneMaterial) -> Option<[f32; 4]> {
        let tex = m.texture.as_ref()?;
        let uv = self.triangle_uvs(tri)?;
        let mut mean = [0.0f32; 2];
        for corner in uv {
            mean[0] += corner[0] / 3.0;
            mean[1] += corner[1] / 3.0;
        }
        Some(tex.sample(mean))
    }

    /// The UV at each corner of triangle `tri`, for a shader to interpolate.
    pub fn triangle_uvs(&self, tri: usize) -> Option<[[f32; 2]; 3]> {
        let BObject3D::Loaded { mesh, uvs, .. } = self else { return None };
        let indices = mesh.indices.get(tri * 3..tri * 3 + 3)?;
        let mut out = [[0.0f32; 2]; 3];
        for (slot, &i) in out.iter_mut().zip(indices) {
            *slot = *uvs.get(i as usize)?;
        }
        Some(out)
    }

    /// How triangle `tri`'s surface responds to light.
    pub fn triangle_surface(&self, tri: usize) -> (f32, f32) {
        match self.triangle_slot(tri) {
            Some(m) => (m.roughness, m.metallic),
            // Blender's Principled defaults — a plain dielectric.
            None => (0.5, 0.0),
        }
    }

    /// The base-colour texture covering triangle `tri`, if it has one.
    pub fn triangle_texture(
        &self,
        tri: usize,
        shading: BShading,
    ) -> Option<&crate::phiphx::scene::SceneTexture> {
        // Solid shading paints the material's viewport colour, which is what
        // Blender shows there — no texture is involved.
        if shading.uses_viewport_color() {
            return None;
        }
        self.triangle_slot(tri)?.texture.as_ref()
    }

    pub(crate) fn triangle_slot(&self, tri: usize) -> Option<&SceneMaterial> {
        let BObject3D::Loaded {
            materials,
            material_indices,
            ..
        } = self
        else {
            return None;
        };
        if materials.is_empty() {
            return None;
        }
        let slot = material_indices
            .get(tri)
            .copied()
            .map(usize::from)
            .unwrap_or(0);
        materials.get(slot).or_else(|| materials.first())
    }
}
