//! The Outliner's Filter popover (`OUTLINER_PT_filter`) decides what the
//! View Layer tree holds: object types, Objects, Object Contents,
//! Collections, the object state and its Invert, and how the search text
//! matches (Exact Match, Case Sensitive). Over Blender's factory startup
//! file: Cube, Light and Camera in "Collection".

use std::path::PathBuf;

use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::part::outliner::filter::{BOutlineFilter, popover, set};
use phiber::v2::part::outliner::tree::{BRowKind, flatten};
use phiber::v2::scene::source::set_loaded_scene;
use phiber::v2::state::{BKey as _, BState, CtxKey, OutlinerKey, val::BValue};
use phiber::v2::view::BView;

fn load_startup() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/blend/startup-5.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    set_loaded_scene(Some(SceneModel::from_blend_owned(scene, &path.to_string_lossy())));
}

fn objects(s: &BState) -> Vec<(String, u16)> {
    flatten(s).into_iter().filter(|r| r.kind == BRowKind::Object).map(|r| (r.label, r.depth)).collect()
}

fn names(s: &BState) -> Vec<String> {
    objects(s).into_iter().map(|(n, _)| n).collect()
}

fn set_box(s: &mut BState, rna: &'static str, on: bool) {
    s.set(OutlinerKey::Filter(rna).path(), BValue::Bool(on).encode());
}

#[test]
fn the_type_boxes_objects_and_contents_decide_the_rows() {
    load_startup();
    let mut s = BState::new();
    assert_eq!(names(&s), ["Camera", "Cube", "Light"], "everything, sorted");

    set_box(&mut s, "use_filter_object_mesh", false);
    set_box(&mut s, "use_filter_object_camera", false);
    assert_eq!(names(&s), ["Light"], "Meshes and Cameras off");
    set_box(&mut s, "use_filter_object_mesh", true);
    set_box(&mut s, "use_filter_object_camera", true);

    // Object Contents off: the Cube row is a leaf, with no mesh under it.
    let cube = |s: &BState| flatten(s).into_iter().find(|r| r.label == "Cube").unwrap();
    assert!(cube(&s).expanded.is_some(), "the Cube opens onto its mesh");
    set_box(&mut s, "use_filter_object_content", false);
    assert!(cube(&s).expanded.is_none() && cube(&s).summary.is_empty());
    set_box(&mut s, "use_filter_object_content", true);

    // Collections off: the objects hang straight off the Scene Collection.
    let depth = objects(&s)[0].1;
    set_box(&mut s, "use_filter_collection", false);
    assert!(flatten(&s).iter().all(|r| r.kind != BRowKind::Collection));
    assert_eq!(objects(&s)[0].1, depth - 1);
    set_box(&mut s, "use_filter_collection", true);

    set_box(&mut s, "use_filter_object", false);
    assert!(names(&s).is_empty(), "Objects off: collections only");
}

#[test]
fn the_object_state_and_its_invert_filter_by_selection() {
    load_startup();
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    s.set(OutlinerKey::Filter(set::STATE_KEY).path(), BValue::Enum("ACTIVE".into()).encode());
    assert_eq!(names(&s), ["Cube"]);
    set_box(&mut s, "filter_invert", true);
    assert_eq!(names(&s), ["Camera", "Light"]);
    s.set(OutlinerKey::Filter(set::STATE_KEY).path(), BValue::Enum("ALL".into()).encode());
    assert_eq!(names(&s).len(), 3, "Invert means nothing over All");
}

#[test]
fn exact_match_and_case_sensitive_change_how_search_matches() {
    load_startup();
    let mut s = BState::new();
    s.set(OutlinerKey::Search, "cu");
    assert_eq!(names(&s), ["Cube"], "part of the name, any case");
    set_box(&mut s, "use_filter_case_sensitive", true);
    assert!(names(&s).is_empty(), "\"cu\" is not in \"Cube\" once case counts");
    s.set(OutlinerKey::Search, "Cu");
    assert_eq!(names(&s), ["Cube"]);
    set_box(&mut s, "use_filter_complete", true);
    assert!(names(&s).is_empty(), "Exact Match wants the whole name");
    s.set(OutlinerKey::Search, "Cube");
    assert_eq!(names(&s), ["Cube"]);
    assert!(BOutlineFilter::now(&s).hits("Cube") && !BOutlineFilter::now(&s).hits("cube"));
}

#[test]
fn the_popover_lays_out_blenders_sub_panels() {
    let s = BState::new();
    let headings: Vec<String> = popover::body(&s)
        .iter()
        .filter_map(|row| match row {
            BView::Split { list } => match list.first() {
                Some(BView::Label { text }) if !text.is_empty() => Some(text.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(headings, ["Columns", "Sort", "Search", "Filter", "Objects", "Types"]);
}

/// In the app: the Filter button opens the popover, its sections are
/// painted, and ticking a box keeps it open (a panel, not a menu).
#[gpui::test]
async fn the_popover_opens_and_stays_open_while_boxes_are_ticked(cx: &mut gpui::TestAppContext) {
    use gpui::{px, size};
    use phiber::v2::app::BPhiberApp;
    use phiber::v2::state::ModalKey;
    use phiber::v2::view::BAction;
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.handle_action(&BAction::Set(OutlinerKey::FilterOpen.path(), "true".into()), cx);
    });
    cx.run_until_parked();
    for section in ["section-Search", "section-Filter", "section-Types"] {
        assert!(cx.debug_bounds(section).is_some(), "{section} is painted");
    }
    app.update(cx, |app, cx| {
        app.handle_action(&BAction::Set(OutlinerKey::Filter("use_filter_object_camera").path(), BValue::Bool(false).encode()), cx);
    });
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(OutlinerKey::FilterOpen).map(str::to_string)), Some("true".into()));
    assert!(!app.read_with(cx, |a, _| BOutlineFilter::is_on(&a.state, &set::TYPES[3].0)), "Cameras off");
}
