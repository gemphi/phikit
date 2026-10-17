//! Blender's easing curves — `BLI_easing_*` (`blenlib/intern/easing.cc`,
//! after Robert Penner's), as an F-curve key with an easing interpolation
//! (`BEZT_IPO_BACK … BEZT_IPO_SINE`) runs into the next key. Every function
//! takes `(time, begin, change, duration)`: the value `time` into a segment
//! that starts at `begin` and changes by `change` over `duration`.

use std::f32::consts::PI;

/// `eBezTriple_Interpolation`, the easing ones.
pub const IPO_BACK: u8 = 3;
pub const IPO_BOUNCE: u8 = 4;
pub const IPO_CIRC: u8 = 5;
pub const IPO_CUBIC: u8 = 6;
pub const IPO_ELASTIC: u8 = 7;
pub const IPO_EXPO: u8 = 8;
pub const IPO_QUAD: u8 = 9;
pub const IPO_QUART: u8 = 10;
pub const IPO_QUINT: u8 = 11;
pub const IPO_SINE: u8 = 12;

/// `eBezTriple_Easing`.
pub const EASE_AUTO: u8 = 0;
pub const EASE_IN: u8 = 1;
pub const EASE_OUT: u8 = 2;
/// Blender writes this easing but no F-curve here is read with it yet;
/// `easing.rs`'s own tests cover the maths.
pub const EASE_IN_OUT: u8 = 3;

/// `fcurve_eval_keyframes_interpolate`'s easing branch: the value of a
/// segment with interpolation `ipo` and easing `easing` (`AUTO` resolves
/// as Blender does — the bouncy kinds ease out, the rest ease in). `back`,
/// `amplitude` and `period` are the key's own.
#[allow(clippy::too_many_arguments)]
pub fn eased(ipo: u8, easing: u8, time: f32, begin: f32, change: f32, duration: f32, back: f32, amplitude: f32, period: f32) -> f32 {
    let easing = match easing {
        EASE_AUTO => match ipo {
            IPO_BACK | IPO_BOUNCE | IPO_ELASTIC => EASE_OUT,
            _ => EASE_IN,
        },
        other => other,
    };
    let (t, b, c, d) = (time, begin, change, duration);
    match (ipo, easing) {
        (IPO_BACK, EASE_IN) => back_in(t, b, c, d, back),
        (IPO_BACK, EASE_OUT) => back_out(t, b, c, d, back),
        (IPO_BACK, _) => back_in_out(t, b, c, d, back),
        (IPO_BOUNCE, EASE_IN) => bounce_in(t, b, c, d),
        (IPO_BOUNCE, EASE_OUT) => bounce_out(t, b, c, d),
        (IPO_BOUNCE, _) => bounce_in_out(t, b, c, d),
        (IPO_CIRC, EASE_IN) => circ_in(t, b, c, d),
        (IPO_CIRC, EASE_OUT) => circ_out(t, b, c, d),
        (IPO_CIRC, _) => circ_in_out(t, b, c, d),
        (IPO_CUBIC, EASE_IN) => power_in(t, b, c, d, 3),
        (IPO_CUBIC, EASE_OUT) => power_out(t, b, c, d, 3),
        (IPO_CUBIC, _) => power_in_out(t, b, c, d, 3),
        (IPO_ELASTIC, EASE_IN) => elastic_in(t, b, c, d, amplitude, period),
        (IPO_ELASTIC, EASE_OUT) => elastic_out(t, b, c, d, amplitude, period),
        (IPO_ELASTIC, _) => elastic_in_out(t, b, c, d, amplitude, period),
        (IPO_EXPO, EASE_IN) => expo_in(t, b, c, d),
        (IPO_EXPO, EASE_OUT) => expo_out(t, b, c, d),
        (IPO_EXPO, _) => expo_in_out(t, b, c, d),
        (IPO_QUAD, EASE_IN) => power_in(t, b, c, d, 2),
        (IPO_QUAD, EASE_OUT) => power_out(t, b, c, d, 2),
        (IPO_QUAD, _) => power_in_out(t, b, c, d, 2),
        (IPO_QUART, EASE_IN) => power_in(t, b, c, d, 4),
        (IPO_QUART, EASE_OUT) => power_out(t, b, c, d, 4),
        (IPO_QUART, _) => power_in_out(t, b, c, d, 4),
        (IPO_QUINT, EASE_IN) => power_in(t, b, c, d, 5),
        (IPO_QUINT, EASE_OUT) => power_out(t, b, c, d, 5),
        (IPO_QUINT, _) => power_in_out(t, b, c, d, 5),
        (IPO_SINE, EASE_IN) => -c * (t / d * (PI / 2.0)).cos() + c + b,
        (IPO_SINE, EASE_OUT) => c * (t / d * (PI / 2.0)).sin() + b,
        (IPO_SINE, _) => -c / 2.0 * ((PI * t / d).cos() - 1.0) + b,
        // Not an easing kind: linear, so a stray value still lands on `b + c`.
        _ => b + c * (t / d),
    }
}

