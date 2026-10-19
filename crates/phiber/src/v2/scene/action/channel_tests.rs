//! Tests for `channel` — split out so the module itself sits under the
//! 240-line read cap (`tests/level_structure.rs`).

use super::*;
use crate::phiphx::blend::{BFCurve, BKey, BlendActionSlot, EXTRAPOLATE_CONSTANT, EXTRAPOLATE_LINEAR, IPO_BEZ};
use crate::phiphx::scene::SceneAction;

fn curve(path: &str) -> BFCurve {
    BFCurve::new(path, 0, vec![BKey::new([0.0; 2], [1.0, 0.0], [2.0; 2], IPO_BEZ)])
}

fn actions() -> Vec<SceneAction> {
    vec![SceneAction {
        name: "Act".into(),
        slots: vec![BlendActionSlot { handle: 1, identifier: "OBCube".into(), curves: 0..2 }],
        curves: vec![curve("location"), curve("scale")],
        ..Default::default()
    }]
}

fn id(path: &str) -> BChannelId {
    BChannelId { action: "Act".into(), slot: "OBCube".into(), path: path.into(), index: 0 }
}

#[test]
fn extend_and_flags_apply_to_the_named_curve_only() {
    let mut acts = actions();
    BChannelEdit { id: id("location"), extend: Some(EXTRAPOLATE_LINEAR), protect: Some(true), mute: None }.apply(&mut acts);
    let loc = acts[0].curves.iter().find(|c| c.path == "location").unwrap();
    let scale = acts[0].curves.iter().find(|c| c.path == "scale").unwrap();
    assert_eq!(loc.extend, EXTRAPOLATE_LINEAR);
    assert!(loc.flag & FLAG_PROTECTED != 0, "location is protected");
    assert_eq!(scale.extend, EXTRAPOLATE_CONSTANT, "scale is untouched");
    assert!(scale.flag & FLAG_PROTECTED == 0);
}

#[test]
fn mute_clears_as_well_as_sets() {
    let mut acts = actions();
    BChannelEdit { id: id("location"), extend: None, protect: None, mute: Some(true) }.apply(&mut acts);
    assert!(acts[0].curves[0].flag & FLAG_MUTED != 0);
    BChannelEdit { id: id("location"), extend: None, protect: None, mute: Some(false) }.apply(&mut acts);
    assert!(acts[0].curves[0].flag & FLAG_MUTED == 0, "off clears the bit");
}

#[test]
fn a_channel_edit_round_trips_through_its_encoding() {
    let e = BChannelEdit { id: id("pose.bones[\"a\"].location"), extend: Some(EXTRAPOLATE_LINEAR), protect: None, mute: Some(true) };
    assert_eq!(BChannelEdit::parse_all(&BChannelEdit::encode_all(std::slice::from_ref(&e))), vec![e]);
}

#[test]
fn toggle_resolves_against_the_current_flag() {
    assert!(resolve("toggle", false));
    assert!(!resolve("toggle", true));
    assert!(resolve("on", false));
    assert!(!resolve("off", true));
}
