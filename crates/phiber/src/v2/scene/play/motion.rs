//! The match played back: each thing that moves reads the tape at the
//! frame the timeline is on (`BPlay`) — a player's rig its pose and where
//! she stands, the ball where it is, its glow whether it is in flight, a
//! scoreboard digit whether its slot shows it.
//!
//! The scene's frames count from 1; the tape's from 0.

use crate::phiphx::blend::rig::{BRig, BRigState};
use crate::phiphx::blend::{BPlay, ObjectTransformRaw};
use glam::{Quat, Vec3};
use phiplay::build::board::BSlot;
use phiplay::play::BTape;
use phiplay::pose::BSkeleton;
use std::sync::Arc;

/// The tape's frame for the scene's `frame`, held at the ends.
fn at(tape: &BTape, frame: f32) -> usize {
    ((frame.round() as i64 - 1).max(0) as usize).min(tape.frames().saturating_sub(1))
}

fn range(tape: &BTape) -> (f32, f32) {
    (1.0, tape.frames() as f32)
}

/// A player: her track's pose on her rig, her place and heading on the
/// floor, her height.
#[derive(Debug)]
pub struct BPlayer {
    pub tape: Arc<BTape>,
    pub track: usize,
    pub skeleton: Arc<BSkeleton>,
    pub height: f32,
}

impl BPlay for BPlayer {
    fn rig(&self, rig: &BRig, frame: f32) -> Option<BRigState> {
        let step = self.tape.players[self.track].steps.get(at(&self.tape, frame))?;
        let solved = phiplay::pose::step_pose(&self.skeleton, step);
        let mut state = rig.rest_state();
        // The skeleton is the rig's bones in the rig's order.
        for (raw, channel) in state.raw.iter_mut().zip(&solved.channels) {
            let q = channel.rotation;
            raw.quat = [q.w, q.x, q.y, q.z];
            raw.rotmode = 0;
            raw.loc = channel.location.into();
        }
        Some(state)
    }

    fn object(&self, frame: f32) -> Option<ObjectTransformRaw> {
        let step = self.tape.players[self.track].steps.get(at(&self.tape, frame))?;
        let q = Quat::from_rotation_z(step.heading);
        Some(ObjectTransformRaw::from_parts([step.at.x, step.at.y, 0.0], [q.w, q.x, q.y, q.z], [self.height; 3]))
    }

    fn range(&self) -> (f32, f32) {
        range(&self.tape)
    }
}

/// The ball, and its glow: the same path, the glow a shell just outside the
/// ball while it flies and tucked inside it otherwise (make_demo's
/// `Basketball Glow`).
#[derive(Debug)]
pub struct BBall {
    pub tape: Arc<BTape>,
    /// `None` for the ball; the glow's scale in flight and at rest.
    pub glow: Option<(f32, f32)>,
}

impl BPlay for BBall {
    fn object(&self, frame: f32) -> Option<ObjectTransformRaw> {
        let f = at(&self.tape, frame);
        let p = *self.tape.ball.get(f)?;
        // A turn about the way it travels, a full one every metre or so.
        let next = self.tape.ball.get(f + 1).copied().unwrap_or(p);
        let travelled: f32 = self.tape.ball.windows(2).take(f).map(|w| w[0].distance(w[1])).sum();
        let way = (next - p).truncate();
        let axis = if way.length() > 1e-4 { Vec3::new(-way.y, way.x, 0.0).normalize() } else { Vec3::X };
        let q = Quat::from_axis_angle(axis, travelled * 3.0);
        let size = match self.glow {
            None => 1.0,
            Some((flying, resting)) => if self.tape.flying.get(f).copied().unwrap_or(false) { flying } else { resting },
        };
        Some(ObjectTransformRaw::from_parts(p.into(), [q.w, q.x, q.y, q.z], [size; 3]))
    }

    fn range(&self) -> (f32, f32) {
        range(&self.tape)
    }
}

/// A façade panel's light at one of its steps: shown while the venue's
/// show (`BLightShow::level`, a pure function of where the panel is and
/// the beat) lights the panel within `band`, and hidden otherwise — the
/// panel drawn at three strengths, one showing at a time, so the show runs
/// on the timeline with no material changing.
#[derive(Debug)]
pub struct BFacade {
    pub frames: usize,
    pub show: phiplay::venue::BLightShow,
    /// Beats a minute.
    pub tempo: f32,
    /// Where the panel is: round the rim and up the band, 0–1.
    pub at: (f32, f32),
    pub band: (f32, f32),
}

impl BPlay for BFacade {
    fn object(&self, frame: f32) -> Option<ObjectTransformRaw> {
        let beat = (frame - 1.0).max(0.0) / phiplay::play::FPS * self.tempo / 60.0;
        let level = self.show.level(self.at.0, self.at.1, beat);
        Some(hidden_unless((self.band.0..self.band.1).contains(&level)))
    }

    fn range(&self) -> (f32, f32) {
        (1.0, self.frames as f32)
    }
}

/// Where it was built when `shown`, else out of sight below the floor and
/// shrunk to nothing.
fn hidden_unless(shown: bool) -> ObjectTransformRaw {
    let (loc, size) = if shown { ([0.0; 3], 1.0) } else { ([0.0, 0.0, -100.0], 1e-3) };
    ObjectTransformRaw::from_parts(loc, [1.0, 0.0, 0.0, 0.0], [size; 3])
}

/// A scoreboard digit: where it was built while its slot shows it, and out
/// of sight below the floor, shrunk to nothing, while it does not.
#[derive(Debug)]
pub struct BDigit {
    pub tape: Arc<BTape>,
    pub slot: BSlot,
    pub value: u8,
}

impl BPlay for BDigit {
    fn object(&self, frame: f32) -> Option<ObjectTransformRaw> {
        let f = at(&self.tape, frame);
        let score = self.tape.score.get(f).copied().unwrap_or([0, 0]);
        Some(hidden_unless(self.slot.shows(score, f) == Some(self.value)))
    }

    fn range(&self) -> (f32, f32) {
        range(&self.tape)
    }
}
