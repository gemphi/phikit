//! `surface.rs`'s tests: a Mix Shader as two materials, and the procedural
//! textures landing in the tracer's slots.

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
    let palette = |shading| super::super::mesh::BMeshPalette { shading, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey, matcap: None, texture: false };
    let scene = super::scene::build(&[card(material.clone())], &palette(BShading::Rendered));
    assert_eq!(scene.mats.len(), 2, "both lobes are materials");
    let face = &scene.mats[(scene.faces[0][3] & 0xffff) as usize];
    assert_eq!((face.color, face.metal), ([0.8, 0.1, 0.1, 1.0], 0.0), "the face's own is the first lobe");
    assert_eq!(face.mix, 0.3);
    let other = &scene.mats[face.second as usize];
    assert_eq!((other.metal, other.rough, other.second), (1.0, 0.02, phiray::NO_MAT), "the mirror, at `with_pbr`'s floor");
    // Workbench: the viewport colour, one lobe, the mix left out.
    let solid = super::scene::build(&[card(material)], &palette(BShading::Solid));
    assert_eq!(solid.mats.len(), 1);
    assert_eq!(solid.mats[0].second, phiray::NO_MAT);
}

/// A ramped Noise on the base colour, a Voronoi Bump on the normal and a
/// Noise on the Mix Shader's factor: the slots of the lobes they belong to,
/// the ramp one 256 × 1 image in the atlas; Workbench reads none.
#[test]
fn procedural_textures_land_in_the_lobes_slots() {
    use crate::phiphx::blend::shade::procedural::{BlendBump, BlendProc, BlendProcCoord, BlendProcKind, BlendProcedural};
    let noise = BlendProc { kind: BlendProcKind::NoiseFac, coords: BlendProcCoord::Object, location: [0.1, 0.2, 0.0], scale3: [2.0, 2.0, 1.0], scale: 8.0, detail: 3.0, roughness: 0.6, lacunarity: 2.5, distortion: 0.0, normalize: true, variant: 1, ramp: Some(vec![200; 256 * 4]), ..Default::default() };
    let voronoi = BlendProc { kind: BlendProcKind::VoronoiDistance, coords: BlendProcCoord::Generated, scale: 12.0, distortion: 0.7, normalize: false, ramp: None, ..noise };
    let fac = BlendProc { ramp: None, ..noise };
    let red = SceneBsdf { base: [0.8, 0.1, 0.1, 1.0], roughness: 1.0, ..Default::default() };
    let mirror = SceneBsdf { base: [1.0; 4], roughness: 0.0, metallic: 1.0, ..Default::default() };
    let mut material = SceneMaterial::neutral();
    material.bsdf = red.mixed(&mirror, 0.5);
    // The Principled is the second lobe: its textures go there, the factor's
    // to the first, which the tracer reads the weight from.
    material.mix = Some(BlendMix { first: mirror, second: red, fac: 0.5, principled: Some(1) });
    material.procedural = BlendProcedural { base: Some(noise), mix: Some(fac), bump: Some(BlendBump { height: voronoi, strength: 0.6, distance: 0.3 }) };
    let grey = white();
    let palette = |shading| super::super::mesh::BMeshPalette { shading, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey, matcap: None, texture: false };
    let mut unramped = material.clone();
    unramped.procedural.base.as_mut().unwrap().ramp = None;
    let bare = super::scene::build(&[card(unramped)], &palette(BShading::Material)).atlas.len();
    let scene = super::scene::build(&[card(material.clone())], &palette(BShading::Material));
    let face = &scene.mats[(scene.faces[0][3] & 0xffff) as usize];
    assert_eq!((face.color_proc.kind(), face.color_proc.target, face.color_proc.ramp, face.color_proc.coords), (1, 2, phiray::NO_TEX, 1), "the factor's Noise on the first lobe, driving the weight");
    assert!(!face.bump_proc.is_some());
    let other = &scene.mats[face.second as usize];
    assert_eq!((other.color_proc.kind(), other.color_proc.target, other.color_proc.scale, other.color_proc.location), (1, 1, 8.0, [0.1, 0.2, 0.0]), "the base's Noise on the Principled's lobe");
    assert_ne!(other.color_proc.ramp, phiray::NO_TEX, "the ramp uploaded");
    assert_eq!((other.bump_proc.kind(), other.bump_proc.strength, f32::from_bits(other.bump_proc.target)), (3, 0.6, 0.3), "Voronoi, Normalize off, as the Bump's height");
    assert!(scene.atlas.len() > bare && other.color_proc.ramp as usize == bare, "the ramp image (with its mip chain) after the rest");
    // The same texture on the base and the factor: one slot drives both.
    material.mix.as_mut().unwrap().principled = Some(0);
    material.procedural.base = material.procedural.mix.clone();
    let scene = super::scene::build(&[card(material.clone())], &palette(BShading::Rendered));
    let face = &scene.mats[(scene.faces[0][3] & 0xffff) as usize];
    assert_eq!((face.color_proc.target, face.bump_proc.kind()), (3, 3));
    // Workbench: no slot, no ramp.
    let mut plain = material.clone();
    plain.procedural = Default::default();
    let bare = super::scene::build(&[card(plain)], &palette(BShading::Solid)).atlas.len();
    let solid = super::scene::build(&[card(material)], &palette(BShading::Solid));
    assert!(!solid.mats[0].color_proc.is_some() && !solid.mats[0].bump_proc.is_some() && solid.atlas.len() == bare);
}

/// A Brick's settings reach the slot's words as `procedural.wgsl` reads
/// them: the kind, the colours, the mortar, the offset and squash.
#[test]
fn a_bricks_settings_reach_its_slot() {
    use crate::phiphx::blend::shade::procedural::{BlendProc, BlendProcKind};
    let brick = BlendProc { kind: BlendProcKind::BrickColor, scale: 5.0, scale3: [1.0; 3], colors: [[0.8; 3], [0.2; 3], [0.1, 0.0, 0.0]], smoothness: 0.05, exponent: 0.1, width: 0.5, height: 0.25, offset: 0.3, direction: 3, gain: 0.8, rings: 4, ..Default::default() };
    let mut material = SceneMaterial::neutral();
    material.procedural.base = Some(brick);
    let grey = white();
    let palette = super::super::mesh::BMeshPalette { shading: BShading::Material, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey, matcap: None, texture: false };
    let scene = super::scene::build(&[card(material)], &palette);
    let p = scene.mats[(scene.faces[0][3] & 0xffff) as usize].color_proc;
    let words: [u32; 40] = bytemuck::cast(p);
    assert_eq!((words[0], words[3], words[17], words[18]), (phiray::proc::PROC_BRICK_COLOR, 1, 3, 4), "kind, target, offset and squash frequencies");
    assert_eq!([20, 21, 22, 23, 25, 26, 34].map(|k| f32::from_bits(words[k])), [0.3, 0.8, 0.05, 0.1, 0.5, 0.25, 0.1], "offset, squash, mortar size and smooth, width, height, mortar red");
}
