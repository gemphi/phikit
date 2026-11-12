//! Kinds: the type of an entity, declared once from a list the app passes in.
//!
//! phiecs reads no files, so the list (one kind a line, the lexfile format)
//! comes from the app. The same names are the database's `kinds` rows
//! (docs/connect.md). A word has at most one kind.
use crate::aspect::Aspect;
use crate::word::Word;
use crate::world::World;
/// A kind, by its place in the [`KindList`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Kind(u16);
impl Kind {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}
/// The declared kinds, a resource. Declaring a name twice is the same kind.
#[derive(Default, Clone, Debug)]
pub struct KindList {
    names: Vec<String>,
}
impl KindList {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn declare(&mut self, name: &str) -> Kind {
        if let Some(k) = self.get(name) {
            return k;
        }
        self.names.push(name.to_string());
        Kind((self.names.len() - 1) as u16)
    }
    pub fn get(&self, name: &str) -> Option<Kind> {
        self.names.iter().position(|n| n == name).map(|i| Kind(i as u16))
    }
    pub fn name(&self, kind: Kind) -> Option<&str> {
        self.names.get(kind.index()).map(String::as_str)
    }
    pub fn len(&self) -> usize {
        self.names.len()
    }
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}
/// The aspect that gives a word its [`Kind`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KindOf(pub Kind);
impl Aspect for KindOf {
    fn name() -> &'static str {
        "kind"
    }
}
impl World {
    /// Declares each of `names` as a kind, once; hands back their kinds.
    pub fn declare_kinds(&mut self, names: &[&str]) -> Vec<Kind> {
        let list = self.kind_list_mut();
        names.iter().map(|n| list.declare(n)).collect()
    }
    /// The kind called `name`, if declared.
    pub fn kind(&self, name: &str) -> Option<Kind> {
        self.resources.get::<KindList>()?.get(name)
    }
    pub fn kind_name(&self, kind: Kind) -> Option<&str> {
        self.resources.get::<KindList>()?.name(kind)
    }
    /// The kind of `word`, if it was spawned or given one.
    pub fn kind_of(&self, word: Word) -> Option<Kind> {
        self.read::<KindOf>(word).map(|k| k.0)
    }
    /// Gives `word` the kind `kind`.
    pub fn set_kind(&mut self, word: Word, kind: Kind) {
        self.bind(word, KindOf(kind));
    }
    fn kind_list_mut(&mut self) -> &mut KindList {
        if self.resources.get::<KindList>().is_none() {
            self.resources.insert(KindList::new());
        }
        self.resources.get_mut::<KindList>().expect("inserted above")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kinds_come_from_a_list_and_a_word_has_one() {
        let mut w = World::new();
        let k = w.declare_kinds(&["bone", "light", "bone"]);
        assert_eq!(k[0], k[2], "a name twice is one kind");
        assert_eq!(w.kind("light"), Some(k[1]));
        assert_eq!(w.kind("nope"), None);
        let a = w.coin_bare("a");
        assert_eq!(w.kind_of(a), None);
        w.set_kind(a, k[1]);
        assert_eq!(w.kind_of(a).and_then(|k| w.kind_name(k)), Some("light"));
    }
}
