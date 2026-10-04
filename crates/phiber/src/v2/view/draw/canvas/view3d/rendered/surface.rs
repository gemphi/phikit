//! A face's material as the tracer takes it (`scene.rs`): one `Tri`
//! carrying the surface's parameters, its base image, the roughness /
//! metallic / emission maps and the normal map — or, for a Mix Shader
//! (`SceneMaterial::mix`), the first lobe's, with the second lobe its own
//! material in the scene (`Tri::with_second`). The textures belong to the
//! lobe that holds the Principled they were read from.

use super::scene::TEXTURE_LIMIT;
use crate::phiphx::scene::{SceneBsdf, SceneMaterial, SceneTexture};
use crate::v2::scene::BShading;
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
pub(super) type BAtlasIds = std::collections::HashMap<*const Vec<u8>, u32>;

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
                .with_optics(b.emitted(), b.transmission, b.ior)
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

    /// The face's material: the surface as one lobe, or a Mix Shader's two
    /// — the second added to the scene as a material of its own, the
    /// textures on the lobe with the Principled they came from. Workbench
    /// keeps the viewport colour as the first lobe's, since Solid reads only
    /// that.
    pub fn tri(&self, out: &mut Scene, ids: &mut BAtlasIds) -> Tri {
        let viewport = self.shading.uses_viewport_color();
        match self.slot.and_then(|m| m.mix.as_ref()).filter(|_| !viewport) {
            Some(mix) => {
                let (first, second) = (self.lobe(mix.first.base, Some(&mix.first), self.pbr(Some(&mix.first))), self.lobe(mix.second.base, Some(&mix.second), self.pbr(Some(&mix.second))));
                let (first, second) = match mix.principled {
                    Some(1) => (first, self.textured(second, out, ids)),
                    _ => (self.textured(first, out, ids), second),
                };
                let other = out.material(second.mat());
                first.with_second(other, mix.fac)
            }
            None => {
                let bsdf = self.slot.map(|m| &m.bsdf);
                self.textured(self.lobe(self.color, bsdf, self.pbr(bsdf)), out, ids)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::phiphx::blend::shade::surface::BlendMix;
    use crate::phiphx::scene::{SceneBsdf, SceneCorners, SceneMaterial, SceneMesh, SceneNormals};
    use crate::v2::scene::{BObject3D, BShading};
    use gpui::white;

    fn card(material: SceneMaterial) -> BObject3D {
        BObject3D::Loaded {
            name: "Card".into(),
            mesh: std::sync::Arc::new(SceneMesh {
                vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                indices: vec![0, 1, 2],
                normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
                corners: SceneCorners::default(),
            }),
            location: [0.0; 3],
            scale: [1.0; 3],
            matrix_world: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            materials: vec![material],
            material_indices: Default::default(),
            uvs: Default::default(),
            color: [1.0; 4],
            shadow: Default::default(),
        }
    }

    /// A Mix Shader of a red diffuse and a white mirror: two materials in
    /// the scene, the face's the first with the second named at the weight;
    /// Workbench shades the viewport colour alone.
    #[test]
    fn a_mix_shader_is_two_materials_and_a_weight() {
        let red = SceneBsdf { base: [0.8, 0.1, 0.1, 1.0], roughness: 1.0, ..Default::default() };
        let mirror = SceneBsdf { base: [1.0; 4], roughness: 0.0, metallic: 1.0, ..Default::default() };
        let mut material = SceneMaterial::neutral();
        material.bsdf = red.mixed(&mirror, 0.3);
        material.mix = Some(BlendMix { first: red, second: mirror, fac: 0.3, principled: Some(0) });
        let grey = white();
        let palette = |shading| super::super::super::mesh::BMeshPalette { shading, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey, matcap: None, texture: false };
        let scene = super::super::scene::build(&[card(material.clone())], &palette(BShading::Rendered));
        assert_eq!(scene.mats.len(), 2, "both lobes are materials");
        let face = &scene.mats[(scene.faces[0][3] & 0xffff) as usize];
        assert_eq!((face.color, face.metal), ([0.8, 0.1, 0.1, 1.0], 0.0), "the face's own is the first lobe");
        assert_eq!(face.mix, 0.3);
        let other = &scene.mats[face.second as usize];
        assert_eq!((other.metal, other.rough, other.second), (1.0, 0.02, phiray::NO_MAT), "the mirror, at `with_pbr`'s floor");
        // Workbench: the viewport colour, one lobe, the mix left out.
        let solid = super::super::scene::build(&[card(material)], &palette(BShading::Solid));
        assert_eq!(solid.mats.len(), 1);
        assert_eq!(solid.mats[0].second, phiray::NO_MAT);
    }
}
