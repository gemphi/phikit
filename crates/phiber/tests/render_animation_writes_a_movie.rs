//! Render ▸ Render Animation (Ctrl F12) with Output ▸ File Format set to
//! FFmpeg Video gathers every frame and writes one movie named as Blender
//! names it (`shot_0001-0003.mkv`), encoded by ffmpeg where the machine has
//! it and as a Motion-JPEG AVI where it does not — never a folder of
//! stills and no movie.

use gpui::{TestAppContext, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::video::BMovieJob;
use phiber::v2::state::val::BValue;
use phiber::v2::state::{BKey as _, ButsKey, CtxKey, ModalKey, OpKey};
use phiber::v2::view::BAction;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[gpui::test]
async fn ctrl_f12_with_ffmpeg_video_writes_one_movie(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_movie_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let blend = dir.join("cube.blend");
    std::fs::copy(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parented.blend"), &blend).unwrap();
    let scene = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap();
    let model = SceneModel::from_blend_owned(scene, &blend.to_string_lossy());

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(&phiber::v2::scene::loaded_path_key(), blend.to_string_lossy().to_string());
        phiber::v2::app::seed::seed_output_props(&mut app.state, &model);
        app.scene_cache = Some((blend.to_string_lossy().to_string(), std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    // Three tiny frames, one sample each, into `//out/shot_` as a movie.
    let set = |key: &'static str, v: &str| BAction::Set(ButsKey::Value(key).path(), v.to_string());
    let int = |key: &'static str, v: i64| BAction::Set(ButsKey::Value(key).path(), BValue::Int(v).encode());
    app.update(cx, |app, cx| {
        for action in [int("resolution_x", 64), int("resolution_y", 48), int("resolution_percentage", 100), int("cycles.samples", 1), set("filepath", "//out/shot_"), set("image_settings.file_format", "FFMPEG"), set("ffmpeg.format", "MKV"), set("ffmpeg.codec", "H264")] {
            app.handle_action(&action, cx);
        }
        app.state.set(CtxKey::FrameStart, "1");
        app.state.set(CtxKey::FrameEnd, "3");
    });
    cx.run_until_parked();

    app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::Render.path(), "animation".into()), cx));
    let expected = if BMovieJob::ffmpeg_present() { dir.join("out/shot_0001-0003.mkv") } else { dir.join("out/shot_0001-0003.avi") };
    let deadline = Instant::now() + Duration::from_secs(120);
    while !expected.exists() && Instant::now() < deadline {
        // The render thread needs real time; the app's poll timer, the test
        // clock's.
        std::thread::sleep(Duration::from_millis(100));
        cx.background_executor.advance_clock(Duration::from_millis(100));
        cx.run_until_parked();
    }
    assert!(expected.exists(), "the movie {} was written", expected.display());
    assert!(std::fs::metadata(&expected).unwrap().len() > 200, "and holds frames");
    assert!(!dir.join("out/shot_0001.png").exists(), "no stills beside it: the frames went into the movie");
    let _ = std::fs::remove_dir_all(&dir);
}
