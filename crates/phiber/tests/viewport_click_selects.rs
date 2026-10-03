//! A click on a mesh in the 3D viewport selects it: the object under the
//! pointer becomes the active object, the Outliner's row for it lights —
//! the active row a step lighter than the other selected rows — and the
//! viewport outlines it, whichever painter, CPU or GPU, draws the surfaces.
//! A click on empty canvas deselects everything, the active object staying
//! active (Blender's `deselect_all`).
//!
//! Blender ref: `view3d_select.cc`, left-click select.

use gpui::{Modifiers, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::part::outliner::BOutlinerItem;
use phiber::v2::scene::camera::{setup_from_state, with_view};
use phiber::v2::scene::pick::{BPickView, pick};
use phiber::v2::scene::{OBJECTS, current_scene, project3_full, view_pan, view_pitch, view_yaw, view_zoom};
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState, CtxKey, ModalKey};
use phiber::v2::view::{BRowShade, BView, BViewItem};

fn select_view3d(state: &mut BState) {
    let space = BSpaceType::Layout;
    state.set(CtxKey::Space, space.slug());
    state.set(ModalKey::Open, "");
    for area in space.spec().areas.iter().filter(|a| a.hosts.contains(&BHostType::View3D)) {
        state.set(&phiber::v2::area::area_key(space, area), BHostType::View3D.slug());
    }
}

/// The canvas point the object named `name` projects to — its centre.
fn screen_point_of(state: &BState, name: &str) -> (f32, f32) {
    let object = OBJECTS.iter().find(|o| o.name == name).expect(name);
    let setup = setup_from_state(state);
    with_view(setup, || {
        let (x, y, _) = project3_full(object.x, 0.0, object.y, view_yaw(state), view_pitch(state), view_pan(state), view_zoom(state));
        (x, y)
    })
}

/// The lit rows: `(name, active)`.
fn selected_rows(state: &BState) -> Vec<(String, bool)> {
    let BView::Column { list } = BOutlinerItem::new().build(state).shown().clone() else { panic!("the outliner is a column") };
    list.into_iter()
        .filter_map(|row| match row {
            BView::TreeRow { shade: shade @ (BRowShade::Selected | BRowShade::Active), list, .. } => list.iter().find_map(|cell| match cell {
                BView::Button(control) => control.label.clone().map(|name| (name, shade == BRowShade::Active)),
                _ => None,
            }),
            _ => None,
        })
        .collect()
}

#[gpui::test]
async fn clicking_a_mesh_makes_it_the_active_object_and_lights_its_outliner_row(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        select_view3d(&mut app.state);
        app.state.set(CtxKey::Object, "Cube");
        cx.notify();
    });
    cx.run_until_parked();
    let canvas = cx.debug_bounds("view3d-canvas").expect("the canvas painted");
    assert_eq!(app.read_with(cx, |a, _| selected_rows(&a.state)), [("Cube".to_string(), true)]);

    // The Sphere's centre, in canvas coordinates, then in window pixels.
    let (sx, sy) = app.read_with(cx, |a, _| screen_point_of(&a.state, "Sphere"));
    let target = gpui::point(
        canvas.origin.x + canvas.size.width * (0.5 + sx),
        canvas.origin.y + canvas.size.height * (0.5 + sy),
    );
    // The pick agrees with the projection before the click is even made.
    let picked = app.read_with(cx, |a, _| {
        let state = &a.state;
        let setup = setup_from_state(state);
        let view = BPickView {
            yaw: view_yaw(state),
            pitch: view_pitch(state),
            pan: view_pan(state),
            zoom: view_zoom(state),
            setup,
            aspect: f32::from(canvas.size.width) / f32::from(canvas.size.height),
        };
        pick(&current_scene(state, None), view.ray(sx, sy)).map(|hit| hit.name)
    });
    assert_eq!(picked.as_deref(), Some("Sphere"), "the ray through the Sphere's centre meets the Sphere");

    cx.simulate_click(target, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Object).map(str::to_string)), Some("Sphere".into()));
    assert_eq!(app.read_with(cx, |a, _| selected_rows(&a.state)), [("Sphere".to_string(), true)], "the Outliner follows");

    // Shift-click the Cube: both selected, the Cube active and lit lighter.
    let (cx_, cy_) = app.read_with(cx, |a, _| screen_point_of(&a.state, "Cube"));
    let cube = gpui::point(canvas.origin.x + canvas.size.width * (0.5 + cx_), canvas.origin.y + canvas.size.height * (0.5 + cy_));
    cx.simulate_click(cube, Modifiers::shift());
    cx.run_until_parked();
    let rows = app.read_with(cx, |a, _| selected_rows(&a.state));
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert!(rows.contains(&("Cube".to_string(), true)) && rows.contains(&("Sphere".to_string(), false)), "{rows:?}");

    // Empty canvas: the middle of the bottom edge shows nothing (the corners
    // hold the overlay and the gizmo); everything is deselected, the active
    // object stays active.
    let corner = gpui::point(canvas.origin.x + canvas.size.width / 2.0, canvas.origin.y + canvas.size.height - px(6.0));
    cx.simulate_click(corner, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Object).map(str::to_string)), Some("Cube".into()));
    assert!(app.read_with(cx, |a, _| selected_rows(&a.state)).is_empty(), "nothing lit");
}

