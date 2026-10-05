//! The procedural textures a surface reads per pixel (`shade/procedural.rs`)
//! out of `PROCEDURAL.blend` (`fixtures/make_procedural.py`): a Noise through
//! a Color Ramp on the base colour with a Mapping over Object coordinates and
//! a Voronoi Bump on the normal, a Noise on a Mix Shader's factor through a
//! reroute, and a Voronoi's colour over UVs.

use phiphx::blend::BlendFile;
use phiphx::blend::shade::procedural::{BlendProcCoord, BlendProcKind, RAMP_STOPS};
use phiphx::scene::{SceneMaterial, SceneModel};
use std::path::PathBuf;

fn open() -> SceneModel {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/PROCEDURAL.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    SceneModel::from_blend_owned(scene, &path.to_string_lossy())
}

fn material<'a>(m: &'a SceneModel, name: &str) -> &'a SceneMaterial {
    m.objects.iter().flat_map(|o| &o.materials).find(|ma| ma.name == name).unwrap_or_else(|| panic!("{name}"))
}

#[test]
fn a_ramped_noise_on_the_base_and_a_voronoi_bump_on_the_normal() {
    let model = open();
    let ramped = material(&model, "Ramped");
    let base = ramped.procedural.base.as_ref().expect("a texture on Base Color");
    assert_eq!((base.kind, base.coords), (BlendProcKind::NoiseFac, BlendProcCoord::Object));
    assert_eq!((base.scale, base.detail, base.roughness, base.lacunarity, base.distortion), (8.0, 3.0, 0.6, 2.5, 0.0));
    assert_eq!((base.location, base.scale3), ([0.1, 0.2, 0.0], [2.0, 2.0, 1.0]), "the Mapping node's");
    // The ramp: black at 0, orange at the middle, white at 1, eased
    // between — 256 linear RGBA stops.
    let ramp = base.ramp.as_ref().expect("the Color Ramp baked");
    assert_eq!(ramp.len(), RAMP_STOPS * 4);
    assert_eq!((&ramp[..4], &ramp[ramp.len() - 4..]), (&[0, 0, 0, 255][..], &[255, 255, 255, 255][..]));
    let mid = &ramp[128 * 4..129 * 4];
    assert!(mid[0] == 255 && (mid[1] as i32 - 128).abs() <= 2 && mid[2] == 0, "orange at the middle: {mid:?}");
    // Ease: a quarter of the way to orange is 3t² − 2t³ = 0.156 of it,
    // where linear would be 0.25.
    let eighth = &ramp[32 * 4..33 * 4];
    assert!(eighth[0] > 35 && eighth[0] < 45, "eased toward orange: {eighth:?}");
    // The colour fold still gives one base colour — the ramp at the
    // noise's mean, orange — so Solid and the material list agree.
    assert!(ramped.base_color[0] > 0.9 && ramped.base_color[2] < 0.1, "{:?}", ramped.base_color);
    let bump = ramped.procedural.bump.as_ref().expect("a Bump on Normal");
    assert_eq!((bump.strength, bump.distance), (0.6, 0.3));
    assert_eq!((bump.height.kind, bump.height.coords, bump.height.scale, bump.height.distortion, bump.height.normalize), (BlendProcKind::VoronoiDistance, BlendProcCoord::Generated, 12.0, 0.7, false), "Normalize is off on a new Voronoi node");
    assert!(bump.height.ramp.is_none() && ramped.procedural.mix.is_none());
    assert!(ramped.normal_map.is_none() && ramped.normal_texture.is_none(), "a Bump is not a normal map");
}

#[test]
fn a_noise_on_the_mix_factor_and_a_voronoi_colour_over_uvs() {
    let model = open();
    let mixed = material(&model, "Mixed");
    let lobes = mixed.mix.as_ref().expect("a Mix Shader on the output");
    assert!((lobes.fac - 0.5).abs() < 1e-6, "the fold's stand-in for the noise: {}", lobes.fac);
    let fac = mixed.procedural.mix.as_ref().expect("the Noise on Fac, through the reroute");
    assert_eq!((fac.kind, fac.coords, fac.scale, fac.detail), (BlendProcKind::NoiseFac, BlendProcCoord::Generated, 5.0, 2.0));
    assert!(fac.ramp.is_none() && mixed.procedural.base.is_none() && mixed.procedural.bump.is_none());

    let colored = material(&model, "Colored");
    let base = colored.procedural.base.as_ref().expect("a texture on Base Color");
    assert_eq!((base.kind, base.coords, base.scale), (BlendProcKind::VoronoiColor, BlendProcCoord::Uv, 5.0));
    assert_eq!((base.location, base.scale3), ([0.0; 3], [1.0; 3]), "no Mapping: the identity");
}
