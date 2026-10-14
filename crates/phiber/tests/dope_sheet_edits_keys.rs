//! The Dope Sheet edits keys as Blender's does (`scene::action::dope`): a
//! click picks a key column, a drag on it moves it by whole frames — one
//! undo step — a drag on nothing sweeps a box, A / Alt A pick all and none,
//! G moves the picked keys with the pointer and Escape puts them back, X
//! deletes them (and a curve left without keys goes), and Ctrl S writes the
//! curves they leave. Over phiphx's `fixtures/actions.blend`
//! (`make_actions.py`): the Cube moves by CubeAction's location keys at 1
//! and 20.

use gpui::{Modifiers, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext, px, size};
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::scene::action::BSceneActions;
use phiber::v2::scene::action::dope::BKeySelection;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{CtxKey, ModalKey};
use std::path::{Path, PathBuf};

type Cx<'a> = &'a mut gpui::VisualTestContext;

/// `ANIM_UI_get_channel_name_width`, `_height`, `_skip`, `UI_TIME_SCRUB_MARGIN_Y`.
const CHANNEL_WIDTH: f32 = 200.0;
const ROW: f32 = 16.0;
const SKIP: f32 = 2.0;
/// The frame-number strip's height. Blender's `UI_TIME_SCRUB_MARGIN_Y` is
/// 23 at scale 1, but the strip reads as a header for the window below it,
/// so the renderer fills it from the theme's `header_height` instead
/// (`the_scrub_strip_is_as_tall_as_a_header`). This constant followed that
/// change; at 23 every row below the strip was 13 px out and every row
/// assertion here missed.
const SCRUB: f32 = 36.0;

/// Frame `frame` of row `row` on screen (the view is 1 − 10 .. 250 + 10).
fn at(cx: Cx, frame: f32, row: usize) -> gpui::Point<gpui::Pixels> {
    let window = cx.debug_bounds("time-window").expect("the Dope Sheet");
    let (left, width) = (f32::from(window.origin.x) + CHANNEL_WIDTH, f32::from(window.size.width) - CHANNEL_WIDTH);
    let y = f32::from(window.origin.y) + SCRUB + SKIP + row as f32 * (ROW + SKIP) + ROW / 2.0;
    gpui::point(px(left + (frame + 9.0) / 269.0 * width), px(y))
}

fn click(cx: Cx, frame: f32, row: usize) {
    let p = at(cx, frame, row);
    cx.simulate_click(p, Modifiers::none());
    cx.run_until_parked();
}