/// The orbit layer still orbits: a click is not a drag and a drag is not a click.
#[test]
fn the_click_pick_lives_on_the_orbit_surface_and_the_hit_squares_are_gone() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/src/v2/view/draw/canvas/view3d/");
    let orbit = std::fs::read_to_string(format!("{root}orbit.rs")).unwrap();
    assert!(orbit.contains(".on_click(hit.on_click(ctx))") && orbit.contains(".on_drag("), "one surface, two gestures");
    let hit = std::fs::read_to_string(format!("{root}hit.rs")).unwrap();
    assert!(hit.contains("pick_where(&self.scene, view.ray(su, sv)"), "the click is a ray pick");
    assert!(!hit.contains("relative(0.5 + self.x)"), "no floor-anchored squares stand in for the meshes");
}

/// B, then a drag across the canvas: the box takes every object with
/// something inside it (`view3d.select_box`), Shift adds, and the active
/// object is left as it was.
#[gpui::test]
async fn b_then_a_drag_box_selects_what_the_box_covers(cx: &mut TestAppContext) {
    use gpui::{MouseButton, MouseDownEvent, MouseUpEvent};
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        select_view3d(&mut app.state);
        app.state.set(CtxKey::Object, "Gem");
        cx.notify();
    });
    cx.run_until_parked();
    let canvas = cx.debug_bounds("view3d-canvas").expect("the canvas painted");
    cx.simulate_keystrokes("b");
    cx.run_until_parked();
    assert!(app.read_with(cx, |a, _| phiber::v2::scene::boxsel::armed(&a.state)), "B arms the box");
    assert!(cx.debug_bounds("view3d-box-select").is_some(), "the box surface replaced the orbit surface");

    // A box from the Cube's centre to the Sphere's: what it covers is what
    // `boxed_names` says it covers — the same projection the gesture uses.
    let (cx0, cy0) = app.read_with(cx, |a, _| screen_point_of(&a.state, "Cube"));
    let (sx1, sy1) = app.read_with(cx, |a, _| screen_point_of(&a.state, "Sphere"));
    let at = |x: f32, y: f32| gpui::point(canvas.origin.x + canvas.size.width * (0.5 + x), canvas.origin.y + canvas.size.height * (0.5 + y));
    let (a, b) = ((cx0, cy0), (sx1, sy1));
    let expected: Vec<String> = app.read_with(cx, |app, _| {
        let state = &app.state;
        let view = BPickView {
            yaw: view_yaw(state),
            pitch: view_pitch(state),
            pan: view_pan(state),
            zoom: view_zoom(state),
            setup: setup_from_state(state),
            aspect: f32::from(canvas.size.width) / f32::from(canvas.size.height),
        };
        phiber::v2::scene::boxsel::boxed_names(&current_scene(state, None), &view, phiber::v2::scene::boxsel::BBox2 { a, b }, |_| true)
    });
    assert!(expected.contains(&"Cube".to_string()) && expected.contains(&"Sphere".to_string()), "{expected:?}");
    let (from, to) = (at(a.0, a.1), at(b.0, b.1));
    cx.simulate_event(MouseDownEvent { position: from, modifiers: Modifiers::none(), button: MouseButton::Left, click_count: 1, first_mouse: false });
    cx.simulate_mouse_move(from + gpui::point(px(12.0), px(12.0)), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(to, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert!(app.read_with(cx, |a, _| phiber::v2::scene::boxsel::dragged(&a.state)).is_some(), "the rectangle is drawn while dragging");
    cx.simulate_event(MouseUpEvent { position: to, modifiers: Modifiers::none(), button: MouseButton::Left, click_count: 1 });
    cx.run_until_parked();
    let rows = app.read_with(cx, |a, _| selected_rows(&a.state));
    let mut lit: Vec<String> = rows.iter().map(|(n, _)| n.clone()).collect();
    lit.sort();
    let mut expected = expected;
    expected.sort();
    assert_eq!(lit, expected, "{rows:?}");
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Object).map(str::to_string)), Some("Gem".into()), "the active object stays");
    assert!(!app.read_with(cx, |a, _| phiber::v2::scene::boxsel::armed(&a.state)), "the gesture is over");

    // B again, a thin box along the top of the canvas, clear of the overlay
    // text: what it covers (usually nothing) is the selection, and an empty
    // box deselects.
    cx.simulate_keystrokes("b");
    cx.run_until_parked();
    let (a, b) = ((-0.1, -0.49), (0.1, -0.47));
    let expected: Vec<String> = app.read_with(cx, |app, _| {
        let state = &app.state;
        let view = BPickView {
            yaw: view_yaw(state),
            pitch: view_pitch(state),
            pan: view_pan(state),
            zoom: view_zoom(state),
            setup: setup_from_state(state),
            aspect: f32::from(canvas.size.width) / f32::from(canvas.size.height),
        };
        phiber::v2::scene::boxsel::boxed_names(&current_scene(state, None), &view, phiber::v2::scene::boxsel::BBox2 { a, b }, |_| true)
    });
    let (from, to) = (at(a.0, a.1), at(b.0, b.1));
    cx.simulate_event(MouseDownEvent { position: from, modifiers: Modifiers::none(), button: MouseButton::Left, click_count: 1, first_mouse: false });
    cx.simulate_mouse_move(from + gpui::point(px(12.0), px(2.0)), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(to, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_event(MouseUpEvent { position: to, modifiers: Modifiers::none(), button: MouseButton::Left, click_count: 1 });
    cx.run_until_parked();
    let mut lit: Vec<String> = app.read_with(cx, |a, _| selected_rows(&a.state)).iter().map(|(n, _)| n.clone()).collect();
    lit.sort();
    let mut expected = expected;
    expected.sort();
    assert_eq!(lit, expected, "the second box, too, selects what it covers");
    assert!(!app.read_with(cx, |a, _| phiber::v2::scene::boxsel::armed(&a.state)), "the gesture is over");
}
