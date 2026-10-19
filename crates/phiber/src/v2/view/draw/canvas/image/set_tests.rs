//! Tests for `set` — split out so the module itself sits under the
//! 240-line read cap (`tests/level_structure.rs`).

use super::*;

#[test]
fn channels_and_passes_pick_the_bytes_blender_shows() {
    let shot = BRenderShot { w: 1, h: 1, bgra: vec![10, 20, 30, 128], depth: vec![0.5], normal: vec![[1.0, 0.0, -1.0]], albedo: vec![[0.0, 1.0, 0.0]], ..Default::default() };
    assert_eq!(pixels(&shot, "Combined", BChannels::Color), vec![10, 20, 30, 255]);
    assert_eq!(pixels(&shot, "Combined", BChannels::ColorAlpha), vec![10, 20, 30, 128]);
    assert_eq!(pixels(&shot, "Combined", BChannels::Alpha), vec![128, 128, 128, 255]);
    assert_eq!(pixels(&shot, "Combined", BChannels::Red), vec![30, 30, 30, 255]);
    assert_eq!(pixels(&shot, DEPTH, BChannels::Color), vec![0, 0, 0, 255], "one hit is the nearest, so black");
    assert_eq!(pixels(&shot, NORMAL, BChannels::Color), vec![0, 0, 255, 255], "a negative component clips to black");
    assert_eq!(pixels(&shot, DIFFUSE_COLOR, BChannels::Green), vec![255, 255, 255, 255]);
}

#[test]
fn the_data_passes_are_read_for_the_eye() {
    // Three pixels: near, far, and a miss.
    let shot = BRenderShot {
        w: 3,
        h: 1,
        bgra: vec![0; 12],
        depth: vec![2.0, 6.0, 1e10],
        normal: vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.7, 0.0, 0.3]],
        albedo: vec![[1.0, 0.0, 0.0]; 3],
        ..Default::default()
    };
    let depth = pixels(&shot, DEPTH, BChannels::ColorAlpha);
    assert_eq!(&depth[0..4], &[0, 0, 0, 255], "the nearest hit is black");
    assert_eq!(&depth[4..8], &[255, 255, 255, 255], "the farthest is white, not clipped past a metre");
    assert_eq!(&depth[8..12], &MISS, "and nothing hit is the field, clear");
    let normal = pixels(&shot, NORMAL, BChannels::Color);
    assert_eq!(&normal[0..4], &[0, 255, 0, 255]);
    assert_eq!(&normal[8..12], &[128, 128, 128, 255], "the kernel's leftover behind the scene is not shown as a normal");
    assert_eq!(&pixels(&shot, DIFFUSE_COLOR, BChannels::Color)[8..12], &[128, 128, 128, 255]);
}


#[test]
fn the_info_bar_reads_as_blenders() {
    let mut view = BImageView { zoom: None, pan: (0.0, 0.0), channels: BChannels::Color, pass: "Combined".into(), slot_filled: true, status: None, saved: false, phase: crate::v2::scene::BRenderPhase::Rendering, shown: crate::v2::part::image::shown::BShownImage::RenderResult };
    assert_eq!(view.info_text(&("Scene".into(), "ViewLayer".into()), "1"), None, "no render, no bar");
    view.status = Some(render_status::BRenderStatus { w: 4, h: 4, taken: 3, target: 8, done: false, has_image: true, depth: false, normal: false, albedo: false, bytes: 64, seq: 1 });
    let text = view.info_text(&("Scene".into(), "ViewLayer".into()), "12").unwrap();
    assert!(text.starts_with("Frame:12 | Time:") && text.ends_with("| Mem:1M, Peak: 1M | Scene, ViewLayer | Sample 3/8"), "{text}");
    // The tail is the PHASE's line, not a re-reading of `done`, so a
    // test that changes what the screen is has to say so — which is the
    // point: eight states, one value, and no second opinion about which
    // one is up (`scene::render_phase`).
    view.status.as_mut().unwrap().done = true;
    view.phase = crate::v2::scene::BRenderPhase::Finished;
    assert!(view.info_text(&("Scene".into(), "ViewLayer".into()), "12").unwrap().ends_with("| Finished"));
    view.phase = crate::v2::scene::BRenderPhase::Saved;
    assert!(view.info_text(&("Scene".into(), "ViewLayer".into()), "12").unwrap().ends_with("| Saved"));
    view.phase = crate::v2::scene::BRenderPhase::Cancelled;
    assert!(view.info_text(&("Scene".into(), "ViewLayer".into()), "12").unwrap().ends_with("| Cancelled at sample 3/8"));
}
