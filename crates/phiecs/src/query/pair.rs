#[allow(unused_imports)]
use super::*;

use crate::relation::Relation;

/// Words carrying both `A` and `B`.
pub struct With2<'a, A, B> {
    world: &'a World,
    _a: std::marker::PhantomData<(A, B)>,
}

impl<'a, A: Aspect, B: Aspect> With2<'a, A, B> {
    pub fn new(world: &'a World) -> Self {
        Self { world, _a: std::marker::PhantomData }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Word, &'a A, &'a B)> + '_ {
        self.world
            .with::<A>()
            .filter_map(move |(w, a)| self.world.read::<B>(w).map(|b| (w, a, b)))
    }
}

/// The join along a relation kind: every `(from, to)` pair standing under
/// `kind`, with the weight behind it. Apple has no counterpart — its
/// hierarchy is a single-parent tree, so there is nothing to join on.
pub struct Related<'a> {
    world: &'a World,
    kind: Relation,
}

impl<'a> Related<'a> {
    pub fn new(world: &'a World, kind: Relation) -> Self {
        Self { world, kind }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Word, Word, f64)> + '_ {
        self.world
            .words()
            .iter()
            .flat_map(move |w| self.world.edges.out_of(*w))
            .filter(move |e| e.kind == self.kind)
            .map(|e| (e.from, e.to, e.weight))
    }

    /// The words standing on the left of this kind at all.
    pub fn heads(&self) -> Vec<Word> {
        let mut v: Vec<Word> = self.iter().map(|(f, _, _)| f).collect();
        v.sort();
        v.dedup();
        v
    }
}

impl World {
    pub fn with2<A: Aspect, B: Aspect>(&self) -> With2<'_, A, B> {
        With2::new(self)
    }

    /// Every pair under `kind`.
    pub fn related(&self, kind: Relation) -> Related<'_> {
        Related::new(self, kind)
    }
}
