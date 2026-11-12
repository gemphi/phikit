//! The world: its words, their aspects and edges, its resources and systems.
mod relate;
#[cfg(test)]
mod tests;
use crate::aspect::{Aspect, AspectStore, Dense};
use crate::comp::time::Clock;
use crate::relation::{Graph, Register};
use crate::tree::Tree;
use crate::resources::Resources;
use crate::system::System;
use crate::system::event::{DidAdd, EventQueue, WillRemove};
use crate::word::{Lexicon, Word};
use std::any::TypeId;
use std::collections::HashMap;
pub struct World {
    pub lexicon: Lexicon,
    pub relations: Register,
    pub edges: Graph,
    pub resources: Resources,
    pub tree: Tree,
    stores: HashMap<TypeId, Box<dyn AspectStore>>,
    words: Vec<Word>,
    pub(crate) systems: Vec<Box<dyn System>>,
}
impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
impl World {
    /// An empty world. Its [`Clock`] and [`EventQueue`] are already resources.
    pub fn new() -> Self {
        let mut resources = Resources::new();
        resources.insert(Clock::default());
        resources.insert(EventQueue::new());
        Self {
            lexicon: Lexicon::new(),
            relations: Register::new(),
            edges: Graph::new(),
            resources,
            tree: Tree::new(),
            stores: HashMap::new(),
            words: Vec::new(),
            systems: Vec::new(),
        }
    }
    /// The word spelled `spelling`, coined now if it is new, to bind
    /// aspects to as it is made.
    pub fn coin(&mut self, spelling: &str) -> WordBuilder<'_> {
        let word = self.coin_bare(spelling);
        WordBuilder { world: self, word }
    }
    /// The word spelled `spelling`, coined now if it is new. A new one is
    /// announced as [`DidAdd`].
    pub fn coin_bare(&mut self, spelling: &str) -> Word {
        let word = self.lexicon.intern(spelling);
        if !self.words.contains(&word) {
            self.words.push(word);
            self.emit(DidAdd(word));
        }
        word
    }
    /// Drops `word` from the world with its aspects and edges. Its spelling
    /// stays in the lexicon, so its id is never reused. [`WillRemove`] is
    /// emitted first, while the word still holds its aspects.
    pub fn forget(&mut self, word: Word) -> bool {
        let known = self.words.iter().position(|w| *w == word);
        if known.is_some() {
            self.emit(WillRemove(word));
        }
        for store in self.stores.values_mut() {
            store.forget(word);
        }
        self.edges.forget(word);
        self.tree.forget(word);
        match known {
            Some(i) => {
                self.words.remove(i);
                true
            }
            None => false,
        }
    }
    /// The coined words, in the order they were coined.
    pub fn words(&self) -> &[Word] {
        &self.words
    }
    pub fn spelling(&self, word: Word) -> &str {
        self.lexicon.spelling(word).unwrap_or("?")
    }
    pub fn register<A: Aspect>(&mut self) {
        self.stores.entry(TypeId::of::<A>()).or_insert_with(|| Box::new(Dense::<A>::default()));
    }
    /// Binds `aspect` to `word`, handing back the one it replaces.
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
    /// Every word with an `A`, and its `A`.
    pub fn each<A: Aspect>(&self) -> impl Iterator<Item = (Word, &A)> {
        self.store::<A>().into_iter().flat_map(Dense::iter)
    }
    fn store<A: Aspect>(&self) -> Option<&Dense<A>> {
        self.stores.get(&TypeId::of::<A>())?.as_any().downcast_ref()
    }
    fn store_mut<A: Aspect>(&mut self) -> Option<&mut Dense<A>> {
        self.stores.get_mut(&TypeId::of::<A>())?.as_any_mut().downcast_mut()
    }
    /// The world's clock.
    pub fn clock(&self) -> Clock {
        self.resources.get::<Clock>().copied().unwrap_or_default()
    }
}
/// A word being coined, so aspects can be bound to it in one expression.
pub struct WordBuilder<'a> {
    world: &'a mut World,
    word: Word,
}
impl WordBuilder<'_> {
    pub fn bind<A: Aspect>(self, aspect: A) -> Self {
        self.world.bind(self.word, aspect);
        self
    }
    pub fn word(self) -> Word {
        self.word
    }
}
