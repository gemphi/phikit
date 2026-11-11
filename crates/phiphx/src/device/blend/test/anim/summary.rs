use super::*;
use crate::device::blend::anim::fcurve::{BFCurve, BKey};

fn key(frame: f32) -> BKey {
    BKey::new([frame - 1.0, 0.0], [frame, 0.0], [frame + 1.0, 0.0], 2)
}

#[test]
fn keys_are_the_distinct_frames_across_curves_and_slots() {
    let mut action = BAction { name: "Walk".into(), ..Default::default() };
    action.by_slot.insert(1, vec![
        BFCurve::new("location", 0, vec![key(1.0), key(10.0)]),
        BFCurve::new("location", 1, vec![key(10.0), key(20.0004)]),
    ]);
    action.by_slot.insert(2, vec![BFCurve::new("scale", 0, vec![key(5.0)])]);
    let summary = BlendActionSummary::of(&action);
    assert_eq!(summary.keys, vec![1.0, 5.0, 10.0, 20.0]);
    assert_eq!((summary.frame_start, summary.frame_end), (1.0, 20.0));
    assert_eq!(summary.fcurve_count, 3);
    assert_eq!(summary.name, "Walk");
}

#[test]
fn an_action_without_keys_spans_nothing() {
    let action = BAction { name: "Empty".into(), ..Default::default() };
    let summary = BlendActionSummary::of(&action);
    assert!(summary.keys.is_empty());
    assert_eq!((summary.frame_start, summary.frame_end), (0.0, 0.0));
    assert!(!summary.used_by("Cube"));
}
