//! `Condition` — the "under what conditions" of the paradigm, as state.
//!
//! Apple's only state machinery is GameplayKit's `GKStateMachine`, which keys
//! states by name and finds them with a linear scan over `&str`
//! (`gameplay_kit/mod.rs:31`). The shape is right and the lookup is not: here
//! a condition is an interned [`Word`], so entering one is a comparison, and
//! the condition a reading is taken under is itself a word the world knows.

mod set;
pub use set::{CONDITION_NAMES, Conditions};

use crate::aspect::Aspect;
use crate::word::Word;

/// The condition a word is currently read under, and what it was before.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Condition {
    now: Word,
    was: Option<Word>,
    /// Steps spent in the current condition.
    pub steps: u32,
}

impl Aspect for Condition {}

impl Condition {
    pub fn new(now: Word) -> Self {
        Self { now, was: None, steps: 0 }
    }

    pub fn now(&self) -> Word {
        self.now
    }

    pub fn was(&self) -> Option<Word> {
        self.was
    }

    /// Enter `next`; `false` when already there (nothing moves, nothing is
    /// counted twice).
    pub fn enter(&mut self, next: Word) -> bool {
        if next == self.now {
            return false;
        }
        self.was = Some(self.now);
        self.now = next;
        self.steps = 0;
        true
    }

    /// One step under the current condition.
    pub fn tick(&mut self) {
        self.steps = self.steps.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entering_records_where_it_came_from() {
        let (a, b) = (Word::from_id(1), Word::from_id(2));
        let mut c = Condition::new(a);
        assert!(!c.enter(a));
        c.tick();
        assert_eq!(c.steps, 1);
        assert!(c.enter(b));
        assert_eq!((c.now(), c.was(), c.steps), (b, Some(a), 0));
    }
}
