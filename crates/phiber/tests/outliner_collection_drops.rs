//! Collections in the Outliner's drag and drop, as Blender has them
//! (`outliner_dragdrop.cc`): a collection dragged onto another nests in it,
//! its objects along; one dragged into itself, or the Scene Collection
//! anywhere, stays; an object Ctrl-dragged onto a collection is linked
//! there and stays where it was, so hiding one of its collections leaves
//! it shown; a ghost of the row follows the pointer; Undo steps it; and
//! Ctrl S writes it all (Blender reads the same writes back in
//! `phiphx/tests/blender_opens_saves.rs`). Over the factory scene with
//! Props and Lamps beside Collection, the Light in Lamps
//! (`phiphx/tests/fixtures/make_collections.py`).

use gpui::{Modifiers, MouseButton, TestAppContext, px, size};
use phiber::phiphx::blend::{BlendCollection, BlendFile};
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::part::outliner::key::{BHidden, k_restrict};
use phiber::v2::part::outliner::tree::{BOutlineRow, BRowKind, flatten};
use phiber::v2::scene::links::BSceneLinks;
use phiber::v2::state::{BKey as _, ModalKey, OpKey};
use phiber::v2::view::BAction;
use std::path::PathBuf;

const PROPS: &str = "Scene Collection/Props";
const LAMPS: &str = "Scene Collection/Lamps";

fn row(cx: &mut gpui::VisualTestContext, payload: &str) -> gpui::Point<gpui::Pixels> {
    cx.debug_bounds(Box::leak(format!("outliner-row-{payload}").into_boxed_str())).unwrap_or_else(|| panic!("row {payload}")).center()
}

/// Press on `from`, move onto `onto`, and let go there with `held` down;
/// whether the ghost showed on the way.
fn drag(cx: &mut gpui::VisualTestContext, from: &str, onto: &str, held: Modifiers) -> bool {
    let (a, b) = (row(cx, from), row(cx, onto));
    cx.simulate_mouse_down(a, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(gpui::Point::new(a.x, a.y + px(6.0)), Some(MouseButton::Left), Modifiers::none());
    cx.simulate_mouse_move(b, Some(MouseButton::Left), held);
    cx.run_until_parked();
    let ghost = cx.debug_bounds("outliner-drag-ghost").is_some();
    cx.simulate_mouse_up(b, MouseButton::Left, held);
    cx.run_until_parked();
    ghost
}

fn rows(app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext) -> Vec<BOutlineRow> {
    app.read_with(cx, |a, _| flatten(&a.state))
}

/// `(depth, label)` of the rows from the Scene Collection down.
fn outline(rows: &[BOutlineRow]) -> Vec<(u16, String)> {
    rows.iter().filter(|r| matches!(r.kind, BRowKind::SceneCollection | BRowKind::Collection | BRowKind::Object)).map(|r| (r.depth, r.label.clone())).collect()
}

fn find<'a>(c: &'a BlendCollection, name: &str) -> Option<&'a BlendCollection> {
    if c.name == name {
        return Some(c);
    }
    c.children.iter().find_map(|k| find(k, name))
}

#[gpui::test]
async fn collections_nest_objects_link_and_the_save_keeps_it(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_outliner_collections_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let blend = dir.join("collections.blend");
    std::fs::copy(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/collections.blend"), &blend).unwrap();
    let path = blend.to_string_lossy().to_string();
    let scene = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap();
    let model = SceneModel::from_blend_owned(scene, &path);
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(&phiber::v2::scene::loaded_path_key(), path.clone());
        app.scene_cache = Some((path.clone(), std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    let before = outline(&rows(&app, cx));
    assert!(before.contains(&(1, "Lamps".into())) && before.contains(&(2, "Light".into())), "{before:?}");

    // Lamps onto Props: nested, the Light along, and a ghost on the way.
    assert!(drag(cx, &format!("col:{LAMPS}"), &format!("col:{PROPS}"), Modifiers::none()), "the row's ghost follows the pointer");
    let now = outline(&rows(&app, cx));
    let props = now.iter().position(|r| *r == (1, "Props".to_string())).expect("Props");
    assert_eq!(now[props + 1..props + 3], [(2, "Lamps".to_string()), (3, "Light".to_string())], "{now:?}");
    assert_eq!(app.read_with(cx, |a, _| BSceneLinks::now(&a.state).nested), [(LAMPS.to_string(), PROPS.to_string())]);

    // Props into Lamps, now under it: refused. The Scene Collection: stays.
    for op in [format!("col:{PROPS}>col:{LAMPS}"), format!("col:Scene Collection>col:{PROPS}")] {
        app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::OutlinerDrop.path(), op), cx));
        cx.run_until_parked();
    }
    assert_eq!(app.read_with(cx, |a, _| BSceneLinks::now(&a.state).nested).len(), 1, "no loop, no moved root");

    // Undo puts Lamps back; Redo nests it again.
    cx.simulate_keystrokes("ctrl-z");
    cx.run_until_parked();
    assert_eq!(outline(&rows(&app, cx)), before, "Undo");
    cx.simulate_keystrokes("ctrl-shift-z");
    cx.run_until_parked();
    assert_eq!(outline(&rows(&app, cx)), now, "Redo");

    // Ctrl-drag the Cube onto Props: in Props and still in Collection.
    drag(cx, "ob:Cube", &format!("col:{PROPS}"), Modifiers::control());
    let cubes = rows(&app, cx).iter().filter(|r| r.label == "Cube" && r.kind == BRowKind::Object).count();
    assert_eq!(cubes, 2, "linked: a row under each collection");
    assert_eq!(app.read_with(cx, |a, _| BSceneLinks::now(&a.state).linked), [("Cube".to_string(), PROPS.to_string())]);
    // Hiding Props leaves it shown (it is in Collection too); the Light,
    // only in Lamps under Props, goes.
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(k_restrict(&format!("col/{PROPS}"), "hide_get"), "true".into()), cx));
    cx.run_until_parked();
    let hidden = app.read_with(cx, |a, _| {
        let hidden = BHidden::now(&a.state);
        (hidden.hides(&a.state, "Cube"), hidden.hides(&a.state, "Light"))
    });
    assert_eq!(hidden, (false, true), "hidden only where every collection holding it hides it");

    // Ctrl S: the file has the nesting and the link.
    cx.simulate_keystrokes("ctrl-s");
    cx.run_until_parked();
    let saved = BlendFile::parse_bytes(&std::fs::read(&blend).unwrap()).unwrap().extract_scene().unwrap();
    let root = &saved.collections[0];
    let props = find(root, "Props").expect("Props saved");
    assert!(props.children.iter().any(|k| k.name == "Lamps"), "Lamps saved inside Props");
    assert!(!root.children.iter().any(|k| k.name == "Lamps"), "and no longer at the top");
    assert!(find(props, "Lamps").unwrap().object_names.contains(&"Light".to_string()), "the Light came along");
    assert!(props.object_names.contains(&"Cube".to_string()), "the Cube linked into Props: {:?}", props.object_names);
    assert!(find(root, "Collection").unwrap().object_names.contains(&"Cube".to_string()), "and still in Collection");
    let _ = std::fs::remove_dir_all(&dir);
}
