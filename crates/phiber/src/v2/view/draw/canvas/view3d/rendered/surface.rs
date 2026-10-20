//! A face's material as the tracer takes it (`scene.rs`): one `Tri`
//! carrying the surface's parameters, its base image, the roughness /
//! metallic / emission maps and the normal map — or, for a Mix Shader
//! (`SceneMaterial::mix`), the first lobe's, with the second lobe its own
//! material in the scene (`Tri::with_second`). The textures belong to the
//! lobe that holds the Principled they were read from, and so do the
//! procedural textures (`SceneMaterial::procedural`): a Noise / Voronoi on
//! the base colour and a Bump on the normal go with the Principled, one on
//! the Mix Shader's factor with the first lobe, which the tracer reads the
//! weight from. A colour ramp is one 256 × 1 image in the atlas.

use super::scene::TEXTURE_LIMIT;
use crate::phiphx::scene::{SceneBsdf, SceneMaterial, SceneTexture};
use crate::v2::scene::BShading;
use super::slot::BProcSlot;
use super::stand::BMatOverride;
use phiray::{NO_TEX, Scene, Tri};

/// What a face's material build needs of its surroundings.
pub(super) struct BSurface<'a> {
    pub slot: Option<&'a SceneMaterial>,
    pub shading: BShading,
    /// Color ▸ Texture under Solid: the base image alone.
    pub texture: bool,
    /// Images are uploaded in this mode and the face has UVs.
    pub uv_ok: bool,
    pub color: [f32; 4],
    pub uvs: [[f32; 2]; 3],
}

/// Atlas ids for the textures added so far, by their pixel buffer.
pub(crate) type BAtlasIds = std::collections::HashMap<*const Vec<u8>, u32>;

