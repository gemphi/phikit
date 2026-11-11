use crate::device::blend::anim::motion::*;
use crate::device::blend::anim::fcurve::BKey;

fn located(x: f32) -> ObjectTransformRaw {
    ObjectTransformRaw { loc: [x, 0.0, 0.0], ..Default::default() }
}

fn moving_x(from: f32, to: f32) -> BFCurve {
    BFCurve::new(
        "location",
        0,
        vec![BKey::new([0.0, from], [1.0, from], [2.0, from], 1), BKey::new([10.0, to], [11.0, to], [12.0, to], 1)],
    )
}

#[test]
fn a_still_object_keeps_its_rest_transform_at_every_frame() {
    let motion = BlendMotion::still("Cube", located(3.0));
    assert!(!motion.animated());
    assert_eq!(motion.local_at(1.0)[3][0], 3.0);
    assert_eq!(motion.local_at(250.0)[3][0], 3.0);
    assert_eq!(motion.range(), None);
}

#[test]
fn a_keyed_object_moves_between_its_keys_and_holds_outside_them() {
    let motion = BlendMotion { curves: vec![moving_x(0.0, 10.0)], ..BlendMotion::still("Cube", located(3.0)) };
    assert!(motion.animated());
    assert_eq!(motion.range(), Some((1.0, 11.0)));
    assert!((motion.local_at(1.0)[3][0] - 0.0).abs() < 1e-5, "the curve overrides the rest location");
    assert!((motion.local_at(6.0)[3][0] - 5.0).abs() < 1e-5, "linear between the keys");
    assert!((motion.local_at(100.0)[3][0] - 10.0).abs() < 1e-5, "holds the last key");
}

#[test]
fn a_child_moves_with_its_animated_parent() {
    let parent = BlendMotion { curves: vec![moving_x(0.0, 10.0)], ..BlendMotion::still("Parent", located(0.0)) };
    let child = BlendMotion { parent: Some(0), ..BlendMotion::still("Child", located(1.0)) };
    let list = BlendMotionList { list: vec![parent, child] };
    assert!(list.animated());
    let at_start = list.world_at(1.0);
    let at_end = list.world_at(11.0);
    assert!((at_start[1][3][0] - 1.0).abs() < 1e-5);
    assert!((at_end[1][3][0] - 11.0).abs() < 1e-5, "the child rides the parent's 10 units");
    assert_eq!(list.named("Child").map(|m| m.parent), Some(Some(0)));
}

#[test]
fn a_parent_cycle_leaves_each_on_its_local() {
    let a = BlendMotion { parent: Some(1), ..BlendMotion::still("A", located(1.0)) };
    let b = BlendMotion { parent: Some(0), ..BlendMotion::still("B", located(2.0)) };
    let world = BlendMotionList { list: vec![a, b] }.world_at(1.0);
    // Solving A asks for B, which asks for A again: the cycle is cut
    // there, B composes on A's local, A on B's world.
    assert_eq!((world[0][3][0], world[1][3][0]), (4.0, 3.0));
}
