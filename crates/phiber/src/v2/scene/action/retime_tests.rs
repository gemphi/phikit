use super::*;
use super::super::keys::BKeyed;
use crate::phiphx::blend::{BFCurve, BlendActionSlot};

fn action() -> SceneAction {
    // Two slots: the Cube's two curves keyed at 1, 10, 20; the Camera's
    // one at 10.
    let curve = |path: &str, keys: &[(f32, f32)]| {
        let mut c = BFCurve::keyed(path, 0);
        keys.iter().for_each(|(f, v)| c.insert(*f, *v));
        c
    };
    let cube = [(1.0, 0.0), (10.0, 2.0), (20.0, 3.0)];
    SceneAction {
        name: "Act".into(),
        curves: vec![curve("location", &cube), curve("scale", &cube), curve("location", &[(10.0, 5.0)])],
        slots: vec![
            BlendActionSlot { handle: 1, identifier: "OBCube".into(), curves: 0..2 },
            BlendActionSlot { handle: 2, identifier: "OBCamera".into(), curves: 2..3 },
        ],
        ..Default::default()
    }
}

fn frames(a: &SceneAction, i: usize) -> Vec<f32> {
    a.curves[i].keys.iter().map(|k| k.co[0]).collect()
}

#[test]
fn a_move_carries_the_keys_and_their_handles_and_drops_what_it_lands_on() {
    let mut list = vec![action()];
    let before = list[0].curves[0].keys[1];
    BKeyRetime { action: "Act".into(), slot: "OBCube".into(), frames: vec![10.0], by: Some(5.0) }.apply(&mut list);
    assert_eq!(frames(&list[0], 0), [1.0, 15.0, 20.0]);
    assert_eq!(frames(&list[0], 2), [10.0], "the Camera's slot is left alone");
    let moved = list[0].curves[0].keys[1];
    assert_eq!(moved.co[1], before.co[1], "its value goes with it");
    assert!(moved.h1[0] < 15.0 && moved.h2[0] > 15.0, "its handles go with it, worked out again");
    // Onto frame 20: the key standing there goes, the moved one stays.
    BKeyRetime { action: "Act".into(), slot: "OBCube".into(), frames: vec![15.0], by: Some(5.0) }.apply(&mut list);
    assert_eq!(frames(&list[0], 0), [1.0, 20.0]);
    assert_eq!(list[0].curves[0].keys[1].co[1], 2.0, "the moved key's value");
    assert_eq!(list[0].keys, [1.0, 10.0, 20.0], "the action's columns recounted");
}

#[test]
fn a_delete_takes_the_keys_and_a_curve_left_empty_goes_its_slot_closing_up() {
    let mut list = vec![action()];
    BKeyRetime { action: "Act".into(), slot: "OBCamera".into(), frames: vec![10.0], by: None }.apply(&mut list);
    assert_eq!(list[0].curves.len(), 2, "the Camera's curve is empty and gone");
    assert_eq!((list[0].slots[0].curves.clone(), list[0].slots[1].curves.clone()), (0..2, 2..2));
    BKeyRetime { action: "Act".into(), slot: "OBCube".into(), frames: vec![1.0, 20.0], by: None }.apply(&mut list);
    assert_eq!(frames(&list[0], 0), [10.0]);
    BKeyRetime { action: "Act".into(), slot: "OBCube".into(), frames: vec![10.0], by: None }.apply(&mut list);
    assert!(list[0].curves.is_empty() && list[0].keys.is_empty());
    assert_eq!(list[0].slots[0].curves, 0..0);
}

#[test]
fn steps_keep_their_order_through_their_key() {
    let insert = BKeyInsert { action: "Act".into(), slot: "OBCube".into(), frame: 30.0, channels: vec![BKeyed { path: "location".into(), index: 0, value: 1.0 }] };
    let list = vec![
        BKeyStep::Insert(insert),
        BKeyStep::Retime(BKeyRetime { action: "Act".into(), slot: "OBCube".into(), frames: vec![1.0, 30.0], by: Some(-2.0) }),
        BKeyStep::Retime(BKeyRetime { action: "Act".into(), slot: String::new(), frames: vec![8.0], by: None }),
    ];
    assert_eq!(BKeyStep::parse_all(&BKeyStep::encode_all(&list)), list);
    let mut actions = vec![action()];
    list.iter().for_each(|s| s.apply(&mut actions));
    assert_eq!(frames(&actions[0], 0), [-1.0, 10.0, 20.0, 28.0], "inserted at 30, then 1 and 30 moved two back; nothing at 8 to delete");
}
