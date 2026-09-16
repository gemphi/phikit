//! State — a flat `HashMap<String, String>` keyed by encoded `BPath`.
//!
//! Every key is `ctx.<seg>.<seg>...`. Host state naturally nests under the
//! host's slug: `ctx.<host>.<tail>`. Read via `state.host(host).get(tail)`.
//! Three-letter accessors: `get`, `set`, `has`.

pub mod action;
pub mod area_resize;
pub mod file_nav;
pub mod key;
pub mod menu;
pub mod path;
pub mod seed;
pub mod space_switch;
pub mod val;
pub mod zone_size;

pub use area_resize::{
    K_DRAG_INDEX, K_DRAG_SPACE, K_DRAG_ZONE, ResizeError, apply_delta, apply_drag, area_fraction,
    area_size_key, area_size_prefix, boundary_pair, clamp_to_min, normalize, reset_equal,
    snap_to_equal, zone_visibility_key,
};
pub use key::*;
pub use menu::{
    apply_open, apply_toggle, hover_open_if_menubar_active, is_menubar_active,
    is_menubar_active_in, k_menu, k_menu_path,
};
pub use path::BPath;
pub use seed::default_state;
pub use space_switch::switch_space;
pub use val::*;
pub use zone_size::{
    apply_zone_drag, is_resizable as is_zone_resizable, reset_zone_size, zone_size, zone_size_key,
};

use crate::v2::host::{BHostType, host_slug};
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

    pub fn get(&self, path: &BPath) -> Option<&str> {
        self.inner.get(&path.encode()).map(String::as_str)
    }
    pub fn set(&mut self, path: &BPath, val: impl Into<String>) {
        self.inner.insert(path.encode(), val.into());
    }
    pub fn set_encoded(&mut self, encoded: &str, val: impl Into<String>) {
        self.inner.insert(encoded.to_string(), val.into());
    }
    /// Remove a key from the store, clearing it entirely. A cleared key reads
    /// back as absent (`get` returns `None`, `has` is false), which is how the
    /// scene loading/error flags represent their "off" state — never a stale
    /// value such as `"true"`. No-op when the key is not present.
    pub fn remove(&mut self, path: &BPath) {
        self.inner.remove(&path.encode());
    }
    pub fn get_encoded(&self, encoded: &str) -> Option<&str> {
        self.inner.get(encoded).map(String::as_str)
    }
    pub fn iter_keys(&self) -> impl Iterator<Item = &String> {
        self.inner.keys()
    }
    pub fn has(&self, path: &BPath, expected: &str) -> bool {
        self.get(path) == Some(expected)
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
        let mut segs: Vec<&'static str> = vec![host_slug(self.host)];
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
