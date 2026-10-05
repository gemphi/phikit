//! A file's light shapes reach the tracer's light records
//! (`rendered/world.rs`'s `light`), which is what gives each lamp a
//! penumbra the width of its own shape (`phiray`'s `shadow.wgsl`,
//! `rendering.md` §44). A sun's shape is its Angle in radians; the other
//! kinds carry a length.

use phiber::phiphx::Phx;
use phiber::v2::view::draw::canvas::view3d::rendered::BLamp;

#[test]
fn a_suns_angle_and_the_other_shapes_come_from_the_file() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../phiphx/tests/fixtures/LIGHTS.blend");
    let model = Phx::open(std::path::Path::new(path)).expect("fixture loads");
    let of = |name: &str| {
        let l = model.lights.iter().find(|l| l.name == name).unwrap_or_else(|| panic!("{name}"));
        l.record()
    };
    // Kinds: 0 sun, 1 point, 2 spot, 3 area.
    let sun = of("Sun");
    assert_eq!(sun.kind, 0);
    assert!((sun.size - 10.0_f32.to_radians()).abs() < 1e-6, "the file's 10° Angle, not Blender's 0.526° default: {}", sun.size);
    assert_eq!((of("Bulb").kind, of("Bulb").size), (1, 0.4));
    assert_eq!((of("Panel").kind, of("Panel").size), (3, 2.5));
    let cone = of("Cone");
    assert_eq!((cone.kind, cone.size), (2, 0.2));
    assert!((cone.cone - 30.0_f32.to_radians().cos()).abs() < 1e-6, "the spot's half-angle cosine: {}", cone.cone);
}

/// An area light's shape reaches the record with its own axes and the
/// object's scale in its edges, as EEVEE's does (`eevee_light.cc`), since
/// the taps spread over that shape in that plane (`shadow.wgsl`).
#[test]
fn an_area_lights_shape_axes_and_scale_reach_the_record() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../phiphx/tests/fixtures/LIGHTS.blend");
    let model = Phx::open(std::path::Path::new(path)).expect("fixture loads");
    let of = |name: &str| model.lights.iter().find(|l| l.name == name).unwrap_or_else(|| panic!("{name}")).record();

    // Strip: a Rectangle 3 × 0.5 on an object scaled 2 in X — the edge
    // along its own X carries that scale.
    let strip = of("Strip");
    assert_eq!((strip.kind, strip.shape), (3, phiray::AREA_RECT));
    assert_eq!((strip.size, strip.size_y), (6.0, 0.5), "the X edge scaled by the object's 2");
    assert!((strip.right[0].abs() - 1.0).abs() < 1e-6, "its own X axis, normalized: {:?}", strip.right);

    let round = of("Round");
    assert_eq!((round.shape, round.size, round.size_y), (phiray::AREA_DISK, 1.5, 1.5));
    let oval = of("Oval");
    assert_eq!((oval.shape, oval.size, oval.size_y), (phiray::AREA_ELLIPSE, 2.0, 0.75));
    // The kinds without a plane leave the area fields alone.
    let sun = of("Sun");
    assert_eq!((sun.shape, sun.size_y, sun.right), (phiray::AREA_SQUARE, 0.0, [0.0; 3]));
}
