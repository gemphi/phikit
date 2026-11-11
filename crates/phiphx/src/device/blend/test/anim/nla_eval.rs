use crate::device::blend::anim::nla_eval::*;

fn moving_x(from: (f32, f32), to: (f32, f32)) -> BFCurve {
    let key = |(f, v): (f32, f32)| {
        let mut k = super::super::fcurve::BKey::new([f - 1.0, v], [f, v], [f + 1.0, v], 1);
        k.ipo = super::super::fcurve::IPO_LIN;
        k
    };
    BFCurve::new("location", 0, vec![key(from), key(to)])
}

#[test]
fn a_held_strip_plays_its_action_mapped_and_holds_either_side() {
    // CubeAction pushed down: keys 1 → 0 and 20 → 3, strip 5–30 over
    // action frames 5–30 (its manual range).
    let strip = BNlaStrip::clip("CubeAction", (1, "OBCube"), (5.0, 30.0), true, true);
    let track = BMotionTrack { flag: 0, clips: vec![BMotionClip { strip, curves: vec![moving_x((1.0, 0.0), (20.0, 3.0))] }] };
    let x = |frame: f32| {
        let mut raw = ObjectTransformRaw::default();
        BMotionTrack::play(std::slice::from_ref(&track), false, &mut raw, frame);
        raw.loc[0]
    };
    let at = |f: f32| 3.0 * (f - 1.0) / 19.0;
    assert!((x(0.0) - at(5.0)).abs() < 1e-5, "held before: the strip's first frame");
    assert!((x(10.0) - at(10.0)).abs() < 1e-5);
    assert!((x(40.0) - 3.0).abs() < 1e-5, "held after");
    let muted = BMotionTrack { flag: TRACK_MUTED, ..track.clone() };
    let mut raw = ObjectTransformRaw::default();
    BMotionTrack::play(&[muted], false, &mut raw, 10.0);
    assert_eq!(raw.loc[0], 0.0, "a muted track plays nothing");
    // Twice as slow, repeated: the action's time wraps.
    let slow = BNlaStrip { scale: 2.0, repeat: 2.0, end: 5.0 + 2.0 * 2.0 * 25.0, ..BNlaStrip::clip("A", (1, "OBA"), (5.0, 30.0), true, false) };
    assert!((slow.action_frame(5.0 + 2.0 * 10.0) - 15.0).abs() < 1e-5);
    assert!((slow.action_frame(5.0 + 2.0 * 30.0) - 10.0).abs() < 1e-5, "the second time through");
}
