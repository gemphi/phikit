//! `Word` — the entity of this ECS.
//!
//! | RealityKit | phiecs |
//! |---|---|
//! | `Entity` | `Word` |
//! | `EntityGenerator` | [`Lexicon`] |
//!
//! Apple's `Entity` carries a `generation` alongside its id so a recycled id
//! can be told from a live one (`reality-foundation/entity.rs`). Nothing here
//! recycles: a word, once interned, keeps its id for the life of the lexicon,
//! because the id *is* the word's identity in every table that stores it. The
//! generation field is therefore deliberately absent — do not add it back.

mod set;
pub use set::Lexicon;

/// One word of a language: an opaque id issued by a [`Lexicon`].
///
/// Opaque by construction, like `RelationType` in phiano: you cannot mint one,
/// because a word means nothing outside the lexicon that interned it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Word(pub(crate) u32);

impl Word {
    /// Its index in the issuing lexicon — also its slot in a dense store.
    pub fn id(self) -> u32 {
        self.0
    }

    /// Rebuild a word from an id the same lexicon issued. Wrong ids are not
    /// unsafe, only meaningless: every lookup will miss.
    pub fn from_id(id: u32) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "w{}", self.0)
    }
}
