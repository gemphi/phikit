#[allow(unused_imports)]
use super::*;

use crate::word::Lexicon;

/// The conditions a quest runs under before it has learned any of its own.
/// They are ordinary words: a condition is a word the world knows, so that
/// "under *question*" and "under *answer*" can themselves be related.
pub const CONDITION_NAMES: &[&str] = &["asking", "answered", "grounded", "contradicted", "idle"];

/// The starting conditions, interned.
#[derive(Clone, Copy, Debug)]
pub struct Conditions {
    pub asking: Word,
    pub answered: Word,
    pub grounded: Word,
    pub contradicted: Word,
    pub idle: Word,
}

impl Conditions {
    /// Intern [`CONDITION_NAMES`] into `lexicon`.
    pub fn intern(lexicon: &mut Lexicon) -> Self {
        Self {
            asking: lexicon.intern(CONDITION_NAMES[0]),
            answered: lexicon.intern(CONDITION_NAMES[1]),
            grounded: lexicon.intern(CONDITION_NAMES[2]),
            contradicted: lexicon.intern(CONDITION_NAMES[3]),
            idle: lexicon.intern(CONDITION_NAMES[4]),
        }
    }
}
