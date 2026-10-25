//! The court moves, as make_demo wrote them — each a pose at a moment
//! and a height off the floor. `RELEASE` and `TAKE` say when through a
//! move the ball leaves the hands and when it arrives in them.

use super::pieces::{Side, arm, leg, stride, trunk};
use super::{BDeltas, BMoment, mix, ramp};
use std::f32::consts::PI;

/// A move.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BMove {
    Idle,
    Run,
    Dribble,
    Pass,
    Catch,
    JumpShot,
    Layup,
    Defend,
    Contest,
    PickUp,
    Celebrate,
    // The other sports' (`sport.rs`).
    Pivot,
    Shoot,
    Serve,
    Dig,
    Set,
    Spike,
    Block,
    Kick,
    Header,
    Save,
    // The crowd's, seated.
    Clap,
    Cheer,
}

/// When through a move the ball leaves the hands (or foot).
pub fn release(m: BMove) -> Option<f32> {
    match m {
        BMove::Pass => Some(0.4),
        BMove::JumpShot => Some(0.55),
        BMove::Layup => Some(0.68),
        BMove::Shoot => Some(0.6),
        BMove::Serve => Some(0.55),
        BMove::Dig => Some(0.5),
        BMove::Set => Some(0.5),
        BMove::Spike => Some(0.6),
        BMove::Kick => Some(0.45),
        BMove::Header => Some(0.55),
        _ => None,
    }
}

/// When through a move the ball arrives in the hands.
pub fn take(m: BMove) -> Option<f32> {
    match m {
        BMove::Catch => Some(0.36),
        BMove::PickUp => Some(0.45),
        BMove::Save => Some(0.5),
        _ => None,
    }
}

