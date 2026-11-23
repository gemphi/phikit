//! # Reactive State
//!
//! Observable state primitives for the declarative layout engine.
//! When a `State<T>` value changes, it signals that the owning view
//! needs to rebuild its `body()`.
//!
//! ## Usage
//!
//! ```ignore
//! use rust_appkit::layout_engine::State;
//!
//! let counter = State::new(0);
//! counter.set(1);  // marks dirty
//! assert_eq!(*counter.get(), 1);
//! assert!(counter.is_dirty());
//! counter.clean();  // reset after rebuild
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Observable state wrapper — triggers view rebuild when mutated.
///
/// `State<T>` wraps a value and tracks whether it has been modified
/// since the last `clean()` call. The view host checks `is_dirty()`
/// each frame and calls `ViewBuilder::body()` to rebuild if needed.
#[derive(Debug)]
pub struct State<T> {
    value: Arc<Mutex<T>>,
    dirty: Arc<AtomicBool>,
}

impl<T> State<T> {
    /// Create a new state with an initial value.
    pub fn new(initial: T) -> Self {
        Self {
            value: Arc::new(Mutex::new(initial)),
            dirty: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get a clone of the current value.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.value.lock().unwrap().clone()
    }

    /// Get a reference to the value (locks the mutex).
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.value.lock().unwrap();
        f(&*guard)
    }

    /// Set a new value and mark as dirty.
    pub fn set(&self, new_value: T) {
        *self.value.lock().unwrap() = new_value;
        self.dirty.store(true, Ordering::Release);
    }

    /// Mutate the value in-place and mark as dirty.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let mut guard = self.value.lock().unwrap();
        f(&mut *guard);
        self.dirty.store(true, Ordering::Release);
    }

    /// Check if the state has been modified since last `clean()`.
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Acquire)
    }

    /// Reset the dirty flag (called after view rebuild).
    pub fn clean(&self) {
        self.dirty.store(false, Ordering::Release);
    }
}

impl<T: Clone> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            dirty: self.dirty.clone(),
        }
    }
}

impl<T: Default> Default for State<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// A collection of state objects that can be checked for any dirty state.
/// Used by view hosts to determine if a rebuild is needed.
#[derive(Debug, Default)]
pub struct StateGroup {
    flags: Vec<Arc<AtomicBool>>,
}

impl StateGroup {
    pub fn new() -> Self {
        Self { flags: Vec::new() }
    }

    /// Register a state's dirty flag with this group.
    pub fn track<T>(&mut self, state: &State<T>) {
        self.flags.push(state.dirty.clone());
    }

    /// Returns true if any tracked state is dirty.
    pub fn any_dirty(&self) -> bool {
        self.flags.iter().any(|f| f.load(Ordering::Acquire))
    }

    /// Clean all tracked states.
    pub fn clean_all(&self) {
        for flag in &self.flags {
            flag.store(false, Ordering::Release);
        }
    }
}
