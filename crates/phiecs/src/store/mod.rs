//! Where aspects live.
//!
//! Apple stores components as `Vec<(Entity, T)>` and scans it linearly for
//! every get, and every insert first `retain`s the vector to drop a previous
//! value — O(n) per insert, O(n) per read (`reality-foundation/component.rs:44`).
//! Words are dense ids from a [`crate::word::Lexicon`], so here the id *is*
//! the slot: [`Dense`] is a `Vec<Option<A>>` and every operation is O(1).

mod dense;
pub use dense::Dense;

use std::any::Any;

/// The type-erased face of a store, so `World` can keep one per aspect type.
pub(crate) trait AspectStore: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// Drop whatever this store holds for `word`; `true` when it held some.
    fn forget(&mut self, word: crate::word::Word) -> bool;
}
