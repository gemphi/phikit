use crate::device::blend::anim::easing::*;

const KINDS: [u8; 10] = [IPO_BACK, IPO_BOUNCE, IPO_CIRC, IPO_CUBIC, IPO_ELASTIC, IPO_EXPO, IPO_QUAD, IPO_QUART, IPO_QUINT, IPO_SINE];

#[test]
fn every_easing_starts_at_begin_and_lands_on_begin_plus_change() {
    for ipo in KINDS {
        for easing in [EASE_AUTO, EASE_IN, EASE_OUT, EASE_IN_OUT] {
            let at = |t: f32| eased(ipo, easing, t, 2.0, 6.0, 10.0, 1.70158, 0.8, 4.1);
            assert!((at(0.0) - 2.0).abs() < 1e-3, "ipo {ipo} easing {easing} starts at begin: {}", at(0.0));
            assert!((at(10.0) - 8.0).abs() < 1e-3, "ipo {ipo} easing {easing} ends at begin + change: {}", at(10.0));
        }
    }
}

#[test]
fn the_plain_kinds_are_monotonic_and_in_out_is_symmetric() {
    for ipo in [IPO_CIRC, IPO_CUBIC, IPO_EXPO, IPO_QUAD, IPO_QUART, IPO_QUINT, IPO_SINE] {
        let mut last = f32::NEG_INFINITY;
        for step in 0..=20 {
            let v = eased(ipo, EASE_IN_OUT, step as f32 * 0.5, 0.0, 1.0, 10.0, 0.0, 0.0, 0.0);
            assert!(v >= last - 1e-6, "ipo {ipo} climbs: {last} then {v}");
            last = v;
        }
        let half = eased(ipo, EASE_IN_OUT, 5.0, 0.0, 1.0, 10.0, 0.0, 0.0, 0.0);
        assert!((half - 0.5).abs() < 1e-4, "ipo {ipo} in-out is half way at half time: {half}");
    }
}

#[test]
fn back_overshoots_and_bounce_never_goes_below_its_floor() {
    let mut below = false;
    let mut peak = 0.0f32;
    for step in 0..=100 {
        let t = step as f32 * 0.1;
        below |= eased(IPO_BACK, EASE_IN, t, 0.0, 1.0, 10.0, 1.70158, 0.0, 0.0) < -1e-6;
        peak = peak.max(eased(IPO_BOUNCE, EASE_OUT, t, 0.0, 1.0, 10.0, 0.0, 0.0, 0.0));
        assert!(eased(IPO_BOUNCE, EASE_OUT, t, 0.0, 1.0, 10.0, 0.0, 0.0, 0.0) >= -1e-6);
    }
    assert!(below, "back eases in by dipping under its start");
    assert!((peak - 1.0).abs() < 1e-5, "a bounce out reaches its target");
    // Auto: back, bounce and elastic ease out; the rest ease in.
    assert_eq!(eased(IPO_BACK, EASE_AUTO, 3.0, 0.0, 1.0, 10.0, 1.7, 0.0, 0.0), eased(IPO_BACK, EASE_OUT, 3.0, 0.0, 1.0, 10.0, 1.7, 0.0, 0.0));
    assert_eq!(eased(IPO_QUAD, EASE_AUTO, 3.0, 0.0, 1.0, 10.0, 0.0, 0.0, 0.0), eased(IPO_QUAD, EASE_IN, 3.0, 0.0, 1.0, 10.0, 0.0, 0.0, 0.0));
}
