//! State — a flat `HashMap<String, String>` keyed by encoded `BPath`.
//!
//! Every key is `ctx.<seg>.<seg>...`. Host state naturally nests under the
//! host's slug: `ctx.<host>.<tail>`. Read via `state.host(host).get(tail)`.
//! Three-letter accessors: `get`, `set`, `has`.

pub mod action;
pub mod edit;
pub mod key;
pub use key::{
    ActivityKey, AppKey, AreaKey, BKey, ButsKey, CtxKey, FileKey, MenuKey, ModalKey, OutlinerKey,
    ImageKey, SceneKey, UvKey, View3dKey, ZoneKey,
};
pub mod log;
pub mod menu;
pub mod path;
pub mod rule;
pub mod seed;
pub mod val;

pub use key::*;
pub use menu::{
    apply_open, apply_toggle, close_overlays, hover_open_if_menubar_active, is_menubar_active,
    is_menubar_active_in,
};
pub use path::BPath;
pub use seed::default_state;
pub use val::*;

use crate::v2::host::BHostType;
use std::collections::HashMap;

/// Flat KV store. Keys are encoded `BPath` strings; values are strings.
#[derive(Clone, Debug, Default)]
pub struct BState {
    inner: HashMap<String, String>,
}

impl BState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: impl BKey) -> Option<&str> {
        self.inner.get(&key.path().encode()).map(String::as_str)
    }
    pub fn set(&mut self, key: impl BKey, val: impl Into<String>) {
        self.inner.insert(key.path().encode(), val.into());
    }
    pub fn set_encoded(&mut self, encoded: &str, val: impl Into<String>) {
        self.inner.insert(encoded.to_string(), val.into());
    }
    /// Remove a key from the store, clearing it entirely. A cleared key reads
    /// back as absent (`get` returns `None`, `has` is false), which is how the
    /// scene loading/error flags represent their "off" state — never a stale
    /// value such as `"true"`. No-op when the key is not present.
    pub fn remove(&mut self, key: impl BKey) {
        self.inner.remove(&key.path().encode());
    }
    /// Remove an encoded key, as `remove` does a typed one.
    pub fn remove_encoded(&mut self, encoded: &str) {
        self.inner.remove(encoded);
    }
    pub fn get_encoded(&self, encoded: &str) -> Option<&str> {
        self.inner.get(encoded).map(String::as_str)
    }
    pub fn iter_keys(&self) -> impl Iterator<Item = &String> {
        self.inner.keys()
    }
    pub fn has(&self, key: impl BKey, expected: &str) -> bool {
        self.get(key) == Some(expected)
    }
    pub fn host(&self, host: BHostType) -> BHostState<'_> {
        BHostState { state: self, host }
    }
}

/// Host-scoped read view — every lookup nests under `ctx.<host>.<tail>`.
/// Return borrows tie to the underlying `BState`'s lifetime `'a` so a
/// short-lived `BHostState` binding does not shorten the borrow.
pub struct BHostState<'a> {
    state: &'a BState,
    host: BHostType,
}

impl<'a> BHostState<'a> {
    fn path(&self, tail: &'static str) -> BPath {
        let mut segs: Vec<&'static str> = vec![self.host.slug()];
        segs.extend(tail.split('.'));
        BPath::owned(segs)
    }
    pub fn get(&self, tail: &'static str) -> Option<&'a str> {
        self.state.get(&self.path(tail))
    }
    pub fn has(&self, tail: &'static str, expected: &str) -> bool {
        self.get(tail) == Some(expected)
    }
}
