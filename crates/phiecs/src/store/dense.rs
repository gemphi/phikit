#[allow(unused_imports)]
use super::*;

use crate::aspect::Aspect;
use crate::word::Word;

/// One aspect type for every word that carries it, indexed by `Word::id`.
pub struct Dense<A> {
    slots: Vec<Option<A>>,
    len: usize,
}

impl<A> Default for Dense<A> {
    fn default() -> Self {
        Self { slots: Vec::new(), len: 0 }
    }
}

impl<A: Aspect> Dense<A> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind `aspect` to `word`, returning what it replaced.
    pub fn insert(&mut self, word: Word, aspect: A) -> Option<A> {
        let i = word.id() as usize;
        if i >= self.slots.len() {
            self.slots.resize_with(i + 1, || None);
        }
        aspect.on_bind(word);
        let previous = self.slots[i].replace(aspect);
        if previous.is_none() {
            self.len += 1;
        }
        previous
    }

    pub fn get(&self, word: Word) -> Option<&A> {
        self.slots.get(word.id() as usize)?.as_ref()
    }

    pub fn get_mut(&mut self, word: Word) -> Option<&mut A> {
        self.slots.get_mut(word.id() as usize)?.as_mut()
    }

    pub fn remove(&mut self, word: Word) -> Option<A> {
        let taken = self.slots.get_mut(word.id() as usize)?.take();
        if let Some(a) = &taken {
            a.on_unbind(word);
            self.len -= 1;
        }
        taken
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Every bound word and its aspect, in word order.
    pub fn iter(&self) -> impl Iterator<Item = (Word, &A)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_ref().map(|a| (Word::from_id(i as u32), a)))
    }

    /// The same, mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Word, &mut A)> {
        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(|(i, s)| s.as_mut().map(|a| (Word::from_id(i as u32), a)))
    }
}

impl<A: Aspect> AspectStore for Dense<A> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn forget(&mut self, word: Word) -> bool {
        self.remove(word).is_some()
    }
}
