//! `Tree`: one parent, many children, the hierarchy a graph does not give.
//!
//! A [`Graph`](crate::relation::Graph) relates anything to anything. A tree is
//! the stricter shape a body or a scene needs: every word has at most one
//! parent and none is its own ancestor, so "where does this part hang" (its
//! parents up to the root) and "what moves with it" (everything under it) each
//! have one answer. Apple keeps this as `parent` and `children`
//! (RF:15395-15414). From phinet's `node/tree.rs`, over [`Word`].
use crate::comp::space::{Pose, WorldMatrix};
use crate::word::Word;
use crate::world::World;

/// Parent and children per word, indexed by the word's id.
#[derive(Clone, Debug, Default)]
pub struct Tree {
    parent: Vec<Option<Word>>,
    children: Vec<Vec<Word>>,
}

impl Tree {
    pub fn new() -> Self {
        Self::default()
    }

    fn grow(&mut self, w: Word) {
        let i = w.id() as usize;
        if i >= self.parent.len() {
            self.parent.resize(i + 1, None);
            self.children.resize_with(i + 1, Vec::new);
        }
    }

    /// Hangs `child` under `parent`, taking it from any parent it had; its own
    /// children come with it. Refused (false, nothing changed) when `parent`
    /// is `child` or stands under it, which would close a loop.
    pub fn attach(&mut self, child: Word, parent: Word) -> bool {
        if child == parent || self.is_ancestor(child, parent) {
            return false;
        }
        self.detach(child);
        self.grow(child);
        self.grow(parent);
        self.parent[child.id() as usize] = Some(parent);
        self.children[parent.id() as usize].push(child);
        true
    }

    /// Takes `w` from its parent, returning the parent it had; its own
    /// children stay with it.
    pub fn detach(&mut self, w: Word) -> Option<Word> {
        let p = self.parent.get_mut(w.id() as usize)?.take()?;
        self.children[p.id() as usize].retain(|c| *c != w);
        Some(p)
    }

    /// Removes `w` from the tree: it leaves its parent, and its children are
    /// left without one.
    pub fn forget(&mut self, w: Word) {
        self.detach(w);
        for c in self.children(w).to_vec() {
            self.detach(c);
        }
    }

    pub fn parent(&self, w: Word) -> Option<Word> {
        self.parent.get(w.id() as usize).copied().flatten()
    }

    /// `w`'s children, in the order they were attached.
    pub fn children(&self, w: Word) -> &[Word] {
        self.children.get(w.id() as usize).map_or(&[], Vec::as_slice)
    }

    /// `w`'s parents, nearest first, up to its root.
    pub fn ancestors(&self, w: Word) -> impl Iterator<Item = Word> + '_ {
        std::iter::successors(self.parent(w), move |p| self.parent(*p))
    }

    /// Whether `a` stands above `b`.
    pub fn is_ancestor(&self, a: Word, b: Word) -> bool {
        self.ancestors(b).any(|p| p == a)
    }

    /// Everything under `w`, depth first in attach order; `w` is not included.
    pub fn descendants(&self, w: Word) -> Vec<Word> {
        let mut out = Vec::new();
        let mut stack: Vec<Word> = self.children(w).iter().rev().copied().collect();
        while let Some(n) = stack.pop() {
            out.push(n);
            stack.extend(self.children(n).iter().rev().copied());
        }
        out
    }

    /// The top of each tree: words with children and no parent.
    pub fn roots(&self) -> Vec<Word> {
        (0..self.children.len())
            .filter(|&i| self.parent[i].is_none() && !self.children[i].is_empty())
            .map(|i| Word::from_id(i as u32))
            .collect()
    }
}

/// A word was moved under another, or out from under one (Apple's
/// `DidReparentEntity`, RF:7110).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DidReparent {
    pub child: Word,
    pub old: Option<Word>,
    pub new: Option<Word>,
}

impl World {
    /// Where `word` is in the world, worked out now from the [`Pose`] of
    /// every word from its root down (a word with no `Pose` is the identity).
    /// [`WorldMatrix`] is what `propagate` stored last step; this is fresh.
    pub fn world_of(&self, word: Word) -> WorldMatrix {
        let local = |w: Word| self.read::<Pose>(w).map_or(WorldMatrix::IDENTITY, Pose::matrix);
        let up: Vec<Word> = std::iter::once(word).chain(self.tree.ancestors(word)).collect();
        up.iter().rev().fold(WorldMatrix::IDENTITY, |m, w| m.mul(&local(*w)))
    }

    /// Hangs `child` under `parent`; false, nothing changed, when that would
    /// close a loop. With `keep_world` the child stays where it is: its
    /// `Pose` becomes `inverse(new parent world) x old world`. Without, its
    /// `Pose` is unchanged and it jumps with its new parent. Apple's
    /// `setParent(_:preservingWorldTransform:)` (RF:15395). Emits
    /// [`DidReparent`].
    pub fn set_parent(&mut self, child: Word, parent: Word, keep_world: bool) -> bool {
        let before = self.world_of(child);
        let old = self.tree.parent(child);
        if !self.tree.attach(child, parent) {
            return false;
        }
        if keep_world {
            self.keep_world(child, parent, before);
        }
        self.emit(DidReparent { child, old, new: Some(parent) });
        true
    }

    /// Takes `child` out from under its parent, true if it had one. With
    /// `keep_world` its `Pose` becomes its old world matrix.
    pub fn clear_parent(&mut self, child: Word, keep_world: bool) -> bool {
        let before = self.world_of(child);
        let Some(old) = self.tree.detach(child) else { return false };
        if keep_world {
            self.set_pose(child, before);
        }
        self.emit(DidReparent { child, old: Some(old), new: None });
        true
    }

    /// A pin: a word named `name` under `parent`, its `offset` the place on
    /// the parent where a part can be hung (GeometricPin, RF:9088). A part
    /// that is a child of the pin cannot drift from it: its world matrix is
    /// derived, there is no second one to disagree.
    pub fn pin(&mut self, parent: Word, name: &str, offset: Pose) -> Word {
        let spelling = format!("{}.{name}", self.spelling(parent));
        let pin = self.coin_bare(&spelling);
        self.bind(pin, offset);
        self.tree.attach(pin, parent);
        pin
    }

    fn keep_world(&mut self, child: Word, parent: Word, before: WorldMatrix) {
        if let Some(inverse) = self.world_of(parent).inverse() {
            self.set_pose(child, inverse.mul(&before));
        }
    }

    fn set_pose(&mut self, word: Word, local: WorldMatrix) {
        self.bind(word, local.pose());
    }
}

#[cfg(test)]
mod tests;
