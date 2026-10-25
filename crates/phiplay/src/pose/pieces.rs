//! The pose pieces, as make_demo wrote them: an arm, a leg, the trunk, a
//! running stride. Angles about the armature's axes, in radians.

use super::BDeltas;
use glam::{Quat, Vec3};

pub fn q(axis: Vec3, angle: f32) -> Quat {
    Quat::from_axis_angle(axis, angle)
}

const X: Vec3 = Vec3::X;
const Y: Vec3 = Vec3::Y;
const Z: Vec3 = Vec3::Z;

/// The finger bones of a hand (Rigify's deform names), curled together.
pub const FINGERS: [[&str; 15]; 2] = [
    [
        "DEF-thumb.01.L", "DEF-thumb.02.L", "DEF-thumb.03.L", "DEF-f_index.01.L", "DEF-f_index.02.L", "DEF-f_index.03.L", "DEF-f_middle.01.L",
        "DEF-f_middle.02.L", "DEF-f_middle.03.L", "DEF-f_ring.01.L", "DEF-f_ring.02.L", "DEF-f_ring.03.L", "DEF-f_pinky.01.L", "DEF-f_pinky.02.L",
        "DEF-f_pinky.03.L",
    ],
    [
        "DEF-thumb.01.R", "DEF-thumb.02.R", "DEF-thumb.03.R", "DEF-f_index.01.R", "DEF-f_index.02.R", "DEF-f_index.03.R", "DEF-f_middle.01.R",
        "DEF-f_middle.02.R", "DEF-f_middle.03.R", "DEF-f_ring.01.R", "DEF-f_ring.02.R", "DEF-f_ring.03.R", "DEF-f_pinky.01.R", "DEF-f_pinky.02.R",
        "DEF-f_pinky.03.R",
    ],
];

/// Which side: 0 left (+x), 1 right.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    L,
    R,
}

impl Side {
    fn sign(self) -> f32 {
        if self == Side::L { 1.0 } else { -1.0 }
    }
    fn bone(self, l: &'static str, r: &'static str) -> &'static str {
        if self == Side::L { l } else { r }
    }
}

/// An arm `lower`ed from the T (about Y; 1.25 hangs it), `swing` about X
/// (negative forward and up), the elbow flexed; the clavicle takes a share
/// of the swing and rises when the arm goes overhead.
pub fn arm(d: &mut BDeltas, side: Side, lower: f32, swing: f32, flex: f32, curl: f32, wrist: f32) {
    let s = side.sign();
    let overhead = super::ramp(-swing, 1.4, 2.7);
    let clav = (0.15 * swing).clamp(-0.3, 0.3);
    d.set(side.bone("DEF-shoulder.L", "DEF-shoulder.R"), q(X, clav) * q(Y, s * (0.18 * (lower / 1.25).clamp(0.0, 1.0) - 0.30 * overhead)));
    d.set(side.bone("DEF-upper_arm.L", "DEF-upper_arm.R"), q(X, swing - clav) * q(Y, s * (lower - 0.18)));
    d.set(side.bone("DEF-forearm.L", "DEF-forearm.R"), q(Z, -s * flex));
    d.set(side.bone("DEF-hand.L", "DEF-hand.R"), q(Y, s * wrist));
    for finger in FINGERS[usize::from(side == Side::R)] {
        d.set(finger, q(Y, s * if finger.contains("thumb") { 0.45 * curl } else { curl }));
    }
}

/// A leg: the thigh swung forward by `thigh` (and `out` to the side), the
/// knee folded by `knee`, the ankle by `foot`.
pub fn leg(d: &mut BDeltas, side: Side, thigh: f32, knee: f32, foot: f32, out: f32) {
    let s = side.sign();
    d.set(side.bone("DEF-thigh.L", "DEF-thigh.R"), q(X, -thigh) * q(Y, -s * out));
    d.set(side.bone("DEF-shin.L", "DEF-shin.R"), q(X, knee));
    d.set(side.bone("DEF-foot.L", "DEF-foot.R"), q(X, foot));
}

/// The spine bent forward by `lean` over its four bones, banked into a turn
/// and twisted; the neck takes the lean back so the eyes stay level, and
/// the head turns by `look`.
pub fn trunk(d: &mut BDeltas, lean: f32, twist: f32, bank: f32, look: f32, chin: f32) {
    d.set("DEF-spine", q(X, 0.30 * lean) * q(Y, -0.6 * bank) * q(Z, 0.3 * twist));
    d.set("DEF-spine.001", q(X, 0.25 * lean) * q(Z, -0.25 * twist));
    d.set("DEF-spine.002", q(X, 0.25 * lean) * q(Y, -0.4 * bank) * q(Z, -0.4 * twist));
    d.set("DEF-spine.003", q(X, 0.20 * lean) * q(Z, -0.35 * twist));
    let back = 0.1 * twist;
    for (bone, share) in [("DEF-spine.004", 0.4), ("DEF-spine.005", 0.35), ("DEF-spine.006", 0.25)] {
        d.set(bone, q(X, (-0.8 * lean + chin) * share) * q(Z, (look - back) * share));
    }
}

/// The legs and arms of a running stride at `phase`, `amp` its size (0
/// stands), knees bent a further `knee`.
pub fn stride(d: &mut BDeltas, phase: f32, amp: f32, knee: f32) {
    let s = phase.sin();
    for side in [Side::L, Side::R] {
        let sign = side.sign();
        let swing = s * sign;
        let recovery = (phase * sign + 0.9 * sign).sin().max(0.0);
        leg(
            d,
            side,
            (0.65 * swing + 0.15) * amp + 0.6 * knee,
            (0.25 + 1.25 * recovery) * amp + 0.05 + knee,
            (-0.25 * (-swing).max(0.0) + 0.15 * swing.max(0.0)) * amp,
            0.0,
        );
        arm(d, side, 1.25, 0.47 * swing * amp, 0.25 + (0.75 + 0.2 * (-swing).max(0.0)) * amp, 0.55, 0.0);
    }
}
