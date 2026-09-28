//! The UV editor as Blender 5.0 has it (`IMAGE_HT_header`,
//! `overlay_edit_uv.cc`): the model carries each mesh's UV map face by face,
//! exactly as Blender reads the startup cube's; switching to UV Editing puts
//! the cube in Edit Mode (`WorkSpace.object_mode`), where the UV editor
//! edits it — no mode menu, the selection settings before the menus, the
//! pivot, snapping and proportional editing after — and back in Layout it
//! is in Object Mode, the UVs shown greyed and the settings gone.

use gpui::{Modifiers, TestAppContext, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::part::image::uv::BUvShown;
use phiber::v2::part::modebar::active_mode;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BKey as _, CtxKey, ModalKey, UvKey};
use phiber::v2::view::BAction;
use std::path::PathBuf;

/// `[uv for uv in cube.data.uv_layers.active.data]` in Blender 5.0.
const CUBE_UVS: [[f32; 2]; 24] = [
    [0.625, 0.5], [0.875, 0.5], [0.875, 0.75], [0.625, 0.75],
    [0.375, 0.75], [0.625, 0.75], [0.625, 1.0], [0.375, 1.0],
    [0.375, 0.0], [0.625, 0.0], [0.625, 0.25], [0.375, 0.25],
    [0.125, 0.5], [0.375, 0.5], [0.375, 0.75], [0.125, 0.75],
    [0.375, 0.5], [0.625, 0.5], [0.625, 0.75], [0.375, 0.75],
    [0.375, 0.25], [0.625, 0.25], [0.625, 0.5], [0.375, 0.5],
];

fn shown(cx: &mut gpui::VisualTestContext, id: &str) -> bool {
    cx.debug_bounds(Box::leak(id.to_string().into_boxed_str())).is_some()
}

#[gpui::test]
async fn the_uv_editor_shows_and_edits_the_cubes_uvs(cx: &mut TestAppContext) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/blend/startup-5.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    let key = path.to_string_lossy().to_string();
    let model = SceneModel::from_blend_owned(scene, &key);

    // The map, face by face, as Blender reads it.
    let cube = model.objects.iter().position(|o| o.name == "Cube").expect("the Cube");
    let map = &model.uv_maps[cube];
    assert_eq!(map.starts, [0, 4, 8, 12, 16, 20, 24], "six quads");
    for (ours, blender) in map.uvs.iter().zip(CUBE_UVS) {
        assert!((ours[0] - blender[0]).abs() < 1e-4 && (ours[1] - blender[1]).abs() < 1e-4, "{ours:?} vs {blender:?}");
    }

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(&phiber::v2::scene::loaded_path_key(), key.clone());
        app.scene_cache = Some((key, std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    let act = |app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext, action: BAction| {
        app.update(cx, |app, cx| app.handle_action(&action, cx));
        cx.run_until_parked();
    };
    act(&app, cx, BAction::Set(CtxKey::Object.path(), "Cube".into()));

    // UV Editing: Edit Mode, and the UV editor edits the Cube.
    act(&app, cx, BAction::Set(CtxKey::Space.path(), BSpaceType::UvEditing.slug().into()));
    assert_eq!(app.read_with(cx, |a, _| active_mode(&a.state, BHostType::View3D)), "Edit Mode", "the workspace's mode");
    let uv = app.read_with(cx, |a, _| BUvShown::now(&a.state));
    assert!(uv.edits() && uv.maps.len() == 1, "the Cube's map, edited");
    assert!(shown(cx, "image-window"), "the UV editor's window draws");
    let sync = "btn-icon-set-ctx.uv.sync-false-arrows_horizontal";
    assert!(shown(cx, sync), "UV Sync Selection, on in a new 5.0 scene");
    assert!(shown(cx, "btn-icon-set-ctx.uv.mesh_select_mode-Edge-edge_slide"), "syncing: the mesh's select modes");
    for id in ["btn-icon-toggle-ctx.menu.uv.Pivot Point-target", "btn-icon-set-ctx.uv.snap-true-magnet", "btn-icon-set-ctx.uv.proportional-true-circle_dashed"] {
        assert!(shown(cx, id), "{id} after the spacer");
    }

    // Sync off: the UV select modes; Edge picked.
    let at = cx.debug_bounds(Box::leak(sync.to_string().into_boxed_str())).unwrap().center();
    cx.simulate_click(at, Modifiers::none());
    cx.run_until_parked();
    let edge = "btn-icon-set-ctx.uv.select_mode-Edge-edge_slide";
    let at = cx.debug_bounds(edge).expect("the UV select modes").center();
    cx.simulate_click(at, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(UvKey::SelectMode).map(str::to_string)), Some("Edge".into()));

    // The pivot menu picks Median Point.
    let at = cx.debug_bounds("btn-icon-toggle-ctx.menu.uv.Pivot Point-target").unwrap().center();
    cx.simulate_click(at, Modifiers::none());
    cx.run_until_parked();
    let at = cx.debug_bounds("menu-Median Point").expect("the pivot's options").center();
    cx.simulate_click(at, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(UvKey::Pivot).map(str::to_string)), Some("Median Point".into()));

    // Layout: Object Mode; the UVs are shown, not edited, and the settings go.
    act(&app, cx, BAction::Set(CtxKey::Space.path(), BSpaceType::Layout.slug().into()));
    assert_ne!(app.read_with(cx, |a, _| active_mode(&a.state, BHostType::View3D)), "Edit Mode");
    act(&app, cx, BAction::Set(CtxKey::Space.path(), BSpaceType::UvEditing.slug().into()));
    act(&app, cx, BAction::Set(phiber::v2::part::modebar::set::mode_key(BHostType::View3D), "Object Mode".into()));
    let uv = app.read_with(cx, |a, _| BUvShown::now(&a.state));
    assert!(!uv.edits() && uv.maps.len() == 1, "Object Mode shows the map and edits nothing");
    assert!(!shown(cx, "btn-icon-toggle-ctx.menu.uv.Pivot Point-target"), "no settings while not editing");
}
