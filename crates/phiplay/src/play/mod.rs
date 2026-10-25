//! The match, played: a phiecs world (`world`) stepped at the frame rate,
//! its systems by stage — the ball's flight (Ingest), who is open and who
//! guards whom (Pre-couple), what the ball's holder and everyone else does
//! next (Couple), where they run (Post-couple), and the frame written down
//! (Readout) — into a tape (`BTape`): every player's place, heading and
//! move, frame by frame, the ball's path and the score.
//!
//! The tape is what Phiber plays: a rig asks it for a frame and poses the
//! player from her move there (`pose`), so a frame can be drawn in any
//! order and scrubbed backwards. The game logic lives in two kinds:
//! invasion games where a side carries the ball into the other's end
//! (`invasion`: basketball, netball, soccer) and rally games over a net
//! (`rally`: volleyball, beach volleyball).

pub mod ball;
pub mod invasion;
pub mod rally;
pub mod state;
pub mod world;

use crate::choice::BChoice;
use crate::pose::BMoment;
use crate::pose::moves::BMove;
use glam::{Vec2, Vec3};

/// The frame rate the match is played and drawn at.
pub const FPS: f32 = 24.0;

/// A player at one frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BStep {
    pub at: Vec2,
    pub heading: f32,
    pub mv: BMove,
    pub moment: BMoment,
    /// The move before, still giving way: (move, its moment, how much of
    /// it remains, 1 → 0).
    pub from: Option<(BMove, BMoment, f32)>,
}

/// One player's track through the match.
#[derive(Clone, Debug, PartialEq)]
pub struct BTrack {
    /// 0 home, 1 away.
    pub team: usize,
    /// Her place in the roster (`look::roster`).
    pub roster: usize,
    /// Her position in the sport's line-up.
    pub slot: usize,
    pub steps: Vec<BStep>,
}

/// Something the match marks: a score, a whistle.
#[derive(Clone, Debug, PartialEq)]
pub struct BEvent {
    pub frame: usize,
    pub team: usize,
    pub what: String,
}

/// The match as played.
#[derive(Clone, Debug, PartialEq)]
pub struct BTape {
    pub players: Vec<BTrack>,
    /// The ball's centre, frame by frame.
    pub ball: Vec<Vec3>,
    /// Whether the ball is in flight from a throw, kick or hit — it glows.
    pub flying: Vec<bool>,
    /// The score after each frame.
    pub score: Vec<[u16; 2]>,
    pub events: Vec<BEvent>,
}

impl BTape {
    pub fn frames(&self) -> usize {
        self.ball.len()
    }
}

/// The match the choice picks, `seconds` of it.
pub fn play(choice: &BChoice, seconds: f32) -> BTape {
    let frames = (seconds * FPS).round() as usize;
    if choice.sport.spec().goal_is_net() { rally::play(choice, frames) } else { invasion::play(choice, frames) }
}

/// A repeatable number in 0–1 for (`a`, `b`): the match's dice, so the same
/// choice plays the same match.
pub fn dice(a: usize, b: usize) -> f32 {
    crate::build::field::hash(a as i32 * 7919 + 13, b as i32 * 104_729 + 7)
}

/// The turn about z that brings −y round onto `v` (make_demo's
/// `heading_to`): where a player faces.
pub fn heading_to(v: Vec2) -> f32 {
    v.x.atan2(-v.y)
}

/// `angle` moved by whole turns to within half a turn of `near`.
pub fn unwrap(angle: f32, near: f32) -> f32 {
    let mut a = angle;
    while a - near > std::f32::consts::PI {
        a -= std::f32::consts::TAU;
    }
    while a - near < -std::f32::consts::PI {
        a += std::f32::consts::TAU;
    }
    a
}

#[cfg(test)]
mod tests;
