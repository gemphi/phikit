//! Every texture node the tracer evaluates, read with its settings out of
//! `PROCEDURAL.blend` (`fixtures/make_procedural.py`, Blender 5.0.1):
//! Checker, Brick, Gradient, Magic, Wave, a Ridged Multifractal Noise and
//! two Voronoi features (`shade/texnode.rs`). The numbers are the ones the
//! script set; the enums are `DNA_node_types.h`'s.

use phiphx::blend::BlendFile;
use phiphx::blend::shade::procedural::{BlendProc, BlendProcKind};
use phiphx::scene::SceneModel;
use std::path::PathBuf;

fn base(name: &str) -> BlendProc {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/PROCEDURAL.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    let model = SceneModel::from_blend_owned(scene, &path.to_string_lossy());
    let ma = model.objects.iter().flat_map(|o| &o.materials).find(|m| m.name == name).unwrap_or_else(|| panic!("{name}"));
    ma.procedural.base.clone().unwrap_or_else(|| panic!("{name}: a texture on Base Color"))
}

#[test]
fn checker_brick_and_gradient_with_their_settings() {
    let checker = base("Checkered");
    assert_eq!((checker.kind, checker.scale), (BlendProcKind::CheckerColor, 3.0));
    assert_eq!((checker.colors[0], checker.colors[1]), ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]));

    let brick = base("Bricks");
    assert_eq!(brick.kind, BlendProcKind::BrickColor);
    assert_eq!((brick.offset, brick.direction, brick.gain, brick.rings), (0.3, 3, 0.8, 4), "offset and squash, with their frequencies");
    assert_eq!((brick.smoothness, brick.exponent, brick.width, brick.height), (0.05, 0.1, 0.5, 0.25), "mortar size and smooth, brick width, row height");
    assert_eq!((brick.colors[0], brick.colors[2]), ([0.8; 3], [0.0; 3]));

    let gradient = base("Graded");
    assert_eq!((gradient.kind, gradient.variant, gradient.scale), (BlendProcKind::Gradient, 5, 1.0), "SHD_BLEND_QUADRATIC_SPHERE, no Scale");
}

#[test]
fn magic_wave_and_the_noise_and_voronoi_variants() {
    let magic = base("Magical");
    assert_eq!((magic.kind, magic.rings, magic.distortion), (BlendProcKind::MagicColor, 4, 1.5));

    let wave = base("Waved");
    assert_eq!((wave.kind, wave.variant, wave.rings, wave.profile), (BlendProcKind::Wave, 1, 1, 1), "rings, along Y, saw");
    assert_eq!((wave.distortion, wave.smoothness, wave.offset, wave.gain), (2.0, 1.0, 1.0, 0.5), "distortion, phase, detail scale and roughness");

    let ridged = base("Ridged");
    assert_eq!((ridged.kind, ridged.variant, ridged.offset, ridged.gain), (BlendProcKind::NoiseFac, 3, 0.5, 2.0), "SHD_NOISE_RIDGED_MULTIFRACTAL");
    assert!(ridged.normalize);

    let edged = base("Edged");
    assert_eq!((edged.kind, edged.variant, edged.normalize), (BlendProcKind::VoronoiDistance, 3, true), "SHD_VORONOI_DISTANCE_TO_EDGE");

    let smooth = base("Smoothed");
    assert_eq!((smooth.variant, smooth.direction, smooth.exponent, smooth.smoothness, smooth.normalize), (2, 3, 1.5, 0.4, false), "Smooth F1, Minkowski");

    // The planes of batch 142 read as before: an fBm Noise, a Voronoi F1.
    let colored = base("Colored");
    assert_eq!((colored.kind, colored.variant, colored.direction), (BlendProcKind::VoronoiColor, 0, 0));
}

/// The Mapping node whole: its Type (`bNode.custom1`), Location, Rotation
/// and Scale, read out of `Turned` — a Checker whose Vector comes through
/// a Texture-type Mapping turned 45° about Z. Rotation and Type were both
/// dropped on the floor before batch 151, so a turned texture drew
/// straight and a Texture mapping drew as a Point one.
#[test]
fn the_mapping_nodes_type_rotation_location_and_scale() {
    let turned = base("Turned");
    assert_eq!(turned.kind, BlendProcKind::CheckerColor);
    assert_eq!(turned.mapping, 1, "NODE_MAPPING_TYPE_TEXTURE");
    assert_eq!((turned.location, turned.scale3), ([0.1, 0.0, 0.0], [2.0, 2.0, 1.0]));
    let z = turned.rotation[2];
    assert_eq!((turned.rotation[0], turned.rotation[1]), (0.0, 0.0));
    assert!((z - std::f32::consts::FRAC_PI_4).abs() < 1e-6, "45° about Z in radians: {z}");

    // The planes that have no Mapping node, and the one that has a Point
    // Mapping, read as the identity and as Point.
    let colored = base("Colored");
    assert_eq!((colored.mapping, colored.rotation, colored.scale3), (0, [0.0; 3], [1.0; 3]));
    let ramped = base("Ramped");
    assert_eq!((ramped.mapping, ramped.rotation), (0, [0.0; 3]), "a Point Mapping with no Rotation set");
    assert_eq!((ramped.location, ramped.scale3), ([0.1, 0.2, 0.0], [2.0, 2.0, 1.0]));
}

/// White Noise and Gabor, the last two texture nodes the tracer had no
/// reader for (`svm/white_noise.h`, `svm/gabor.h`). White Noise keeps its
/// dimensions in `bNode.custom1` rather than a storage struct, and its W
/// in the slot's Offset word; a Gabor keeps its type in `NodeTexGabor` and
/// its Frequency, Anisotropy and Orientation in the Detail, Roughness,
/// Lacunarity and Color1 words.
#[test]
fn white_noise_and_gabor_with_their_settings() {
    let hashed = base("Hashed");
    assert_eq!((hashed.kind, hashed.variant), (BlendProcKind::WhiteNoiseFac, 4), "Value, four dimensions");
    assert_eq!((hashed.offset, hashed.scale), (1.5, 1.0), "W, and no Scale socket at all");

    let gabor = base("Gabored");
    assert_eq!((gabor.kind, gabor.variant), (BlendProcKind::GaborValue, 1), "Value, SHD_GABOR_TYPE_3D");
    assert_eq!((gabor.scale, gabor.detail, gabor.roughness), (12.0, 4.0, 0.25), "Scale, Frequency, Anisotropy");
    assert_eq!(gabor.colors[0], [0.0, 1.0, 0.0], "Orientation 3D");
}

/// A Math node between the texture and the socket (`svm_math`,
/// `NodeMathOperation`): before the walk went through one, the texture was
/// lost from the material altogether — `at` broke on a node `texture` has
/// no reader for and the material came out with no procedural at all. The
/// node's operation is in `bNode.custom1`, Clamp in `custom2`, and its
/// other two operands are sockets all three named `Value`, so only the
/// identifiers `Value_001` and `Value_002` tell them apart.
#[test]
fn a_math_node_between_a_texture_and_its_socket() {
    let mathed = base("Mathed");
    assert_eq!((mathed.kind, mathed.scale), (BlendProcKind::NoiseFac, 6.0), "the Noise behind it is still read");
    let math = mathed.math.expect("the Math node");
    assert_eq!((math.op, math.b, math.clamp), (2, 2.5, true), "NODE_MATH_MULTIPLY, by 2.5, clamped");
}
