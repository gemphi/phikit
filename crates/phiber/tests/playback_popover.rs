//! The Timeline's Playback popover is Blender's `TIME_PT_playback` panel:
//! it opens from the header's Playback button, lays out its headed rows,
//! stays open while a box in it is ticked (a panel, not a menu), and goes
//! when a menu opens. What it holds comes from the file and goes back to
//! it on save.

use gpui::{Modifiers, MouseButton, TestAppContext, px, size};
use phiber::phiphx::blend::{BlendFile, BlendPlayback, BlendSyncMode, TIME_ALL_3D_WIN};
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::part::playback::settings;
use phiber::v2::state::{BKey as _, CtxKey, ModalKey, val::BValue};
use phiber::v2::view::BAction;
use std::path::PathBuf;

fn click(cx: &mut gpui::VisualTestContext, selector: &'static str) {
    let bounds = cx.debug_bounds(selector).unwrap_or_else(|| panic!("{selector} is painted"));
    cx.simulate_mouse_down(bounds.center(), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(bounds.center(), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
}

#[gpui::test]
async fn the_playback_popover_opens_stays_open_for_its_boxes_and_closes_for_a_menu(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        cx.notify();
    });
    cx.run_until_parked();

    let anchor = "btn-label-set-ctx.header.time.panel.Playback-true-text";
    assert!(cx.debug_bounds("section-Sync").is_none(), "closed until asked");
    click(cx, anchor);
    // Sync is a dropdown row (a split row); the switch groups' headings are
    // a line over their first switch, so the switches are what is painted.
    assert!(cx.debug_bounds("section-Sync").is_some(), "the Sync row is painted");
    for switch in ["switch-ctx.buts.use_audio_scrub", "switch-ctx.buts.use_play_3d_editors"] {
        assert!(cx.debug_bounds(switch).is_some(), "the {switch} row is painted");
    }
    // The factory settings: 3D Viewport plays, Scrubbing is off.
    assert!(app.read_with(cx, |a, _| settings::is_on(&a.state, "use_play_3d_editors") && !settings::is_on(&a.state, "use_audio_scrub")));
    // Tick Scrubbing: it turns on and the panel is still there. A boolean
    // row's control is a switch over its key (`view/draw/ctrl/mod.rs`), so
    // the target is `switch-<key>`, not a labelled button.
    click(cx, "switch-ctx.buts.use_audio_scrub");
    assert!(app.read_with(cx, |a, _| settings::is_on(&a.state, "use_audio_scrub")));
    assert!(cx.debug_bounds("section-Sync").is_some(), "a panel stays open while its boxes are ticked");
    assert!(cx.debug_bounds("switch-ctx.buts.use_audio_scrub").is_some(), "and the box is still there to untick");
    // A menu opening takes the panel down.
    app.update(cx, |app, cx| app.handle_action(&BAction::Toggle(phiber::v2::state::MenuKey::Top("time", "View").path()), cx));
    cx.run_until_parked();
    assert!(cx.debug_bounds("section-Sync").is_none(), "a menu opening closes the panel");
}

/// The settings come from the file and go back to it.
#[gpui::test]
async fn the_settings_are_read_from_the_file_and_written_back_on_save(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_playback_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/nla_cube.blend");
    let blend = dir.join("cube.blend");
    std::fs::copy(source, &blend).unwrap();
    let path = blend.to_string_lossy().to_string();
    let scene = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap();
    let in_file = scene.props.playback;
    assert_eq!(in_file.sync, BlendSyncMode::EveryFrame, "the fixture was saved with Play Every Frame");
    assert_eq!(in_file.play_in, BlendPlayback::default().play_in);
    let model = SceneModel::from_blend_owned(scene, &path);

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(&phiber::v2::scene::loaded_path_key(), path.clone());
        settings::seed(&mut app.state, &model.playback);
        app.scene_cache = Some((path.clone(), std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| settings::now(&a.state)), in_file, "what the file has, not the factory");

    // Frame Dropping, Limit to Frame Range, only the 3D Viewport playing — then Ctrl S.
    app.update(cx, |app, cx| {
        app.handle_action(&BAction::Set(settings::key(settings::SYNC_KEY), BValue::Enum("FRAME_DROP".into()).encode()), cx);
        app.handle_action(&BAction::Set(settings::key("lock_frame_selection_to_range"), BValue::Bool(true).encode()), cx);
        app.handle_action(&BAction::Set(settings::key("use_play_animation_editors"), BValue::Bool(false).encode()), cx);
        // The record button, its mode and Time Jump go to the file too.
        app.handle_action(&BAction::Set(CtxKey::AutoKey.path(), "true".into()), cx);
        app.handle_action(&BAction::Set(settings::key(settings::AUTO_KEY_MODE_KEY), BValue::Enum("REPLACE_KEYS".into()).encode()), cx);
        app.handle_action(&BAction::Set(settings::key(settings::JUMP_UNIT_KEY), BValue::Enum("FRAME".into()).encode()), cx);
        app.handle_action(&BAction::Set(settings::key(settings::JUMP_DELTA_KEY), BValue::Float(4.0).encode()), cx);
    });
    cx.run_until_parked();
    cx.simulate_keystrokes("ctrl-s");
    cx.run_until_parked();
    let saved = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap().props.playback;
    assert_eq!(saved.sync, BlendSyncMode::FrameDrop);
    assert!(saved.lock_frame_selection);
    assert_eq!(saved.play_in, TIME_ALL_3D_WIN);
    assert!(saved.auto_key && saved.auto_key_replace, "record on, Replace");
    assert!(saved.jump_in_frames && saved.jump_delta == 4.0, "Time Jump: 4 frames");
    assert_eq!(saved, app.read_with(cx, |a, _| settings::now(&a.state)));
    let _ = std::fs::remove_dir_all(&dir);
}
