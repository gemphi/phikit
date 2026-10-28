//! Data bound to words, one dense store per type.

use crate::word::Word;
use std::any::Any;

/// A type that can be bound to a word.
pub trait Aspect: Send + Sync + 'static {
    fn on_bind(&self, _word: Word) {}
    fn on_unbind(&self, _word: Word) {}
}

/// One aspect type's values, indexed by word id.
pub struct Dense<A> {
    slots: Vec<Option<A>>,
}

impl<A> Default for Dense<A> {
    fn default() -> Self {
        Self { slots: Vec::new() }
    }
}

impl<A: Aspect> Dense<A> {
    pub fn insert(&mut self, word: Word, aspect: A) -> Option<A> {
        let i = word.id() as usize;
        if i >= self.slots.len() {
            self.slots.resize_with(i + 1, || None);
        }
        aspect.on_bind(word);
        self.slots[i].replace(aspect)
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
        }
        taken
    }

    pub fn iter(&self) -> impl Iterator<Item = (Word, &A)> {
        self.slots.iter().enumerate().filter_map(|(i, s)| s.as_ref().map(|a| (Word::from_id(i as u32), a)))
    }
}

/// A store of any aspect type, as the world keeps them side by side.
pub(crate) trait AspectStore: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn forget(&mut self, word: Word) -> bool;
}

impl<A: Aspect> AspectStore for Dense<A> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn forget(&mut self, word: Word) -> bool {
        self.remove(word).is_some()
    }
}
