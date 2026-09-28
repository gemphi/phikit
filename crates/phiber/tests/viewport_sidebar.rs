//! The 3D Viewport's sidebar (N panel, `SpaceView3D.show_region_ui`): N
//! opens it beside the tab strip, picking a tab opens it on that tab and
//! picking the showing tab puts it away. Item ▸ Transform is the Object
//! tab's Transform over the same keys, View ▸ Focal Length is the area's own
//! lens, and a file's saved screen decides whether it opens.

use std::path::PathBuf;

use gpui::{Modifiers, MouseButton, TestAppContext, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::part::sidebar::show::BSidebarShown;
use phiber::v2::scene::camera::setup_from_state;
use phiber::v2::scene::view_slot::BViewSlot;
use phiber::v2::state::{BKey as _, BState, ButsKey, CtxKey, ModalKey, View3dKey};
use phiber::v2::view::BAction;

fn shown(state: &BState) -> bool {
    BSidebarShown(BHostType::View3D).on(state)
}

fn right(b: gpui::Bounds<gpui::Pixels>) -> f32 {
    f32::from(b.origin.x) + f32::from(b.size.width)
}

async fn window(cx: &mut TestAppContext) -> (gpui::Entity<BPhiberApp>, &mut gpui::VisualTestContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.handle_action(&BAction::Set(CtxKey::Object.path(), "Cube".into()), cx);
    });
    cx.run_until_parked();
    (app, cx)
}

#[gpui::test]
async fn n_opens_the_item_tab_with_the_objects_transform(cx: &mut TestAppContext) {
    let (app, cx) = window(cx).await;
    assert!(!app.read_with(cx, |a, _| shown(&a.state)), "shut until opened, as Blender's startup file has it");
    assert!(cx.debug_bounds("panel-sidebar-column").is_none());
    let compass = cx.debug_bounds("gizmo-compass").expect("the navigation gizmo");

    cx.simulate_keystrokes("n");
    cx.run_until_parked();
    assert!(app.read_with(cx, |a, _| shown(&a.state)), "N shows the sidebar");
    let column = cx.debug_bounds("panel-sidebar-column").expect("the sidebar's panels");
    let tab = cx.debug_bounds("tab-view_layer-view_layer").expect("the tab strip");
    assert!(right(column) <= f32::from(tab.origin.x), "the panels stand left of the tab strip");
    let transform = cx.debug_bounds("card-ctx-view3d-item-transform-expanded").expect("Item ▸ Transform");
    assert!(transform.origin.x >= column.origin.x && right(transform) <= right(column));
    let x = cx
        .debug_bounds("btn-icon-set-ctx.edit-ctx.buts.location.x-number")
        .expect("the Location X field");
    assert!(x.origin.x >= column.origin.x, "the Location field is the sidebar's");
    // The gizmo keeps its distance from the strip, now the strip and the panels.
    let moved = cx.debug_bounds("gizmo-compass").expect("the navigation gizmo");
    assert!(f32::from(moved.origin.x) < f32::from(compass.origin.x) - f32::from(column.size.width) + 1.0, "the gizmo stands clear of the panels");

    // The field is the Object tab's own key (`ctx.buts.location.x`), which
    // moves the object (`transform_panel_moves_objects.rs`).
    assert_eq!(ButsKey::Axis("location", "x").path().encode(), "ctx.buts.location.x");

    cx.simulate_keystrokes("n");
    cx.run_until_parked();
    assert!(!app.read_with(cx, |a, _| shown(&a.state)), "N again puts it away");
    assert!(cx.debug_bounds("panel-sidebar-column").is_none());
}

#[gpui::test]
async fn a_tab_opens_the_sidebar_on_it_and_the_showing_tab_puts_it_away(cx: &mut TestAppContext) {
    let (app, cx) = window(cx).await;
    let view = cx.debug_bounds("tab-view_layer-view_layer").expect("the View tab").center();
    cx.simulate_click(view, Modifiers::none());
    cx.run_until_parked();
    assert!(app.read_with(cx, |a, _| shown(&a.state)), "picking a tab opens the sidebar");
    assert!(cx.debug_bounds("card-ctx-view3d-view-view-expanded").is_some(), "on the View tab");

    // Focal Length is this area's own lens.
    // The viewport of the workspace showing: its own view slot.
    let scope = app.read_with(cx, |a, _| phiber::v2::space::BSpaceList::active(&a.state).slug());
    let slot = BViewSlot::new(scope, "Center");
    let lens = slot.path(View3dKey::Lens);
    let id: &'static str = Box::leak(format!("btn-icon-set-ctx.edit-{}-number", lens.encode()).into_boxed_str());
    assert!(cx.debug_bounds(id).is_some(), "the Focal Length field edits {}", lens.encode());
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(lens.clone(), "35".into()), cx));
    cx.run_until_parked();
    let fov = app.read_with(cx, |a, _| slot.with(|| setup_from_state(&a.state).fov));
    assert!((fov - 72.0 / 35.0).abs() < 1e-4, "a 35 mm lens widens the view: {fov}");

    let view = cx.debug_bounds("tab-view_layer-view_layer").expect("the View tab").center();
    cx.simulate_mouse_move(view, None, Modifiers::none());
    cx.simulate_mouse_down(view, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(view, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert!(!app.read_with(cx, |a, _| shown(&a.state)), "picking the showing tab puts it away");
}

#[test]
fn a_files_saved_screen_decides_whether_the_sidebar_opens() {
    let read = |name: &str| {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
        let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
        SceneModel::from_blend_owned(scene, &path.to_string_lossy())
    };
    // `make_sidebar_open.py`: the factory startup, every viewport's N panel shown.
    let open = read("sidebar_open.blend");
    assert!(open.ui.view.as_ref().expect("a saved view").sidebar);
    let mut s = BState::new();
    phiber::v2::app::load_ui::restore_ui(&mut s, &open);
    assert!(shown(&s), "the sidebar opens as the file left it");
    // The factory startup itself keeps it shut.
    let shut = read("parented.blend");
    assert!(!shut.ui.view.as_ref().expect("a saved view").sidebar);
    phiber::v2::app::load_ui::restore_ui(&mut s, &shut);
    assert!(!shown(&s));
}
