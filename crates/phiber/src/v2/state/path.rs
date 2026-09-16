//! `BPath` — hierarchical state key.
//!
//! Every key is `ctx.<seg>.<seg>...`. Segments cascade with the store
//! hierarchy; the deepest keys look like `ctx.<host>.<pane>.<part>.<view>`.
//!
//! Internally the segment slice is a `Cow<'static, [&'static str]>` so
//! `const BPath::from_static(&[...])` is legal — every fixed key can be
//! declared as a `pub const` (see `state::key`). Dynamic paths (host-scoped
//! reads, per-menu-label lookups) use `BPath::of(&[..])` which allocates.

use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BPath {
    pub segs: Cow<'static, [&'static str]>,
}

impl BPath {
    pub const fn from_static(segs: &'static [&'static str]) -> Self {
        Self {
            segs: Cow::Borrowed(segs),
        }
    }
    pub fn of(segs: &[&'static str]) -> Self {
        Self {
            segs: Cow::Owned(segs.to_vec()),
        }
    }
    pub fn owned(segs: Vec<&'static str>) -> Self {
        Self {
            segs: Cow::Owned(segs),
        }
    }
    pub fn encode(&self) -> String {
        let mut o = String::from("ctx");
        for s in self.segs.iter() {
            o.push('.');
            o.push_str(s);
        }
        o
    }
}

impl Default for BPath {
    fn default() -> Self {
        Self::from_static(&[])
    }
}
