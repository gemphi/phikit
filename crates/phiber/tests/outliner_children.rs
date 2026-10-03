//! Object Children (`use_filter_children`): the Outliner is a table, and a
//! child's row comes straight after its parent's, one level deeper — the
//! depth is the whole tree. The fixture (`tests/fixtures/make_parented.py`,
//! Blender 5.0) is the factory scene with Light parented to Cube, a mesh
//! "Arm" parented to Cube and an empty "Hand" parented to Arm.

use std::path::PathBuf;

use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::part::outliner::tree::{BRowKind, flatten};
use phiber::v2::scene::source::set_loaded_scene;
use phiber::v2::state::{BKey as _, BState, OutlinerKey, val::BValue};

fn load() -> SceneModel {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parented.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    let model = SceneModel::from_blend_owned(scene, &path.to_string_lossy());
    set_loaded_scene(Some(model.clone()));
    model
}

/// `(label, depth)` of the object and data rows.
fn rows(s: &BState) -> Vec<(String, u16, bool)> {
    flatten(s).into_iter().filter(|r| r.kind != BRowKind::Collection && r.kind != BRowKind::SceneCollection).map(|r| (r.label, r.depth, r.kind == BRowKind::Data)).collect()
}

fn open(s: &mut BState, id: &'static str) {
    s.set(OutlinerKey::Expanded(id), "true");
}

#[test]
fn the_file_names_each_objects_parent() {
    let model = load();
    assert_eq!(model.parent_of("Arm"), Some("Cube"));
    assert_eq!(model.parent_of("Hand"), Some("Arm"));
    assert_eq!(model.parent_of("Light"), Some("Cube"));
    assert_eq!(model.parent_of("Cube"), None);
}

#[test]
fn children_follow_their_parent_one_level_deeper() {
    load();
    let mut s = BState::new();
    // Closed: only the roots, and the Cube's closed row counts what it holds.
    let top: Vec<String> = rows(&s).into_iter().map(|(n, _, _)| n).collect();
    assert_eq!(top, ["Camera", "Cube"]);
    let cube = flatten(&s).into_iter().find(|r| r.label == "Cube").unwrap();
    assert_eq!(cube.expanded, Some(false));
    assert_eq!(cube.summary.iter().map(|(_, n)| n).sum::<usize>(), 3, "its mesh, Arm and Light");

    open(&mut s, "Cube");
    open(&mut s, "Arm");
    let d = rows(&s)[1].1;
    assert_eq!(
        rows(&s),
        [
            ("Camera".into(), d, false),
            ("Cube".into(), d, false),
            ("Cube".into(), d + 1, true),
            ("Arm".into(), d + 1, false),
            ("Arm".into(), d + 2, true),
            ("Hand".into(), d + 2, false),
            ("Light".into(), d + 1, false),
        ]
    );

    // Object Children off: one flat list again.
    s.set(OutlinerKey::Filter("use_filter_children").path(), BValue::Bool(false).encode());
    let flat: Vec<(String, u16)> = rows(&s).into_iter().filter(|(_, _, data)| !data).map(|(n, depth, _)| (n, depth)).collect();
    assert_eq!(flat, [("Arm".into(), d), ("Camera".into(), d), ("Cube".into(), d), ("Hand".into(), d), ("Light".into(), d)]);
}

#[test]
fn a_search_lists_the_matches_flat() {
    load();
    let mut s = BState::new();
    s.set(OutlinerKey::Search, "hand");
    let found: Vec<String> = rows(&s).into_iter().map(|(n, _, _)| n).collect();
    assert_eq!(found, ["Hand"]);
}

/// A pick (a click in the viewport) opens the rows above the object — its
/// collection and its parents — so the Outliner can show its row
/// (`outliner_show_active`), and counts the pick for the list to follow.
#[test]
fn a_pick_opens_the_rows_above_it() {
    use phiber::v2::state::{CtxKey, action::apply_action};
    use phiber::v2::view::prim::BAction;
    load();
    let mut s = BState::new();
    let home = flatten(&s).into_iter().find(|r| r.kind == BRowKind::Collection).unwrap();
    s.set(OutlinerKey::Expanded(home.id), "false");
    assert!(!rows(&s).iter().any(|(n, _, _)| n == "Hand"));

    apply_action(&mut s, &BAction::Set(CtxKey::Object.path(), "Hand".into()));
    let listed: Vec<String> = rows(&s).into_iter().filter(|(_, _, data)| !data).map(|(n, _, _)| n).collect();
    assert_eq!(listed, ["Camera", "Cube", "Arm", "Hand", "Light"], "the collection, Cube and Arm open");
    assert_eq!(s.get(OutlinerKey::Reveal), Some("1"));
    apply_action(&mut s, &BAction::Set(CtxKey::Object.path(), "Camera".into()));
    assert_eq!(s.get(OutlinerKey::Reveal), Some("2"), "every pick counts");
}
