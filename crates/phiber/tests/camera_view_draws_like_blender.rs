//! The Animation workspace's left viewport looks through the scene camera,
//! as `startup.blend` saves it (`RegionView3D.persp = RV3D_CAMOB`): the
//! canvas is built as a camera view — only when there is a camera to look
//! through — the frame lands where `view3d_camera_border` puts it, the
//! passepartout darkens what lies outside it, and the overlay names the
//! view as `view3d_get_name` does.

use phiber::phiphx::scene::{SceneCamera, SceneModel};
use phiber::v2::area::BAreaType;
use phiber::v2::host::BHostType;
use phiber::v2::scene::camera_view::{BCameraFrame, BCameraSlot, camera_view_key, set::BCameraView};
use phiber::v2::view::BAction;
use phiber::v2::scene::source::{clear_loaded_scene, set_loaded_scene};
use phiber::v2::space::{BSpaceItem, BSpaceType};
use phiber::v2::state::{BState, CtxKey, V_FALSE, V_TRUE};
use phiber::v2::theme::{BPaletteName, resolve_ui_style, ui_style_defaults};
use phiber::v2::view::{BView, BViewItem};

fn camera() -> SceneCamera {
    SceneCamera {
        name: "Camera".into(),
        location: [7.36, -6.93, 4.96],
        matrix_world: [[0.686, 0.728, 0.0, 0.0], [-0.324, 0.305, 0.895, 0.0], [0.651, -0.614, 0.445, 0.0], [7.36, -6.93, 4.96, 1.0]],
        kind: "PERSP".into(),
        focal_length_mm: 50.0,
        sensor_width: 36.0,
        ortho_scale: 6.0,
        clip_start: 0.1,
        clip_end: 100.0,
        display_size: 1.0,
        dof: Default::default(),
    }
}

fn scene_with_camera() -> SceneModel {
    let mut model = SceneModel::default();
    model.cameras.push(camera());
    model.render.camera = "Camera".into();
    model
}

/// Every 3D viewport canvas in `view`, with the area kind it sits in.
fn viewports(view: &BView, area: Option<BAreaType>, out: &mut Vec<(BAreaType, bool)>) {
    match view {
        BView::Area { kind, child } => viewports(child, Some(*kind), out),
        BView::Canvas { host: BHostType::View3D, camera } => {
            assert!(camera.key.is_some(), "a viewport in an area carries the key that toggles its camera view");
            out.push((area.expect("a canvas sits in an area"), camera.through))
        }
        BView::Space { list, .. } | BView::Zone { list, .. } | BView::Host { list, .. } | BView::Region { list, .. } | BView::Pane { list, .. } | BView::Row { list } | BView::Column { list } => {
            for child in list {
                viewports(child, area, out);
            }
        }
        BView::Part { body, .. } => viewports(body, area, out),
        _ => {}
    }
}

fn animation(state: &BState) -> Vec<(BAreaType, bool)> {
    let root = BSpaceItem::new(BSpaceType::Animation).build(state);
    let mut out = Vec::new();
    viewports(&root, None, &mut out);
    out.sort_by_key(|(kind, _)| kind.label());
    out
}

#[test]
fn the_animation_workspaces_left_viewport_looks_through_the_camera_when_there_is_one() {
    let mut state = BState::new();
    state.set(CtxKey::Space, BSpaceType::Animation.slug());
    // No camera: both viewports show the orbit.
    clear_loaded_scene();
    assert_eq!(animation(&state), [(BAreaType::Center, false), (BAreaType::Left, false)]);
    // A camera: the left one looks through it, as startup.blend saves it.
    set_loaded_scene(Some(scene_with_camera()));
    assert_eq!(animation(&state), [(BAreaType::Center, false), (BAreaType::Left, true)]);
    // Each viewport keeps its own choice.
    let left = BSpaceType::Animation.spec().areas_in(phiber::v2::zone::BZoneType::Left)[0];
    state.set(&camera_view_key(BSpaceType::Animation.slug(), &left), V_FALSE);
    assert_eq!(animation(&state), [(BAreaType::Center, false), (BAreaType::Left, false)]);
    let center = BSpaceType::Animation.spec().areas_in(phiber::v2::zone::BZoneType::Center)[0];
    state.set(&camera_view_key(BSpaceType::Animation.slug(), &center), V_TRUE);
    assert_eq!(animation(&state), [(BAreaType::Center, true), (BAreaType::Left, false)]);
    // Layout never opens a camera view on its own.
    state.set(CtxKey::Space, BSpaceType::Layout.slug());
    let root = BSpaceItem::new(BSpaceType::Layout).build(&state);
    let mut out = Vec::new();
    viewports(&root, None, &mut out);
    assert!(out.iter().all(|(_, camera)| !camera), "{out:?}");
    clear_loaded_scene();
}

