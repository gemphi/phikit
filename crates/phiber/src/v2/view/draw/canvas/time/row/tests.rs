//! Channel row tests — split out so `mod.rs` stays inside the 240-line read
//! budget (`level_structure`).

use super::*;
use crate::phiphx::blend::BNlaTrack;

/// The band the tests measure past — Blender's own number, so these stay
/// readable while the live view takes its height from the theme.
const SCRUB: f32 = super::super::SCRUB_HEIGHT;

fn action(user: &str, keys: &[f32]) -> SceneAction {
    let mut curve = crate::phiphx::blend::BFCurve::keyed("location", 0);
    keys.iter().for_each(|f| curve.insert(*f, 0.0));
    SceneAction { name: format!("{user}Action"), keys: keys.to_vec(), users: vec![user.into()], curves: vec![curve], ..Default::default() }
}

#[test]
fn an_object_row_carries_its_own_columns_and_rows_step_by_the_channel_step() {
    let list = [action("rig", &[0.0, 10.0]), action("Cube", &[5.0])];
    let row = BTimeRow::object("rig", &list);
    assert_eq!(row.keys, vec![0.0, 10.0]);
    assert_eq!(row.name(), "rig");
    assert_eq!(BTimeRow::top(0, ROW_HEIGHT, SCRUB), SCRUB + ROW_SKIP);
    assert_eq!(BTimeRow::top(1, ROW_HEIGHT, SCRUB) - BTimeRow::top(0, ROW_HEIGHT, SCRUB), ROW_HEIGHT + ROW_SKIP);
    assert_eq!(BTimeRow::center(0, ROW_HEIGHT, SCRUB) - BTimeRow::top(0, ROW_HEIGHT, SCRUB), ROW_HEIGHT / 2.0);
    assert_eq!(BTimeRow::height(BHostType::Nla), NLA_ROW_HEIGHT, "NLA rows are `1.2 × widget_unit`");
    assert_eq!(BTimeRow::height(BHostType::Action), ROW_HEIGHT);
}

#[test]
fn the_timeline_lists_the_collapsed_summary_alone() {
    let s = BState::default();
    let axis = super::super::curve::BAxisColors { x: [1.0; 3], y: [1.0; 3], z: [1.0; 3], w: [1.0; 3], other: [1.0; 3] };
    let list = BTimeRow::list(&s, BHostType::Time, axis);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].kind, BRowKind::Summary { expanded: false });
    let sheet = BTimeRow::list(&s, BHostType::Action, axis);
    assert_eq!(sheet[0].kind, BRowKind::Summary { expanded: true });
    assert!(BTimeRow::list(&s, BHostType::Ipo, axis).is_empty(), "the Graph editor lists no summary");
    assert!(BTimeRow::list(&s, BHostType::Drivers, axis).is_empty(), "nor does the Drivers editor, with nothing loaded");
    assert_eq!(BTimeRow { kind: BRowKind::Driver { name: "Hide (a)".into(), color: [0.0; 3] }, keys: Vec::new() }.name(), "Hide (a)");
}

#[test]
fn the_nla_lists_the_object_its_action_line_and_its_tracks_last_first() {
    use crate::phiphx::blend::{BNlaStrip, BlendNla};
    let track = |name: &str| BNlaTrack { name: name.into(), flag: 0, strips: vec![BNlaStrip { name: "Walk".into(), start: 1.0, end: 49.0, ..Default::default() }], address: 0 };
    let nla = BlendNla { object: "Cube".into(), action: Some("Idle".into()), extend: 0, flag: 0, tracks: vec![track("Base Layer"), track("Overlay")], ..Default::default() };
    let idle = SceneAction { name: "Idle".into(), keys: vec![10.0, 30.0], users: vec!["Cube".into()], ..Default::default() };
    let rows = BTimeRow::nla(&nla, &[idle]);
    assert_eq!(rows.iter().map(BTimeRow::name).collect::<Vec<_>>(), ["Cube", "Idle", "Overlay", "Base Layer"]);
    assert_eq!((BTimeRow::track_at(&rows, 2), BTimeRow::track_at(&rows, 3)), (Some(("Cube".into(), 1)), Some(("Cube".into(), 0))), "counted from the bottom");
    assert_eq!(BTimeRow::track_at(&rows, 1), None, "the action line is no track");
    let h = NLA_ROW_HEIGHT;
    assert_eq!((BTimeRow::index_at(BTimeRow::center(3, h, SCRUB), h, SCRUB), BTimeRow::index_at(SCRUB, h, SCRUB)), (Some(3), None));
    assert_eq!(rows[1].keys, vec![10.0, 30.0], "the action line carries the action's key columns");
    let bare = BlendNla { object: "Sphere".into(), ..Default::default() };
    assert_eq!(BTimeRow::nla(&bare, &[])[1].name(), "<No Action>");
}

