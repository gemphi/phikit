//! Asking the world what it holds.
//!
//! Apple's `Query::iter` builds a `Vec<(Entity, &C)>` and hands it back
//! (`reality-foundation/query.rs:23`), so every query allocates and none can
//! be composed. These are real iterators, and there is one Apple has no
//! equivalent for: [`Related`], the join along an edge kind.

mod pair;
pub use pair::{Related, With2};

use crate::aspect::Aspect;
use crate::word::Word;
use crate::world::World;

impl World {
    /// Every word carrying `A`.
    pub fn with<A: Aspect>(&self) -> impl Iterator<Item = (Word, &A)> {
        self.store::<A>().into_iter().flat_map(|s| s.iter())
    }

    /// The same, mutably.
    pub fn with_mut<A: Aspect>(&mut self) -> impl Iterator<Item = (Word, &mut A)> {
        self.store_mut::<A>().into_iter().flat_map(|s| s.iter_mut())
    }

    /// How many words carry `A`.
    pub fn count_with<A: Aspect>(&self) -> usize {
        self.store::<A>().map_or(0, |s| s.len())
    }
}
