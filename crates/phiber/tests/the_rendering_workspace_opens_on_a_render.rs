//! The Rendering workspace opens on a render of the scene — its own test
//! binary, since the render job is one per process.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::render_job;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BKey as _, CtxKey};

/// Opening the Rendering workspace with a scene loaded renders it into the
/// Render Result at once, so the Image editor shows the scene rather than
/// New / Open (`BPhiberApp::render_on_entry`); another workspace does not.
#[gpui::test]
async fn opening_the_rendering_workspace_renders_the_scene(cx: &mut TestAppContext) {
    use phiber::phiphx::blend::BlendFile;
    use phiber::phiphx::scene::SceneModel;
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parented.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    let model = SceneModel::from_blend_owned(scene, &path.to_string_lossy());
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(phiber::v2::state::ModalKey::Open);
        app.state.set(&phiber::v2::scene::loaded_path_key(), path.to_string_lossy().to_string());
        app.state.set(phiber::v2::state::ButsKey::Value("cycles.samples"), phiber::v2::state::val::BValue::Int(1).encode());
        app.scene_cache = Some((path.to_string_lossy().to_string(), std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    assert!(render_job::shot().0.is_none(), "Layout renders nothing by itself");
    let before = render_job::shot().1;
    app.update(cx, |app, cx| app.handle_action(&phiber::v2::view::BAction::Set(CtxKey::Space.path(), BSpaceType::Rendering.slug().into()), cx));
    cx.run_until_parked();
    let (shot, seq) = render_job::shot();
    let shot = shot.expect("the Rendering workspace started a render");
    assert!(seq > before && shot.w > 0 && shot.h > 0, "{} × {}", shot.w, shot.h);
    assert_eq!(app.read_with(cx, |a, _| a.state.get(phiber::v2::state::ModalKey::Open).map(str::to_string)), None, "no render window: the result is the workspace's");

    // While it renders, the window is drawn when a sample lands — by the
    // same task that serves the viewport (`ensure_ray_ticker`) — rather
    // than every vsync, which laid the whole window out sixty times a
    // second for the minutes a render takes; the task ends with the render.
    assert!(app.read_with(cx, |a, _| a.draws_on_frames()), "the render started the ticker");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while render_job::rendering() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(50));
        cx.executor().advance_clock(std::time::Duration::from_millis(8));
        cx.run_until_parked();
    }
    assert!(!render_job::rendering(), "the render never finished");
    for _ in 0..3 {
        cx.executor().advance_clock(std::time::Duration::from_millis(8));
        cx.run_until_parked();
    }
    assert!(!app.read_with(cx, |a, _| a.draws_on_frames()), "the ticker ends with the render");
}
