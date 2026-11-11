use crate::device::blend::anim::fcurve::*;

fn key(x: f32, y: f32, ipo: u8) -> BKey {
    BKey::new([x - 1.0, y], [x, y], [x + 1.0, y], ipo)
}

#[test]
fn holds_its_ends_and_interpolates_between() {
    let c = BFCurve::new("location", 0, vec![key(1.0, 0.0, 1), key(11.0, 10.0, 1)]);
    assert_eq!(c.eval(-5.0), Some(0.0));
    assert_eq!(c.eval(50.0), Some(10.0));
    assert!((c.eval(6.0).unwrap() - 5.0).abs() < 1e-5);
}

#[test]
fn a_flat_bezier_eases_in_and_out() {
    // Flat handles: the curve leaves 0 slowly and arrives at 10 slowly,
    // so it is below the line early and above it late.
    let c = BFCurve::new("location", 0, vec![key(0.0, 0.0, 2), key(10.0, 10.0, 2)]);
    let early = c.eval(2.0).unwrap();
    let late = c.eval(8.0).unwrap();
    assert!(early < 2.0 && early > 0.0, "{early}");
    assert!(late > 8.0 && late < 10.0, "{late}");
    assert!((c.eval(5.0).unwrap() - 5.0).abs() < 1e-4);
}

#[test]
fn evaluation_is_zero_without_keys_and_the_generator_replaces_the_keys() {
    let empty = BFCurve::new("influence", 0, vec![]);
    assert_eq!(empty.evaluate(0.3), 0.0, "`evaluate_fcurve` on a keyless driver");
    let mut inverted = BFCurve::new("mute", 0, vec![key(0.0, 5.0, 1), key(1.0, 5.0, 1)]);
    inverted.generator = Some(vec![1.0, -1.0]);
    assert!((inverted.evaluate(0.25) - 0.75).abs() < 1e-6, "the Generator's `1 − x`, not the keys' 5");
    let keyed = BFCurve::new("location", 0, vec![key(1.0, 0.0, 1), key(11.0, 10.0, 1)]);
    assert!((keyed.evaluate(6.0) - 5.0).abs() < 1e-5);
}

#[test]
fn paths_split_into_bone_and_property() {
    let c = BFCurve::new("pose.bones[\"chest\"].rotation_quaternion", 1, vec![]);
    assert_eq!(c.bone(), Some("chest"));
    assert_eq!(c.property(), "rotation_quaternion");
    let o = BFCurve::new("location", 0, vec![]);
    assert_eq!(o.bone(), None);
    assert_eq!(o.property(), "location");
}

#[test]
fn an_easing_key_eases_into_the_next_and_a_muted_curve_is_switched_off() {
    let mut a = key(0.0, 0.0, super::super::easing::IPO_QUAD);
    a.easing = super::super::easing::EASE_IN;
    let c = BFCurve::new("location", 0, vec![a, key(10.0, 10.0, 1)]);
    assert!((c.eval(5.0).unwrap() - 2.5).abs() < 1e-5, "quad in: a quarter of the way at half time");
    assert_eq!(c.eval(10.0), Some(10.0));
    let mut muted = c.clone();
    muted.flag |= FLAG_MUTED;
    assert!(muted.muted() && !c.muted());
}

#[test]
fn linear_extrapolation_runs_on_along_the_end_slopes() {
    // A Bézier key's handles give the slope; a linear key takes the next key's.
    let mut c = BFCurve::new(
        "location",
        0,
        vec![BKey::new([-1.0, -2.0], [0.0, 0.0], [1.0, 2.0], 2), BKey::new([9.0, 10.0], [10.0, 10.0], [11.0, 10.0], 2)],
    );
    assert_eq!(c.eval(-5.0), Some(0.0), "constant extrapolation holds");
    c.extend = EXTRAPOLATE_LINEAR;
    assert!((c.eval(-5.0).unwrap() + 10.0).abs() < 1e-5, "slope 2 from the left handle: {:?}", c.eval(-5.0));
    assert!((c.eval(15.0).unwrap() - 10.0).abs() < 1e-5, "a flat right handle holds the value");
    let mut lin = BFCurve::new("location", 0, vec![key(0.0, 0.0, 1), key(10.0, 10.0, 1)]);
    lin.extend = EXTRAPOLATE_LINEAR;
    assert!((lin.eval(20.0).unwrap() - 20.0).abs() < 1e-5, "a linear key runs on through the previous key");
    assert!((lin.eval(-10.0).unwrap() + 10.0).abs() < 1e-5);
}
