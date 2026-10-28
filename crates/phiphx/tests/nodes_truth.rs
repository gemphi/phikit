//! The value, vector and colour nodes and the node groups the fold reads
//! (`shade/fold.rs`, `shade/group.rs`, `shade/procedural.rs`) out of
//! `NODES.blend` (`fixtures/make_nodes.py`), held to Cycles' own
//! evaluation: `truth/nodes.txt` is the Emit bake of each emission plane
//! (`truth/blender_nodes.py`), so every chain of constant inputs — Map
//! Range in its four types with and without Clamp, Separate / Combine
//! Color in HSV and HSL, Combine XYZ through Vector Rotate and Vector
//! Math, a dot product, Separate XYZ, the Normal node's Dot, and a node
//! group three ways (its inputs set, linked, nested) — folds to the
//! colour Blender bakes. The per-pixel nodes are read structurally:
//! Displacement as a Bump, Fresnel and Layer Weight on a Mix Shader's
//! factor as slots, a Map Range before a Color Ramp composed into the
//! ramp, one after a texture as the slot's tail, a Vertex Color as white,
//! a White Noise reached through a group.

use phiphx::blend::BlendFile;
use phiphx::blend::shade::procedural::{BlendProcCoord, BlendProcKind, RAMP_STOPS};
use phiphx::scene::{SceneMaterial, SceneModel};
use std::path::PathBuf;

fn open() -> SceneModel {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/NODES.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    SceneModel::from_blend_owned(scene, &path.to_string_lossy())
}

fn material<'a>(m: &'a SceneModel, name: &str) -> &'a SceneMaterial {
    m.objects.iter().flat_map(|o| &o.materials).find(|ma| ma.name == name).unwrap_or_else(|| panic!("{name}"))
}

/// `truth/nodes.txt`: material → the colour Cycles baked.
fn truth() -> Vec<(String, [f32; 3])> {
    let text = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/truth/nodes.txt")).unwrap();
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let w: Vec<&str> = l.split_whitespace().collect();
            (w[0].to_string(), [w[1].parse().unwrap(), w[2].parse().unwrap(), w[3].parse().unwrap()])
        })
        .collect()
}

/// Every constant chain folds to what Cycles bakes, within a byte's
/// worth of float rounding (HSV through two conversions).
#[test]
fn constant_node_chains_fold_to_what_cycles_bakes() {
    let model = open();
    let truth = truth();
    assert_eq!(truth.len(), 11, "eleven emission planes");
    for (name, want) in truth {
        let got = material(&model, &name).bsdf.emitted();
        for k in 0..3 {
            assert!((got[k] - want[k]).abs() < 2e-3, "{name}: folded {got:?}, Cycles baked {want:?}");
        }
    }
}

/// A Displacement on the Material Output is the Bump Cycles makes of it:
/// the Height's texture at Strength 1 and Distance Scale.
#[test]
fn displacement_becomes_a_bump_at_its_scale() {
    let model = open();
    let m = material(&model, "Displaced");
    let bump = m.procedural.bump.as_ref().expect("the Displacement as a Bump");
    assert_eq!((bump.strength, bump.distance), (1.0, 0.4));
    assert_eq!((bump.height.kind, bump.height.scale), (BlendProcKind::NoiseFac, 7.0));
}

