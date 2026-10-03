//! The NLA editor edits the stack as Blender 5.0's does, on
//! `nla_cube.blend` (the Cube: Base Layer with Walk 1–97 under a muted
//! Overlay with Walk Again 120–168): a click picks a strip and its track
//! (`nla.click_select`), Shift adds, a click on nothing clears; a click on a
//! track's name picks the track (`anim.channels_click`); A, Alt A, Ctrl I
//! (`nla.select_all`); H mutes (`nla.mute_toggle`) and the Cube stops
//! walking; X deletes (`nla.delete`); Tracks ▸ Add Above Selected and
//! Delete (`nla.tracks_add`, `nla.tracks_delete`). Undo steps them and
//! Ctrl S writes the stack — twice over, the same. phiphx
//! `anim/nla_edit.rs` holds each operator to bpy's results and
//! `blender_opens_saves` has Blender read such a stack back.

use gpui::{Modifiers, TestAppContext, px, size};
use phiber::phiphx::blend::{BlendFile, STRIP_ACTIVE, STRIP_MUTED, STRIP_SELECT, TRACK_ACTIVE, TRACK_SELECTED};
use phiber::phiphx::scene::{SceneModel, SceneNla};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::scene::{BObject3D, scene_objects};
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{CtxKey, ModalKey};
use std::path::PathBuf;

type Cx<'a> = &'a mut gpui::VisualTestContext;

/// `NLATRACK_NAMEWIDTH`, `NLATRACK_HEIGHT`, `NLATRACK_SKIP`, `UI_TIME_SCRUB_MARGIN_Y`.
const CHANNEL_WIDTH: f32 = 200.0;
const ROW: f32 = 24.0;
const SKIP: f32 = 2.0;
const SCRUB: f32 = 23.0;

/// Where frame `frame` of row `row` is on screen (the view is 1 − 10 ..
/// 250 + 10, `nla_create`).
fn at(cx: Cx, frame: f32, row: usize) -> gpui::Point<gpui::Pixels> {
    let window = cx.debug_bounds("time-window").expect("the NLA window");
    let (left, width) = (f32::from(window.origin.x) + CHANNEL_WIDTH, f32::from(window.size.width) - CHANNEL_WIDTH);
    let y = f32::from(window.origin.y) + SCRUB + SKIP + row as f32 * (ROW + SKIP) + ROW / 2.0;
    gpui::point(px(left + (frame + 9.0) / 269.0 * width), px(y))
}

/// A click on row `row`'s name in the channel list.
fn name(cx: Cx, row: usize) -> gpui::Point<gpui::Pixels> {
    let window = cx.debug_bounds("time-window").expect("the NLA window");
    let y = f32::from(window.origin.y) + SCRUB + SKIP + row as f32 * (ROW + SKIP) + ROW / 2.0;
    gpui::point(window.origin.x + px(60.0), px(y))
}

fn click(cx: Cx, point: gpui::Point<gpui::Pixels>, shift: bool) {
    cx.simulate_click(point, if shift { Modifiers::shift() } else { Modifiers::none() });
    cx.run_until_parked();
}

fn keys(cx: Cx, keys: &str) {
    let over = cx.debug_bounds("time-window").expect("the NLA window").center();
    cx.simulate_mouse_move(over, None, Modifiers::none());
    cx.simulate_keystrokes(keys);
    cx.run_until_parked();
}

fn menu(cx: Cx, title: &str, item: &str) {
    let title = cx.debug_bounds(Box::leak(format!("btn-label-toggle-ctx.menu.nla.{title}-text").into_boxed_str())).expect("the menu").center();
    cx.simulate_click(title, Modifiers::none());
    cx.run_until_parked();
    let item = cx.debug_bounds(Box::leak(format!("menu-{item}").into_boxed_str())).unwrap_or_else(|| panic!("{item}")).center();
    cx.simulate_click(item, Modifiers::none());
    cx.run_until_parked();
}

/// The Cube's stack as the editor reads it: each track's name, whether it
/// is selected and active, and its strips' `(name, selected, active, muted)`.
fn stack(cx: Cx) -> Vec<(String, bool, bool, Vec<(String, bool, bool, bool)>)> {
    cx.run_until_parked();
    let nla: SceneNla = phiber::v2::scene::source::with_loaded(|m| m.and_then(|m| m.nla.iter().find(|n| n.object == "Cube").cloned())).expect("the Cube's NLA");
    let bit = |flag: u32, b: u32| flag & b != 0;
    nla.tracks
        .iter()
        .map(|t| (t.name.clone(), bit(t.flag, TRACK_SELECTED), bit(t.flag, TRACK_ACTIVE), t.strips.iter().map(|s| (s.name.clone(), bit(s.flag, STRIP_SELECT), bit(s.flag, STRIP_ACTIVE), bit(s.flag, STRIP_MUTED))).collect()))
        .collect()
}