/// `x^n` easing in: quad, cubic, quart, quint share one shape.
fn power_in(t: f32, b: f32, c: f32, d: f32, n: i32) -> f32 {
    let t = t / d;
    c * t.powi(n) + b
}
fn power_out(t: f32, b: f32, c: f32, d: f32, n: i32) -> f32 {
    let t = t / d - 1.0;
    // Even powers flip sign (`-c * (t^4 - 1)`), odd powers do not (`c * (t^3 + 1)`).
    if n % 2 == 0 { -c * (t.powi(n) - 1.0) + b } else { c * (t.powi(n) + 1.0) + b }
}
fn power_in_out(t: f32, b: f32, c: f32, d: f32, n: i32) -> f32 {
    let t = t / (d / 2.0);
    if t < 1.0 {
        return c / 2.0 * t.powi(n) + b;
    }
    let t = t - 2.0;
    if n % 2 == 0 { -c / 2.0 * (t.powi(n) - 2.0) + b } else { c / 2.0 * (t.powi(n) + 2.0) + b }
}

fn back_in(t: f32, b: f32, c: f32, d: f32, s: f32) -> f32 {
    let t = t / d;
    c * t * t * ((s + 1.0) * t - s) + b
}
fn back_out(t: f32, b: f32, c: f32, d: f32, s: f32) -> f32 {
    let t = t / d - 1.0;
    c * (t * t * ((s + 1.0) * t + s) + 1.0) + b
}
fn back_in_out(t: f32, b: f32, c: f32, d: f32, s: f32) -> f32 {
    let s = s * 1.525;
    let t = t / (d / 2.0);
    if t < 1.0 {
        return c / 2.0 * (t * t * ((s + 1.0) * t - s)) + b;
    }
    let t = t - 2.0;
    c / 2.0 * (t * t * ((s + 1.0) * t + s) + 2.0) + b
}

fn bounce_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let t = t / d;
    if t < 1.0 / 2.75 {
        c * (7.5625 * t * t) + b
    } else if t < 2.0 / 2.75 {
        let t = t - 1.5 / 2.75;
        c * (7.5625 * t * t + 0.75) + b
    } else if t < 2.5 / 2.75 {
        let t = t - 2.25 / 2.75;
        c * (7.5625 * t * t + 0.9375) + b
    } else {
        let t = t - 2.625 / 2.75;
        c * (7.5625 * t * t + 0.984375) + b
    }
}
fn bounce_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    c - bounce_out(d - t, 0.0, c, d) + b
}
fn bounce_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    if t < d / 2.0 { bounce_in(t * 2.0, 0.0, c, d) * 0.5 + b } else { bounce_out(t * 2.0 - d, 0.0, c, d) * 0.5 + c * 0.5 + b }
}

fn circ_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let t = t / d;
    -c * ((1.0 - t * t).max(0.0).sqrt() - 1.0) + b
}
fn circ_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let t = t / d - 1.0;
    c * (1.0 - t * t).max(0.0).sqrt() + b
}
fn circ_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let t = t / (d / 2.0);
    if t < 1.0 {
        return -c / 2.0 * ((1.0 - t * t).max(0.0).sqrt() - 1.0) + b;
    }
    let t = t - 2.0;
    c / 2.0 * ((1.0 - t * t).max(0.0).sqrt() + 1.0) + b
}

