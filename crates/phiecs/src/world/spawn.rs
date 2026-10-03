#[allow(unused_imports)]
use super::*;

impl World {
    /// Coin a word (RealityKit: `spawn`), interning its spelling. Coining the
    /// same spelling twice gives the same word — a lexicon has one of each.
    pub fn coin(&mut self, spelling: &str) -> WordBuilder<'_> {
        let word = self.coin_bare(spelling);
        WordBuilder { world: self, word }
    }

    /// The word alone, with no aspects bound.
    pub fn coin_bare(&mut self, spelling: &str) -> Word {
        let word = self.lexicon.intern(spelling);
        if !self.words.contains(&word) {
            self.words.push(word);
        }
        word
    }

    /// Forget a word: drop its aspects, drop every edge that touched it, and
    /// take it off the roll. Its spelling stays interned, so any id already
    /// handed out still reads back — a word that was said cannot be unsaid.
    pub fn forget(&mut self, word: Word) -> bool {
        let known = self.words.iter().position(|w| *w == word);
        for store in self.stores.values_mut() {
            store.forget(word);
        }
        self.edges.forget(word);
        match known {
            Some(i) => {
                self.words.remove(i);
                true
            }
            None => false,
        }
    }
}

/// Coin a word and bind its aspects in one expression (Apple's
/// `EntityBuilder`, `reality-foundation/world.rs:13`).
pub struct WordBuilder<'a> {
    world: &'a mut World,
    word: Word,
}

impl WordBuilder<'_> {
    pub fn bind<A: Aspect>(self, aspect: A) -> Self {
        let word = self.word;
        let world = self.world;
        world.bind(word, aspect);
        Self { world, word }
    }

    /// The word itself, ending the chain.
    pub fn word(self) -> Word {
        self.word
    }
}
