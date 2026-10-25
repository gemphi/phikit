//! Poses for the players, ported from the demo's choreography
//! (`phiber/assets/source/make_demo.py`): the same pieces (`pieces`: an
//! arm, a leg, the trunk, a running stride), the same moves (`moves`), the
//! same solve (`solve`: rotations composed down the skeleton, the body set
//! on the floor by its lowest foot point) — so a basketball game played
//! here poses its players exactly as the demo's keyed one did — and new
//! moves for the sports the demo did not have (`sport`).
//!
//! A move is a function of a moment (`BMoment`: how far through the move,
//! the stride's phase, the pace, where the ball is) giving rotations for
//! the bones it moves and a height off the floor; the solve turns those
//! into each bone's channels, which Phiber's rig takes as a play
//! (`phiphx` `BPlay`).

pub mod moves;
pub mod pieces;
pub mod solve;
pub mod sport;

use glam::{Mat4, Quat};

/// A bone of the rest skeleton, parents before children.
#[derive(Clone, Debug, PartialEq)]
pub struct BSkelBone {
    pub name: String,
    pub parent: Option<usize>,
    /// Its rest matrix in the armature (Blender's `matrix_local`).
    pub rest: Mat4,
    pub length: f32,
}

/// The skeleton the poses are solved on.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BSkeleton {
    pub bones: Vec<BSkelBone>,
}

impl BSkeleton {
    pub fn index(&self, name: &str) -> Option<usize> {
        self.bones.iter().position(|b| b.name == name)
    }
}

/// One frame of one player, as the moves read it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BMoment {
    /// How far through the move, 0–1.
    pub t: f32,
    /// The pace, 0 (standing) to 1 (sprinting).
    pub amp: f32,
    /// The stride's phase, radians.
    pub phase: f32,
    /// The dribble's bounce, 0 (ball on the floor) to 1 (in the hand).
    pub bounce: f32,
    /// Leaning into a turn, radians.
    pub bank: f32,
    /// The head turned toward the ball, radians.
    pub look: f32,
}

/// The rotations a move gives the bones it moves, by name, applied in the
/// armature's frame down the chain (`solve`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BDeltas(pub Vec<(&'static str, Quat)>);

impl BDeltas {
    pub fn set(&mut self, bone: &'static str, q: Quat) {
        match self.0.iter_mut().find(|(b, _)| *b == bone) {
            Some(slot) => slot.1 = q,
            None => self.0.push((bone, q)),
        }
    }

    pub fn get(&self, bone: &str) -> Option<Quat> {
        self.0.iter().find(|(b, _)| *b == bone).map(|(_, q)| *q)
    }

    /// `self` turned toward `other` by `w`, bone by bone (a bone only one
    /// side moves turns from or to rest).
    pub fn blend(&self, other: &BDeltas, w: f32) -> BDeltas {
        let mut out = BDeltas::default();
        for (bone, _) in self.0.iter().chain(&other.0) {
            if out.get(bone).is_none() {
                let a = self.get(bone).unwrap_or(Quat::IDENTITY);
                let b = other.get(bone).unwrap_or(Quat::IDENTITY);
                out.set(bone, a.slerp(b, w));
            }
        }
        out
    }
}

/// `x` eased: 0 at 0, 1 at 1, flat at both ends.
pub fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// 0 before `a`, 1 after `b`, eased between.
pub fn ramp(t: f32, a: f32, b: f32) -> f32 {
    if b > a { smooth((t - a) / (b - a)) } else { f32::from(u8::from(t >= b)) }
}

pub fn mix(a: f32, b: f32, w: f32) -> f32 {
    a + (b - a) * w
}

/// A frame of a player's track as rotations and a height: her move at its
/// moment, still giving way to the move before for the blend's first
/// frames.
pub fn step_deltas(step: &crate::play::BStep) -> (BDeltas, f32) {
    let (now, air) = sport::pose(step.mv, &step.moment);
    match step.from {
        Some((mv, moment, left)) if left > 0.0 => {
            let (before, was) = sport::pose(mv, &moment);
            (now.blend(&before, left), mix(air, was, left))
        }
        _ => (now, air),
    }
}

/// Her bones' channels at a frame of her track.
pub fn step_pose(skeleton: &BSkeleton, step: &crate::play::BStep) -> solve::BSolved {
    let (deltas, air) = step_deltas(step);
    solve::solve(skeleton, &deltas, air)
}
