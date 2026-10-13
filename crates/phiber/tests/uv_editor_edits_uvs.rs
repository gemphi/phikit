//! The UV editor edits UVs as Blender's does (`scene::uv_edit`,
//! `canvas/image/uv_pick.rs`): in Edit Mode a click on the square picks the
//! nearest corner (`uv_select_pick`), Shift adds to the selection, a plain
//! click replaces it, a click on empty square picks nothing, and a drag
//! that began on a selected corner moves the whole selection. In Object
//! Mode the square is read-only, as Blender shows the UVs greyed and does
//! not edit them (`ED_space_image_show_uvedit`).
//!
//! Over the startup cube, whose map `uv_editor_shows_uvs.rs` already checks
//! against Blender's own: six quads, 24 corners, none of them at the square's
//! own corners — so the click points here are the cube's UVs, not (0, 0).

use gpui::{Modifiers, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::part::image::uv::BUvShown;
use phiber::v2::part::modebar::active_mode;
use phiber::v2::scene::uv_edit::BUvEdit;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BKey as _, CtxKey, ModalKey};
use phiber::v2::view::BAction;
use std::path::PathBuf;

type Cx<'a> = &'a mut gpui::VisualTestContext;

/// `UV_TILE`: the square is one tile, fitted into the window and centred.
const UV_TILE: f32 = 256.0;

/// Where UV `(u, v)` lands in the UV editor's window — the square fitted at
/// a power of two and centred, `v` running up where `y` runs down.
fn at(cx: Cx, u: f32, v: f32) -> gpui::Point<gpui::Pixels> {
    let w = cx.debug_bounds("image-window").expect("the UV editor");
    let (winx, winy) = (f32::from(w.size.width), f32::from(w.size.height));
    let mut zoom = 1.0f32;
    while UV_TILE * zoom > winx.min(winy) {
        zoom /= 2.0;
    }
    let side = (UV_TILE * zoom).round();
    let x = f32::from(w.origin.x) + ((winx - side) / 2.0).round();
    let y = f32::from(w.origin.y) + ((winy - side) / 2.0).round();
    gpui::point(px(x + u * side), px(y + (1.0 - v) * side))
}

fn edit(app: &gpui::Entity<BPhiberApp>, cx: Cx) -> BUvEdit {
    app.read_with(cx, |app, _| BUvEdit::now(&app.state))
}

/// The startup cube in the UV Editing workspace, which puts it in Edit
/// Mode — `add_window_view` hands back a borrow of the context, so the
/// window is made by the caller and this fills it.
fn editing(app: &gpui::Entity<BPhiberApp>, cx: Cx) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/blend/startup-5.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    let key = path.to_string_lossy().to_string();
    let model = SceneModel::from_blend_owned(scene, &key);
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(&phiber::v2::scene::loaded_path_key(), key.clone());
        app.scene_cache = Some((key, std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    for action in [BAction::Set(CtxKey::Object.path(), "Cube".into()), BAction::Set(CtxKey::Space.path(), BSpaceType::UvEditing.slug().into())] {
        app.update(cx, |app, cx| app.handle_action(&action, cx));
        cx.run_until_parked();
    }
    assert_eq!(app.read_with(cx, |a, _| active_mode(&a.state, BHostType::View3D)), "Edit Mode");
    assert!(app.read_with(cx, |a, _| BUvShown::now(&a.state).edits()), "the UV editor edits the Cube");
}

#[gpui::test]
async fn a_click_picks_a_uv_corner_shift_adds_to_it_and_a_drag_moves_the_selection(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    editing(&app, cx);
    assert!(edit(&app, cx).selected.is_empty(), "nothing picked to begin with");

    // The cube's corner at UV (0.875, 0.75).
    let p = at(cx, 0.875, 0.75);
    cx.simulate_click(p, Modifiers::none());
    cx.run_until_parked();
    let picked = edit(&app, cx).selected;
    assert_eq!(picked.len(), 1, "one corner picked: {picked:?}");

    // Shift on another of its corners adds it, keeping the first.
    let p = at(cx, 0.125, 0.5);
    cx.simulate_click(p, Modifiers::shift());
    cx.run_until_parked();
    let both = edit(&app, cx).selected;
    assert_eq!(both.len(), 2, "Shift added the second: {both:?}");
    assert!(both.starts_with(&picked), "and kept the first, in the order picked");

    // A plain click on a corner replaces the lot.
    let p = at(cx, 0.125, 0.75);
    cx.simulate_click(p, Modifiers::none());
    cx.run_until_parked();
    let one = edit(&app, cx).selected;
    assert_eq!(one.len(), 1, "a plain click replaces: {one:?}");

    // A click where the cube has no corner leaves the selection alone —
    // there is nothing within the radius, and picking the least far thing
    // on the whole square would make a stray click move something.
    let p = at(cx, 0.03, 0.03);
    cx.simulate_click(p, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(edit(&app, cx).selected, one, "a click on empty square picks nothing new");

    // A drag from the selected corner moves it, and only it.
    let (from, to) = (at(cx, 0.125, 0.75), at(cx, 0.25, 0.875));
    cx.simulate_event(MouseDownEvent { position: from, modifiers: Modifiers::none(), button: MouseButton::Left, click_count: 1, first_mouse: false });
    cx.simulate_mouse_move(to, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_event(MouseUpEvent { position: to, modifiers: Modifiers::none(), button: MouseButton::Left, click_count: 1 });
    cx.run_until_parked();
    let moved = edit(&app, cx);
    assert_eq!(moved.moved.len(), 1, "one corner moved: {:?}", moved.moved);
    let ((map, corner), by) = moved.moved[0];
    assert!(by[0] > 0.05 && by[1] > 0.05, "right and up, as the pointer went: {by:?}");
    // And the corner reads back where it was moved to.
    let uv = app.read_with(cx, |a, _| BUvShown::now(&a.state).maps[map].uvs[corner]);
    assert_eq!(moved.uv_of((map, corner), uv), [uv[0] + by[0], uv[1] + by[1]]);
}

/// Object Mode shows the UVs without editing them.
#[gpui::test]
async fn object_mode_shows_the_uvs_without_editing_them(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    editing(&app, cx);
    let action = BAction::Set(CtxKey::Space.path(), BSpaceType::Layout.slug().into());
    app.update(cx, |app, cx| app.handle_action(&action, cx));
    cx.run_until_parked();
    assert!(!app.read_with(cx, |a, _| BUvShown::now(&a.state).edits()), "back in Object Mode");
    // The UV editor is not the Layout workspace's centre, so there is no
    // square to click: what is pinned is that the mode decides, which is
    // `BUvGesture::attach`'s own guard.
    assert!(app.read_with(cx, |a, _| BUvShown::now(&a.state).edit) == false);
}
