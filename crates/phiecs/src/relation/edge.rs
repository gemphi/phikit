#[allow(unused_imports)]
use super::*;

use crate::word::Word;
use std::collections::HashMap;

/// One relation, directed: `from` stands to `to` under `kind`, with the
/// weight the evidence gives it.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Edge {
    pub from: Word,
    pub kind: Relation,
    pub to: Word,
    pub weight: f64,
}

/// Every edge, indexed both ways so "what does this word relate to" and "what
/// relates to this word" are both one lookup. This replaces Apple's
/// `Parent`/`Children` pair, which can hold only one kind and one parent.
#[derive(Default, Clone, Debug)]
pub struct Edges {
    out: HashMap<Word, Vec<Edge>>,
    into: HashMap<Word, Vec<Edge>>,
}

impl Edges {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add `from -kind-> to`, or add `weight` to it if it is already there.
    pub fn relate(&mut self, from: Word, kind: Relation, to: Word, weight: f64) {
        let e = Edge { from, kind, to, weight };
        Self::push(self.out.entry(from).or_default(), e);
        Self::push(self.into.entry(to).or_default(), e);
    }

    fn push(list: &mut Vec<Edge>, e: Edge) {
        match list.iter_mut().find(|x| x.kind == e.kind && x.to == e.to && x.from == e.from) {
            Some(x) => x.weight += e.weight,
            None => list.push(e),
        }
    }

    /// Remove `from -kind-> to`; `true` when it was there.
    pub fn unrelate(&mut self, from: Word, kind: Relation, to: Word) -> bool {
        let a = Self::drop_from(self.out.get_mut(&from), kind, |e| e.to == to);
        let b = Self::drop_from(self.into.get_mut(&to), kind, |e| e.from == from);
        a || b
    }

    fn drop_from(list: Option<&mut Vec<Edge>>, kind: Relation, m: impl Fn(&Edge) -> bool) -> bool {
        match list {
            Some(v) => {
                let before = v.len();
                v.retain(|e| !(e.kind == kind && m(e)));
                v.len() != before
            }
            None => false,
        }
    }

    /// Everything `word` relates to.
    pub fn out_of(&self, word: Word) -> &[Edge] {
        self.out.get(&word).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Everything that relates to `word`.
    pub fn into_of(&self, word: Word) -> &[Edge] {
        self.into.get(&word).map(Vec::as_slice).unwrap_or(&[])
    }

    /// What `word` calls for under `kind`, heaviest first — the X ⇒ Y read.
    pub fn calls_for(&self, word: Word, kind: Relation) -> Vec<(Word, f64)> {
        let mut v: Vec<(Word, f64)> =
            self.out_of(word).iter().filter(|e| e.kind == kind).map(|e| (e.to, e.weight)).collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        v
    }

    /// Drop every edge touching `word` (a word being forgotten).
    pub fn forget(&mut self, word: Word) {
        for e in self.out.remove(&word).unwrap_or_default() {
            if let Some(v) = self.into.get_mut(&e.to) {
                v.retain(|x| x.from != word);
            }
        }
        for e in self.into.remove(&word).unwrap_or_default() {
            if let Some(v) = self.out.get_mut(&e.from) {
                v.retain(|x| x.to != word);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.out.values().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