/// The pose of `m` at `at`, and the height off the floor.
pub fn pose(m: BMove, at: &BMoment) -> (BDeltas, f32) {
    let mut d = BDeltas::default();
    let air = match m {
        BMove::Idle => {
            trunk(&mut d, 0.05, 0.0, 0.0, at.look, 0.0);
            stride(&mut d, at.phase, 0.0, 0.05);
            0.0
        }
        BMove::Run => {
            trunk(&mut d, 0.14 * at.amp + 0.02, 0.30 * at.phase.sin() * at.amp, at.bank, at.look, 0.0);
            stride(&mut d, at.phase, at.amp, 0.0);
            0.0
        }
        BMove::Dribble => {
            trunk(&mut d, 0.30 + 0.10 * at.amp, 0.15 * at.phase.sin() * at.amp, at.bank, at.look, 0.0);
            stride(&mut d, at.phase, 0.8 * at.amp, 0.30);
            arm(&mut d, Side::R, 1.05, -0.45, 0.25 + 0.75 * at.bounce, 0.15, -0.3 + 0.5 * at.bounce);
            arm(&mut d, Side::L, 1.0, -0.75, 1.3, 0.35, 0.0);
            0.0
        }
        BMove::Pass => {
            let out = ramp(at.t, 0.25, 0.42);
            trunk(&mut d, 0.15 + 0.25 * out, 0.0, 0.0, at.look, 0.0);
            stride(&mut d, at.phase, 0.5 * at.amp, 0.15);
            for side in [Side::L, Side::R] {
                arm(&mut d, side, mix(1.40, 1.25, out), mix(-0.95, -1.45, out), mix(2.0, 0.2, out), mix(0.3, 0.1, out), mix(0.0, 0.5, out));
            }
            0.0
        }
        BMove::Catch => {
            let pull = ramp(at.t, 0.3, 0.6);
            trunk(&mut d, 0.15, 0.0, 0.0, at.look, 0.0);
            stride(&mut d, at.phase, 0.5 * at.amp, 0.2);
            for side in [Side::L, Side::R] {
                arm(&mut d, side, mix(1.35, 1.42, pull), mix(-1.40, -0.75, pull), mix(0.35, 1.65, pull), mix(0.1, 0.35, pull), 0.0);
            }
            0.0
        }
        BMove::JumpShot => {
            let dip = ramp(at.t, 0.0, 0.25) * (1.0 - ramp(at.t, 0.25, 0.38));
            let land = ramp(at.t, 0.78, 0.86) * (1.0 - ramp(at.t, 0.88, 1.0));
            let bend = dip.max(0.8 * land);
            let air = if at.t > 0.36 && at.t < 0.78 { 0.42 * (PI * (at.t - 0.36) / 0.42).sin().max(0.0) } else { 0.0 };
            trunk(&mut d, 0.15 + 0.35 * bend, 0.0, 0.0, at.look, -0.1);
            for side in [Side::L, Side::R] {
                leg(&mut d, side, 0.65 * bend + 0.05, 1.2 * bend + 0.08, if air > 0.0 { -0.35 } else { 0.0 }, 0.0);
            }
            let rise = ramp(at.t, 0.1, 0.4);
            let shoot = ramp(at.t, 0.5, 0.58);
            arm(&mut d, Side::R, mix(1.35, 1.10, rise), mix(-0.9, -2.0, rise) - 0.65 * shoot, mix(2.0, 2.3, rise) - 2.1 * shoot, 0.2, mix(-0.6, 0.9, shoot));
            arm(&mut d, Side::L, mix(1.35, 1.0, rise), mix(-0.9, -1.9, rise), mix(2.0, 1.5, rise), 0.15, 0.0);
            air
        }
        BMove::Layup => {
            let jump = ramp(at.t, 0.48, 0.56);
            let land = ramp(at.t, 0.88, 1.0);
            let air = if at.t > 0.52 && at.t < 0.9 { 0.55 * (PI * (at.t - 0.52) / 0.38).sin().max(0.0) } else { 0.0 };
            trunk(&mut d, 0.25 - 0.15 * jump, 0.0, at.bank, at.look, -0.15 * jump);
            stride(&mut d, at.phase, (1.0 - jump) * at.amp, 0.15);
            if jump > 0.0 {
                let mut up = BDeltas::default();
                leg(&mut up, Side::R, 1.45, 1.6, 0.2, 0.0);
                leg(&mut up, Side::L, 0.05, 0.1, -0.5, 0.0);
                for (bone, rot) in &up.0 {
                    let from = d.get(bone).unwrap_or_default();
                    d.set(bone, from.slerp(*rot, jump * (1.0 - land)));
                }
            }
            let carry = ramp(at.t, 0.45, 0.68);
            arm(&mut d, Side::R, mix(1.40, 1.10, carry), mix(-0.95, -2.85, carry), mix(2.0, 0.15, carry), 0.15, 0.5 * ramp(at.t, 0.66, 0.72));
            arm(&mut d, Side::L, mix(1.40, 1.15, carry), mix(-0.95, -1.6, carry), mix(2.0, 0.8, carry), 0.2, 0.0);
            air
        }
        BMove::Defend => {
            trunk(&mut d, 0.55, 0.0, 0.0, at.look, -0.15);
            let s = at.phase.sin();
            for (side, sign) in [(Side::L, 1.0f32), (Side::R, -1.0)] {
                leg(&mut d, side, 0.70 + 0.18 * s * sign * at.amp, 1.05 + 0.25 * (s * sign).max(0.0) * at.amp, 0.15, 0.28);
            }
            arm(&mut d, Side::L, 0.95, -2.2, 0.3, 0.1, 0.0);
            arm(&mut d, Side::R, 0.55, -0.35, 0.55, 0.1, 0.0);
            0.0
        }
        BMove::Contest | BMove::Block => {
            let dip = ramp(at.t, 0.0, 0.28) * (1.0 - ramp(at.t, 0.28, 0.38));
            let land = ramp(at.t, 0.8, 0.88) * (1.0 - ramp(at.t, 0.9, 1.0));
            let bend = dip.max(0.8 * land);
            let air = if at.t > 0.38 && at.t < 0.8 { 0.38 * (PI * (at.t - 0.38) / 0.42).sin().max(0.0) } else { 0.0 };
            trunk(&mut d, 0.2 + 0.3 * bend, 0.0, 0.0, at.look, -0.2);
            let up = ramp(at.t, 0.15, 0.4) * (1.0 - ramp(at.t, 0.85, 1.0));
            for side in [Side::L, Side::R] {
                leg(&mut d, side, 0.6 * bend + 0.05, 1.1 * bend + 0.08, if air > 0.0 { -0.35 } else { 0.0 }, 0.0);
                arm(&mut d, side, mix(1.0, 1.12, up), mix(-0.6, -2.85, up), mix(0.8, 0.1, up), 0.05, 0.0);
            }
            air
        }
        BMove::PickUp => {
            let down = ramp(at.t, 0.0, 0.4) * (1.0 - ramp(at.t, 0.5, 0.85));
            trunk(&mut d, 0.2 + 0.9 * down, 0.0, 0.0, at.look * (1.0 - down), 0.3 * down);
            stride(&mut d, at.phase, 0.4 * at.amp * (1.0 - down), 0.75 * down);
            let hold = ramp(at.t, 0.45, 0.8);
            for side in [Side::L, Side::R] {
                arm(&mut d, side, 1.32, mix(-0.55, -0.95, hold), mix(0.25, 2.0, hold), mix(0.1, 0.35, hold), 0.0);
            }
            0.0
        }
        BMove::Celebrate => {
            let hop = (2.0 * PI * (at.t * 1.6).min(1.0)).sin().abs();
            trunk(&mut d, -0.05, 0.0, 0.0, at.look, -0.25);
            stride(&mut d, at.phase, 0.3 * at.amp, 0.25 * (1.0 - hop));
            let up = ramp(at.t, 0.0, 0.2);
            for side in [Side::L, Side::R] {
                arm(&mut d, side, mix(1.25, -0.75, up), 0.0, mix(0.3, 0.6 + 0.4 * hop, up), 0.95, 0.0);
            }
            0.14 * hop * (1.0 - ramp(at.t, 0.6, 0.75))
        }
        other => return super::sport::pose(other, at),
    };
    (d, air)
}
