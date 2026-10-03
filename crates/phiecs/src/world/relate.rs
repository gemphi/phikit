#[allow(unused_imports)]
use super::*;

use crate::phasor::Phasor;
use crate::relation::Relation;

/// How far a word moves toward what it was seen with, per observation.
pub const COUPLE_RATE: f64 = 0.08;

impl World {
    /// `from` stands to `to` under `kind`: record the edge, count the kind,
    /// and move the two words' phasors toward each other on that channel.
    /// This is `add_child` in Apple's model (`world.rs:207`) and the whole of
    /// learning in this one.
    pub fn relate(&mut self, from: Word, kind: Relation, to: Word) {
        self.relate_by(from, kind, to, 1.0, COUPLE_RATE);
    }

    /// [`World::relate`] with the edge weight and coupling rate given.
    pub fn relate_by(&mut self, from: Word, kind: Relation, to: Word, weight: f64, rate: f64) {
        self.edges.relate(from, kind, to, weight);
        self.relations.observe(kind);
        self.register::<Phasor>();
        for w in [from, to] {
            if self.read::<Phasor>(w).is_none() {
                let seed = Phasor::seeded(self.spelling(w));
                self.bind(w, seed);
            }
        }
        let (a, b) = match (self.read::<Phasor>(from), self.read::<Phasor>(to)) {
            (Some(a), Some(b)) => (a.theta(kind), b.theta(kind)),
            _ => return,
        };
        if let Some(p) = self.read_mut::<Phasor>(from) {
            p.couple(kind, b, rate);
        }
        if let Some(p) = self.read_mut::<Phasor>(to) {
            p.couple(kind, a, rate);
        }
    }

    /// Take the relation back.
    pub fn unrelate(&mut self, from: Word, kind: Relation, to: Word) -> bool {
        self.edges.unrelate(from, kind, to)
    }

    /// What `word` calls for under `kind`, heaviest first — X ⇒ Y.
    pub fn calls_for(&self, word: Word, kind: Relation) -> Vec<(Word, f64)> {
        self.edges.calls_for(word, kind)
    }

    /// Declare a relation kind by name, interning it as a word too, so a kind
    /// can itself be related to (a condition is a word, see `condition`).
    pub fn declare(&mut self, name: &str) -> Option<Relation> {
        self.lexicon.intern(name);
        self.relations.declare(name)
    }
}
