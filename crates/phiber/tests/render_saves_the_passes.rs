//! A multi-layer EXR carries one layer per render pass, named Blender's way.
//!
//! The names are checked against Blender's own vocabulary in phiphx
//! (`pass_truth.rs`); what this checks is that the writer *uses* them, that a
//! pass the tracer cannot produce is left out rather than written as black,
//! and that each layer holds its own data rather than a copy of the image.

use phiber::v2::scene::render_job::BRenderShot;
use phiber::v2::scene::render_save::{BSaveKind, kind_of, save_to, written_pass};
use phiphx::blend::{BPassKind, BlendImageFormat, BlendOutput, BlendPassSet};
use std::collections::BTreeSet;

/// A 2 × 2 render with every pass the tracer can produce.
fn a_shot() -> BRenderShot {
    let linear: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0], [0.25, 0.5, 0.75], [1.0, 1.0, 1.0], [4.0, 2.0, 8.0]];
    let display: Vec<[f32; 3]> = linear.iter().map(|px| px.map(|v| v.clamp(0.0, 1.0).powf(1.0 / 2.2))).collect();
    let bgra: Vec<u8> = display.iter().flat_map(|px| [px[2], px[1], px[0]].map(|v| (v * 255.0).round() as u8).into_iter().chain([255u8])).collect();
    BRenderShot {
        w: 2,
        h: 2,
        bgra,
        display,
        linear,
        // Deliberately unlike the image, so a layer that is secretly the
        // combined pass cannot pass for the albedo.
        albedo: vec![[0.9, 0.1, 0.1], [0.1, 0.9, 0.1], [0.1, 0.1, 0.9], [0.5, 0.5, 0.5]],
        normal: vec![[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]],
        depth: vec![3.5, 7.25, 1.0e10, 12.0],
        taken: 16,
        target: 16,
        done: true,
    }
}

fn temp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("phiber_render_pass");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

/// Every channel name in the written file.
///
/// The `exr` crate groups a single part's channels into pseudo-layers by
/// their common prefix, so the names to compare are the channels' own —
/// which is also how a compositor sees Blender's files.
fn channels(file: &std::path::Path) -> BTreeSet<String> {
    let image = exr::prelude::read_all_flat_layers_from_file(file).unwrap();
    image.layer_data.iter().flat_map(|layer| layer.channel_data.list.iter().map(|c| c.name.to_string())).collect()
}

fn output() -> BlendOutput {
    BlendOutput { format: BlendImageFormat::OpenExrMultiLayer, depth: 32, ..Default::default() }
}

#[test]
fn a_multilayer_format_is_its_own_kind_of_write() {
    assert_eq!(kind_of(&output()), BSaveKind::ExrMultiLayer { half: false });
    // Its depth still clamps the way a plain EXR's does.
    assert_eq!(kind_of(&BlendOutput { depth: 16, ..output() }), BSaveKind::ExrMultiLayer { half: true });
}

#[test]
fn each_pass_becomes_a_layer_named_the_way_blender_names_it() {
    let shot = a_shot();
    let want = BlendPassSet { flag: BPassKind::Depth.bit() | BPassKind::Normal.bit() | BPassKind::DiffuseColor.bit() };
    let file = temp("passes.exr");
    save_to(&shot, &output(), want, &file).unwrap();
    let got = channels(&file);
    for name in [
        "ViewLayer.Combined.R", "ViewLayer.Combined.A",
        "ViewLayer.Depth.Z",
        "ViewLayer.Normal.X", "ViewLayer.Normal.Y", "ViewLayer.Normal.Z",
        "ViewLayer.Diffuse Color.R", "ViewLayer.Diffuse Color.G", "ViewLayer.Diffuse Color.B",
    ] {
        assert!(got.contains(name), "no channel {name}; the file has {got:?}");
    }
}

