//! Words and the lexicon that spells them.

use std::collections::HashMap;

/// A thing in the world, by its place in the lexicon.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Word(pub(crate) u32);

impl Word {
    pub fn id(self) -> u32 {
        self.0
    }

    pub fn from_id(id: u32) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "w{}", self.0)
    }
}

/// Spellings to words and back. A spelling interned twice is one word, and
/// ids are dense in the order spellings were first seen.
#[derive(Default, Clone, Debug)]
pub struct Lexicon {
    ids: HashMap<String, Word>,
    words: Vec<String>,
}

impl Lexicon {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, spelling: &str) -> Word {
        if let Some(w) = self.ids.get(spelling) {
            return *w;
        }
        let w = Word(self.words.len() as u32);
        self.words.push(spelling.to_string());
        self.ids.insert(spelling.to_string(), w);
        w
    }

    pub fn get(&self, spelling: &str) -> Option<Word> {
        self.ids.get(spelling).copied()
    }

    pub fn spelling(&self, word: Word) -> Option<&str> {
        self.words.get(word.0 as usize).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

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