#[test]
fn the_frame_is_blenders_camera_border_and_the_view_is_the_cameras() {
    // A 559 × 400 region (the Animation workspace's left viewport at 1920)
    // showing a 1920 × 1080 render at the frame-to-fit zoom.
    let region = (559.0, 400.0);
    let render = (1920.0, 1080.0);
    let view = BCameraView::of(&camera(), region, render, None);
    let frame = view.frame;
    // `view3d_center_camera_exec`: four pixels to spare on the tight side.
    assert!((frame.width() - (region.0 - 4.0)).abs() < 0.5, "{frame:?}");
    assert!((frame.width() / frame.height() - 1920.0 / 1080.0).abs() < 1e-3, "the frame keeps the render's aspect");
    assert!(((frame.left + frame.right) / 2.0 - region.0 / 2.0).abs() < 0.5 && ((frame.top + frame.bottom) / 2.0 - region.1 / 2.0).abs() < 0.5, "centred");
    // The eye is the camera: the pivot lies ten units ahead of it along its -Z.
    let back = camera().matrix_world[2];
    let length = (back[0] * back[0] + back[1] * back[1] + back[2] * back[2]).sqrt();
    for axis in 0..3 {
        assert!((view.pivot[axis] - (camera().location[axis] - back[axis] / length * 10.0)).abs() < 1e-3);
    }
    assert!((view.yaw - 313.3).abs() < 1.0 && (view.pitch - 26.4).abs() < 1.0, "the default camera looks down at the origin from (7.36, -6.93, 4.96), in phiphx's yaw convention: {} {}", view.yaw, view.pitch);
    // camzoom 0 halves the frame; the saved zoom is honoured when given.
    let half = BCameraFrame::of(region, render, 0.0);
    assert!((half.width() - region.0 / 2.0).abs() < 1e-3);
}

#[test]
fn the_overlay_names_the_view_and_the_frame_takes_the_themes_darkest_shade() {
    use phiber::v2::view::draw::canvas::overlay::view_name;
    assert_eq!(view_name(Some(true), true), "Camera Perspective");
    assert_eq!(view_name(Some(false), true), "Camera Orthographic");
    assert_eq!(view_name(None, true), "User Perspective");
    assert_eq!(view_name(None, false), "User Orthographic");
    for name in BPaletteName::ALL.iter().copied() {
        let style = resolve_ui_style(ui_style_defaults(name), &BState::new());
        let passepartout = style.camera_passepartout();
        assert_eq!(style.colors.ladder.step_of(passepartout), Some(1), "{name:?}: the passepartout is the darkest shade");
        assert!((passepartout.3 - 0.5).abs() < 1e-6, "at Blender's default passepartalpha");
        assert_eq!(style.colors.ladder.step_of(style.view_overlay()), Some(1));
    }
}

#[test]
fn the_gizmos_camera_button_toggles_this_areas_view_and_an_orbit_leaves_it_carrying_the_view_over() {
    // The button: into the camera view, or out of it, for this area's key.
    let left = BSpaceType::Animation.spec().areas_in(phiber::v2::zone::BZoneType::Left)[0];
    let key = camera_view_key(BSpaceType::Animation.slug(), &left);
    let slot = BCameraSlot { key: Some(key.clone()), through: true };
    assert_eq!(slot.toggle(), Some(BAction::Set(key.clone(), "false".into())));
    let slot = BCameraSlot { key: Some(key.clone()), through: false };
    assert_eq!(slot.toggle(), Some(BAction::Set(key.clone(), "true".into())));
    assert_eq!(BCameraSlot::none().toggle(), None, "a canvas outside an area has nothing to toggle");

    // The orbit: `ED_view3d_persp_switch_from_camera` — the camera's view
    // becomes the orbit's, then the area stops looking through the camera.
    let view = BCameraView::of(&camera(), (559.0, 400.0), (1920.0, 1080.0), None);
    let mut state = BState::new();
    state.set(CtxKey::Space, BSpaceType::Animation.slug());
    for action in view.leave(&key) {
        phiber::v2::state::action::apply_action(&mut state, &action);
    }
    use phiber::v2::scene::{is_perspective, view_pitch, view_yaw, view_zoom};
    assert!((view_yaw(&state) - view.yaw).abs() < 1e-2 && (view_pitch(&state) - view.pitch).abs() < 1e-2);
    assert!((view_zoom(&state) - view.zoom).abs() < 1e-3);
    assert_eq!(is_perspective(&state), view.perspective);
    let pivot = phiber::v2::scene::camera::view_pivot(&state);
    for axis in 0..3 {
        assert!((pivot[axis] - view.pivot[axis]).abs() < 1e-4, "the orbit pivots where the camera view did");
    }
    assert_eq!(state.get(&key), Some(V_FALSE), "and the area no longer looks through the camera");
}
