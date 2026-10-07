//! The world as it lights a render (`blend/shade/world.rs`): the Background
//! node's Color × Strength, and what an Environment Texture does to it.
//!
//! The case that matters is a file whose HDRI is not beside it.
//! `THYMUS-BOLDROID-WAVE-003.blend` is one: its Background Color is linked
//! to an Environment Texture pointing at
//! `//..\..\..\CW\CW-Posing\HDRIs\lonely_road_afternoon_puresky_4k.exr`,
//! which is not shipped. Blender folds a missing image to magenta — the
//! `IMB` "invalid" colour every missing texture takes — and since the world
//! is a dome, that lights the whole scene magenta. Reading it as white
//! instead made the scene about twice as bright and dropped the cast
//! (`rendering.md` §52).

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
fn a_missing_environment_image_lights_the_scene_magenta_as_blender_does() {
    let w = model("THYMUS-BOLDROID-WAVE-003.blend").world.background;
    assert_eq!(w.color, [1.0, 0.0, 1.0], "Blender's missing-image magenta, not white");
    assert_eq!(w.strength, 1.0);
    assert_eq!(w.radiance(), [1.0, 0.0, 1.0], "and that is what lights every ray");
    // The renderer is told there is no image, so it lights from `color`
    // rather than hunting for a file Blender would not find either.
    assert_eq!(w.texture, None, "a file that is not there is not an image to load");
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