use crate::phiphx::blend::BFCurve;

/// One object with two curves on it.
fn rig() -> Vec<SceneAction> {
    vec![SceneAction {
        name: "RigAction".into(),
        users: vec!["Rig".into()],
        curves: vec![
            BFCurve::new(r#"pose.bones["Chest"].location"#, 0, Vec::new()),
            BFCurve::new("rotation_quaternion", 0, Vec::new()),
        ],
        ..Default::default()
    }]
}

#[test]
fn a_closed_object_lists_no_curves_and_an_open_one_lists_them_all() {
    // `ACHANNEL_SETTING_EXPAND`. Closed is the default, as Blender leaves an
    // object's channels closed until asked — a rig with three hundred bone
    // curves would otherwise bury every other object on first paint.
    let actions = rig();
    let rows = BTimeRow::curves("Rig", &actions);
    assert_eq!(rows.len(), 2, "one row per F-curve of every action the object uses");
    // And they are named by the one naming function, not by their raw path.
    assert_eq!(rows[0].name(), "X Location (Chest)");
    assert_eq!(rows[1].name(), "W Quaternion Rotation");
}

#[test]
fn a_curve_row_wears_its_own_curves_colour() {
    let actions = rig();
    let rows = BTimeRow::curves("Rig", &actions);
    let BRowKind::Curve { color, .. } = &rows[0].kind else { panic!("{:?}", rows[0].kind) };
    assert_eq!(*color, actions[0].curves[0].color, "the swatch is the curve's own colour");
}

#[test]
fn only_the_objects_own_actions_are_listed() {
    // `users` is what ties an action to an object; an action nobody uses, or
    // one used by someone else, must not appear under this object.
    let mut actions = rig();
    actions.push(SceneAction {
        name: "OtherAction".into(),
        users: vec!["Cube".into()],
        curves: vec![BFCurve::new("location", 1, Vec::new())],
        ..Default::default()
    });
    assert_eq!(BTimeRow::curves("Rig", &actions).len(), 2);
    assert_eq!(BTimeRow::curves("Cube", &actions).len(), 1);
    assert_eq!(BTimeRow::curves("Nobody", &actions).len(), 0);
}

#[test]
fn expansion_is_read_per_object_and_defaults_closed() {
    use crate::v2::state::{DopeKey, intern, val::V_TRUE};
    let mut state = BState::default();
    assert!(!BTimeRow::is_expanded(&state, "Rig"), "closed until asked");
    state.set(DopeKey::Expanded(intern("Rig")), V_TRUE);
    assert!(BTimeRow::is_expanded(&state, "Rig"));
    // One object's fold is not another's — the key carries the name.
    assert!(!BTimeRow::is_expanded(&state, "Cube"));
}

#[test]
fn the_rows_start_below_whatever_band_the_theme_gives_them() {
    // The band was a constant, so a theme with 36px headers still painted a
    // 23px strip and the rows began under the wrong place — and changing the
    // theme's own `scrubbing_height` did nothing at all, because nothing in
    // this window read it.
    const ROW: f32 = 32.0;
    for band in [SCRUB, 36.0, 48.0] {
        assert_eq!(BTimeRow::top(0, ROW, band), band + ROW_SKIP, "band {band}");
        // The hit-test agrees with the layout, which is the thing that breaks
        // when the two take their band from different places: the middle of row
        // 0 finds row 0, and a point inside the band is no row at all.
        assert_eq!(BTimeRow::index_at(BTimeRow::center(0, ROW, band), ROW, band), Some(0), "band {band}");
        assert_eq!(BTimeRow::index_at(band - 1.0, ROW, band), None, "band {band}");
        assert_eq!(BTimeRow::index_at(BTimeRow::center(3, ROW, band), ROW, band), Some(3), "band {band}");
    }
}

#[test]
fn an_open_object_lists_its_groups_and_an_open_group_its_curves() {
    use crate::v2::state::{DopeKey, intern, val::V_TRUE};
    // `animfilter_act_group`: Blender nests a rig's curves under one group a
    // bone (and `Object Transforms`), each summarising its curves' keys.
    let key = |frame: f32| crate::phiphx::blend::BKey::new([frame, 0.0], [frame, 0.0], [frame, 0.0], 2);
    let grouped = |path: &str, group: Option<&str>, frames: &[f32]| crate::phiphx::blend::BFCurve {
        group: group.map(str::to_string),
        ..BFCurve::new(path, 0, frames.iter().map(|&f| key(f)).collect())
    };
    let actions = vec![SceneAction {
        name: "Walk".into(),
        users: vec!["Rig".into()],
        curves: vec![
            grouped("location", Some("Object Transforms"), &[1.0]),
            grouped(r#"pose.bones["chest"].rotation_euler"#, Some("chest"), &[1.0, 5.0]),
            grouped(r#"pose.bones["chest"].location"#, Some("chest"), &[9.0]),
            grouped("hide_viewport", None, &[3.0]),
        ],
        ..Default::default()
    }];
    let mut state = BState::default();
    let rows = BTimeRow::channels(&state, "Rig", &actions);
    let names: Vec<&str> = rows.iter().map(BTimeRow::name).collect();
    assert_eq!(names, ["Object Transforms", "chest", "Disable in Viewports"], "groups closed, then the ungrouped curve");
    assert_eq!(rows[1].keys, [1.0, 5.0, 9.0], "a closed group summarises its curves' keys");
    state.set(DopeKey::Expanded(intern(&BTimeRow::group_key("Rig", "chest"))), V_TRUE);
    let rows = BTimeRow::channels(&state, "Rig", &actions);
    assert_eq!(rows.len(), 5, "the open group lists its two curves under it");
    assert!(matches!(rows[2].kind, BRowKind::Curve { grouped: true, .. }));
    assert!(matches!(rows[4].kind, BRowKind::Curve { grouped: false, .. }));
}

/// One action keying a camera object and its camera data through two
/// slots, the object bound to the first.
fn camera_action() -> SceneAction {
    let key = |frame: f32| crate::phiphx::blend::BKey::new([frame, 0.0], [frame, 0.0], [frame, 0.0], 2);
    let curve = |path: &str, group: &str, frames: &[f32]| crate::phiphx::blend::BFCurve {
        group: Some(group.to_string()),
        ..BFCurve::new(path, 0, frames.iter().map(|&f| key(f)).collect())
    };
    let slot = |handle: i32, identifier: &str, curves: std::ops::Range<usize>| crate::phiphx::blend::BlendActionSlot { handle, identifier: identifier.into(), curves };
    SceneAction {
        name: "CameraAction".into(),
        users: vec!["Camera".into()],
        bindings: vec![("Camera".into(), 1)],
        curves: vec![curve("location", "Object Transforms", &[1.0, 20.0]), curve("lens", "Camera", &[1.0, 40.0])],
        slots: vec![slot(1, "OBCamera", 0..1), slot(2, "CACamera", 1..2)],
        ..Default::default()
    }
}

#[test]
fn the_action_editor_lists_each_slot_of_the_active_action_over_its_groups() {
    use crate::v2::state::{CtxKey, DopeKey, intern, val::V_FALSE};
    let model = crate::phiphx::scene::SceneModel { actions: vec![camera_action()], ..Default::default() };
    crate::v2::scene::source::set_loaded_scene(Some(model));
    let mut s = BState::default();
    s.set(CtxKey::Object, "Camera");
    s.set(&crate::v2::part::modebar::set::mode_key(BHostType::Action), crate::v2::part::action::template::ACTION_EDITOR);
    let axis = super::super::curve::BAxisColors { x: [1.0; 3], y: [1.0; 3], z: [1.0; 3], w: [1.0; 3], other: [1.0; 3] };
    let rows = BTimeRow::list(&s, BHostType::Action, axis);
    let names: Vec<&str> = rows.iter().map(BTimeRow::name).collect();
    assert_eq!(names, ["Summary", "Camera", "Object Transforms", "Camera", "Camera"], "each slot open over its group");
    assert!(matches!(&rows[1].kind, BRowKind::Slot { identifier, object, .. } if identifier == "OBCamera" && object == "Camera"));
    assert!(matches!(&rows[3].kind, BRowKind::Slot { identifier, .. } if identifier == "CACamera"));
    assert_eq!(rows[1].keys, [1.0, 20.0], "a slot carries its own curves' keys");
    assert_eq!(rows[3].keys, [1.0, 40.0]);
    assert_eq!(rows[0].keys, [1.0, 20.0, 40.0], "the summary, every slot's");
    // Shut, a slot keeps its row and drops its groups.
    s.set(DopeKey::Expanded(intern(&BTimeRow::slot_key("CameraAction", "CACamera"))), V_FALSE);
    assert_eq!(BTimeRow::list(&s, BHostType::Action, axis).len(), 4);
    // The Dope Sheet lists the object, whose curves are its own slot's only.
    s.set(&crate::v2::part::modebar::set::mode_key(BHostType::Action), "Dope Sheet");
    let curves: Vec<String> = BTimeRow::curves("Camera", &[camera_action()]).iter().map(|r| r.name().to_string()).collect();
    assert_eq!(curves, ["X Location"], "not the camera data's lens");
    crate::v2::scene::source::set_loaded_scene(None);
}
