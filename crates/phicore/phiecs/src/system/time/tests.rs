use super::*;
use crate::comp::space::{Pose, WorldMatrix};
use crate::system::sink::MemorySink;
use crate::world::World;
fn mover() -> (World, crate::word::Word) {
    let mut w = World::new();
    let k = w.declare_kinds(&["thing"])[0];
    let a = w.spawn_named(k, "ball");
    w.bind(a, Pose::default());
    w.add_system(record());
    (w, a)
}
fn move_to(w: &mut World, a: crate::word::Word, x: f32) {
    w.read_mut::<Pose>(a).unwrap().t[0] = x;
}
fn go_at(w: &mut World, a: crate::word::Word, ticks: &'static [u64]) {
    use crate::system::{FnSystem, Stage};
    w.add_system(FnSystem::new("go", move |w, _| {
        let t = w.clock().tick;
        if ticks.contains(&t) {
            move_to(w, a, t as f32);
        }
    }).at(Stage::Ingest));
}
#[test]
fn a_step_records_one_row_for_each_aspect_that_changed() {
    let (mut w, a) = mover();
    let sink = MemorySink::new();
    w.attach(sink.clone());
    go_at(&mut w, a, &[1, 3]);
    for _ in 0..4 {
        w.step(1.0);
    }
    let ticks: Vec<_> = sink.rows().iter().filter(|r| r.aspect == "pose").map(|r| r.tick).collect();
    assert_eq!(ticks, [1, 3], "one row a change, none for ticks 2 and 4");
    assert!(sink.events().contains(&(1, "did_add".into(), "thing#1".into())));
}
#[test]
fn a_write_between_steps_belongs_to_the_tick_just_ended() {
    let (mut w, a) = mover();
    move_to(&mut w, a, 1.0);
    w.step(1.0);
    move_to(&mut w, a, 2.0);
    w.step(1.0);
    assert_eq!(w.at::<Pose>(a, 0).unwrap().t[0], 1.0);
    assert_eq!(w.at::<Pose>(a, 1).unwrap().t[0], 2.0);
}
#[test]
fn seek_reads_every_step_back() {
    let (mut w, a) = mover();
    w.add_system(crate::system::space::propagate());
    go_at(&mut w, a, &[1, 2, 3, 4]);
    for _ in 0..4 {
        w.step(1.0);
    }
    assert_eq!(w.read::<Pose>(a).unwrap().t[0], 4.0);
    assert_eq!(w.at::<Pose>(a, 2).unwrap().t[0], 2.0, "at asks the journal, the world stays");
    assert_eq!(w.read::<Pose>(a).unwrap().t[0], 4.0);
    w.seek(2);
    assert_eq!(w.clock().tick, 2);
    assert_eq!(w.read::<Pose>(a).unwrap().t[0], 2.0);
    assert_eq!(w.read::<WorldMatrix>(a).unwrap().0[3][0], 2.0, "the matrix is read back too");
    w.seek(4);
    assert_eq!(w.read::<Pose>(a).unwrap().t[0], 4.0, "forward again");
    w.seek(0);
    assert!(w.read::<Pose>(a).is_none(), "before it had a pose");
}
#[test]
fn stepping_on_from_a_seek_rewrites_the_future() {
    let (mut w, a) = mover();
    for x in [1.0, 2.0, 3.0] {
        move_to(&mut w, a, x);
        w.step(1.0);
    }
    w.seek(1);
    assert_eq!(w.read::<Pose>(a).unwrap().t[0], 2.0);
    move_to(&mut w, a, 9.0);
    w.step(1.0);
    assert_eq!(w.clock().tick, 2);
    assert_eq!(w.at::<Pose>(a, 1).unwrap().t[0], 9.0);
    assert_eq!(w.at::<Pose>(a, 3).unwrap().t[0], 9.0, "the old tick 2 is gone");
    assert!(w.journal_rows().iter().all(|r| r.tick <= 2));
}#[test]
fn dump_and_load_round_trip() {
    let (mut w, a) = mover();
    let k = w.kind("thing").unwrap();
    let hand = w.spawn_named(k, "hand");
    w.set_parent(hand, a, false);
    w.bind(hand, Pose::at([0.0, 1.0, 0.0]));
    w.bind(a, crate::comp::stored::Owner::Remote);
    w.set_enabled(hand, false);
    move_to(&mut w, a, 5.0);
    w.add_system(crate::system::space::propagate());
    w.step(1.0);
    let rows = w.dump();
    let mut other = World::new();
    other.declare_kinds(&["thing"]);
    assert_eq!(other.load(&rows), rows.len());
    let (a2, hand2) = (other.lexicon.get("thing#1").unwrap(), other.lexicon.get("thing#2").unwrap());
    assert_eq!(other.read::<Pose>(a2), w.read::<Pose>(a));
    assert_eq!(other.read::<Pose>(hand2), w.read::<Pose>(hand));
    assert_eq!(other.read::<crate::comp::stored::Owner>(a2), Some(&crate::comp::stored::Owner::Remote));
    assert_eq!(other.tree.parent(hand2), Some(a2));
    assert!(!other.is_enabled(hand2));
    assert_eq!(other.display_name(hand2), "hand");
    assert_eq!(other.kind_of(a2), w.kind_of(a));
    assert_eq!(other.clock().tick, 1);
    assert_eq!(other.stable_id(a2), w.stable_id(a));
    assert_eq!(other.dump(), rows, "the same rows come back");
}
#[test]
fn a_row_for_an_unknown_kind_is_skipped() {
    let (w, _) = mover();
    let rows = w.dump();
    let mut other = World::new(); // no kinds declared
    assert!(other.load(&rows) < rows.len());
    assert!(other.kind_of(other.lexicon.get("thing#1").unwrap()).is_none());
}
