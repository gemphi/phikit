//! G, R and S in the 3D Viewport (`transform.translate` / `rotate` /
//! `resize`): the selected objects follow the pointer from where it first
//! stood, their children with them; X / Y / Z hold the change to a world
//! axis; a click keeps it as one undo step, Escape puts it back. Over the
//! factory scene with Arm parented to the Cube (`make_parented.py`).

use gpui::{Modifiers, MouseButton, Point, TestAppContext, point, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::camera::setup_from_state;
use phiber::v2::scene::pick::BPickView;
use phiber::v2::scene::transform::BTransforms;
use phiber::v2::scene::view_slot::BViewSlot;
use phiber::v2::scene::xform::BTransformModal;
use phiber::v2::scene::{BObject3D, scene_objects, view_pan, view_pitch, view_yaw, view_zoom};
use phiber::v2::state::{BKey as _, BState, CtxKey, ModalKey};
use phiber::v2::view::BAction;
use glam::{Mat4, Vec3};
use std::path::PathBuf;

fn world(s: &BState, name: &str) -> Mat4 {
    match scene_objects(s).iter().find(|o| o.name() == name).unwrap_or_else(|| panic!("{name}")) {
        BObject3D::Loaded { matrix_world, .. } => Mat4::from_cols_array_2d(matrix_world),
        BObject3D::Primitive { .. } => panic!("{name} is a loaded object"),
    }
}

fn at(s: &BState, name: &str) -> Vec3 {
    world(s, name).w_axis.truncate()
}

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
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(CtxKey::Object.path(), "Cube".into()), cx));
    cx.run_until_parked();
    (app, cx)
}

/// The viewport as drawn, and where a world point lands in the window.
fn landing(app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext, p: Vec3) -> (Point<gpui::Pixels>, BPickView) {
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
    (point(canvas.origin.x + px((su + 0.5) * w), canvas.origin.y + px((sv + 0.5) * h)), view)
}

fn step(cx: &mut gpui::VisualTestContext, to: Point<gpui::Pixels>) {
    cx.simulate_mouse_move(to, None, Modifiers::none());
    cx.run_until_parked();
}

fn offset(p: Point<gpui::Pixels>, dx: f32, dy: f32) -> Point<gpui::Pixels> {
    point(p.x + px(dx), p.y + px(dy))
}

