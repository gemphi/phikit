//! The moves the demo's basketball did not need, made from the same
//! pieces: netball's pivot and two-handed shot, volleyball's serve, dig,
//! set and spike, soccer's kick, header and save, and the crowd's seated
//! clap and cheer.

use super::moves::BMove;
use super::pieces::{Side, arm, leg, stride, trunk};
use super::{BDeltas, BMoment, mix, ramp};
use std::f32::consts::PI;

/// A jump's height through a move: rising after `from`, landing by `to`,
/// `top` metres at its peak.
fn hop(t: f32, from: f32, to: f32, top: f32) -> f32 {
    if t > from && t < to { top * (PI * (t - from) / (to - from)).sin().max(0.0) } else { 0.0 }
}

/// Seated: thighs level, shins down, as on a seat's pan.
fn seated(d: &mut BDeltas) {
    for side in [Side::L, Side::R] {
        leg(d, side, 1.5, 1.45, -0.1, 0.12);
    }
}

pub fn pose(m: BMove, at: &BMoment) -> (BDeltas, f32) {
    let mut d = BDeltas::default();
    let air = match m {
        // Netball: landed, the landing foot planted, the ball held at the
        // chest while she turns to find a pass — three seconds at most.
        BMove::Pivot => {
            trunk(&mut d, 0.12, 0.4 * (at.t * PI * 2.0).sin(), 0.0, at.look, 0.0);
            leg(&mut d, Side::L, 0.15, 0.25, 0.0, 0.05);
            leg(&mut d, Side::R, -0.1 + 0.2 * (at.t * PI * 2.0).sin(), 0.35, 0.1, 0.12);
            for side in [Side::L, Side::R] {
                arm(&mut d, side, 1.4, -0.85, 1.9, 0.35, 0.0);
            }
            0.0
        }
        // Netball's shot: the ball lifted above the head in both hands,
        // knees bent, then pushed up and out with a flick on the release.
        BMove::Shoot => {
            let dip = ramp(at.t, 0.15, 0.45) * (1.0 - ramp(at.t, 0.5, 0.65));
            let lift = ramp(at.t, 0.0, 0.35);
            let push = ramp(at.t, 0.5, 0.62);
            trunk(&mut d, 0.05, 0.0, 0.0, at.look, -0.25);
            for side in [Side::L, Side::R] {
                leg(&mut d, side, 0.4 * dip + 0.05, 0.8 * dip + 0.05, 0.0, 0.05);
                arm(&mut d, side, mix(1.4, 1.15, lift), mix(-0.9, -2.7, lift) - 0.2 * push, mix(1.8, 1.9, lift) - 1.6 * push, 0.2, mix(-0.5, 0.8, push));
            }
            0.0
        }
        // An overhand serve: the left hand tosses, the right arm cocks back
        // over the shoulder and swings through at the top (0.55), a little
        // hop with it.
        BMove::Serve => {
            let toss = ramp(at.t, 0.05, 0.3) * (1.0 - ramp(at.t, 0.45, 0.7));
            let cock = ramp(at.t, 0.2, 0.45);
            let hit = ramp(at.t, 0.5, 0.62);
            trunk(&mut d, 0.1 + 0.2 * hit, -0.4 * cock + 0.6 * hit, 0.0, at.look, -0.2);
            stride(&mut d, at.phase, 0.3 * at.amp, 0.1);
            arm(&mut d, Side::L, mix(1.3, 1.05, toss), mix(-0.6, -2.6, toss), 0.15, 0.1, 0.0);
            arm(&mut d, Side::R, mix(1.3, 0.9, cock), mix(-0.5, -2.9, cock) + 1.4 * hit, mix(0.5, 2.0, cock) - 1.8 * hit, 0.05, 0.0);
            hop(at.t, 0.45, 0.8, 0.12)
        }
        // A dig: low, knees bent, the arms straight and joined below the
        // waist, the forearms a platform the ball rebounds from.
        BMove::Dig => {
            let low = ramp(at.t, 0.0, 0.3) * (1.0 - ramp(at.t, 0.7, 1.0));
            let lift = ramp(at.t, 0.4, 0.55) * (1.0 - ramp(at.t, 0.75, 1.0));
            trunk(&mut d, 0.45 * low + 0.15, 0.0, 0.0, at.look, -0.2);
            for side in [Side::L, Side::R] {
                leg(&mut d, side, 0.9 * low + 0.1, 1.4 * low + 0.1, 0.2 * low, 0.3);
                arm(&mut d, side, 1.6, mix(-0.4, -1.0, lift), 0.05, 0.2, 0.0);
            }
            0.0
        }
        // A set: hands above the forehead making a window, the knees giving
        // and then driving up as the ball is pushed out (0.5).
        BMove::Set => {
            let give = ramp(at.t, 0.1, 0.4) * (1.0 - ramp(at.t, 0.45, 0.6));
            let push = ramp(at.t, 0.45, 0.6);
            trunk(&mut d, 0.0, 0.0, 0.0, at.look, -0.4);
            for side in [Side::L, Side::R] {
                leg(&mut d, side, 0.5 * give + 0.05, 0.9 * give + 0.05, 0.0, 0.08);
                arm(&mut d, side, 1.2, mix(-2.4, -2.9, push), mix(1.7, 0.4, push), 0.25, mix(-0.9, 0.3, push));
            }
            0.0
        }
        // A spike: the approach (the stride), both arms swung back, the jump,
        // the hitting arm drawn back like a bow and snapped over the ball at
        // the top (0.6).
        BMove::Spike => {
            let load = ramp(at.t, 0.25, 0.4) * (1.0 - ramp(at.t, 0.42, 0.5));
            let jump = ramp(at.t, 0.42, 0.5);
            let draw = ramp(at.t, 0.45, 0.56);
            let hit = ramp(at.t, 0.58, 0.66);
            trunk(&mut d, 0.3 * load - 0.15 * draw + 0.45 * hit, -0.5 * draw + 0.7 * hit, 0.0, at.look, -0.3 * draw);
            stride(&mut d, at.phase, (1.0 - jump) * at.amp, 0.5 * load);
            arm(&mut d, Side::L, 1.1, mix(0.6 * load, -2.6, jump) + 1.2 * hit, 0.3, 0.1, 0.0);
            arm(&mut d, Side::R, mix(1.2, 0.85, draw), mix(0.7 * load, -2.9, jump) + 1.9 * hit, mix(0.4, 2.2, draw) - 2.0 * hit, 0.05, 0.6 * hit);
            hop(at.t, 0.48, 0.9, 0.6)
        }
        // A kick: the left foot planted beside the ball, the right leg drawn
        // back and swung through it (0.45), the arms out for balance.
        BMove::Kick => {
            let back = ramp(at.t, 0.1, 0.35) * (1.0 - ramp(at.t, 0.35, 0.45));
            let through = ramp(at.t, 0.38, 0.55) * (1.0 - ramp(at.t, 0.75, 1.0));
            trunk(&mut d, 0.15 - 0.15 * through, 0.3 * through, -0.2 * through, at.look, -0.1);
            leg(&mut d, Side::L, 0.15, 0.3, 0.0, 0.05);
            leg(&mut d, Side::R, -0.9 * back + 1.3 * through, 1.6 * back + 0.2 * (1.0 - through), 0.5 * through, 0.0);
            arm(&mut d, Side::L, 0.7, -0.6 * through, 0.6, 0.3, 0.0);
            arm(&mut d, Side::R, 0.8, 0.5 * through, 0.6, 0.3, 0.0);
            0.0
        }
        // A header: a jump, the neck and trunk drawn back and snapped
        // forward through the ball (0.55).
        BMove::Header => {
            let back = ramp(at.t, 0.2, 0.45) * (1.0 - ramp(at.t, 0.5, 0.6));
            let snap = ramp(at.t, 0.5, 0.6) * (1.0 - ramp(at.t, 0.8, 1.0));
            trunk(&mut d, -0.3 * back + 0.35 * snap, 0.0, 0.0, at.look, -0.5 * back + 0.5 * snap);
            for side in [Side::L, Side::R] {
                leg(&mut d, side, 0.3, 0.6 * (1.0 - snap), -0.3, 0.05);
                arm(&mut d, side, 0.9, -1.0, 1.2, 0.4, 0.0);
            }
            hop(at.t, 0.3, 0.85, 0.35)
        }
        // A save: crouched, then springing across with both hands reaching
        // for the ball (taken at 0.5).
        BMove::Save => {
            let crouch = ramp(at.t, 0.0, 0.2) * (1.0 - ramp(at.t, 0.25, 0.35));
            let reach = ramp(at.t, 0.25, 0.45);
            trunk(&mut d, 0.4 * crouch + 0.2, 0.0, 0.5 * reach, at.look, 0.0);
            for side in [Side::L, Side::R] {
                leg(&mut d, side, 0.8 * crouch + 0.1, 1.3 * crouch + 0.1, 0.0, 0.3 + 0.2 * reach);
                arm(&mut d, side, mix(1.2, 0.3, reach), mix(-0.8, -2.4, reach), mix(1.0, 0.2, reach), 0.2, 0.0);
            }
            hop(at.t, 0.25, 0.75, 0.45)
        }
        // The crowd, seated: clapping (hands meeting in front on the beat)
        // and cheering (fists up and pumping).
        BMove::Clap => {
            seated(&mut d);
            trunk(&mut d, 0.1, 0.0, 0.0, at.look, 0.0);
            let beat = (at.t * PI * 8.0).sin().abs();
            for side in [Side::L, Side::R] {
                arm(&mut d, side, 1.1 + 0.25 * beat, -1.2, 1.4, 0.1, 0.0);
            }
            0.0
        }
        BMove::Cheer => {
            seated(&mut d);
            trunk(&mut d, -0.1, 0.0, 0.0, at.look, -0.3);
            let pump = (at.t * PI * 4.0).sin().abs();
            for side in [Side::L, Side::R] {
                arm(&mut d, side, -0.5, -0.2 * pump, 0.4 + 0.5 * pump, 0.95, 0.0);
            }
            0.0
        }
        // Every other move is the court's (`moves.rs`).
        other => return super::moves::pose(other, at),
    };
    (d, air)
}
