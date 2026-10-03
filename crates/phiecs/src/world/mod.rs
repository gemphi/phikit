//! `World` — the lexicon, its aspects, its edges and its systems.
//!
//! The same name as Apple's, and the same four responsibilities
//! (`reality-foundation/world.rs:35`), with two of its defects fixed: a store
//! is dense rather than a linear scan, and `forget` actually drops the word's
//! aspects instead of leaving them behind for the next id to inherit (Apple's
//! `despawn` says so in a comment at `world.rs:174`).

mod relate;
mod schedule;
mod spawn;
pub use relate::COUPLE_RATE;
pub use spawn::WordBuilder;

use crate::aspect::Aspect;
use crate::relation::{Edges, Relations};
use crate::resources::Resources;
use crate::store::{AspectStore, Dense};
use crate::system::System;
use crate::word::{Lexicon, Word};
use std::any::TypeId;
use std::collections::HashMap;

pub struct World {
    pub lexicon: Lexicon,
    pub relations: Relations,
    pub edges: Edges,
    pub resources: Resources,
    pub(crate) stores: HashMap<TypeId, Box<dyn AspectStore>>,
    pub(crate) words: Vec<Word>,
    pub(crate) systems: Vec<Box<dyn System>>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    /// An empty world that already knows the named relation kinds.
    pub fn new() -> Self {
        Self {
            lexicon: Lexicon::new(),
            relations: Relations::named(),
            edges: Edges::new(),
            resources: Resources::new(),
            stores: HashMap::new(),
            words: Vec::new(),
            systems: Vec::new(),
        }
    }

    /// Make room for an aspect type before anything is bound. Optional —
    /// `bind` registers on first use — but it makes a world's shape readable
    /// at its construction site, which is why Apple has `register` too.
    pub fn register<A: Aspect>(&mut self) {
        self.stores.entry(TypeId::of::<A>()).or_insert_with(|| Box::new(Dense::<A>::new()));
    }

    /// Bind `aspect` to `word`, returning what it replaced.
    pub fn bind<A: Aspect>(&mut self, word: Word, aspect: A) -> Option<A> {
        self.register::<A>();
        self.store_mut::<A>()?.insert(word, aspect)
    }

    pub fn read<A: Aspect>(&self, word: Word) -> Option<&A> {
        self.store::<A>()?.get(word)
    }

    pub fn read_mut<A: Aspect>(&mut self, word: Word) -> Option<&mut A> {
        self.store_mut::<A>()?.get_mut(word)
    }

    pub fn unbind<A: Aspect>(&mut self, word: Word) -> Option<A> {
        self.store_mut::<A>()?.remove(word)
    }

    pub(crate) fn store<A: Aspect>(&self) -> Option<&Dense<A>> {
        self.stores.get(&TypeId::of::<A>())?.as_any().downcast_ref()
    }

    pub(crate) fn store_mut<A: Aspect>(&mut self) -> Option<&mut Dense<A>> {
        self.stores.get_mut(&TypeId::of::<A>())?.as_any_mut().downcast_mut()
    }

    /// Every word this world has coined, in id order.
    pub fn words(&self) -> &[Word] {
        &self.words
    }

    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    pub fn contains(&self, word: Word) -> bool {
        (word.id() as usize) < self.lexicon.len()
    }

    /// Its spelling, for anything a person will read.
    pub fn spelling(&self, word: Word) -> &str {
        self.lexicon.spelling(word).unwrap_or("?")
    }
}

#[cfg(test)]
mod tests;
