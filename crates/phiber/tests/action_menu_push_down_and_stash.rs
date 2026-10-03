//! The Action Editor's Action menu as Blender 5.0 has it
//! (`DOPESHEET_MT_action`, drawn while the active object has an action):
//! Push Down Action lays the Cube's CubeAction into a new NLA track named
//! for it and unassigns it, the Cube moving as before — now by its NLA,
//! held before the strip — and Stash Action puts Spin, muted, in
//! "[Action Stash]" at the bottom of the stack, once per action and slot.
//! Undo steps them and Ctrl S writes the stack. Every value is Blender's,
//! read with bpy after `action.push_down` / `action.stash` on the same
//! file (`phiphx/tests/fixtures/actions.blend`; phiphx
//! `blender_opens_saves` has Blender read the same writes).

use gpui::{Modifiers, TestAppContext, px, size};
use phiber::phiphx::blend::{BlendFile, STASH_TRACK, STRIP_ACTIVE, STRIP_SELECT, TRACK_MUTED, TRACK_PROTECTED};
use phiber::phiphx::scene::{SceneModel, SceneNla};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::part::modebar::set::mode_key;
use phiber::v2::scene::action::BSceneActions;
use phiber::v2::scene::{BObject3D, scene_objects};
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BKey as _, CtxKey, ModalKey, OpKey};
use phiber::v2::view::BAction;
use std::path::PathBuf;

type Cx<'a> = &'a mut gpui::VisualTestContext;

const MENU: &str = "btn-label-toggle-ctx.menu.action.Action-text";

/// Action ▸ `item` from the header.
fn pick(cx: Cx, item: &'static str) {
    let menu = cx.debug_bounds(MENU).expect("the Action menu").center();
    cx.simulate_click(menu, Modifiers::none());
    cx.run_until_parked();
    let at = cx.debug_bounds(item).unwrap_or_else(|| panic!("{item}")).center();
    cx.simulate_click(at, Modifiers::none());
    cx.run_until_parked();
}

fn said() -> String {
    phiber::v2::state::log::latest().map(|e| e.text.clone()).unwrap_or_default()
}

/// The Cube's NLA as the editors read it.
fn cube_nla(cx: Cx) -> SceneNla {
    cx.run_until_parked();
    phiber::v2::scene::source::with_loaded(|m| m.and_then(|m| m.nla.iter().find(|n| n.object == "Cube").cloned())).expect("the Cube's NLA")
}

/// The Cube's world X and Z turn at `frame`.
fn cube_at(app: &gpui::Entity<BPhiberApp>, cx: Cx, frame: i32) -> (f32, f32) {
    app.update(cx, |app, cx| {
        app.state.set(CtxKey::Frame, frame.to_string());
        cx.notify();
    });
    cx.run_until_parked();
    app.read_with(cx, |a, _| match scene_objects(&a.state).iter().find(|o| o.name() == "Cube") {
        Some(BObject3D::Loaded { matrix_world, .. }) => (matrix_world[3][0], matrix_world[0][1].atan2(matrix_world[0][0])),
        _ => panic!("the Cube"),
    })
}

fn near(got: (f32, f32), want: (f32, f32)) -> bool {
    (got.0 - want.0).abs() < 1e-3 && (got.1 - want.1).abs() < 1e-3
}

