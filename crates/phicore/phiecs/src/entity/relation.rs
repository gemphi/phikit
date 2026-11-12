//! Kinds of link between words, and the weighted edges that carry them.

use crate::word::Word;
use std::collections::HashMap;

/// A kind of link, by its place in the [`Register`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Relation(pub(crate) u32);

impl Relation {
    pub fn channel(self) -> usize {
        self.0 as usize
    }
}

/// The declared kinds, by name.
#[derive(Default, Clone, Debug)]
pub struct Register {
    names: Vec<String>,
    index: HashMap<String, u32>,
}

impl Register {
    pub fn new() -> Self {
        Self::default()
    }

    /// The kind called `name`, declared now if it is new.
    pub fn declare(&mut self, name: &str) -> Option<Relation> {
        if let Some(i) = self.index.get(name) {
            return Some(Relation(*i));
        }
        let i = u32::try_from(self.names.len()).ok()?;
        self.index.insert(name.to_string(), i);
        self.names.push(name.to_string());
        Some(Relation(i))
    }

    pub fn get(&self, name: &str) -> Option<Relation> {
        self.index.get(name).map(|i| Relation(*i))
    }

    pub fn name(&self, kind: Relation) -> Option<&str> {
        self.names.get(kind.channel()).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Relation, &str)> {
        self.names.iter().enumerate().map(|(i, n)| (Relation(i as u32), n.as_str()))
    }
}

/// `from` is `kind` to `to`, as strongly as `weight`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Edge {
    pub from: Word,
    pub kind: Relation,
    pub to: Word,
    pub weight: f64,
}

/// Every edge, kept both ways round. Relating the same pair again adds to
/// the edge's weight rather than adding a second edge.
#[derive(Default, Clone, Debug)]
pub struct Graph {
    out: HashMap<Word, Vec<Edge>>,
    into: HashMap<Word, Vec<Edge>>,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn out_of(&self, word: Word) -> &[Edge] {
        self.out.get(&word).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn into_of(&self, word: Word) -> &[Edge] {
        self.into.get(&word).map(Vec::as_slice).unwrap_or(&[])
    }

    /// What `word` is `kind` to, heaviest first; ties keep the order they
    /// were related in.
    pub fn calls_for(&self, word: Word, kind: Relation) -> Vec<(Word, f64)> {
        let mut v: Vec<(Word, f64)> =
            self.out_of(word).iter().filter(|e| e.kind == kind).map(|e| (e.to, e.weight)).collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        v
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_are_stable_and_edges_add_up() {
        let mut r = Register::new();
        let eats = r.declare("eats").unwrap();
        assert_eq!(Some(eats), r.declare("eats"));
        assert_eq!(r.name(eats), Some("eats"));

        let mut g = Graph::new();
        let [a, b, c] = [0, 1, 2].map(Word);
        g.relate(a, eats, b, 1.0);
        g.relate(a, eats, c, 1.0);
        g.relate(a, eats, c, 1.0);
        assert_eq!(g.calls_for(a, eats), vec![(c, 2.0), (b, 1.0)]);
        assert!(g.unrelate(a, eats, c));
        assert_eq!(g.into_of(c), &[]);
        g.forget(b);
        assert!(g.is_empty());
    }
}
