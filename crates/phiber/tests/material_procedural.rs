//! A file's Noise / Voronoi textures reach the scene the viewport builds
//! from (`PROCEDURAL.blend`, `phiphx/tests/fixtures/make_procedural.py`):
//! the material the tracer's slots are filled from carries them
//! (`rendered/surface.rs`), the ramp baked, the Bump with its numbers.

use phiber::phiphx::Phx;
use phiber::phiphx::blend::shade::procedural::{BlendProcCoord, BlendProcKind};
use phiber::v2::scene::{BObject3D, current_scene};
use phiber::v2::state::BState;

#[test]
fn the_scene_carries_the_procedural_textures() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../phiphx/tests/fixtures/PROCEDURAL.blend");
    let model = Phx::open(std::path::Path::new(path)).expect("fixture loads");
    let scene = current_scene(&BState::default(), Some(&model));
    let material = |n: &str| match scene.iter().find(|o| matches!(o, BObject3D::Loaded { name, .. } if name == n)).expect(n) {
        BObject3D::Loaded { materials, .. } => materials[0].clone(),
        _ => unreachable!(),
    };
    let ramped = material("Ramped");
    let base = ramped.procedural.base.as_ref().expect("the ramped Noise on Base Color");
    assert_eq!((base.kind, base.coords, base.scale), (BlendProcKind::NoiseFac, BlendProcCoord::Object, 8.0));
    assert_eq!(base.ramp.as_ref().map(Vec::len), Some(1024));
    let bump = ramped.procedural.bump.as_ref().expect("the Voronoi Bump");
    assert_eq!((bump.height.kind, bump.strength, bump.distance), (BlendProcKind::VoronoiDistance, 0.6, 0.3));
    let mixed = material("Mixed");
    assert!(mixed.mix.is_some() && mixed.procedural.mix.as_ref().is_some_and(|p| p.kind == BlendProcKind::NoiseFac));
    assert_eq!(material("Colored").procedural.base.as_ref().map(|p| (p.kind, p.coords)), Some((BlendProcKind::VoronoiColor, BlendProcCoord::Uv)));
}
