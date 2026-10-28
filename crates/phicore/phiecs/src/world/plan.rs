//! Chains of one kind of link from one word to another.

use super::World;
use crate::relation::Relation;
use crate::word::Word;

impl World {
    /// The heaviest chain of `kind` links from `from` to `to`, at most
    /// `depth` links long; its weight is the product of its links'.
    pub fn plan(&self, from: Word, kind: Relation, to: Word, depth: usize) -> Option<Vec<Word>> {
        self.plan_avoiding(from, kind, to, depth, &[])
    }

    /// [`plan`](Self::plan), never passing through a word in `avoid`.
    pub fn plan_avoiding(&self, from: Word, kind: Relation, to: Word, depth: usize, avoid: &[Word]) -> Option<Vec<Word>> {
        let mut frontier: Vec<(f64, Vec<Word>)> = vec![(1.0, vec![from])];
        let mut best: Option<(f64, Vec<Word>)> = None;
        while let Some((score, path)) = frontier.pop() {
            let last = *path.last()?;
            if last == to && path.len() > 1 {
                if best.as_ref().is_none_or(|(s, _)| score > *s) {
                    best = Some((score, path));
                }
                continue;
            }
            if path.len() > depth {
                continue;
            }
            for (next, w) in self.calls_for(last, kind) {
                if (path.contains(&next) && next != to) || avoid.contains(&next) {
                    continue;
                }
                let mut p = path.clone();
                p.push(next);
                frontier.push((score * w, p));
            }
            frontier.sort_by(|a, b| a.0.total_cmp(&b.0));
        }
        best.map(|(_, p)| p)
    }

    /// Every word `from` reaches by `kind` within `depth` links, with the
    /// best chain weight to it, heaviest first.
    pub fn reach(&self, from: Word, kind: Relation, depth: usize) -> Vec<(Word, f64)> {
        let mut seen: Vec<(Word, f64)> = Vec::new();
        let mut layer: Vec<(Word, f64)> = vec![(from, 1.0)];
        for _ in 0..depth {
            let mut next: Vec<(Word, f64)> = Vec::new();
            for (w, s) in &layer {
                for (n, weight) in self.calls_for(*w, kind) {
                    if n == from {
                        continue;
                    }
                    let score = s * weight;
                    match seen.iter_mut().find(|(x, _)| *x == n) {
                        Some(e) => e.1 = e.1.max(score),
                        None => {
                            seen.push((n, score));
                            next.push((n, score));
                        }
                    }
                }
            }
            layer = next;
        }
        seen.sort_by(|a, b| b.1.total_cmp(&a.1));
        seen
    }
}
