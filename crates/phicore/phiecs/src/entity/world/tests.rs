use super::*;
use crate::aspect::Aspect;

struct Colour(&'static str);
impl Aspect for Colour {}

#[test]
fn a_plan_is_the_heaviest_chain_shown() {
    let mut w = World::new();
    let before = w.declare("before").unwrap();
    let [a, b, c, d, e] = ["a", "b", "c", "d", "e"].map(|s| w.coin_bare(s));
    for _ in 0..2 {
        w.relate(a, before, b);
        w.relate(b, before, d);
    }
    w.relate(a, before, c);
    w.relate(c, before, d);
    w.relate(d, before, e);
    assert_eq!(w.plan(a, before, e, 4), Some(vec![a, b, d, e]));
    assert_eq!(w.plan_avoiding(a, before, e, 4, &[b]), Some(vec![a, c, d, e]));
    assert_eq!(w.plan_avoiding(a, before, e, 4, &[d]), None, "every road to e runs through d");
    assert_eq!(w.plan(a, before, e, 2), None, "too short");
    assert_eq!(w.plan(e, before, a, 4), None, "never shown");
    let r = w.reach(a, before, 3);
    assert_eq!(r[0].0, d, "{r:?}");
    assert_eq!(r.len(), 4);
}

/// Ids are what phiplay's dice roll on, so they must come out as phinet's
/// phiecs gives them: a declared kind's name takes a lexicon id, though it
/// is not a coined word.
#[test]
fn ids_follow_the_lexicon_and_kinds_spell_into_it() {
    let mut w = World::new();
    let a = w.coin_bare("a");
    let on = w.declare("on").unwrap();
    let b = w.coin_bare("b");
    assert_eq!((a.id(), b.id()), (0, 2));
    assert_eq!(w.words(), &[a, b]);
    assert_eq!(w.coin_bare("a"), a);
    w.relate(a, on, b);
    assert!(w.holds(a, on, b) && !w.holds(b, on, a));
    assert_eq!(w.degree(a, on, b), 1.0);
}

#[test]
fn aspects_bind_read_and_go_with_their_word() {
    let mut w = World::new();
    let sky = w.coin("sky").bind(Colour("blue")).word();
    let sea = w.coin_bare("sea");
    assert_eq!(w.read::<Colour>(sky).map(|c| c.0), Some("blue"));
    assert!(w.read::<Colour>(sea).is_none());
    assert_eq!(w.bind(sky, Colour("grey")).map(|c| c.0), Some("blue"));
    assert_eq!(w.each::<Colour>().map(|(word, c)| (word, c.0)).collect::<Vec<_>>(), vec![(sky, "grey")]);
    assert!(w.forget(sky));
    assert!(w.read::<Colour>(sky).is_none() && w.words() == [sea]);
}

#[test]
fn resources_are_one_per_type() {
    let mut w = World::new();
    w.resources.insert(3u32);
    assert_eq!(w.resources.insert(4u32), Some(3));
    *w.resources.get_mut::<u32>().unwrap() += 1;
    assert_eq!(w.resources.remove::<u32>(), Some(5));
    assert!(w.resources.get::<u32>().is_none());
}
struct Mass(f32);
impl Aspect for Mass {
    fn name() -> &'static str {
        "mass"
    }
}
struct Scratch;
impl Aspect for Scratch {
    const TRANSIENT: bool = true;
}
#[test]
fn aspects_have_names_and_a_word_lists_them() {
    let mut w = World::new();
    let a = w.coin_bare("a");
    w.bind(a, Mass(1.0));
    w.bind(a, Colour("red"));
    w.bind(a, Scratch);
    assert_eq!(Mass::name(), "mass");
    assert!(Colour::name().ends_with("Colour"), "default is the type path");
    let list = w.aspect_list(a);
    assert_eq!(list.len(), 3);
    assert!(list.contains(&"mass"));
    assert!(Scratch::TRANSIENT && !Mass::TRANSIENT);
    w.unbind::<Mass>(a);
    assert!(!w.aspect_list(a).contains(&"mass"));
}
#[test]
fn a_write_stamps_the_tick_and_says_so_once() {
    use crate::system::event::DidChange;
    let mut w = World::new();
    let (a, b) = (w.coin_bare("a"), w.coin_bare("b"));
    w.bind(a, Mass(1.0));
    w.bind(b, Mass(2.0));
    assert_eq!(w.changed::<Mass>(a), Some(0), "bound before any step");
    w.step(1.0);
    w.step(1.0);
    assert_eq!(w.changed::<Mass>(a), Some(0));
    w.read_mut::<Mass>(a).unwrap().0 = 5.0;
    w.read_mut::<Mass>(a).unwrap().0 = 6.0;
    assert_eq!(w.changed::<Mass>(a), Some(2));
    assert_eq!(w.changed_since::<Mass>(1), [(a, 2)], "b did not change since tick 1");
    let seen: Vec<_> = w.events::<DidChange>().filter(|e| e.word == a && e.aspect == "mass").collect();
    assert_eq!(seen.len(), 1, "two writes in one tick, one event");
    let c = w.coin_bare("c");
    assert_eq!(w.changed::<Mass>(c), None);
    w.unbind::<Mass>(a);
    assert_eq!(w.changed::<Mass>(a), None);
}