/// Fresnel and Layer Weight on a Mix Shader's factor are per-pixel slots
/// carrying the IOR / Blend; the fold stands in with the hemispherical
/// mean (`fresnel_dielectric_Fss`: 0.136 at 1.8; Facing at Blend 0.3 is
/// `1 − 2 / (0.6 + 2)` = 0.231).
#[test]
fn fresnel_and_layer_weight_drive_the_mix_per_pixel() {
    let model = open();
    let fresnel = material(&model, "FresnelMixed");
    let slot = fresnel.procedural.mix.as_ref().expect("the Fresnel on Fac");
    assert_eq!((slot.kind, slot.detail), (BlendProcKind::Fresnel, 1.8));
    let fac = fresnel.mix.as_ref().expect("a Mix Shader").fac;
    assert!((fac - 0.8 / (4.08567 + 1.00071 * 1.8)).abs() < 1e-4, "{fac}");
    let facing = material(&model, "Facing");
    let slot = facing.procedural.mix.as_ref().expect("the Layer Weight on Fac");
    assert_eq!((slot.kind, slot.detail), (BlendProcKind::LayerFacing, 0.3));
    let fac = facing.mix.as_ref().expect("a Mix Shader").fac;
    assert!((fac - (1.0 - 2.0 / 2.6)).abs() < 1e-5, "{fac}");
}

/// A Map Range between a Noise and a Color Ramp is composed into the
/// ramp's bake — a black-to-white ramp read through `x · 0.5` ends at
/// mid grey; one between a Noise and the socket is the slot's Math tail,
/// a Multiply Add by its slope and offset.
#[test]
fn map_range_folds_into_the_ramp_or_the_tail() {
    let model = open();
    let remapped = material(&model, "Remapped");
    let base = remapped.procedural.base.as_ref().expect("the Noise through the ramp");
    let ramp = base.ramp.as_ref().expect("the ramp baked");
    assert_eq!(ramp.len(), RAMP_STOPS * 4);
    let last = &ramp[(RAMP_STOPS - 1) * 4..];
    assert!((last[0] as i32 - 128).abs() <= 1, "the ramp at 0.5: {last:?}");
    assert!(base.math.is_none(), "nothing after the ramp");
    let tailed = material(&model, "Tailed");
    let base = tailed.procedural.base.as_ref().expect("the Noise through the Map Range");
    let math = base.math.expect("the Map Range as the tail");
    assert_eq!((math.op, math.b, math.c, math.clamp), (36, 0.5, 0.25, false), "x · 0.5 + 0.25; the clamp to 0.25..0.75 is not the tail's 0..1");
}

/// A Base Color that is a Mix of two textures reaches the record as both,
/// with the node's factor and blend mode: the planks (a Wave through a
/// Color Ramp) and the grain (a Noise over a Mapping stretched 22× in Y),
/// which one texture per socket could only carry half of.
#[test]
fn a_base_colour_mix_carries_both_textures() {
    let model = open();
    let wood = material(&model, "MixedWood");
    let planks = wood.procedural.base.as_ref().expect("the Wave through the ramp");
    assert_eq!((planks.kind, planks.scale, planks.distortion), (BlendProcKind::Wave, 7.0, 0.4));
    assert_eq!(planks.direction, 1, "bands along Y");
    assert!(planks.ramp.is_some(), "the Color Ramp baked");
    let grain = wood.procedural.base_mix.as_ref().expect("the Noise as the second texture");
    assert_eq!((grain.fac, grain.blend), (0.35, 0), "Mix by 0.35");
    assert_eq!((grain.texture.kind, grain.texture.scale, grain.texture.detail), (BlendProcKind::NoiseColor, 3.0, 8.0));
    assert_eq!(grain.texture.scale3, [1.0, 22.0, 1.0], "the Mapping's non-uniform scale");
    assert_eq!(grain.texture.coords, BlendProcCoord::Object);
}

/// A Vertex Color the graph cannot read per vertex folds to white, and a
/// texture inside a group reaches the procedural slot.
#[test]
fn vertex_colour_is_white_and_a_grouped_texture_is_read() {
    let model = open();
    let vc = material(&model, "VertexColored");
    assert_eq!(vc.base_color, [1.0, 1.0, 1.0, 1.0]);
    let hashed = material(&model, "WhiteNoised");
    let base = hashed.procedural.base.as_ref().expect("the White Noise through the group");
    assert_eq!((base.kind, base.variant), (BlendProcKind::WhiteNoiseFac, 3));
}
