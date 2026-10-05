//! A light's shape, read from the file (`blend/object/data.rs`,
//! `scene/import_extras.rs`) out of `LIGHTS.blend`
//! (`fixtures/make_lights.py`, Blender 5.0.1). The shape is what decides
//! how soft the light's shadow is (`phiray`'s `shadow.wgsl`), and each
//! light in the fixture carries one the default would not give: a sun's
//! Angle is `Light.sun_angle` in radians, not the `radius` the other kinds
//! use, and reading `radius` for a sun gave every sun Blender's default
//! 0.526° however wide the file's was.

use phiphx::blend::BlendFile;
use phiphx::scene::{SceneLight, SceneModel};
use std::path::PathBuf;

fn lights() -> Vec<SceneLight> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/LIGHTS.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    SceneModel::from_blend_owned(scene, &path.to_string_lossy()).lights
}

#[test]
fn every_kind_carries_the_shape_the_file_gives_it() {
    let lights = lights();
    let of = |name: &str| lights.iter().find(|l| l.name == name).unwrap_or_else(|| panic!("{name}"));

    let sun = of("Sun");
    assert_eq!(sun.kind, "SUN");
    assert!((sun.size - 10.0_f32.to_radians()).abs() < 1e-6, "the file's Angle, not the default: {}", sun.size);
    assert_eq!(sun.energy, 3.0);

    let bulb = of("Bulb");
    assert_eq!((bulb.kind.as_str(), bulb.size, bulb.energy), ("POINT", 0.4, 200.0));

    // An area light's size is its edge. Its stored `energy` is not the
    // Watts the UI shows: Blender keeps 4/π of it (500 W → 636.6), so the
    // reader passing the stored number straight through is right.
    let panel = of("Panel");
    assert_eq!((panel.kind.as_str(), panel.size), ("AREA", 2.5));
    assert!((panel.energy - 500.0 * 4.0 / std::f32::consts::PI).abs() < 0.1, "{}", panel.energy);

    let cone = of("Cone");
    assert_eq!((cone.kind.as_str(), cone.size), ("SPOT", 0.2));
    assert!((cone.spot_size - 60.0_f32.to_radians()).abs() < 1e-6, "{}", cone.spot_size);
    assert_eq!((panel.shape.as_str(), panel.size_y), ("SQUARE", 2.5), "a square's Size Y is its size");
}

/// An area light's shape and its second edge (`Light.area_shape`,
/// `area_sizey`). Blender reads Size Y only for a Rectangle and an
/// Ellipse, so a Disk's stays at the default and must read back as its
/// one size both ways (`eevee_light.cc`).
#[test]
fn an_area_lights_shape_and_its_second_edge() {
    let lights = lights();
    let of = |name: &str| lights.iter().find(|l| l.name == name).unwrap_or_else(|| panic!("{name}")).clone();

    let strip = of("Strip");
    assert_eq!((strip.kind.as_str(), strip.shape.as_str()), ("AREA", "RECTANGLE"));
    assert_eq!((strip.size, strip.size_y), (3.0, 0.5), "the file's two edges, before the object's scale");

    let round = of("Round");
    assert_eq!((round.shape.as_str(), round.size, round.size_y), ("DISK", 1.5, 1.5), "a disk takes one size both ways");

    let oval = of("Oval");
    assert_eq!((oval.shape.as_str(), oval.size, oval.size_y), ("ELLIPSE", 2.0, 0.75));
}