impl BSurface<'_> {
    /// Workbench (Solid / Wireframe) shades from the material's viewport
    /// colour and reads no image at all — so no image goes to the GPU in
    /// those modes. Uploading a character's normal and roughness maps for a
    /// mode that never samples them is what pushed the atlas past one
    /// binding and made the GPU decline the whole scene. Color ▸ Texture is
    /// the exception: Solid then shows the base-colour image, and that
    /// image alone.
    fn has_uv(&self) -> bool {
        self.uv_ok && !self.shading.uses_viewport_color()
    }

    fn has_base_image(&self) -> bool {
        self.has_uv() || (self.uv_ok && self.texture && self.shading == BShading::Solid)
    }

    /// The parameters of one lobe: `bsdf`'s, over `color` (the viewport
    /// colour under Workbench, else the lobe's base).
    fn lobe(&self, color: [f32; 4], bsdf: Option<&SceneBsdf>, pbr: (f32, f32)) -> Tri {
        let mut tri = Tri::new([[0.0; 3]; 3], [[0.0; 3]; 3], color).with_pbr(pbr.0, pbr.1);
        if let Some(b) = bsdf {
            tri.color[3] = b.alpha;
            tri = tri
                // Glass refracts at its own IOR; anything else reflects at the
                // IOR Specular IOR Level makes of it (`specular_eta`), which
                // is the only thing `ior` does on an opaque surface.
                .with_optics(b.emitted(), b.transmission, if b.transmission > 0.0 { b.ior } else { b.specular_eta() })
                .with_subsurface(b.subsurface, b.scatter())
                .with_sheen(b.sheen, b.sheen_roughness, b.sheen_tint)
                .with_coat(b.coat, b.coat_roughness)
                .with_anisotropy(b.anisotropic, b.anisotropic_rotation);
        }
        tri
    }

    /// Solid and Wireframe (Workbench) shade with the material's viewport
    /// roughness and metallic, the rest with the BSDF's.
    fn pbr(&self, bsdf: Option<&SceneBsdf>) -> (f32, f32) {
        match (self.slot, self.shading.uses_viewport_color()) {
            (Some(m), true) => (m.viewport_pbr[0], m.viewport_pbr[1]),
            (Some(_), false) => bsdf.map_or((0.5, 0.0), |b| (b.roughness.clamp(0.0, 1.0), b.metallic.clamp(0.0, 1.0))),
            (None, true) => (0.4, 0.0),
            (None, false) => (0.5, 0.0),
        }
    }

    /// The material's textures onto `tri`: the base image (tinted by the
    /// base colour, which is white when the image was folded), the maps —
    /// a linked image replaces the socket value, so the scalar becomes 1
    /// (emission keeps its strength) and the shader multiplies by the image
    /// — and the normal map.
    fn textured(&self, mut tri: Tri, out: &mut Scene, ids: &mut BAtlasIds) -> Tri {
        let Some(m) = self.slot else { return tri };
        let mut add = |t: &SceneTexture| *ids.entry(std::sync::Arc::as_ptr(&t.rgba)).or_insert_with(|| out.atlas.add_bounded(t.width, t.height, &t.rgba, TEXTURE_LIMIT));
        if let (Some(tex), true) = (m.texture.as_ref(), self.has_base_image()) {
            let tint = m.base_color;
            tri.color = [tint[0], tint[1], tint[2], tri.color[3]];
            tri = tri.with_texture(add(tex), self.uvs);
        }
        let images = m.maps.images();
        if self.has_uv() && images.iter().any(Option::is_some) {
            let maps = [0, 1, 2].map(|k| images[k].map_or(NO_TEX, &mut add));
            tri.rough = if images[0].is_some() { 1.0 } else { tri.rough };
            tri.metal = if images[1].is_some() { 1.0 } else { tri.metal };
            if images[2].is_some() {
                tri.emit = [m.bsdf.emission_strength; 3];
            }
            tri = tri.with_maps(maps, self.uvs);
        }
        if let (Some(map), Some((_, strength)), true) = (m.normal_map.as_ref(), m.normal_texture.as_ref(), self.has_uv()) {
            tri = tri.with_normal_map(add(map), *strength, self.uvs);
        }
        tri
    }

    /// The material under a shading mode that evaluates its nodes
    /// (Material Preview, Rendered); Workbench reads none of them.
    fn shaded(&self) -> Option<&SceneMaterial> {
        self.slot.filter(|_| !self.shading.uses_viewport_color())
    }

    /// The Principled's procedural textures onto its lobe: the base
    /// colour's and the Bump's.
    fn procedural(&self, tri: Tri, out: &mut Scene, ids: &mut BAtlasIds) -> Tri {
        let Some(m) = self.shaded() else { return tri };
        let base = m.procedural.base.as_ref().map(|p| p.slot(1, out, ids));
        let bump = m.procedural.bump.as_ref().map(|b| b.height.slot(0, out, ids).bump(b.strength, b.distance));
        tri.with_procs(base.or(Some(tri.color_proc)), bump)
    }

    /// The Mix Shader's factor onto the first lobe. One slot holds the
    /// colour texture: the same texture on both drives both; two different
    /// ones leave the weight at its folded constant.
    fn mixed(&self, mix: &crate::phiphx::blend::shade::surface::BlendMix, tri: Tri, out: &mut Scene, ids: &mut BAtlasIds) -> Tri {
        let Some(m) = self.shaded() else { return tri };
        let Some(fac) = m.procedural.mix.as_ref() else { return tri };
        let target = match (&m.procedural.base, mix.principled) {
            (Some(base), Some(0)) if base == fac => 3,
            (Some(_), Some(0)) => return tri,
            _ => 2,
        };
        tri.with_procs(Some(fac.slot(target, out, ids)), Some(tri.bump_proc))
    }

    /// The face's material: the surface as one lobe, or a Mix Shader's two
    /// — the second added to the scene as a material of its own, the
    /// textures on the lobe with the Principled they came from. Workbench
    /// keeps the viewport colour as the first lobe's, since Solid reads only
    /// that.
    pub fn tri(&self, out: &mut Scene, ids: &mut BAtlasIds) -> Tri {
        if let Some(stand) = self.overridden() {
            return stand;
        }
        let viewport = self.shading.uses_viewport_color();
        match self.slot.and_then(|m| m.mix.as_ref()).filter(|_| !viewport) {
            Some(mix) => {
                let (first, second) = (self.lobe(mix.first.base, Some(&mix.first), self.pbr(Some(&mix.first))), self.lobe(mix.second.base, Some(&mix.second), self.pbr(Some(&mix.second))));
                let (first, second) = match mix.principled {
                    Some(1) => (first, self.procedural(self.textured(second, out, ids), out, ids)),
                    _ => (self.procedural(self.textured(first, out, ids), out, ids), second),
                };
                let first = self.mixed(mix, first, out, ids);
                let other = out.material(second.mat());
                first.with_second(other, mix.fac)
            }
            None => {
                let bsdf = self.slot.map(|m| &m.bsdf);
                self.procedural(self.textured(self.lobe(self.color, bsdf, self.pbr(bsdf)), out, ids), out, ids)
            }
        }
    }
}