#[test]
fn a_pass_the_tracer_cannot_produce_is_left_out_not_written_black() {
    let shot = a_shot();
    // Mist and the two index passes are asked for but not traced.
    let want = BlendPassSet { flag: BPassKind::Mist.bit() | BPassKind::ObjectIndex.bit() | BPassKind::Depth.bit() };
    assert_eq!(written_pass(&shot, want), vec![BPassKind::Combined, BPassKind::Depth]);

    let file = temp("partial.exr");
    save_to(&shot, &output(), want, &file).unwrap();
    let got = channels(&file);
    assert!(got.contains("ViewLayer.Depth.Z"));
    // An empty layer in a compositor reads as a render that came out wrong,
    // which costs more time than a missing one does.
    assert!(!got.iter().any(|c| c.contains("Mist")), "an untraced pass was written anyway: {got:?}");
    assert!(!got.iter().any(|c| c.contains("Object Index")), "an untraced pass was written anyway: {got:?}");
}

#[test]
fn asking_for_nothing_still_writes_the_image() {
    let shot = a_shot();
    let file = temp("combined_only.exr");
    save_to(&shot, &output(), BlendPassSet { flag: 0 }, &file).unwrap();
    let got = channels(&file);
    assert!(got.contains("ViewLayer.Combined.R"));
    assert_eq!(got.len(), 4, "Combined alone is four channels, not {got:?}");
}

#[test]
fn the_layers_hold_their_own_data() {
    let shot = a_shot();
    let want = BlendPassSet { flag: BPassKind::Depth.bit() | BPassKind::DiffuseColor.bit() };
    let file = temp("distinct.exr");
    save_to(&shot, &output(), want, &file).unwrap();
    let image = exr::prelude::read_all_flat_layers_from_file(&file).unwrap();
    let plane = |name: &str| {
        let channel = image
            .layer_data
            .iter()
            .flat_map(|layer| layer.channel_data.list.iter())
            .find(|c| c.name.to_string() == name)
            .unwrap_or_else(|| panic!("no channel {name}"));
        match &channel.sample_data {
            exr::prelude::FlatSamples::F32(v) => v.clone(),
            other => panic!("{name} is not float: {other:?}"),
        }
    };
    // Depth carries the distances, including Cycles' 1e10 for a ray that met
    // nothing — not the image, and not zero.
    let depth = plane("ViewLayer.Depth.Z");
    assert_eq!(depth.len(), 4);
    assert!((depth[0] - 3.5).abs() < 1e-4, "{depth:?}");
    assert!(depth[2] > 1.0e9, "a miss should read 1e10, not {}", depth[2]);
    // The albedo is the albedo, not a second copy of the combined pass.
    let red = plane("ViewLayer.Diffuse Color.R");
    let combined = plane("ViewLayer.Combined.R");
    assert!((red[0] - 0.9).abs() < 1e-4, "{red:?}");
    assert!((red[0] - combined[0]).abs() > 0.5, "the Diffuse Color layer looks like the Combined one");
    // Combined is scene-linear, so the over-bright pixel survives here too.
    assert!(combined[3] > 3.5, "{combined:?}");
}


#[test]
fn each_pass_is_its_own_part_the_way_blender_writes_one() {
    // The first version of this writer put every channel in one part. The
    // names all read back correctly and the channel test passed — but a
    // reader groups a single part into one layer, so the passes vanished.
    // Blender writes one part per pass (`R_IMF_EXR_FLAG_MULTIPART`), so the
    // structure is checked as well as the names.
    let shot = a_shot();
    let want = BlendPassSet { flag: BPassKind::Depth.bit() | BPassKind::Normal.bit() | BPassKind::DiffuseColor.bit() };
    let file = temp("parts.exr");
    save_to(&shot, &output(), want, &file).unwrap();
    let image = exr::prelude::read_all_flat_layers_from_file(&file).unwrap();
    let names: Vec<String> = image.layer_data.iter().map(|layer| layer.attributes.layer_name.clone().map(|n| n.to_string()).unwrap_or_default()).collect();
    assert_eq!(
        names,
        vec!["ViewLayer.Combined", "ViewLayer.Depth", "ViewLayer.Normal", "ViewLayer.Diffuse Color"],
        "the passes should be separate parts, in Blender's order"
    );
    for layer in image.layer_data.iter() {
        assert_eq!(layer.size, exr::prelude::Vec2(2, 2));
    }
}
