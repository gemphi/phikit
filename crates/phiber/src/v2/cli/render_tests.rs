//! The parts of a headless render that need no GPU: the frame walk, the
//! sample default, and the `times.txt` line the comparisons parse.

use super::*;

#[test]
fn the_frame_walk_takes_a_range_the_way_ctrl_f12_does() {
    // `end` equal to `start` is **one** frame, not "unset" — reading it as
    // unset is what once rendered a hundred frames when one was asked for.
    assert_eq!(BHeadlessRender::range(5.0, 5.0, 1.0, 250.0), vec![5.0]);
    assert_eq!(BHeadlessRender::range(1.0, 4.0, 1.0, 250.0), vec![1.0, 2.0, 3.0, 4.0]);
    // An `end` below `start` means the file's own end.
    assert_eq!(BHeadlessRender::range(1.0, 0.0, 1.0, 3.0), vec![1.0, 2.0, 3.0]);
    // A step walks it, and a zero or negative step is one.
    assert_eq!(BHeadlessRender::range(1.0, 7.0, 3.0, 250.0), vec![1.0, 4.0, 7.0]);
    assert_eq!(BHeadlessRender::range(1.0, 3.0, 0.0, 250.0), vec![1.0, 2.0, 3.0]);
}

fn render(samples: Option<u32>, draft: bool) -> BHeadlessRender {
    BHeadlessRender {
        file: "x.blend".into(),
        out: "/tmp/x".into(),
        frame: vec![1.0],
        samples,
        draft,
        camera: None,
        look: Default::default(),
    }
}

#[test]
fn a_headless_render_takes_the_samples_the_command_line_takes() {
    assert_eq!(render(None, false).sample_count(), 2048, "silent: the default");
    assert_eq!(render(Some(64), false).sample_count(), 64, "asked: honoured");
    assert_eq!(render(Some(4096), false).sample_count(), 4096, "asked: honoured");
    assert_eq!(render(Some(64), true).sample_count(), 64, "--draft changes nothing");
}

#[test]
fn the_times_line_is_the_shape_the_comparisons_parse() {
    // `frame posed_ms render_ms samples`, which `animation_compare` and
    // `truth/blender_animation.py` both read positionally.
    let cost = BFrameCost { frame: 7.0, posed_ms: 12.4, drew_ms: 980.6, samples: 2048 };
    assert_eq!(cost.line(), "7 12 981 2048\n");
}

#[test]
fn the_measurement_switches_are_all_off_unless_asked_for() {
    let plain = BLookOverride::default();
    assert!(!plain.no_world && !plain.no_lights);
    assert!(plain.view.is_none() && plain.material.is_none() && plain.denoise.is_none());
    // A render for its own sake must not divide the resolution.
    assert_eq!(render(None, false).look.scale.max(1), 1);
}

/// `-o` names frames Blender's way: the last component is a prefix, not a
/// folder. `-o …\pose26_phiray_` made a *directory* of that name and wrote
/// `f0001.png` into it, so the file the caller asked for never appeared.
#[test]
fn the_output_path_is_a_name_prefix_unless_it_ends_in_a_separator() {
    let p = |out: &str, frame: i64| BHeadlessRender::frame_path(out, frame).1.to_string_lossy().replace('\\', "/");
    let d = |out: &str| BHeadlessRender::frame_path(out, 1).0.to_string_lossy().replace('\\', "/");

    // A prefix: the frame follows the name, in the folder above it.
    assert_eq!(p("/renders/pose26_phiray_", 1), "/renders/pose26_phiray_0001.png");
    assert_eq!(d("/renders/pose26_phiray_"), "/renders");
    assert_eq!(p("/renders/shot", 140), "/renders/shot0140.png");

    // A trailing separator means a folder, and the frame is the whole name.
    assert_eq!(p("/renders/", 7), "/renders/0007.png");
    assert_eq!(d("/renders/"), "/renders");

    // `####` sets its own padding, wherever it sits in the name.
    assert_eq!(p("/renders/img_##_test", 4), "/renders/img_04_test.png");
    assert_eq!(p("/renders/f######", 12), "/renders/f000012.png");

    // A bare name with no folder lands beside the working directory.
    assert_eq!(d("shot_"), ".");
}