fn expo_in(t: f32, b: f32, c: f32, d: f32) -> f32 {
    if t == 0.0 { b } else { c * 2f32.powf(10.0 * (t / d - 1.0)) + b }
}
fn expo_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    if t == d { b + c } else { c * (-(2f32.powf(-10.0 * t / d)) + 1.0) + b }
}
fn expo_in_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    if t == 0.0 {
        return b;
    }
    if t == d {
        return b + c;
    }
    let t = t / (d / 2.0);
    if t < 1.0 {
        return c / 2.0 * 2f32.powf(10.0 * (t - 1.0)) + b;
    }
    let t = t - 1.0;
    c / 2.0 * (-(2f32.powf(-10.0 * t)) + 2.0) + b
}

/// `elastic_blend`: near the start of the segment the sine is blended in
/// from a straight line, so a segment with an amplitude smaller than its
/// change does not jump.
fn elastic_blend(time: f32, change: f32, duration: f32, amplitude: f32, s: f32, mut f: f32) -> f32 {
    if change != 0.0 {
        let t = s.abs();
        if amplitude != 0.0 { f *= amplitude / change.abs() } else { f = 0.0 }
        if (time * duration).abs() < t {
            let l = (time * duration).abs() / t;
            f = f * l + (1.0 - l);
        }
    }
    f
}
fn elastic_in(time: f32, b: f32, c: f32, d: f32, mut amplitude: f32, mut period: f32) -> f32 {
    if time == 0.0 {
        return b;
    }
    let mut t = time / d;
    if t == 1.0 {
        return b + c;
    }
    t -= 1.0;
    if period == 0.0 {
        period = d * 0.3;
    }
    let mut f = 1.0;
    let s = if amplitude == 0.0 || amplitude < c.abs() {
        let s = period / 4.0;
        f = elastic_blend(t, c, d, amplitude, s, f);
        amplitude = c;
        s
    } else {
        period / (2.0 * PI) * (c / amplitude).asin()
    };
    -f * (amplitude * 2f32.powf(10.0 * t) * ((t * d - s) * (2.0 * PI) / period).sin()) + b
}
fn elastic_out(time: f32, b: f32, c: f32, d: f32, mut amplitude: f32, mut period: f32) -> f32 {
    if time == 0.0 {
        return b;
    }
    let mut t = time / d;
    if t == 1.0 {
        return b + c;
    }
    t = -t;
    if period == 0.0 {
        period = d * 0.3;
    }
    let mut f = 1.0;
    let s = if amplitude == 0.0 || amplitude < c.abs() {
        let s = period / 4.0;
        f = elastic_blend(t, c, d, amplitude, s, f);
        amplitude = c;
        s
    } else {
        period / (2.0 * PI) * (c / amplitude).asin()
    };
    f * (amplitude * 2f32.powf(10.0 * t) * ((t * d - s) * (2.0 * PI) / period).sin()) + c + b
}
fn elastic_in_out(time: f32, b: f32, c: f32, d: f32, mut amplitude: f32, mut period: f32) -> f32 {
    if time == 0.0 {
        return b;
    }
    let mut t = time / (d / 2.0);
    if t == 2.0 {
        return b + c;
    }
    t -= 1.0;
    if period == 0.0 {
        period = d * (0.3 * 1.5);
    }
    let mut f = 1.0;
    let s = if amplitude == 0.0 || amplitude < c.abs() {
        let s = period / 4.0;
        f = elastic_blend(t, c, d, amplitude, s, f);
        amplitude = c;
        s
    } else {
        period / (2.0 * PI) * (c / amplitude).asin()
    };
    if t < 0.0 {
        return -0.5 * (f * (amplitude * 2f32.powf(10.0 * t) * ((t * d - s) * (2.0 * PI) / period).sin())) + b;
    }
    let t = -t;
    (f * (amplitude * 2f32.powf(-10.0 * t) * ((t * d - s) * (2.0 * PI) / period).sin())) * 0.5 + c + b
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
