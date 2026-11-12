//! A two-joint arm as words: a child link follows its parent, and
//! reparenting with `keep_world` keeps where a link is.
use phiace::comp::joint::{Angle, Target};
use phiace::entity::bot::{Bot, Joint, spawn};
use phiace::system::actuate::{drive, hinge};
use phiecs::prelude::*;

const HALF_TURN: f32 = std::f32::consts::FRAC_PI_2;

fn near(a: [f32; 3], b: [f32; 3]) -> bool {
    a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-3)
}

fn z(lo: f32, hi: f32) -> Joint {
    Joint { axis: [0.0, 0.0, 1.0], lo, hi }
}

/// base, upper at the base, forearm one unit along upper, a tip pin one unit
/// along forearm; the world steps drive, hinge, propagate.
fn arm() -> (World, Vec<Word>, Word) {
    let bot = Bot::new("arm")
        .root("base", Pose::default())
        .link("upper", "base", Pose::default(), z(-3.0, 3.0))
        .link("fore", "upper", Pose::at([1.0, 0.0, 0.0]), z(-3.0, 3.0));
    let mut w = World::new();
    let links = spawn(&mut w, &bot).unwrap();
    let tip = w.pin(links[2], "tip", Pose::at([1.0, 0.0, 0.0]));
    w.add_system(drive(1.0));
    w.add_system(hinge());
    w.add_system(propagate());
    (w, links, tip)
}

fn tip_at(w: &World, tip: Word) -> [f32; 3] {
    w.read::<WorldMatrix>(tip).unwrap().point([0.0; 3])
}

#[test]
fn a_child_link_follows_its_parent() {
    let (mut w, links, tip) = arm();
    w.step(0.1);
    assert!(near(tip_at(&w, tip), [2.0, 0.0, 0.0]), "straight out");
    w.bind(links[1], Target(HALF_TURN));
    for _ in 0..20 {
        w.step(0.1);
    }
    assert!((w.read::<Angle>(links[1]).unwrap().0 - HALF_TURN).abs() < 1e-5);
    assert!(near(tip_at(&w, tip), [0.0, 2.0, 0.0]), "the whole arm turned up: {:?}", tip_at(&w, tip));
    w.bind(links[2], Target(HALF_TURN));
    for _ in 0..20 {
        w.step(0.1);
    }
    assert!(near(tip_at(&w, tip), [-1.0, 1.0, 0.0]), "the forearm folded: {:?}", tip_at(&w, tip));
}

#[test]
fn a_joint_stops_at_its_limit_and_at_its_rate() {
    let (mut w, links, _) = arm();
    w.bind(links[1], Target(10.0));
    w.step(1.0);
    assert!((w.read::<Angle>(links[1]).unwrap().0 - 1.0).abs() < 1e-6, "one radian a second");
    for _ in 0..10 {
        w.step(1.0);
    }
    assert_eq!(w.read::<Angle>(links[1]).unwrap().0, 3.0, "held by its limit");
}

#[test]
fn reparenting_with_keep_world_keeps_a_link_where_it_is() {
    let (mut w, links, tip) = arm();
    w.bind(links[1], Target(HALF_TURN));
    for _ in 0..20 {
        w.step(0.1);
    }
    let before = tip_at(&w, tip);
    assert!(w.set_parent(tip, links[0], true), "the pin moves from fore to base");
    w.step(0.1);
    assert!(near(tip_at(&w, tip), before), "kept: {:?} vs {before:?}", tip_at(&w, tip));
    w.read_mut::<Pose>(links[0]).unwrap().t = [0.0, 0.0, 5.0];
    w.step(0.1);
    assert!(near(tip_at(&w, tip), [before[0], before[1], 5.0]), "now it rides on base, not on the forearm");
}

#[test]
fn a_link_naming_an_unspawned_parent_is_refused() {
    let bot = Bot::new("odd").link("a", "missing", Pose::default(), z(0.0, 1.0));
    assert!(spawn(&mut World::new(), &bot).is_none());
}
