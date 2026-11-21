use super::*;
use crate::choice::BChoice;
use crate::play::world::{act, build, game, record};
use crate::pose::moves::BMove;
use crate::pose::sport;
use crate::pose::{BMoment, step_deltas};
use glam::Quat;
/// The turn between two rotations, exact near zero (acos is not).
fn angle(a: Quat, b: Quat) -> f32 {
    let d = a.inverse() * b;
    2.0 * d.xyz().length().atan2(d.w.abs())
}
/// How far apart two poses are: the sum of the angles between each bone's
/// rotation, and the height difference.
fn apart(a: &BSnap, b: &BSnap) -> f32 {
    let mut bones: Vec<&str> = a.deltas.0.iter().chain(&b.deltas.0).map(|(n, _)| *n).collect();
    bones.sort_unstable();
    bones.dedup();
    let turn: f32 = bones.iter().map(|n| angle(a.deltas.get(n).unwrap_or(Quat::IDENTITY), b.deltas.get(n).unwrap_or(Quat::IDENTITY))).sum();
    turn + (a.air - b.air).abs()
}
fn pose_of(mv: BMove, m: &BMoment) -> BSnap {
    let (deltas, air) = sport::pose(mv, m);
    BSnap { deltas, air }
}
fn at(t: f32) -> BMoment {
    BMoment { t, amp: 0.4, phase: 1.0, ..BMoment::default() }
}
/// The first blended frame of the OLD logic: the interrupted move at its
/// END (`t` forced to 1), faded by `left`, under the new move.
fn old_first_frame(old: BMove, cut: &BMoment, new: &BSnap, left: f32) -> BSnap {
    let source = pose_of(old, &BMoment { t: 1.0, ..*cut });
    BSnap { deltas: new.deltas.blend(&source.deltas, left), air: new.air + (source.air - new.air) * left }
}
#[test]
fn the_old_blend_began_from_the_end_of_the_cut_move_and_the_pop_is_gone() {
    // A pass cut a third of the way through, then a run.
    let cut = at(0.3);
    let drawn = pose_of(BMove::Pass, &cut);
    let run = pose_of(BMove::Run, &at(0.0));
    let first = left(5, 5); // the first frame of the blend
    let old = apart(&old_first_frame(BMove::Pass, &cut, &run, first), &drawn);
    let source = BFrom { snap: drawn.clone(), left: first };
    let new_pose = BSnap { deltas: run.deltas.blend(&source.snap.deltas, first), air: run.air + (drawn.air - run.air) * first };
    let new = apart(&new_pose, &drawn);
    eprintln!("pop at the switch: old {old:.3}, snapshot {new:.3}");
    assert!(old > 2.0 * new, "the old blend jumped {old}, the snapshot {new}");
    assert!(new <= (1.0 - first) * apart(&drawn, &run) + 1e-4, "the snapshot moves only by the new move's share");
}
#[test]
fn a_third_move_inside_a_blend_starts_from_the_blended_pose() {
    // Pass, cut for Run; two frames into that blend, cut for Idle.
    let (pass, run) = (pose_of(BMove::Pass, &at(0.3)), pose_of(BMove::Run, &at(0.0)));
    let l = left(5, 6);
    let drawn = BSnap { deltas: run.deltas.blend(&pass.deltas, l), air: run.air + (pass.air - run.air) * l };
    // Old: one slot, so the third move saw only the Run, at its end.
    let old_source = pose_of(BMove::Run, &at(1.0));
    assert!(apart(&old_source, &drawn) > 0.05, "the old source was not what was drawn");
    // New: the snapshot is the last drawn pose itself.
    assert_eq!(apart(&drawn.clone(), &drawn), 0.0);
}
fn drive(w: &mut phiecs::world::World, frames: usize) {
    for _ in 0..frames {
        record(w, 1.0 / 24.0);
    }
}
fn last_two(w: &phiecs::world::World, p: phiecs::word::Word) -> (BSnap, BSnap, BSnap) {
    let g = game(w);
    let i = g.players.iter().position(|q| *q == p).unwrap();
    let steps = &g.tape.players[i].steps;
    let (a, b) = (&steps[steps.len() - 2], &steps[steps.len() - 1]);
    let now = |s: &crate::play::BStep| {
        let (deltas, air) = sport::pose(s.mv, &s.moment);
        BSnap { deltas, air }
    };
    let (d, air) = step_deltas(b);
    (emitted(a), BSnap { deltas: d, air }, now(b))
}
#[test]
fn in_the_world_a_move_change_does_not_jump_and_a_third_move_does_not_either() {
    let mut w = build(&BChoice::default());
    let p = game(&w).players[0];
    assert!(act(&mut w, p, BMove::Pass));
    drive(&mut w, 4); // the pass is under way: 4 of 13 frames
    for (next, wait) in [(BMove::Run, 2), (BMove::Idle, 0)] {
        assert!(act(&mut w, p, next), "{next:?}");
        drive(&mut w, 1);
        let (before, shown, pure) = last_two(&w, p);
        let jump = apart(&shown, &before);
        let share = (1.0 - left(game(&w).frame - 1, game(&w).frame - 1)) * apart(&pure, &before);
        assert!(jump <= share + 1e-3, "{next:?}: the pose jumped {jump}, the new move's share is {share}");
        drive(&mut w, wait); // still inside the blend: the next move is the third
    }
    assert!(w.read::<BBlend>(p).is_some());
}
#[test]
fn two_moves_in_one_frame_never_show_the_first() {
    let mut w = build(&BChoice::default());
    let p = game(&w).players[0];
    drive(&mut w, 3);
    assert!(act(&mut w, p, BMove::Run));
    drive(&mut w, 8); // the blend is over
    let before = w.read::<BSnap>(p).cloned().unwrap();
    assert!(act(&mut w, p, BMove::Idle));
    assert!(act(&mut w, p, BMove::Celebrate));
    let blend = w.read::<BBlend>(p).unwrap();
    assert_eq!(blend.snap, before, "the blend starts from what was drawn, not from the Idle in between");
}
