//! The ball: held, dribbled, in flight, loose or dead — and where that puts
//! it each frame. A flight is make_demo's: from one point to another in a
//! given time under gravity. When a flight lands it waits as `Arrived` for
//! the sport to say what that meant (a catch, a score, a miss), so the
//! ball system (Ingest) knows physics and nothing of rules.

use super::FPS;
use super::world::{BAct, BBody, game, game_mut};
use crate::pose::moves::BMove;
use crate::pose::ramp;
use glam::{Vec2, Vec3};
use phiecs::prelude::*;

const GRAVITY: f32 = 9.81;
/// How fast a rolling ball loses its speed, per second.
const ROLLING: f32 = 0.8;

/// What a flight was for, read by the sport when it lands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BAim {
    /// To a player's hands (or feet).
    To(Word),
    /// At the goal by `by`: whether it goes in, and for how many.
    Shot { by: Word, team: usize, made: bool, points: u16 },
    /// Into the other side's court (a rally's hit or serve).
    Court { team: usize },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BBallState {
    Dead { at: Vec3 },
    Held { by: Word, at: Vec3 },
    Dribbled { by: Word, at: Vec3 },
    Flight { from: Vec3, to: Vec3, start: usize, end: usize, aim: BAim, at: Vec3 },
    Arrived { aim: BAim, at: Vec3 },
    Loose { vel: Vec3, at: Vec3 },
}

impl BBallState {
    pub fn at(&self) -> Vec3 {
        match *self {
            BBallState::Dead { at }
            | BBallState::Held { at, .. }
            | BBallState::Dribbled { at, .. }
            | BBallState::Flight { at, .. }
            | BBallState::Arrived { at, .. }
            | BBallState::Loose { at, .. } => at,
        }
    }

    /// Thrown, kicked or hit and not yet arrived: the ball glows.
    pub fn flying(&self) -> bool {
        matches!(self, BBallState::Flight { .. })
    }

    /// Who has it in hand or at foot.
    pub fn holder(&self) -> Option<Word> {
        match *self {
            BBallState::Held { by, .. } | BBallState::Dribbled { by, .. } => Some(by),
            _ => None,
        }
    }
}

/// The point at `frame` of a flight from `from` (frame `start`) to `to`
/// (frame `end`) under gravity.
pub fn flight(from: Vec3, to: Vec3, start: usize, end: usize, frame: usize) -> Vec3 {
    let total = (end.saturating_sub(start)).max(1) as f32 / FPS;
    let t = (frame.saturating_sub(start)) as f32 / FPS;
    let v0 = (to - from) / total + Vec3::Z * (0.5 * GRAVITY * total);
    from + v0 * t - Vec3::Z * (0.5 * GRAVITY * t * t)
}

/// Where a holder carries the ball: at her chest, raised through a shot,
/// out in front for a pass.
pub fn in_hands(w: &World, by: Word) -> Vec3 {
    let b = w.read::<BBody>(by).expect("a body");
    let a = w.read::<BAct>(by).expect("acting");
    let t = super::world::progress(w, by);
    let forward = Vec2::new(b.heading.sin(), -b.heading.cos());
    let (lift, reach) = match a.mv {
        BMove::JumpShot | BMove::Shoot => (1.15 * ramp(t, 0.1, 0.5), 0.1),
        BMove::Layup => (1.4 * ramp(t, 0.45, 0.68), 0.25),
        BMove::Pass => (0.0, 0.2 + 0.3 * ramp(t, 0.25, 0.42)),
        BMove::Serve => (1.3 * ramp(t, 0.05, 0.3), 0.15),
        _ => (0.0, 0.0),
    };
    let p = b.at + forward * (0.32 + reach);
    Vec3::new(p.x, p.y, 1.12 + lift)
}

/// Ingest: the ball moved one frame.
pub fn advance(w: &mut World, dt: f32) {
    let frame = game(w).frame;
    let radius = game(w).sport.ball.radius;
    let bounce = game(w).sport.ball.bounce;
    let next = match game(w).ball {
        BBallState::Held { by, .. } => BBallState::Held { by, at: in_hands(w, by) },
        BBallState::Dribbled { by, .. } => {
            let b = *w.read::<BBody>(by).expect("a body");
            let a = *w.read::<BAct>(by).expect("acting");
            let forward = Vec2::new(b.heading.sin(), -b.heading.cos());
            // Her right, facing `forward`: −x when she faces −y.
            let right = Vec2::new(forward.y, -forward.x);
            if game(w).sport.rules.steps.is_none() {
                // At her feet.
                let p = b.at + forward * 0.45;
                BBallState::Dribbled { by, at: Vec3::new(p.x, p.y, radius) }
            } else {
                // Under her right palm at the top of the bounce, on the floor at
                // the bottom, on the dribble's clock.
                let p = b.at + forward * 0.35 + right * 0.25;
                BBallState::Dribbled { by, at: Vec3::new(p.x, p.y, radius + (0.95 - radius) * a.dphase.sin().abs()) }
            }
        }
        BBallState::Flight { from, to, start, end, aim, .. } => {
            if frame >= end {
                BBallState::Arrived { aim, at: to }
            } else {
                BBallState::Flight { from, to, start, end, aim, at: flight(from, to, start, end, frame) }
            }
        }
        BBallState::Loose { mut vel, at } => {
            vel.z -= GRAVITY * dt;
            let mut at = at + vel * dt;
            if at.z < radius {
                at.z = radius;
                vel.z = -vel.z * bounce;
                vel.x *= 0.85;
                vel.y *= 0.85;
                if vel.z < 0.3 {
                    vel.z = 0.0;
                }
            }
            // Rolling, it slows.
            if at.z <= radius && vel.z == 0.0 {
                vel *= (-ROLLING * dt).exp();
            }
            BBallState::Loose { vel, at }
        }
        other => other,
    };
    game_mut(w).ball = next;
}

/// Throws the ball now from where it is to `to`, arriving in `frames`.
pub fn throw(w: &mut World, to: Vec3, frames: usize, aim: BAim) {
    let g = game_mut(w);
    let from = g.ball.at();
    let start = g.frame;
    g.ball = BBallState::Flight { from, to, start, end: start + frames.max(2), aim, at: from };
}
