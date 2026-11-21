//! Data bound to words, one dense store per type.
use crate::word::Word;
use std::any::Any;
/// A type that can be bound to a word.
pub trait Aspect: Send + Sync + 'static {
    /// True for a transient aspect (Apple's `TransientComponent`, RF:6571):
    /// it is never dumped or journalled.
    const TRANSIENT: bool = false;
    /// The aspect's name, so it can be listed, saved and addressed by text
    /// (Apple's `registerComponent`, RF:6568). The default is the type's path.
    fn name() -> &'static str
    where
        Self: Sized,
    {
        std::any::type_name::<Self>()
    }
    fn on_bind(&self, _word: Word) {}
    fn on_unbind(&self, _word: Word) {}
}
/// One aspect type's values, indexed by word id.
pub struct Dense<A> {
    slots: Vec<Option<A>>,
    /// The tick each slot was last set or taken `&mut`, if ever.
    changed: Vec<Option<u64>>,
}
impl<A> Default for Dense<A> {
    fn default() -> Self {
        Self { slots: Vec::new(), changed: Vec::new() }
    }
}
impl<A: Aspect> Dense<A> {
    pub fn insert(&mut self, word: Word, aspect: A) -> Option<A> {
        let i = word.id() as usize;
        if i >= self.slots.len() {
            self.slots.resize_with(i + 1, || None);
            self.changed.resize(i + 1, None);
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
        self.changed[word.id() as usize] = None;
        if let Some(a) = &taken {
            a.on_unbind(word);
        }
        taken
    }
    /// Marks `word` changed at `tick`. True when that is news: it was not
    /// already marked for this tick.
    pub fn touch(&mut self, word: Word, tick: u64) -> bool {
        match self.changed.get_mut(word.id() as usize) {
            Some(c) if *c == Some(tick) => false,
            Some(c) => {
                *c = Some(tick);
                true
            }
            None => false,
        }
    }
    /// The tick `word`'s value was last set or taken `&mut`.
    pub fn changed(&self, word: Word) -> Option<u64> {
        self.changed.get(word.id() as usize).copied().flatten()
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
    fn name(&self) -> &'static str;
    fn has(&self, word: Word) -> bool;
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
    fn name(&self) -> &'static str {
        A::name()
    }
    fn has(&self, word: Word) -> bool {
        self.get(word).is_some()
    }
}
