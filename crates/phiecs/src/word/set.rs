//! The lexicon: the names, and the ids they are known by.
//!
//! `mod.rs` holds the type; this file holds its content, as everywhere else in
//! this workspace. Apple's `EntityGenerator` is one `AtomicU64` that counts up
//! (`reality-foundation/entity.rs:39`); a language needs the inverse map too —
//! id → spelling — because every report a person reads is in words.

use super::Word;
use std::collections::HashMap;

/// Interns spellings and issues [`Word`]s. Ids are dense from 0, so they
/// index a [`crate::store::Dense`] directly.
#[derive(Default, Clone, Debug)]
pub struct Lexicon {
    ids: HashMap<String, Word>,
    words: Vec<String>,
}

impl Lexicon {
    pub fn new() -> Self {
        Self::default()
    }

    /// The word for `spelling`, interning it if this is its first sighting.
    pub fn intern(&mut self, spelling: &str) -> Word {
        if let Some(w) = self.ids.get(spelling) {
            return *w;
        }
        let w = Word(self.words.len() as u32);
        self.words.push(spelling.to_string());
        self.ids.insert(spelling.to_string(), w);
        w
    }

    /// The word for `spelling`, only if it has been seen.
    pub fn get(&self, spelling: &str) -> Option<Word> {
        self.ids.get(spelling).copied()
    }

    /// Its spelling, or `None` for a word this lexicon did not issue.
    pub fn spelling(&self, word: Word) -> Option<&str> {
        self.words.get(word.0 as usize).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Every word, in id order.
    pub fn iter(&self) -> impl Iterator<Item = (Word, &str)> {
        self.words.iter().enumerate().map(|(i, s)| (Word(i as u32), s.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interning_is_stable_and_dense() {
        let mut lex = Lexicon::new();
        let a = lex.intern("morning");
        let b = lex.intern("regard");
        assert_eq!(a, lex.intern("morning"));
        assert_eq!((a.id(), b.id()), (0, 1));
        assert_eq!(lex.spelling(b), Some("regard"));
        assert_eq!(lex.get("absent"), None);
    }
}
