#[allow(unused_imports)]
use super::*;
use crate::phasor::Phasor;

#[test]
fn coining_is_idempotent_and_forgetting_is_total() {
    let mut w = World::new();
    let cat = w.coin("cat").bind(Phasor::seeded("cat")).word();
    assert_eq!(cat, w.coin_bare("cat"), "a lexicon holds one of each word");
    assert_eq!(w.word_count(), 1);
    assert!(w.read::<Phasor>(cat).is_some());
    let mat = w.coin_bare("mat");
    let on = w.declare("on").unwrap();
    w.relate(cat, on, mat);
    assert_eq!(w.calls_for(cat, on), vec![(mat, 1.0)]);

    assert!(w.forget(cat));
    assert!(w.read::<Phasor>(cat).is_none(), "aspects go with the word");
    assert!(w.edges.out_of(cat).is_empty());
    assert!(w.edges.into_of(mat).is_empty(), "the other side is cleaned too");
    assert_eq!(w.spelling(cat), "cat", "what was said stays readable");
}

#[test]
fn relating_moves_both_words_on_that_channel_only() {
    let mut w = World::new();
    let (a, b) = (w.coin_bare("morning"), w.coin_bare("regard"));
    let under = w.declare("under").unwrap();
    let other = w.declare("beside").unwrap();
    let before = w.read::<Phasor>(a).map(|p| p.theta(under));
    for _ in 0..30 {
        w.relate(a, under, b);
    }
    let pa = w.read::<Phasor>(a).unwrap();
    let pb = w.read::<Phasor>(b).unwrap();
    assert!(pa.agreement(pb, under).unwrap() > 0.9);
    assert_eq!(pa.agreement(pb, other), None, "an unobserved channel stays silent");
    assert_ne!(Some(pa.theta(under)), before);
    assert_eq!(w.relations.evidence(under), 30);
}
