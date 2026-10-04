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
use std::collections::HashMap;
use std::sync::Mutex;

/// Intern a runtime name to a `&'static str` so it can be a `BPath` segment.
///
/// A plain `Box::leak` per frame would leak every frame; interning leaks each
/// distinct name at most once, so the leaked set is bounded by the number of
/// distinct names ever shown.
///
/// This lives here rather than in whichever part first needed it, because
/// several now do: the Outliner's rows, the animation editors' channels and the
/// node editors' cards are all named by the FILE, so none of their names is
/// known at compile time. One pool means one leak per distinct name however
/// many of them ask for it.
pub fn intern(name: &str) -> &'static str {
    static POOL: Mutex<Option<HashMap<String, &'static str>>> = Mutex::new(None);
    let mut guard = POOL.lock().unwrap();
    let map = guard.get_or_insert_with(HashMap::new);
    if let Some(&interned) = map.get(name) {
        return interned;
    }
    let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
    map.insert(name.to_string(), leaked);
    leaked
}

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
