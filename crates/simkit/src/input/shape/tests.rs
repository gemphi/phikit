//! What [`AxisShape`](super::AxisShape) promises: the order of the
//! steps, and what each knob does on its own.

use super::*;

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
}

#[test]
fn a_dead_zone_swallows_a_resting_stick_and_keeps_its_ends() {
    let s = AxisShape::stick().with_dead_zone(0.2).with_expo(0.0);
    assert_eq!(s.apply(0.0), 0.0, "a centred stick asks for nothing");
    assert_eq!(s.apply(0.19), 0.0, "and so does one inside the zone");
    assert_eq!(s.apply(-0.2), 0.0, "either side of centre");
    assert!(close(s.apply(1.0), 1.0), "full travel still reaches full");
    assert!(close(s.apply(-1.0), -1.0));
    // Half of what is left of the travel is half the demand.
    assert!(close(s.apply(0.6), 0.5), "got {}", s.apply(0.6));
}

#[test]
fn a_levers_dead_zone_sits_at_its_stop() {
    let s = AxisShape::lever().with_dead_zone(0.1);
    assert_eq!(s.apply(0.0), 0.0, "a closed throttle is closed");
    assert_eq!(s.apply(0.09), 0.0, "stiction off the stop reads closed");
    assert!(close(s.apply(1.0), 1.0), "and open is open");
    assert!(close(s.apply(0.55), 0.5));
    assert_eq!(s.apply(-3.0), 0.0, "a lever never goes below its stop");
}

#[test]
fn expo_softens_the_middle_and_leaves_the_ends() {
    let straight = AxisShape::stick().with_dead_zone(0.0).with_expo(0.0);
    let curved = AxisShape::stick().with_dead_zone(0.0).with_expo(1.0);
    assert!(close(curved.apply(1.0), 1.0), "the end is still the end");
    assert!(close(curved.apply(-1.0), -1.0));
    assert!(
        curved.apply(0.5) < straight.apply(0.5),
        "half stick asks for less with expo on: {} vs {}",
        curved.apply(0.5),
        straight.apply(0.5)
    );
    assert!(close(curved.apply(0.5), 0.125), "a full cube at half travel");
    assert!(
        close(curved.apply(-0.5), -0.125),
        "and the curve is odd, so pulling matches pushing"
    );
}

#[test]
fn trim_biases_the_demand_and_scale_gains_it() {
    let s = AxisShape::stick().with_dead_zone(0.0).with_expo(0.0).with_trim(0.1);
    assert!(close(s.apply(0.0), 0.1), "a trimmed stick asks for something at rest");
    assert!(close(s.apply(0.4), 0.5));
    assert!(close(s.apply(1.0), 1.0), "and the trim cannot push past the stop");

    let half = AxisShape::stick().with_dead_zone(0.0).with_expo(0.0).with_scale(0.5);
    assert!(close(half.apply(1.0), 0.5), "a halved axis gives half");
}

#[test]
fn inversion_flips_a_stick_and_a_lever_alike() {
    let stick = AxisShape::stick().with_dead_zone(0.0).with_expo(0.0).inverted();
    assert!(close(stick.apply(1.0), -1.0));
    assert!(close(stick.apply(-1.0), 1.0));
    assert!(close(stick.apply(0.0), 0.0), "inverting leaves the centre alone");

    let lever = AxisShape::lever().with_dead_zone(0.0).inverted();
    assert!(close(lever.apply(0.0), 1.0), "a reversed lever is open at its stop");
    assert!(close(lever.apply(1.0), 0.0));
}

#[test]
fn a_centred_stick_reads_as_a_lever_with_centre_at_half() {
    let s = AxisShape::stick_as_lever().with_dead_zone(0.0);
    assert!(close(s.apply(0.0), 0.5), "centre is half, so centre holds height");
    assert!(close(s.apply(1.0), 1.0));
    assert!(close(s.apply(-1.0), 0.0));
}

#[test]
fn the_steps_run_in_a_radios_order() {
    // Dead zone before expo: at 0.6 of travel past a 0.2 zone the
    // live value is 0.5, and a full cube of that is 0.125 — not the
    // 0.216 a cube of the raw 0.6 would give.
    let s = AxisShape::stick().with_dead_zone(0.2).with_expo(1.0);
    assert!(close(s.apply(0.6), 0.125), "got {}", s.apply(0.6));

    // Trim after expo, so trimming out a drift is a straight offset
    // and not something the curve bends.
    let t = s.with_trim(0.25);
    assert!(close(t.apply(0.6), 0.375), "got {}", t.apply(0.6));
}
