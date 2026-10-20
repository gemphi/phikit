//! The world as it lights a render (`blend/shade/world.rs`): the Background
//! node's Color × Strength, and what an Environment Texture does to it.
//!
//! The case that matters is a file whose HDRI is not beside it.
//! `THYMUS-BOLDROID-WAVE-003.blend` is one: its Background Color is linked
//! to an Environment Texture pointing at
//! `//..\..\..\CW\CW-Posing\HDRIs\lonely_road_afternoon_puresky_4k.exr`,
//! which is not shipped. Blender folds a missing image to magenta — the
//! `IMB` "invalid" colour every missing texture takes — and since the world
//! is a dome, that lights the whole scene magenta (`rendering.md` §52).
//!
//! phiber used to reproduce that. It does not any more, and deliberately:
//! pose 26 of CW-POSES (`autumn_field_puresky_4k.exr` at an `E:\` path from
//! the machine it was made on) came out of F12 magenta from floor to hair,
//! and nobody wants the picture Blender draws for a broken path. The world
//! records the missing file (`BlendWorld::missing`), the colour is the white
//! tint a stand-in environment is multiplied by, and the renderer lights
//! with Blender's own studio light (`forest`, what Material Preview lights
//! with); the read report says so.

use phiphx::phx::Phx;
use phiphx::scene::SceneModel;
use std::path::PathBuf;

/// Through `Phx::open`, not `parse_bytes`: whether an image is there is
/// answered against the `.blend`'s own folder, which only the open path
/// records (`blend::set_image_source`). Reading the file directly leaves
/// that unset, and every image then counts as present.
fn model(name: &str) -> SceneModel {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    Phx::open(&path).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

#[test]
fn a_missing_environment_image_is_recorded_and_lit_by_a_stand_in_not_magenta() {
    let w = model("THYMUS-BOLDROID-WAVE-003.blend").world.background;
    let missing = w.missing.as_deref().expect("the HDRI Blender could not find is named");
    assert!(missing.ends_with("lonely_road_afternoon_puresky_4k.exr"), "{missing}");
    assert_eq!(w.color, [1.0; 3], "the white tint a stand-in environment is multiplied by, not magenta");
    assert_eq!(w.strength, 1.0);
    // The renderer is told there is no image of the file's own to load.
    assert_eq!(w.texture, None, "a file that is not there is not an image to load");
    // And the read report says what happened and what was done about it.
    let report = model("THYMUS-BOLDROID-WAVE-003.blend").report;
    assert!(report.warnings.iter().any(|w| w.starts_with("World: cannot read") && w.contains("studio light")), "{:?}", report.warnings);
}

/// A world with no Environment Texture at all reads its Background Color
/// straight, and a file with no world node tree takes Blender's default
/// near-black grey.
#[test]
fn a_world_without_an_image_reads_its_own_colour() {
    let w = model("LIGHTS.blend").world.background;
    assert_eq!(w.texture, None);
    assert!(w.color.iter().all(|c| (*c - 0.05).abs() < 0.01), "Blender's default world grey: {:?}", w.color);
    assert_eq!(w.strength, 1.0);
}

/// The Background that lights is the one the World Output reaches, not the
/// first the file saved. `WORLD-CAMERA-SPLIT.blend` (made in Blender 5.2)
/// is pose 23_2's world in miniature: a spare Background and HDRI saved
/// first and linked to nothing, then a Mix Shader keyed on Light Path ▸ Is
/// Camera Ray choosing between the HDRI that lights (Strength 2.5, turned
/// 1.25 rad by its own Mapping) and one the camera alone sees. None of the
/// three images is shipped, so `missing` names the one that was read.
#[test]
fn the_world_is_lit_by_the_background_its_output_reaches() {
    let w = model("WORLD-CAMERA-SPLIT.blend").world.background;
    let read = w.missing.as_deref().expect("the lighting HDRI is named");
    assert!(read.ends_with("lights_the_scene.exr"), "the Mix's Fac-0 side lights every ray but the camera's: {read}");
    assert_eq!(w.strength, 2.5, "that Background's own Strength");
    assert!((w.rotation - 1.25).abs() < 1e-6, "the Mapping on that texture, not the first in the tree: {}", w.rotation);
}