fn drag(cx: Cx, from: (f32, usize), to: (f32, usize)) {
    let (a, b) = (at(cx, from.0, from.1), at(cx, to.0, to.1));
    cx.simulate_event(MouseDownEvent { position: a, modifiers: Modifiers::none(), button: MouseButton::Left, click_count: 1, first_mouse: false });
    cx.simulate_mouse_move(a + gpui::point(px(6.0), px(0.0)), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(b, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_event(MouseUpEvent { position: b, modifiers: Modifiers::none(), button: MouseButton::Left, click_count: 1 });
    cx.run_until_parked();
}

fn keys(cx: Cx, keys: &str) {
    let over = cx.debug_bounds("time-window").expect("the Dope Sheet").center();
    cx.simulate_mouse_move(over, None, Modifiers::none());
    cx.simulate_keystrokes(keys);
    cx.run_until_parked();
}

/// The Cube's X location keys as the session has them.
fn cube_x(app: &gpui::Entity<BPhiberApp>, cx: Cx) -> Vec<f32> {
    app.read_with(cx, |a, _| {
        let model = &a.scene_cache.as_ref().unwrap().1;
        let actions = BSceneActions::now(&a.state).keyed_actions(model);
        let action = actions.into_iter().find(|a| a.name == "CubeAction").unwrap();
        action.curves.iter().find(|c| c.path == "location" && c.index == 0).map_or(Vec::new(), |c| c.keys.iter().map(|k| k.co[0]).collect())
    })
}

fn picks(app: &gpui::Entity<BPhiberApp>, cx: Cx) -> Vec<(String, f32)> {
    app.read_with(cx, |a, _| BKeySelection::now(&a.state).0.into_iter().map(|p| (p.object, p.frame)).collect())
}

/// A copy of the saved file for `truth/blender_dope_keys.py` to open.
fn keep(blend: &Path, name: &str) {
    std::fs::copy(blend, std::env::temp_dir().join(format!("phiber_dope_keys_{name}.blend"))).ok();
}

fn saved(blend: &Path) -> SceneModel {
    let scene = BlendFile::parse_bytes(&std::fs::read(blend).unwrap()).unwrap().extract_scene().unwrap();
    SceneModel::from_blend_owned(scene, &blend.to_string_lossy())
}

#[gpui::test]
async fn the_dope_sheet_picks_moves_and_deletes_keys_and_saves_them(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_dope_keys_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let blend = dir.join("actions.blend");
    std::fs::copy(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/actions.blend"), &blend).unwrap();
    let path = blend.to_string_lossy().to_string();
    let model = saved(&blend);
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    let space = BSpaceType::Animation;
    let area = space.spec().areas.iter().find(|a| a.default_host == BHostType::Action).copied().expect("the Dope Sheet area");
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(CtxKey::Space, space.slug());
        app.state.set(&phiber::v2::area::area_key(space, &area), BHostType::Action.slug());
        app.state.set(&phiber::v2::scene::loaded_path_key(), path.clone());
        app.state.set(CtxKey::Object, "Cube");
        app.state.set(CtxKey::FrameStart, "1");
        app.state.set(CtxKey::FrameEnd, "250");
        app.scene_cache = Some((path.clone(), std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(cube_x(&app, cx), [1.0, 20.0]);

    // Rows: the summary, the Cube. A click on the Cube's key at 20.
    click(cx, 20.0, 1);
    assert_eq!(picks(&app, cx), [("Cube".to_string(), 20.0)]);
    // Dragged to 30: moved, still picked, one undo step.
    drag(cx, (20.0, 1), (30.0, 1));
    assert_eq!(cube_x(&app, cx), [1.0, 30.0]);
    assert_eq!(picks(&app, cx), [("Cube".to_string(), 30.0)]);
    keys(cx, "ctrl-s");
    keep(&blend, "moved");
    keys(cx, "ctrl-z");
    assert_eq!(cube_x(&app, cx), [1.0, 20.0], "Undo puts the whole move back");

    // A and Alt A.
    keys(cx, "a");
    assert_eq!(picks(&app, cx).len(), 2);
    keys(cx, "alt-a");
    assert!(picks(&app, cx).is_empty());

    // G with the pointer, then Escape.
    click(cx, 20.0, 1);
    keys(cx, "g");
    let (from, to) = (at(cx, 20.0, 1), at(cx, 45.0, 1));
    cx.simulate_mouse_move(from, None, Modifiers::none());
    cx.simulate_mouse_move(to, None, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(cube_x(&app, cx), [1.0, 45.0], "G: the key follows the pointer");
    keys(cx, "escape");
    assert_eq!(cube_x(&app, cx), [1.0, 20.0], "Escape puts it back");

    // A box from nothing at 120 back over frame 20, both rows.
    keys(cx, "alt-a");
    drag(cx, (120.0, 1), (12.0, 0));
    assert_eq!(picks(&app, cx), [("Cube".to_string(), 20.0)], "the box holds frame 20 alone");

    // The summary's key at 1, then X.
    click(cx, 1.0, 0);
    assert_eq!(picks(&app, cx), [("Cube".to_string(), 1.0)]);
    keys(cx, "x");
    assert_eq!(cube_x(&app, cx), [20.0]);

    // Ctrl S: Blender's file holds the key at 20 alone.
    keys(cx, "ctrl-s");
    let x = saved(&blend).actions.into_iter().find(|a| a.name == "CubeAction").unwrap();
    let x = x.curves.iter().find(|c| c.path == "location" && c.index == 0).expect("the X curve").clone();
    assert_eq!(x.keys.iter().map(|k| k.co).collect::<Vec<_>>(), [[20.0, 3.0]]);
    keep(&blend, "deleted");

    // Every key deleted: the curves go, and the saved action has none.
    keys(cx, "a");
    keys(cx, "x");
    assert!(cube_x(&app, cx).is_empty());
    keys(cx, "ctrl-s");
    let action = saved(&blend).actions.into_iter().find(|a| a.name == "CubeAction").unwrap();
    assert!(action.curves.is_empty(), "{:?}", action.curves.iter().map(|c| &c.path).collect::<Vec<_>>());
    keep(&blend, "emptied");
    let _ = std::fs::remove_dir_all(&dir);
}
