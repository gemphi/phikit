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
