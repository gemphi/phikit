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
    // The Blend as a width in cosine, as EEVEE precomputes it
    // (`spot_blend = (1 - spot_size) * spotblend`): the file's 0.6 of the
    // way from the cone's edge to straight ahead.
    let want = (1.0 - cone.cone) * 0.6;
    assert!((cone.blend - want).abs() < 1e-6, "{} vs {want}", cone.blend);
    // Blend 0 is a hard edge; a kind with no cone carries none.
    assert_eq!((of("Sharp").blend, of("Bulb").blend), (1e-8, 0.0));
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

/// A spot's cone follows its object's scale, so the record carries
/// `scale.z / scale.xy` and its own axes — EEVEE's `spot_size_inv`
/// (`eevee_light.cc`), which makes the cone's cross-section an ellipse
/// (`spot_mask` in `light.wgsl`, `rendering.md` §50). `Squashed` is a spot
/// whose object is 1.5× in Y.
#[test]
fn a_scaled_spots_cone_reaches_the_record_as_an_ellipse() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../phiphx/tests/fixtures/LIGHTS.blend");
    let model = Phx::open(std::path::Path::new(path)).expect("fixture loads");
    let of = |name: &str| model.lights.iter().find(|l| l.name == name).unwrap_or_else(|| panic!("{name}")).record();

    let squashed = of("Squashed");
    assert_eq!(squashed.kind, 2);
    let [sx, sy] = squashed.spot_scale;
    assert!((sx - 1.0).abs() < 1e-6 && (sy - 1.0 / 1.5).abs() < 1e-6, "scale.z over scale.xy: {sx} {sy}");
    // The ellipse needs the light's own axes, so a spot shares `right`
    // with an area light even though it has no plane of its own.
    assert!((squashed.right[0].abs() - 1.0).abs() < 1e-6, "its own X axis, normalized: {:?}", squashed.right);

    // An unscaled spot is round — `[1, 1]` is what leaves `spot_mask` at
    // batch 147's plain smoothstep.
    assert_eq!(of("Cone").spot_scale, [1.0, 1.0], "an unscaled cone stays round");
    // The kinds with no cone carry none of it.
    assert_eq!((of("Sun").spot_scale, of("Bulb").spot_scale, of("Panel").spot_scale), ([0.0; 2], [0.0; 2], [0.0; 2]));
}
