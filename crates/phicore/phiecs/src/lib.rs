//! A world of words: each thing is a [`Word`], what links two things is a
//! [`Relation`], data on a thing is an [`Aspect`], data on the whole world is
//! a resource, and logic is a [`System`] run in [`Stage`] order by
//! [`World::step`].
//!
//! ```
//! use phiecs::prelude::*;
//!
//! let mut world = World::new();
//! let cat = world.coin_bare("cat");
//! let mat = world.coin_bare("mat");
//! let on = world.declare("on").unwrap();
//! world.relate(cat, on, mat);
//! assert_eq!(world.calls_for(cat, on).first().map(|(w, _)| *w), Some(mat));
//! ```

pub mod comp;
pub mod entity;
pub mod system;

// Today's paths stay: `crate::word`, `crate::aspect` and the rest.
pub use comp::{aspect, resources};
pub use entity::{relation, word, world};

pub mod prelude {
    pub use crate::aspect::Aspect;
    pub use crate::comp::time::Clock;
    pub use crate::relation::{Edge, Relation, Register};
    pub use crate::resources::Resources;
    pub use crate::system::{DidAdd, EventQueue, FnSystem, Stage, System, WillRemove};
    pub use crate::word::{Lexicon, Word};
    pub use crate::world::{World, WordBuilder};
}
