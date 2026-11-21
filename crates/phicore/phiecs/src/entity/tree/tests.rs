use super::*;

fn near(a: [f32; 3], b: [f32; 3]) -> bool {
    a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-4)
}

fn words(w: &mut World, names: &[&str]) -> Vec<Word> {
    names.iter().map(|n| w.coin_bare(n)).collect()
}

#[test]
fn children_keep_attach_order_and_a_loop_is_refused() {
    let mut w = World::new();
    let [a, b, c]: [Word; 3] = words(&mut w, &["a", "b", "c"]).try_into().unwrap();
    assert!(w.tree.attach(b, a) && w.tree.attach(c, b));
    assert_eq!(w.tree.descendants(a), [b, c]);
    assert_eq!(w.tree.roots(), [a]);
    assert!(!w.tree.attach(a, c), "a is above c");
    assert!(!w.tree.attach(a, a));
    assert_eq!(w.tree.parent(a), None, "nothing changed");
    assert!(w.set_parent(c, a, false) && w.tree.children(b).is_empty());
    assert_eq!(w.tree.children(a), [b, c]);
    assert!(!w.set_parent(a, c, true), "the loop is refused through the world too");
}

#[test]
fn without_keep_the_child_jumps_with_its_new_parent() {
    let mut w = World::new();
    let a = w.coin("a").bind(Pose::at([10.0, 0.0, 0.0])).word();
    let c = w.coin("c").bind(Pose::at([0.0, 1.0, 0.0])).word();
    assert!(w.set_parent(c, a, false));
    assert_eq!(w.read::<Pose>(c).unwrap().t, [0.0, 1.0, 0.0]);
    assert!(near(w.world_of(c).point([0.0; 3]), [10.0, 1.0, 0.0]));
}

#[test]
fn with_keep_the_child_stays_where_it_is() {
    let mut w = World::new();
    let turn = Pose::turned([0.0, 0.0, 1.0], 1.0);
    let a = w.coin("a").bind(Pose { t: [10.0, 0.0, 0.0], r: turn, s: [2.0, 2.0, 2.0] }).word();
    let c = w.coin("c").bind(Pose::at([0.0, 1.0, 0.0])).word();
    let before = w.world_of(c);
    assert!(w.set_parent(c, a, true));
    let after = w.world_of(c);
    assert!(near(before.point([1.0, 2.0, 3.0]), after.point([1.0, 2.0, 3.0])));
    assert_ne!(w.read::<Pose>(c).unwrap().t, [0.0, 1.0, 0.0], "the local pose was solved");
    assert!(w.clear_parent(c, true));
    assert!(near(w.world_of(c).point([1.0, 2.0, 3.0]), before.point([1.0, 2.0, 3.0])));
}

#[test]
fn reparenting_is_announced_and_forgetting_frees_the_children() {
    let mut w = World::new();
    let [a, b, c]: [Word; 3] = words(&mut w, &["a", "b", "c"]).try_into().unwrap();
    w.set_parent(c, a, false);
    w.set_parent(c, b, false);
    let seen: Vec<_> = w.events::<DidReparent>().copied().collect();
    assert_eq!(seen[0], DidReparent { child: c, old: None, new: Some(a) });
    assert_eq!(seen[1], DidReparent { child: c, old: Some(a), new: Some(b) });
    w.forget(b);
    assert_eq!(w.tree.parent(c), None);
}

#[test]
fn a_pin_is_a_named_child_with_an_offset() {
    let mut w = World::new();
    let hand = w.coin("hand").bind(Pose::at([1.0, 0.0, 0.0])).word();
    let grip = w.pin(hand, "grip", Pose::at([0.0, 0.5, 0.0]));
    let tool = w.coin("tool").word();
    w.set_parent(tool, grip, false);
    assert_eq!(w.spelling(grip), "hand.grip");
    assert_eq!(w.tree.parent(grip), Some(hand));
    assert!(near(w.world_of(tool).point([0.0; 3]), [1.0, 0.5, 0.0]), "the tool is on the pin");
}
