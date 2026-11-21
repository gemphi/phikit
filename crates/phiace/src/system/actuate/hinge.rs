//! A joint's angle, and the pose it makes.
//!
//! No physics yet (PLAN order 1): [`drive`] is the stand-in Simulate, moving
//! each angle toward its target at a fixed rate. [`hinge`] turns the angle
//! into the link's `Pose`; it is added before `propagate` so both run in
//! PostCouple, in that order.
use crate::comp::joint::{Angle, Hinge, Limit, Target};
use phiecs::prelude::{FnSystem, Pose, Stage, World};

/// Moves every `Angle` toward its `Target` by at most `rate` radians a
/// second, kept inside its `Limit`. At [`Stage::Couple`].
pub fn drive(rate: f32) -> FnSystem<impl FnMut(&mut World, f32) + Send + Sync> {
    FnSystem::new("drive", move |world, dt| {
        let moves: Vec<_> = world.each::<Target>().map(|(w, t)| (w, t.0)).collect();
        for (word, target) in moves {
            let limit = world.read::<Limit>(word).copied().unwrap_or(Limit { lo: f32::MIN, hi: f32::MAX });
            if let Some(angle) = world.read_mut::<Angle>(word) {
                let step = (target - angle.0).clamp(-rate * dt, rate * dt);
                angle.0 = (angle.0 + step).clamp(limit.lo, limit.hi);
            }
        }
    })
    .at(Stage::Couple)
}

/// Writes the rotation of every `Hinge` from its `Angle`, leaving the
/// link's translation as hung. At [`Stage::PostCouple`].
pub fn hinge() -> FnSystem<impl FnMut(&mut World, f32) + Send + Sync> {
    FnSystem::new("hinge", |world, _| {
        let turns: Vec<_> = world.each::<Hinge>().filter_map(|(w, h)| Some((w, Pose::turned(h.axis, world.read::<Angle>(w)?.0)))).collect();
        for (word, r) in turns {
            if let Some(pose) = world.read_mut::<Pose>(word) {
                pose.r = r;
            }
        }
    })
    .at(Stage::PostCouple)
}
