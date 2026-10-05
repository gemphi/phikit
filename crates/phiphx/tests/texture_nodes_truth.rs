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
