//! `Relation` — the kind of an edge, and the thing Apple's model has no name
//! for.
//!
//! RealityKit's hierarchy is `Parent` / `Children`: one edge kind, one parent
//! (`reality-foundation/components.rs:113`). A language is a multigraph — a
//! word stands in many relations to many words at once, and the *kind* is the
//! payload. In Phient's terms the kind is the condition: `w` means nothing
//! except what it relates to, under which `c`.

mod edge;
mod set;
pub use edge::{Edge, Edges};
pub use set::{DEFAULT_KINDS, KIND_NAMES};

/// How many relation kinds one language is assumed to hold. Phiano measures
/// this bound rather than asserting it (docs/types); 64 is the channel count
/// every phasor already carries.
pub const MAX_KINDS: usize = 64;

/// One relation kind, opaque: it is issued by a [`Relations`] table, because
/// what a kind *means* lives in the table's evidence, not in the number.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Relation(pub(crate) u8);

impl Relation {
    /// Its channel on the relation circle.
    pub fn channel(self) -> usize {
        self.0 as usize
    }
}

/// The kinds this world knows, with the evidence each was learned from.
#[derive(Default, Clone, Debug)]
pub struct Relations {
    names: Vec<String>,
    /// How many pairs were seen under each kind.
    evidence: Vec<u32>,
}

impl Relations {
    pub fn new() -> Self {
        Self::default()
    }

    /// The table every world starts with: the named kinds of [`KIND_NAMES`].
    pub fn named() -> Self {
        let mut r = Self::new();
        for n in KIND_NAMES {
            r.declare(n);
        }
        r
    }

    /// Issue (or find) the kind called `name`. `None` once the language has
    /// used up its [`MAX_KINDS`] channels — a bound worth hitting loudly.
    pub fn declare(&mut self, name: &str) -> Option<Relation> {
        if let Some(i) = self.names.iter().position(|n| n == name) {
            return Some(Relation(i as u8));
        }
        if self.names.len() >= MAX_KINDS {
            return None;
        }
        self.names.push(name.to_string());
        self.evidence.push(0);
        Some(Relation(self.names.len() as u8 - 1))
    }

    pub fn get(&self, name: &str) -> Option<Relation> {
        self.names.iter().position(|n| n == name).map(|i| Relation(i as u8))
    }

    pub fn name(&self, kind: Relation) -> Option<&str> {
        self.names.get(kind.channel()).map(String::as_str)
    }

    /// Record that one more pair was seen under `kind`.
    pub fn observe(&mut self, kind: Relation) {
        if let Some(e) = self.evidence.get_mut(kind.channel()) {
            *e += 1;
        }
    }

    pub fn evidence(&self, kind: Relation) -> u32 {
        self.evidence.get(kind.channel()).copied().unwrap_or(0)
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Relation, &str)> {
        self.names.iter().enumerate().map(|(i, n)| (Relation(i as u8), n.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_are_bounded_and_stable() {
        let mut r = Relations::named();
        let a = r.declare("eats").unwrap();
        assert_eq!(Some(a), r.declare("eats"));
        assert_eq!(r.name(a), Some("eats"));
        r.observe(a);
        assert_eq!(r.evidence(a), 1);
        while r.len() < MAX_KINDS {
            r.declare(&format!("k{}", r.len())).unwrap();
        }
        assert!(r.declare("one too many").is_none());
    }
}
