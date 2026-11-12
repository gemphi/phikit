//! Stored aspects: named fields of [`Value`], and back, with no dependency.
//!
//! Apple saves with `Codable` (`Transform: Codable` RF:13684, `Entity.write`
//! RF:12125). Here an aspect that implements [`Stored`] turns into `(name,
//! Value)` fields, which map onto phiora's `items.doc`: `{ aspect: { field:
//! value } }` (docs/connect.md). [`Owner`] and the stable id are the two
//! ideas kept from `SynchronizationComponent` (RF:2815-2848).
mod entry;
mod md5;
pub use entry::{StoredEntry, StoredList};
pub use md5::{md5, md5_hex};
use crate::aspect::Aspect;
use crate::word::Word;
use crate::world::World;
/// A saved value: a number, text, or a list of values.
#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Num(f64),
    Text(String),
    List(Vec<Value>),
}
impl Value {
    pub fn num(&self) -> Option<f64> {
        if let Value::Num(n) = self { Some(*n) } else { None }
    }
    pub fn text(&self) -> Option<&str> {
        if let Value::Text(s) = self { Some(s) } else { None }
    }
    pub fn list(&self) -> Option<&[Value]> {
        if let Value::List(l) = self { Some(l) } else { None }
    }
    /// A list of numbers from `v`.
    pub fn nums(v: &[f32]) -> Value {
        Value::List(v.iter().map(|n| Value::Num(f64::from(*n))).collect())
    }
    /// `N` numbers from a list; `None` if it is not a list of exactly `N` numbers.
    pub fn as_nums<const N: usize>(&self) -> Option<[f32; N]> {
        let l = self.list()?;
        if l.len() != N {
            return None;
        }
        let mut out = [0.0; N];
        for (o, v) in out.iter_mut().zip(l) {
            *o = v.num()? as f32;
        }
        Some(out)
    }
}
/// An aspect's saved form: field names and values, in order.
pub type Fields = Vec<(String, Value)>;
/// The value of the field called `name`.
pub fn field<'a>(fields: &'a Fields, name: &str) -> Option<&'a Value> {
    fields.iter().find(|(n, _)| n == name).map(|(_, v)| v)
}
/// An aspect that can be saved. Its [`Aspect::name`] is its key in a doc;
/// `KIND` repeats it as a constant, and the two must agree.
pub trait Stored: Aspect + Sized {
    const KIND: &'static str;
    /// True if the journal keeps this aspect's value at every tick it changes.
    const TIMED: bool = false;
    fn fields(&self) -> Fields;
    /// Builds the aspect from its fields; `None` if one is missing or wrong.
    fn from_fields(fields: &Fields) -> Option<Self>;
}
/// The id a word has in every process and in phiora: the MD5 of `kind:name`,
/// as 32 lowercase hex digits.
pub fn stable_id(kind: &str, name: &str) -> String {
    md5_hex(format!("{kind}:{name}").as_bytes())
}
/// Who may write a word: this process, or another that shares it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Owner {
    Local,
    Remote,
}
impl Aspect for Owner {
    fn name() -> &'static str {
        "owner"
    }
}
impl Stored for Owner {
    const KIND: &'static str = "owner";
    fn fields(&self) -> Fields {
        vec![("local".into(), Value::Num(if *self == Owner::Local { 1.0 } else { 0.0 }))]
    }
    fn from_fields(fields: &Fields) -> Option<Self> {
        Some(if field(fields, "local")?.num()? != 0.0 { Owner::Local } else { Owner::Remote })
    }
}
impl World {
    /// `word`'s [`stable_id`], from its kind and spelling; `None` without a kind.
    pub fn stable_id(&self, word: Word) -> Option<String> {
        let kind = self.kind_name(self.kind_of(word)?)?;
        Some(stable_id(kind, self.spelling(word)))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_stable_id_is_the_md5_of_kind_and_name() {
        assert_eq!(stable_id("a", "b"), md5_hex(b"a:b"));
        assert_eq!(stable_id("abc", ""), md5_hex(b"abc:"));
        let mut w = World::new();
        let k = w.declare_kinds(&["bone"])[0];
        let a = w.spawn(k);
        assert_eq!(w.stable_id(a), Some(md5_hex(b"bone:bone#1")));
        assert_eq!(w.stable_id(w.lexicon.get("bone#1").unwrap()), w.stable_id(a));
    }
    #[test]
    fn numbers_round_trip_through_a_value() {
        let v = Value::nums(&[0.1, -2.5, 3.0]);
        assert_eq!(v.as_nums::<3>(), Some([0.1, -2.5, 3.0]));
        assert_eq!(v.as_nums::<4>(), None);
        assert_eq!(Owner::from_fields(&Owner::Remote.fields()), Some(Owner::Remote));
    }
}
