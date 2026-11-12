//! Spawning: a new word of a kind, named for people, switched on or off.
//!
//! Identity in phiecs is the spelling, so two things called "cube" would be
//! one. Apple's entities have an `id` and a `name` apart (RF:10610, RF:10560):
//! `spawn` coins a unique spelling `kind#n` and keeps the display name as the
//! [`Name`] aspect. [`Enabled`] is `isEnabled` (RF:10569-10581).
use crate::aspect::Aspect;
use crate::comp::stored::{Fields, Stored, Value, field};
use crate::entity::kind::{Kind, KindOf};
use crate::word::Word;
use crate::world::World;
/// What a word is called to people. Many words may share one.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Name(pub String);
impl Aspect for Name {
    fn name() -> &'static str {
        "name"
    }
}
/// Whether a word is switched on. No `Enabled` means on. Off hides the word
/// and its subtree without removing them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Enabled(pub bool);
impl Aspect for Enabled {
    fn name() -> &'static str {
        "enabled"
    }
}
impl Stored for Name {
    const KIND: &'static str = "name";
    fn fields(&self) -> Fields {
        vec![("text".into(), Value::Text(self.0.clone()))]
    }
    fn from_fields(f: &Fields) -> Option<Self> {
        Some(Self(field(f, "text")?.text()?.to_string()))
    }
}
impl Stored for Enabled {
    const KIND: &'static str = "enabled";
    fn fields(&self) -> Fields {
        vec![("on".into(), Value::Num(f64::from(u8::from(self.0))))]
    }
    fn from_fields(f: &Fields) -> Option<Self> {
        Some(Self(field(f, "on")?.num()? != 0.0))
    }
}
/// How many words each kind has spawned, a resource.
#[derive(Default)]
struct Spawned(Vec<u64>);
impl World {
    /// A new word of `kind`, spelled `kind#n` with the next free `n`, named
    /// by its spelling.
    pub fn spawn(&mut self, kind: Kind) -> Word {
        self.spawn_as(kind, None)
    }
    /// A new word of `kind` that people call `display`.
    pub fn spawn_named(&mut self, kind: Kind, display: &str) -> Word {
        self.spawn_as(kind, Some(display))
    }
    fn spawn_as(&mut self, kind: Kind, display: Option<&str>) -> Word {
        let label = self.kind_name(kind).unwrap_or("thing").to_string();
        let spelling = loop {
            let n = self.next_spawn(kind);
            let s = format!("{label}#{n}");
            if self.lexicon.get(&s).is_none() {
                break s;
            }
        };
        let word = self.coin_bare(&spelling);
        self.bind(word, KindOf(kind));
        self.bind(word, Name(display.unwrap_or(&spelling).to_string()));
        word
    }
    fn next_spawn(&mut self, kind: Kind) -> u64 {
        if self.resources.get::<Spawned>().is_none() {
            self.resources.insert(Spawned::default());
        }
        let counts = &mut self.resources.get_mut::<Spawned>().expect("inserted above").0;
        if counts.len() <= kind.index() {
            counts.resize(kind.index() + 1, 0);
        }
        counts[kind.index()] += 1;
        counts[kind.index()]
    }
    /// What people call `word`: its [`Name`], else its spelling.
    pub fn display_name(&self, word: Word) -> &str {
        self.read::<Name>(word).map(|n| n.0.as_str()).unwrap_or_else(|| self.spelling(word))
    }
    pub fn set_enabled(&mut self, word: Word, on: bool) {
        self.bind(word, Enabled(on));
    }
    /// `word`'s own switch (on if it has none).
    pub fn is_enabled(&self, word: Word) -> bool {
        self.read::<Enabled>(word).is_none_or(|e| e.0)
    }
    /// On, and every parent above it on (Apple's `isEnabledInHierarchy`).
    pub fn is_enabled_in_tree(&self, word: Word) -> bool {
        self.is_enabled(word) && self.tree.ancestors(word).all(|a| self.is_enabled(a))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::event::{DidAdd, WillRemove};
    #[test]
    fn two_things_called_cube_are_two_words() {
        let mut w = World::new();
        let k = w.declare_kinds(&["mesh", "lamp"]);
        let a = w.spawn_named(k[0], "cube");
        let b = w.spawn_named(k[0], "cube");
        let l = w.spawn(k[1]);
        assert_ne!(a, b);
        assert_eq!((w.spelling(a), w.spelling(b), w.spelling(l)), ("mesh#1", "mesh#2", "lamp#1"));
        assert_eq!((w.display_name(a), w.display_name(b), w.display_name(l)), ("cube", "cube", "lamp#1"));
        assert_eq!(w.kind_of(b), Some(k[0]));
        assert_eq!(w.aspect_list(a), ["kind", "name"]);
        assert_eq!(w.events::<DidAdd>().count(), 3);
    }
    #[test]
    fn a_spelling_taken_by_hand_is_skipped() {
        let mut w = World::new();
        let k = w.declare_kinds(&["mesh"])[0];
        w.coin_bare("mesh#1");
        let b = w.spawn(k);
        assert_eq!(w.spelling(b), "mesh#2");
    }
    #[test]
    fn forget_announces_and_the_id_is_not_reused() {
        let mut w = World::new();
        let k = w.declare_kinds(&["mesh"])[0];
        let a = w.spawn(k);
        assert!(w.forget(a));
        assert_eq!(w.events::<WillRemove>().map(|e| e.0).collect::<Vec<_>>(), [a]);
        assert!(w.read::<Name>(a).is_none(), "its aspects went");
        let b = w.spawn(k);
        assert_ne!(a, b);
        assert_eq!(w.spelling(b), "mesh#2");
    }
    #[test]
    fn off_hides_the_subtree() {
        let mut w = World::new();
        let k = w.declare_kinds(&["mesh"])[0];
        let (root, child) = (w.spawn(k), w.spawn(k));
        w.set_parent(child, root, false);
        assert!(w.is_enabled_in_tree(child));
        w.set_enabled(root, false);
        assert!(w.is_enabled(child) && !w.is_enabled_in_tree(child));
    }
}