fn picked(cx: Cx) -> Vec<(String, bool)> {
    stack(cx).into_iter().flat_map(|t| t.3).map(|s| (s.0, s.1)).collect()
}

fn said() -> String {
    phiber::v2::state::log::latest().map(|e| e.text.clone()).unwrap_or_default()
}

fn cube_x(app: &gpui::Entity<BPhiberApp>, cx: Cx, frame: i32) -> f32 {
    app.update(cx, |app, cx| {
        app.state.set(CtxKey::Frame, frame.to_string());
        cx.notify();
    });
    cx.run_until_parked();
    app.read_with(cx, |a, _| match scene_objects(&a.state).iter().find(|o| o.name() == "Cube") {
        Some(BObject3D::Loaded { matrix_world, .. }) => matrix_world[3][0],
        _ => panic!("the Cube"),
    })
}

/// `nla_cube.blend`, copied to `dir`, open in the Animation workspace's
/// NLA editor with the Cube active.
fn open<'a>(cx: &'a mut TestAppContext, dir: &std::path::Path) -> (gpui::Entity<BPhiberApp>, &'a mut gpui::VisualTestContext, PathBuf) {
    std::fs::create_dir_all(dir).unwrap();
    let blend = dir.join("nla_cube.blend");
    std::fs::copy(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/nla_cube.blend"), &blend).unwrap();
    let path = blend.to_string_lossy().to_string();
    let scene = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap();
    let model = SceneModel::from_blend_owned(scene, &path);
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    let space = BSpaceType::Animation;
    let area = space.spec().areas.iter().find(|a| a.default_host == BHostType::Action).copied().expect("the Dope Sheet area");
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(CtxKey::Space, space.slug());
        app.state.set(&phiber::v2::area::area_key(space, &area), BHostType::Nla.slug());
        app.state.set(&phiber::v2::scene::loaded_path_key(), path.clone());
        app.state.set(CtxKey::Object, "Cube");
        app.state.set(CtxKey::FrameStart, "1");
        app.state.set(CtxKey::FrameEnd, "250");
        app.scene_cache = Some((path.clone(), std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    (app, cx, blend)
}

#[gpui::test]
async fn the_nla_editor_picks_mutes_deletes_and_adds_as_blender_does(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_nla_editor_{}", std::process::id()));
    let (app, cx, blend) = open(cx, &dir);
    let (s, a, m) = (true, true, true);
    // Rows: the Cube, its action line, Overlay, Base Layer.
    assert_eq!(stack(cx).iter().map(|t| t.0.as_str()).collect::<Vec<_>>(), ["Base Layer", "Overlay"]);
    assert!((cube_x(&app, cx, 25) - 4.0).abs() < 1e-3, "Walk carries the Cube to x 4 at frame 25");

    // A click on Walk: it alone, active, with its track.
    let p = at(cx, 49.0, 3);
    click(cx, p, false);
    assert_eq!(stack(cx), [("Base Layer".into(), s, a, vec![("Walk".into(), s, a, false)]), ("Overlay".into(), false, false, vec![("Walk Again".into(), false, false, m)])]);
    // H: Walk muted — the Cube stays put.
    keys(cx, "h");
    assert_eq!(said(), "Toggled muting on 1 strip");
    assert!(stack(cx)[0].3[0].3 && cube_x(&app, cx, 25).abs() < 1e-3, "muted, it plays no more");
    // Shift adds Walk Again, active now; a click on nothing clears all.
    let p = at(cx, 144.0, 2);
    click(cx, p, true);
    assert_eq!(picked(cx), [("Walk".into(), s), ("Walk Again".into(), s)]);
    assert!(stack(cx)[1].3[0].2 && !stack(cx)[0].3[0].2, "Walk Again active");
    let p = at(cx, 200.0, 3);
    click(cx, p, false);
    assert_eq!(picked(cx), [("Walk".into(), false), ("Walk Again".into(), false)]);
    assert!(stack(cx).iter().all(|t| !t.1 && !t.2), "no track picked");
    // A, Ctrl I, Alt A.
    keys(cx, "a");
    assert_eq!(picked(cx), [("Walk".into(), s), ("Walk Again".into(), s)]);
    keys(cx, "ctrl-i");
    assert_eq!(picked(cx), [("Walk".into(), false), ("Walk Again".into(), false)]);
    keys(cx, "a alt-a");
    assert_eq!(picked(cx), [("Walk".into(), false), ("Walk Again".into(), false)]);

    // Base Layer's name, then Tracks ▸ Add Above Selected.
    let p = name(cx, 3);
    click(cx, p, false);
    assert_eq!(stack(cx).iter().map(|t| (t.1, t.2)).collect::<Vec<_>>(), [(s, a), (false, false)]);
    menu(cx, "Tracks", "Add Above Selected");
    assert_eq!(stack(cx).iter().map(|t| (t.0.as_str(), t.1, t.2)).collect::<Vec<_>>(), [("Base Layer", s, false), ("NlaTrack", s, a), ("Overlay", false, false)]);
    // Walk Again (Overlay is row 2 still) picked and X: gone.
    let p = at(cx, 144.0, 2);
    click(cx, p, false);
    keys(cx, "x");
    assert_eq!(said(), "Deleted 1 strip");
    assert!(stack(cx)[2].3.is_empty());
    // Overlay's name, Tracks ▸ Delete.
    let p = name(cx, 2);
    click(cx, p, false);
    menu(cx, "Tracks", "Delete");
    assert_eq!(said(), "Deleted 1 track");
    assert_eq!(stack(cx).iter().map(|t| t.0.as_str()).collect::<Vec<_>>(), ["Base Layer", "NlaTrack"]);

    // Undo brings Overlay back; Redo takes it again.
    keys(cx, "ctrl-z");
    assert_eq!(stack(cx).len(), 3);
    keys(cx, "ctrl-shift-z");
    assert_eq!(stack(cx).len(), 2);

    // Ctrl S, twice: the file has the stack, the same both times.
    let saved = |cx: Cx| {
        keys(cx, "ctrl-s");
        let scene = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap();
        let nla = scene.props.nla.into_iter().find(|n| n.object == "Cube").expect("the Cube's NLA");
        nla.tracks.iter().map(|t| (t.name.clone(), t.strips.iter().map(|s| (s.name.clone(), s.flag & STRIP_MUTED != 0)).collect::<Vec<_>>())).collect::<Vec<_>>()
    };
    let once = saved(cx);
    assert_eq!(once, [("Base Layer".to_string(), vec![("Walk".to_string(), true)]), ("NlaTrack".to_string(), vec![])]);
    assert_eq!(saved(cx), once, "a second save writes the same stack");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Add ▸ Action (Shift A, `nla.actionclip_add`) at frame 100, as bpy 5.0
/// adds on the same file (phiphx `nla_edit.rs`): Idle (keys 10–30) into the
/// active Overlay before Walk Again; again, into a new track on top as
/// Idle.001; with no active track, nothing and Blender's report.
#[gpui::test]
async fn add_action_puts_a_strip_at_the_frame_in_the_active_track(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_nla_add_{}", std::process::id()));
    let (app, cx, _) = open(cx, &dir);
    app.update(cx, |app, cx| {
        app.state.set(CtxKey::Frame, "100");
        cx.notify();
    });
    cx.run_until_parked();
    // Shift A opens the Add menu; Action ▸ lists the file's actions.
    keys(cx, "shift-a");
    let entry = cx.debug_bounds("menu-Action").expect("Add ▸ Action").center();
    cx.simulate_mouse_move(entry, None, Modifiers::none());
    cx.run_until_parked();
    assert!(cx.debug_bounds("menu-Walk").is_some(), "the actions listed");
    let idle = cx.debug_bounds("menu-Idle").expect("Idle").center();
    click(cx, idle, false);
    assert_eq!(said(), "Added a strip of Idle");
    let overlay: Vec<String> = stack(cx)[1].3.iter().map(|x| x.0.clone()).collect();
    assert_eq!(overlay, ["Idle", "Walk Again"]);
    let range = phiber::v2::scene::source::with_loaded(|m| m.and_then(|m| m.nla.iter().find(|n| n.object == "Cube").map(|n| (n.tracks[1].strips[0].start, n.tracks[1].strips[0].end))));
    assert_eq!(range, Some((100.0, 120.0)));
    // Again from the menu: no room there — a new track on top.
    let title = cx.debug_bounds("btn-label-toggle-ctx.menu.nla.Add-text").expect("the Add menu").center();
    click(cx, title, false);
    let entry = cx.debug_bounds("menu-Action").expect("Add ▸ Action").center();
    cx.simulate_mouse_move(entry, None, Modifiers::none());
    cx.run_until_parked();
    let idle = cx.debug_bounds("menu-Idle").expect("Idle").center();
    click(cx, idle, false);
    let now = stack(cx);
    assert_eq!((now.len(), now[2].0.as_str(), now[2].2, now[2].3[0].0.as_str()), (3, "NlaTrack", true, "Idle.001"));
    // A click on nothing leaves no active track: nothing goes in.
    let p = at(cx, 230.0, 4);
    click(cx, p, false);
    keys(cx, "shift-a");
    let entry = cx.debug_bounds("menu-Action").expect("Add ▸ Action").center();
    cx.simulate_mouse_move(entry, None, Modifiers::none());
    cx.run_until_parked();
    let walk = cx.debug_bounds("menu-Walk").expect("Walk").center();
    click(cx, walk, false);
    assert_eq!(said(), "No active track(s) to add strip to, select an existing track or add one before trying again");
    assert_eq!(stack(cx).iter().map(|t| t.3.len()).sum::<usize>(), 4);
    let _ = std::fs::remove_dir_all(&dir);
}
