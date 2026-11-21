//! Linking words: declaring kinds, relating and unrelating, reading back.

use super::World;
use crate::relation::Relation;
use crate::word::Word;

impl World {
    /// The kind called `name`. Its name is spelled in the lexicon too, as
    /// phinet's phiecs spells it, so word ids come out the same on both.
    pub fn declare(&mut self, name: &str) -> Option<Relation> {
        self.lexicon.intern(name);
        self.relations.declare(name)
    }

    /// `from` is `kind` to `to`, once more.
    pub fn relate(&mut self, from: Word, kind: Relation, to: Word) {
        self.edges.relate(from, kind, to, 1.0);
    }

    /// `from` is `kind` to `to`, by `weight`. The rate is how fast phinet's
    /// phasors pull the two together; there are no phasors here, so it is
    /// taken and left unused.
    pub fn relate_by(&mut self, from: Word, kind: Relation, to: Word, weight: f64, _rate: f64) {
        self.edges.relate(from, kind, to, weight);
    }

    pub fn unrelate(&mut self, from: Word, kind: Relation, to: Word) -> bool {
        self.edges.unrelate(from, kind, to)
    }

    /// What `word` is `kind` to, heaviest first.
    pub fn calls_for(&self, word: Word, kind: Relation) -> Vec<(Word, f64)> {
        self.edges.calls_for(word, kind)
    }

    pub fn holds(&self, about: Word, kind: Relation, other: Word) -> bool {
        self.calls_for(about, kind).iter().any(|(w, _)| *w == other)
    }

    /// How strongly `about` is `kind` to `other`: 0 when not at all.
    pub fn degree(&self, about: Word, kind: Relation, other: Word) -> f64 {
        self.calls_for(about, kind).into_iter().find(|(w, _)| *w == other).map_or(0.0, |(_, w)| w)
    }
}
