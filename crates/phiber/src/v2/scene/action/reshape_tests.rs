//! Tests for `reshape` — split out so the module itself sits under the
//! 240-line read cap (`tests/level_structure.rs`).

use super::*;
use crate::phiphx::blend::{BFCurve, BKey, BlendActionSlot};
use crate::phiphx::scene::SceneAction;

fn curve_at(frames: &[f32]) -> BFCurve {
    let keys = frames.iter().map(|&x| BKey::new([x - 1.0, 0.0], [x, 0.0], [x + 1.0, 0.0], IPO_BEZ)).collect();
    BFCurve::new("location", 0, keys)
}

fn action_with(frames: &[f32]) -> Vec<SceneAction> {
    vec![SceneAction {
        name: "Act".into(),
        slots: vec![BlendActionSlot { handle: 1, identifier: "OBCube".into(), curves: 0..1 }],
        curves: vec![curve_at(frames)],
        ..Default::default()
    }]
}

fn reshape(frames: Vec<f32>, shape: BKeyShape) -> BKeyReshape {
    BKeyReshape { action: "Act".into(), slot: "OBCube".into(), frames, shape }
}

#[test]
fn interpolation_is_set_only_on_the_named_keys() {
    let mut actions = action_with(&[1.0, 5.0, 9.0]);
    reshape(vec![5.0], BKeyShape { ipo: Some(IPO_LIN), ..Default::default() }).apply(&mut actions);
    let keys = &actions[0].curves[0].keys;
    assert_eq!(keys[0].ipo, IPO_BEZ, "the untouched key keeps Bézier");
    assert_eq!(keys[1].ipo, IPO_LIN, "the named key is now linear");
    assert_eq!(keys[2].ipo, IPO_BEZ);
}

#[test]
fn easing_and_handle_set_their_own_fields() {
    let mut actions = action_with(&[1.0]);
    reshape(vec![1.0], BKeyShape { easing: Some(EASE_IN), ..Default::default() }).apply(&mut actions);
    assert_eq!(actions[0].curves[0].keys[0].easing, EASE_IN);
    reshape(vec![1.0], BKeyShape { handle: Some(HANDLE_VECT), ..Default::default() }).apply(&mut actions);
    let key = actions[0].curves[0].keys[0];
    assert_eq!((key.h1_type, key.h2_type), (HANDLE_VECT, HANDLE_VECT), "both handles take the type");
}

#[test]
fn a_reshape_round_trips_through_its_encoding() {
    let r = reshape(vec![1.0, 5.0], BKeyShape { ipo: Some(IPO_SINE), easing: Some(EASE_OUT), handle: None });
    let text = r.encode();
    assert!(BKeyReshape::is_mark(&text));
    assert_eq!(BKeyReshape::parse(&text), Some(r));
}

#[test]
fn a_menu_value_names_its_shape() {
    assert_eq!(BKeyShape::from_value("ipo", "lin"), Some(BKeyShape { ipo: Some(IPO_LIN), ..Default::default() }));
    assert_eq!(BKeyShape::from_value("easing", "inout"), Some(BKeyShape { easing: Some(EASE_IN_OUT), ..Default::default() }));
    assert_eq!(BKeyShape::from_value("handle", "vector"), Some(BKeyShape { handle: Some(HANDLE_VECT), ..Default::default() }));
    assert_eq!(BKeyShape::from_value("ipo", "nonsense"), None);
    assert_eq!(BKeyShape::from_value("what", "lin"), None);
}
