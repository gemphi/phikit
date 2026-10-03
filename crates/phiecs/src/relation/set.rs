#[allow(unused_imports)]
use super::*;

/// The kinds a world knows before it has read anything — the relations a
/// dictionary marks in its own prose, which is where phiano's marker channels
/// come from. Everything past these is discovered from a corpus.
pub const KIND_NAMES: &[&str] = &[
    "is-a",
    "part-of",
    "has",
    "does",
    "done-to",
    "like",
    "unlike",
    "near",
    "before",
    "after",
    "causes",
    "used-for",
    "made-of",
    "belongs-to",
    "more-than",
    "less-than",
];

/// How many of [`KIND_NAMES`] a fresh world declares.
pub const DEFAULT_KINDS: usize = KIND_NAMES.len();
