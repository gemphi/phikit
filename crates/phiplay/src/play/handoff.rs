//! The blend from one move into the next, started from the pose last drawn.
//!
//! Apple's `AnimationHandoffType.snapshotAndReplace` (RF:13799-13812) starts
//! the new animation from the bound fields as they were last written, and
//! replaces the old one. The first version here kept the move that was
//! interrupted and its moment, and `record` then set that moment's `t` to
//! 1: the blend began from the END of the old move, not from where it was
//! cut, so the pose jumped at every change. One slot also held only one old
//! move, so a third move inside a blend threw the first blend away.
//!
//! Now every frame written leaves its pose behind ([`BSnap`], a player's
//! aspect). `act` copies it into a [`BBlend`] when a move begins; `record`
//! fades it out over [`BLEND`] frames. A move started inside a blend
//! snapshots the blended pose, so nothing runs out of slots, and two moves
//! started in one frame (standing, then celebrating) never show the first.
use super::world::BLEND;
use crate::pose::{BDeltas, step_deltas};
use phiecs::prelude::*;
/// A pose as written: the bones' rotations and the height off the floor.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BSnap {
    pub deltas: BDeltas,
    pub air: f32,
}
/// The last pose written for a player: the blend source for her next move.
impl Aspect for BSnap {}
/// The pose a move is giving way from, and how much of it remains (1 -> 0).
#[derive(Clone, Debug, PartialEq)]
pub struct BFrom {
    pub snap: BSnap,
    pub left: f32,
}
/// A blend in progress on a player: the snapshot taken when her move began.
#[derive(Clone, Debug, PartialEq)]
pub struct BBlend {
    pub snap: BSnap,
    /// The frame the move began.
    pub start: usize,
}
impl Aspect for BBlend {}
/// A move begins at `frame`: the pose last written becomes its blend source.
/// Nothing written yet means nothing to blend from.
pub fn begin(w: &mut World, player: Word, frame: usize) {
    match w.read::<BSnap>(player).cloned() {
        Some(snap) => {
            w.bind(player, BBlend { snap, start: frame });
        }
        None => {
            w.unbind::<BBlend>(player);
        }
    }
}
/// How much of the old pose remains `frame` frames into a move that began at
/// `start`: 1 -> 0 over [`BLEND`] frames, then none.
pub fn left(start: usize, frame: usize) -> f32 {
    let since = frame.saturating_sub(start);
    if since < BLEND { 1.0 - (since + 1) as f32 / (BLEND + 1) as f32 } else { 0.0 }
}
/// The blend source for the frame being written, while one remains.
pub fn from_at(w: &World, player: Word, frame: usize) -> Option<BFrom> {
    let blend = w.read::<BBlend>(player)?;
    let left = left(blend.start, frame);
    (left > 0.0).then(|| BFrom { snap: blend.snap.clone(), left })
}
/// The pose a step writes, blend included: next move's snapshot.
pub fn emitted(step: &super::BStep) -> BSnap {
    let (deltas, air) = step_deltas(step);
    BSnap { deltas, air }
}
#[cfg(test)]
mod tests;
