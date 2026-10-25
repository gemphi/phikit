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
    ImageKey, SceneKey, TimeViewKey, UvKey, View3dKey, ZoneKey,
};
pub mod log;
/// The event tree lives with the log it is the structured half of.
pub use log::event;
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
pub use path::{BPath, intern};
pub use seed::default_state;
pub use val::*;

use crate::v2::host::BHostType;
use std::collections::HashMap;

/// Flat KV store. Keys are encoded `BPath` strings; values are strings.
///
/// Beside the values, a revision per key family — the segment after `ctx`
/// (`view3d`, `outliner`, `object` …) — counts every write to a key of
/// that family, so a view built from several families can tell whether
/// any of them changed since it was last built (`revision_except`) without
/// reading them all again. An orbit writes `view3d` keys sixty times a
/// second; the Outliner reads none of them.
#[derive(Clone, Debug, Default)]
pub struct BState {
    inner: HashMap<String, String>,
    revs: HashMap<String, u64>,
}

impl BState {
    pub fn new() -> Self {
        Self::default()
    }

    /// The key family of an encoded key: `ctx.view3d.yaw` → `view3d`.
    fn family(encoded: &str) -> &str {
        let tail = encoded.strip_prefix("ctx.").unwrap_or(encoded);
        tail.split('.').next().unwrap_or(tail)
    }

    fn touched(&mut self, encoded: &str) {
        let family = Self::family(encoded);
        match self.revs.get_mut(family) {
            Some(rev) => *rev += 1,
            None => {
                self.revs.insert(family.to_string(), 1);
            }
        }
    }

    /// How many writes the family `prefix` has had.
    pub fn revision(&self, prefix: &str) -> u64 {
        self.revs.get(prefix).copied().unwrap_or(0)
    }

    /// The writes that can change what the scene's lists hold — the
    /// Outliner's rows, the hidden set, the objects the viewport shows:
    /// every family but the viewport's camera (`view3d`) and the clock
    /// (`TIME_FAMILIES`), so an orbit, a zoom or a frame played leaves it
    /// alone — Blender's Outliner keeps its tree store across frames too.
    pub fn scene_revision(&self) -> u64 {
        self.revision_except(&Self::LIST_SKIPS)
    }

    /// The key families a frame of playback or a key edit writes, none of
    /// which names an object: the playhead and its transport, the ranges,
    /// the markers and the Dope Sheet's keys.
    const LIST_SKIPS: [&'static str; 15] = [
        "view3d", "time_view", "frame", "playing", "frame_step", "frame_start", "frame_end", "fps", "play_from", "auto_key", "preview_range", "preview_start", "preview_end", "markers", "keys",
    ];

    /// One number for the writes to every family but `skip`'s: the same
    /// until any key outside `skip` is set or removed.
    pub fn revision_except(&self, skip: &[&str]) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut sum = 0u64;
        for (family, rev) in &self.revs {
            if skip.contains(&family.as_str()) {
                continue;
            }
            let mut h = std::collections::hash_map::DefaultHasher::new();
            family.hash(&mut h);
            sum = sum.wrapping_add(h.finish().wrapping_mul(rev.wrapping_add(1)));
        }
        sum
    }

    pub fn get(&self, key: impl BKey) -> Option<&str> {
        self.inner.get(&key.path().encode()).map(String::as_str)
    }
    pub fn set(&mut self, key: impl BKey, val: impl Into<String>) {
        let encoded = key.path().encode();
        self.touched(&encoded);
        self.inner.insert(encoded, val.into());
    }
    pub fn set_encoded(&mut self, encoded: &str, val: impl Into<String>) {
        self.touched(encoded);
        self.inner.insert(encoded.to_string(), val.into());
    }
    /// Remove a key from the store, clearing it entirely. A cleared key reads
    /// back as absent (`get` returns `None`, `has` is false), which is how the
    /// scene loading/error flags represent their "off" state — never a stale
    /// value such as `"true"`. No-op when the key is not present.
    pub fn remove(&mut self, key: impl BKey) {
        let encoded = key.path().encode();
        if self.inner.remove(&encoded).is_some() {
            self.touched(&encoded);
        }
    }
    /// Remove an encoded key, as `remove` does a typed one.
    pub fn remove_encoded(&mut self, encoded: &str) {
        if self.inner.remove(encoded).is_some() {
            self.touched(encoded);
        }
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

#[cfg(test)]
mod revision_tests {
    use super::*;

    /// A write to a key counts for its family; `revision_except` is the
    /// same number until a family outside `skip` is written or cleared.
    #[test]
    fn revisions_count_writes_per_key_family() {
        let mut s = BState::new();
        let none = s.revision_except(&["view3d"]);
        s.set(View3dKey::Yaw, "30");
        s.set(View3dKey::Zoom, "2");
        assert_eq!(s.revision("view3d"), 2);
        assert_eq!(s.revision_except(&["view3d"]), none, "the viewport's writes are skipped");
        s.set(CtxKey::Object, "Cube");
        let picked = s.revision_except(&["view3d"]);
        assert_ne!(picked, none, "a write outside the skipped family counts");
        s.set(View3dKey::Yaw, "31");
        assert_eq!(s.revision_except(&["view3d"]), picked);
        s.remove(CtxKey::Object);
        assert_ne!(s.revision_except(&["view3d"]), picked, "a clear counts");
        s.remove(CtxKey::Object);
        assert_eq!(s.revision("object"), 2, "clearing an absent key is no write");
    }
}
