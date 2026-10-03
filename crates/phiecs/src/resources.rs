//! World-wide values that belong to no single word.
//!
//! Apple has this type and never wires it into `World`
//! (`reality-foundation/resources.rs`; its own README flags it). Here the
//! world owns one, because a quest needs somewhere to keep its oracle's
//! answer budget and its step count.

use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Default)]
pub struct Resources {
    data: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Resources {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, value: T) -> Option<T> {
        self.data
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|b| b.downcast().ok().map(|b| *b))
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.data.get(&TypeId::of::<T>())?.downcast_ref()
    }

    pub fn get_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        self.data.get_mut(&TypeId::of::<T>())?.downcast_mut()
    }

    pub fn remove<T: Any + Send + Sync>(&mut self) -> Option<T> {
        self.data.remove(&TypeId::of::<T>()).and_then(|b| b.downcast().ok().map(|b| *b))
    }
}
