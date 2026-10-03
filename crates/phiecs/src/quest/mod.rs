//! The quest: learning a language by asking, one word at a time.
//!
//! A quest starts at one word and asks an [`Oracle`] — Phi-4, or anything that
//! answers text with text — what that word relates to, under which conditions.
//! Every answer becomes edges in the [`World`] and puts its target words on
//! the frontier, so the quest walks outward from where it started and the
//! graph it builds is the transcript of what it asked. Nothing is taken on
//! trust: an answer is evidence, counted, and a word the oracle mentions twice
//! under the same kind moves twice.
//!
//! The point is not to copy the oracle. It is to end with a *smaller* thing
//! than the oracle: 64 relation channels and a lexicon, which can be read,
//! measured and argued with — concepts, not weights.

mod ask;
pub mod builders;
mod oracle;
mod per_kind;
mod parse;
#[cfg(test)]
mod parse_tests;
mod reparse;
mod run;
mod step;
mod synthetic;
mod template;
mod tsv;

pub use oracle::{Oracle, OracleError, Scripted, Spawned};
pub use step::{Answer, Step};
pub use synthetic::{SYNTHETIC_KINDS, Synthetic};
pub use tsv::{Row, parse_tsv};
pub use parse::{Claim, offered_kind, parse_relations};
pub use reparse::{Page, reparse_dir};
pub use template::{CONDITIONS, ONE_KIND, RELATIONS, Template, VOICE};

use crate::condition::Conditions;
use crate::world::World;
use std::collections::VecDeque;

/// How far from the seed a quest will walk before it stops.
pub const DEFAULT_DEPTH: u32 = 3;
/// How many questions one run may ask.
pub const DEFAULT_BUDGET: u32 = 64;
/// Consecutive oracle failures (the program ran and exited with an error)
/// after which a run stops.
pub const MAX_FAILURES_IN_A_ROW: u32 = 5;

/// The kinds a quest offers: those a new [`World`] declares.
pub fn base_kinds() -> Vec<String> {
    World::new().relations.iter().map(|(_, n)| n.to_string()).collect()
}

pub struct Quest {
    pub world: World,
    pub conditions: Conditions,
    /// Words still to ask about, with their distance from the seed.
    frontier: VecDeque<(String, u32)>,
    /// Spellings already asked, so a cycle in the language is not a loop here.
    asked: Vec<String>,
    pub depth: u32,
    pub budget: u32,
    pub steps: Vec<Step>,
    /// The kinds every prompt offers: the world's own at the start.
    pub offered: Vec<String>,
    /// Why the last run ended before its budget, if it did.
    pub stopped: Option<String>,
    /// One-kind-at-a-time mode: the kinds each word is asked, one question
    /// each ([`Quest::one_kind_at_a_time`]).
    pub per_kind: Option<Vec<String>>,
    kind_frontier: VecDeque<(String, String, u32)>,
    queued: std::collections::HashSet<String>,
    /// A persona's voice: with one, every word is also asked for sentences
    /// in it ([`Quest::with_voice`]), kept in `sentences` as `(word, line)`.
    pub voice: Option<String>,
    pub sentences: Vec<(String, String)>,
}

impl Quest {
    /// A quest that begins at `seed`.
    pub fn new(seed: &str) -> Self {
        let mut world = World::new();
        let conditions = Conditions::intern(&mut world.lexicon);
        world.coin_bare(seed);
        let offered = base_kinds();
        Self {
            offered,
            world,
            conditions,
            frontier: VecDeque::from(vec![(seed.to_string(), 0)]),
            asked: Vec::new(),
            depth: DEFAULT_DEPTH,
            budget: DEFAULT_BUDGET,
            steps: Vec::new(),
            stopped: None,
            per_kind: None,
            kind_frontier: VecDeque::new(),
            queued: Default::default(),
            voice: None,
            sentences: Vec::new(),
        }
    }

    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_budget(mut self, budget: u32) -> Self {
        self.budget = budget;
        self
    }

    /// The next word to ask about, if the quest has not run out of room.
    pub(crate) fn next_word(&mut self) -> Option<(String, u32)> {
        while let Some((w, d)) = self.frontier.pop_front() {
            if d > self.depth || self.asked.iter().any(|a| a == &w) {
                continue;
            }
            self.asked.push(w.clone());
            return Some((w, d));
        }
        None
    }

    /// Put a word the oracle mentioned on the frontier.
    pub(crate) fn push(&mut self, spelling: &str, depth: u32) {
        if depth <= self.depth && !self.asked.iter().any(|a| a == spelling) {
            self.frontier.push_back((spelling.to_string(), depth));
        }
    }

    /// How many words are waiting.
    pub fn pending(&self) -> usize {
        self.frontier.len()
    }

    /// Everything asked so far.
    pub fn asked(&self) -> &[String] {
        &self.asked
    }
}
