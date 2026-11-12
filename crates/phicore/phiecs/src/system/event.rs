//! Events: typed, pulled by readers, never called back.
//!
//! Apple's `SceneEvents` and `ComponentEvents` (RF:7086-7114, 7221-7242) are
//! delivered to closures through `subscribe` (RF:7403). Here the queue is a
//! double buffer: an event emitted in step N can be read for the rest of step
//! N and all of step N + 1, then it is dropped. `step` swaps it last.
use crate::word::Word;
use crate::world::World;
use std::any::{Any, TypeId};
use std::collections::HashMap;

/// A word was coined (Apple's DidAddEntity, RF:7086-7114).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DidAdd(pub Word);

/// A word is about to be forgotten, and still holds its aspects.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WillRemove(pub Word);

/// One event type's two buffers.
struct Buffer<E> {
    last: Vec<E>,
    now: Vec<E>,
}

/// A buffer of any event type, as the queue keeps them side by side.
trait Slot: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn swap(&mut self);
    fn len(&self) -> usize;
}

impl<E: Send + Sync + 'static> Slot for Buffer<E> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn swap(&mut self) {
        std::mem::swap(&mut self.last, &mut self.now);
        self.now.clear();
    }
    fn len(&self) -> usize {
        self.last.len() + self.now.len()
    }
}

/// The world's events, a resource. See the module page for the lifetime of an
/// event.
#[derive(Default)]
pub struct EventQueue {
    slots: HashMap<TypeId, Box<dyn Slot>>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues `event` for readers of `E`.
    pub fn emit<E: Send + Sync + 'static>(&mut self, event: E) {
        let slot = self.slots.entry(TypeId::of::<E>()).or_insert_with(|| Box::new(Buffer::<E> { last: Vec::new(), now: Vec::new() }));
        if let Some(b) = slot.as_any_mut().downcast_mut::<Buffer<E>>() {
            b.now.push(event);
        }
    }

    /// Every `E` still readable: last step's, then this step's so far.
    pub fn read<E: Send + Sync + 'static>(&self) -> impl Iterator<Item = &E> {
        let b = self.slots.get(&TypeId::of::<E>()).and_then(|s| s.as_any().downcast_ref::<Buffer<E>>());
        b.into_iter().flat_map(|b| b.last.iter().chain(b.now.iter()))
    }

    /// Ends a step: this step's events become last step's, and last step's
    /// are dropped.
    pub fn swap(&mut self) {
        self.slots.values_mut().for_each(|s| s.swap());
    }

    /// How many events of every type are held.
    pub fn len(&self) -> usize {
        self.slots.values().map(|s| s.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl World {
    /// Queues `event` on the world's [`EventQueue`].
    pub fn emit<E: Send + Sync + 'static>(&mut self, event: E) {
        if let Some(q) = self.resources.get_mut::<EventQueue>() {
            q.emit(event);
        }
    }

    /// Every `E` still readable on the world's queue.
    pub fn events<E: Send + Sync + 'static>(&self) -> impl Iterator<Item = &E> {
        self.resources.get::<EventQueue>().into_iter().flat_map(|q| q.read::<E>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Hit(u32);

    #[test]
    fn readable_for_one_step_then_dropped() {
        let mut q = EventQueue::new();
        q.emit(Hit(1));
        assert_eq!(q.read::<Hit>().collect::<Vec<_>>(), [&Hit(1)], "same step");
        q.swap();
        q.emit(Hit(2));
        assert_eq!(q.read::<Hit>().collect::<Vec<_>>(), [&Hit(1), &Hit(2)], "next step sees both");
        q.swap();
        assert_eq!(q.read::<Hit>().collect::<Vec<_>>(), [&Hit(2)], "the first is gone");
        q.swap();
        assert!(q.is_empty());
    }

    #[test]
    fn types_do_not_mix() {
        let mut q = EventQueue::new();
        q.emit(Hit(1));
        q.emit(7u8);
        assert_eq!(q.read::<u8>().count(), 1);
        assert_eq!(q.len(), 2);
    }
}