#[gpui::test]
async fn g_moves_the_selection_with_the_pointer_and_a_click_keeps_it_as_one_step(cx: &mut TestAppContext) {
    let (app, cx) = scene(cx).await;
    let (cube0, arm0) = app.read_with(cx, |a, _| (at(&a.state, "Cube"), at(&a.state, "Arm")));
    let (centre, _) = landing(&app, cx, cube0);
    step(cx, centre);
    cx.simulate_keystrokes("g");
    cx.run_until_parked();
    assert!(app.read_with(cx, |a, _| BTransformModal::active(&a.state)), "G starts the move");
    step(cx, centre);
    step(cx, offset(centre, 80.0, 0.0));
    let (cube1, arm1) = app.read_with(cx, |a, _| (at(&a.state, "Cube"), at(&a.state, "Arm")));
    assert!((cube1 - cube0).length() > 0.1, "the Cube follows the pointer: {cube0} → {cube1}");
    assert!((cube1.z - cube0.z).abs() < 1e-4, "sideways on screen stays level");
    assert!(((arm1 - arm0) - (cube1 - cube0)).length() < 1e-4, "its child comes along");
    // The Cube's origin is under the pointer.
    let (under, _) = landing(&app, cx, cube1);
    assert!((f32::from(under.x) - f32::from(centre.x) - 80.0).abs() < 0.5 && (under.y - centre.y).abs() < px(0.5));

    // X: along world X alone.
    cx.simulate_keystrokes("x");
    step(cx, offset(centre, 60.0, -40.0));
    let cube2 = app.read_with(cx, |a, _| at(&a.state, "Cube"));
    assert!((cube2.y - cube0.y).abs() < 1e-4 && (cube2.z - cube0.z).abs() < 1e-4 && (cube2.x - cube0.x).abs() > 0.05, "held to X: {cube2}");
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Object).map(str::to_string)).as_deref(), Some("Cube"), "X did not delete it");

    // A click keeps it; one Undo puts all of it back.
    let here = offset(centre, 60.0, -40.0);
    cx.simulate_mouse_down(here, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(here, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert!(!app.read_with(cx, |a, _| BTransformModal::active(&a.state)));
    let kept = app.read_with(cx, |a, _| BTransforms::now(&a.state).get("Cube")).expect("kept");
    assert!((kept.location[0] - cube2.x).abs() < 1e-4, "the edit holds the move");
    cx.simulate_keystrokes("ctrl-z");
    cx.run_until_parked();
    let back = app.read_with(cx, |a, _| at(&a.state, "Cube"));
    assert!((back - cube0).length() < 1e-4, "one undo step: {back} vs {cube0}");
}

#[gpui::test]
async fn escape_puts_it_back(cx: &mut TestAppContext) {
    let (app, cx) = scene(cx).await;
    let cube0 = app.read_with(cx, |a, _| at(&a.state, "Cube"));
    let (centre, _) = landing(&app, cx, cube0);
    step(cx, centre);
    cx.simulate_keystrokes("g");
    step(cx, centre);
    step(cx, offset(centre, 50.0, 30.0));
    assert!(app.read_with(cx, |a, _| (at(&a.state, "Cube") - cube0).length()) > 0.05);
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(!app.read_with(cx, |a, _| BTransformModal::active(&a.state)));
    assert!(app.read_with(cx, |a, _| (at(&a.state, "Cube") - cube0).length()) < 1e-5, "as it was");
    assert!(app.read_with(cx, |a, _| BTransforms::now(&a.state).get("Cube")).is_none(), "no edit left behind");
}

#[gpui::test]
async fn r_turns_about_the_view_axis_and_s_scales_by_the_pointers_reach(cx: &mut TestAppContext) {
    let (app, cx) = scene(cx).await;
    let cube0 = app.read_with(cx, |a, _| at(&a.state, "Cube"));
    let (centre, view) = landing(&app, cx, cube0);

    // R: from right of the Cube to above it — a quarter turn, counter-clockwise as seen.
    step(cx, centre);
    cx.simulate_keystrokes("r");
    step(cx, offset(centre, 60.0, 0.0));
    step(cx, offset(centre, 0.0, -60.0));
    let (_, _, back) = view.axes();
    let x_axis = app.read_with(cx, |a, _| world(&a.state, "Cube").x_axis.truncate().normalize());
    let expected = Mat4::from_axis_angle(back, std::f32::consts::FRAC_PI_2).transform_vector3(Vec3::X);
    assert!((x_axis - expected).length() < 1e-3, "{x_axis} vs {expected}");
    assert!(app.read_with(cx, |a, _| (at(&a.state, "Cube") - cube0).length()) < 1e-4, "it turns in place");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert!(!app.read_with(cx, |a, _| BTransformModal::active(&a.state)), "Return keeps it");

    // S: twice as far from the Cube as it started — twice the size.
    cx.simulate_keystrokes("s");
    step(cx, offset(centre, 50.0, 0.0));
    step(cx, offset(centre, 100.0, 0.0));
    let scale = app.read_with(cx, |a, _| BTransforms::now(&a.state).get("Cube")).expect("scaled").scale;
    assert!(scale.iter().all(|s| (s - 2.0).abs() < 1e-3), "{scale:?}");
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    let turned = app.read_with(cx, |a, _| BTransforms::now(&a.state).get("Cube")).expect("the turn stays");
    assert!(turned.scale.iter().all(|s| (s - 1.0).abs() < 1e-4), "Escape undoes the scale, not the turn before it");
}