#[gpui::test]
async fn push_down_and_stash_lay_the_action_into_the_nla(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_action_menu_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let blend = dir.join("actions.blend");
    std::fs::copy(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/actions.blend"), &blend).unwrap();
    let path = blend.to_string_lossy().to_string();
    let scene = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap();
    let model = SceneModel::from_blend_owned(scene, &path);
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(CtxKey::Space, BSpaceType::Animation.slug());
        app.state.set(&phiber::v2::scene::loaded_path_key(), path.clone());
        app.scene_cache = Some((path.clone(), std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(CtxKey::Object.path(), "Cube".into()), cx));
    cx.run_until_parked();
    assert!(cx.debug_bounds(MENU).is_none(), "the Dope Sheet has no Action menu");
    app.update(cx, |app, cx| {
        app.state.set(&mode_key(BHostType::Action), "Action Editor");
        cx.notify();
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds(MENU).is_some(), "the Action Editor has it over CubeAction");

    // Push Down: CubeAction in a new track named for it, from its Manual
    // Frame Range, selected and active; the Cube has no action, and moves
    // as before, held at the strip's start before it.
    pick(cx, "menu-Push Down Action");
    assert_eq!(said(), "Pushed CubeAction down");
    let nla = cube_nla(cx);
    assert_eq!(nla.action, None);
    let track = &nla.tracks[0];
    let strip = &track.strips[0];
    assert_eq!((nla.tracks.len(), track.name.as_str(), strip.name.as_str(), strip.slot_name.as_str()), (1, "CubeAction", "CubeAction", "OBCube"));
    assert_eq!((strip.start, strip.end, strip.flag & (STRIP_SELECT | STRIP_ACTIVE)), (5.0, 30.0, STRIP_SELECT | STRIP_ACTIVE));
    for (frame, x) in [(0, 0.3429), (10, 1.3817), (12, 1.8523), (20, 3.0), (42, 3.0)] {
        assert!(near(cube_at(&app, cx, frame), (x, 0.0)), "at {frame}: {:?}", cube_at(&app, cx, frame));
    }
    assert!(cx.debug_bounds(MENU).is_none(), "no action, no Action menu");

    // Spin on top of it: the action plays over the NLA.
    let assign = |app: &gpui::Entity<BPhiberApp>, cx: Cx| {
        app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::Action.path(), "assign:Spin".into()), cx));
        cx.run_until_parked();
    };
    assign(&app, cx);
    assert!(near(cube_at(&app, cx, 10), (1.3817, 1.0666)), "{:?}", cube_at(&app, cx, 10));

    // Stash: Spin, unselected, in a muted, locked "[Action Stash]" at the
    // bottom; it plays no more.
    pick(cx, "menu-Stash Action");
    assert_eq!(said(), "Stashed Spin");
    let nla = cube_nla(cx);
    assert_eq!(nla.tracks.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(), [STASH_TRACK, "CubeAction"]);
    let (stash, strip) = (&nla.tracks[0], &nla.tracks[0].strips[0]);
    assert_eq!(stash.flag & (TRACK_MUTED | TRACK_PROTECTED), TRACK_MUTED | TRACK_PROTECTED);
    assert_eq!((strip.name.as_str(), strip.start, strip.end, strip.flag & STRIP_SELECT), ("Spin", 1.0, 24.0, 0));
    assert!(near(cube_at(&app, cx, 10), (1.3817, 0.0)), "{:?}", cube_at(&app, cx, 10));
    assert_eq!(app.read_with(cx, |a, _| BSceneActions::now(&a.state).of("Cube", &SceneModel::default())), None);

    // Stashed once per action and slot: again says so and unassigns.
    assign(&app, cx);
    pick(cx, "menu-Stash Action");
    assert_eq!(said(), "Action+Slot has already been stashed");
    assert_eq!(cube_nla(cx).tracks.len(), 2);
    assert!(cx.debug_bounds(MENU).is_none());

    // Undo back to Spin over the pushed strip.
    for _ in 0..3 {
        cx.simulate_keystrokes("ctrl-z");
        cx.run_until_parked();
    }
    assert_eq!(cube_nla(cx).tracks.len(), 1, "the stash undone");
    assert!(cx.debug_bounds(MENU).is_some(), "Spin is the Cube's again");
    cx.simulate_keystrokes("ctrl-shift-z");
    cx.run_until_parked();

    // Ctrl S: the file has the stack — Blender reads it back as above.
    cx.simulate_keystrokes("ctrl-s");
    cx.run_until_parked();
    let saved = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap();
    let nla = saved.props.nla.iter().find(|n| n.object == "Cube").expect("the Cube's NLA saved");
    assert_eq!(nla.action, None);
    let laid: Vec<(String, Vec<(String, f32, f32)>)> = nla.tracks.iter().map(|t| (t.name.clone(), t.strips.iter().map(|s| (s.action.clone().unwrap_or_default(), s.start, s.end)).collect())).collect();
    assert_eq!(laid, [(STASH_TRACK.to_string(), vec![("Spin".to_string(), 1.0, 24.0)]), ("CubeAction".to_string(), vec![("CubeAction".to_string(), 5.0, 30.0)])]);
    let _ = std::fs::remove_dir_all(&dir);
}
