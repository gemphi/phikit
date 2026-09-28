//! A camera, a light or an empty is picked in the viewport by what is drawn
//! for it (`scene::pick::extra`): a click on one of its lines makes it the
//! active, selected object — over the mesh behind it — and a box round its
//! origin selects it. Over the factory scene (`make_parented.py`).

use gpui::{Modifiers, MouseButton, MouseDownEvent, MouseUpEvent, Point, TestAppContext, point, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::camera::setup_from_state;
use phiber::v2::scene::pick::BPickView;
use phiber::v2::scene::pick::extra::BPickExtra;
use phiber::v2::scene::select::BSelection;
use phiber::v2::scene::source::with_loaded;
use phiber::v2::scene::view_slot::BViewSlot;
use phiber::v2::scene::{view_pan, view_pitch, view_yaw, view_zoom};
use phiber::v2::state::{BKey as _, CtxKey, ModalKey, View3dKey};
use phiber::v2::view::BAction;
use glam::Vec3;
use std::path::PathBuf;

async fn scene(cx: &mut TestAppContext) -> (gpui::Entity<BPhiberApp>, &mut gpui::VisualTestContext) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parented.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    let key = path.to_string_lossy().to_string();
    let model = SceneModel::from_blend_owned(scene, &key);
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(&phiber::v2::scene::loaded_path_key(), key.clone());
        app.scene_cache = Some((key, std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    app.update(cx, |app, cx| {
        app.handle_action(&BAction::Set(CtxKey::Object.path(), "Cube".into()), cx);
        // Far enough out that the camera and the light are in view.
        app.handle_action(&BAction::Set(View3dKey::Zoom.path(), "0.3".into()), cx);
    });
    cx.run_until_parked();
    (app, cx)
}

/// Where a world point lands in the window, through the viewport as drawn.
fn landing(app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext, p: Vec3) -> Point<gpui::Pixels> {
    let canvas = cx.debug_bounds("view3d-canvas").expect("the viewport");
    let (w, h) = (f32::from(canvas.size.width), f32::from(canvas.size.height));
    let view = app.read_with(cx, |a, _| {
        let scope = phiber::v2::space::BSpaceList::active(&a.state).slug();
        BViewSlot::new(scope, "Center").with(|| BPickView {
            yaw: view_yaw(&a.state),
            pitch: view_pitch(&a.state),
            pan: view_pan(&a.state),
            zoom: view_zoom(&a.state),
            setup: setup_from_state(&a.state),
            aspect: w / h,
        })
    });
    let (su, sv) = view.project(p).expect("in front of the eye");
    assert!(su.abs() < 0.45 && sv.abs() < 0.45, "{p} is in view: {su}, {sv}");
    point(canvas.origin.x + px((su + 0.5) * w), canvas.origin.y + px((sv + 0.5) * h))
}

fn extra(app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext, name: &str) -> BPickExtra {
    app.read_with(cx, |a, _| with_loaded(|m| BPickExtra::list(&a.state, m.unwrap())))
        .into_iter()
        .find(|e| e.name == name)
        .unwrap_or_else(|| panic!("{name} is pickable"))
}

#[gpui::test]
async fn a_click_on_a_lights_lines_picks_the_light(cx: &mut TestAppContext) {
    let (app, cx) = scene(cx).await;
    let light = extra(&app, cx, "Light");
    // The middle of one of its lines.
    let (a, b) = light.lines[0];
    let at = landing(&app, cx, (a + b) * 0.5);
    cx.simulate_click(at, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Object).map(str::to_string)).as_deref(), Some("Light"));
    assert!(app.read_with(cx, |a, _| BSelection::now(&a.state).contains("Light")), "and selected");

    // Shift-click on the camera's frame adds it; the light stays selected.
    let camera = extra(&app, cx, "Camera");
    let (a, b) = camera.lines[0];
    let at = landing(&app, cx, (a + b) * 0.5);
    cx.simulate_click(at, Modifiers::shift());
    cx.run_until_parked();
    let selection = app.read_with(cx, |a, _| BSelection::now(&a.state));
    assert!(selection.contains("Camera") && selection.contains("Light"), "{selection:?}");
}

#[gpui::test]
async fn a_box_round_the_cameras_origin_selects_the_camera(cx: &mut TestAppContext) {
    let (app, cx) = scene(cx).await;
    let camera = extra(&app, cx, "Camera");
    let centre = landing(&app, cx, camera.origin);
    cx.simulate_keystrokes("b");
    cx.run_until_parked();
    let (from, to) = (point(centre.x - px(15.0), centre.y - px(15.0)), point(centre.x + px(15.0), centre.y + px(15.0)));
    cx.simulate_event(MouseDownEvent { position: from, modifiers: Modifiers::none(), button: MouseButton::Left, click_count: 1, first_mouse: false });
    cx.simulate_mouse_move(from + point(px(12.0), px(12.0)), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(to, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_event(MouseUpEvent { position: to, modifiers: Modifiers::none(), button: MouseButton::Left, click_count: 1 });
    cx.run_until_parked();
    let selection = app.read_with(cx, |a, _| BSelection::now(&a.state));
    assert!(selection.contains("Camera"), "{selection:?}");
    assert!(!selection.contains("Cube"), "the box was round the camera alone: {selection:?}");
}
