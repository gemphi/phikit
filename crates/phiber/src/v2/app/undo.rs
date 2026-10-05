//! Undo — Blender's `ed.undo` / `ed.redo` over the document, not the UI.
//!
//! Blender's undo stack holds the scene, not which menu is open or where a
//! panel is scrolled; here the document is the set of keys that describe
//! what the file *is* now — the active and selected objects, the frame and
//! range, what was added, deleted, duplicated or hidden, the property
//! edits — and a step is a snapshot of them taken before an action that
//! changes any. `Preferences.edit.undo_steps` is 32 by default; so is
//! [`STEPS`].

use crate::v2::state::BState;
use std::collections::BTreeMap;

/// How many steps back are kept (Blender's default `undo_steps`).
pub const STEPS: usize = 32;

/// The document's keys: the exact ones, the domains taken whole, and the
/// UI keys inside those domains that are not document (which Properties
/// tab is showing, which dropdown is open).
fn document_keys() -> (Vec<String>, Vec<String>, Vec<String>) {
    use crate::v2::state::{BKey as _, ButsKey, CtxKey, OutlinerKey, SceneKey};
    let exact = vec![
        CtxKey::Object.path().encode(),
        CtxKey::Selected.path().encode(),
        CtxKey::Frame.path().encode(),
        CtxKey::FrameStart.path().encode(),
        CtxKey::FrameEnd.path().encode(),
        CtxKey::Fps.path().encode(),
        CtxKey::Markers.path().encode(), CtxKey::KeySelected.path().encode(),
        SceneKey::Added.path().encode(),
        SceneKey::Deleted.path().encode(),
        SceneKey::Copied.path().encode(),
        SceneKey::Renamed.path().encode(),
        SceneKey::Moved.path().encode(),
        SceneKey::Parented.path().encode(),
        SceneKey::Linked.path().encode(),
        SceneKey::Nested.path().encode(),
        SceneKey::Assigned.path().encode(),
        SceneKey::MadeActions.path().encode(),
        SceneKey::ActionRanges.path().encode(),
        SceneKey::Keyed.path().encode(), SceneKey::Nla.path().encode(),
        SceneKey::Transformed.path().encode(), SceneKey::ShadowCast.path().encode(), SceneKey::RenderBorder.path().encode(),
    ];
    let domains = vec![domain_of(OutlinerKey::Search.path().encode()), domain_of(ButsKey::NavbarTab.path().encode())];
    let ui = vec![
        ButsKey::NavbarTab.path().encode(),
        ButsKey::NavbarExpanded.path().encode(),
        // `ctx.buts.dropdown.<key>`: the dropdowns' domain, whichever key.
        ButsKey::Dropdown("x").path().encode().rsplit_once('.').map_or(String::new(), |(head, _)| head.to_string()),
        OutlinerKey::Search.path().encode(),
    ];
    (exact, domains, ui)
}

/// `ctx.outliner.search` → `ctx.outliner.`: the domain a key sits in.
fn domain_of(encoded: String) -> String {
    let mut parts: Vec<&str> = encoded.split('.').collect();
    parts.truncate(2);
    format!("{}.", parts.join("."))
}

/// A gesture under way whose steps are one undo step: a marker grab, a
/// G / R / S, a key move in the Dope Sheet.
pub fn holding(s: &BState) -> bool {
    crate::v2::scene::grab::BMarkerGrab::active(s) || crate::v2::scene::xform::BTransformModal::active(s) || crate::v2::scene::action::dope::BKeyGrab::active(s)
}

/// The document as it stands: its keys and values.
pub type BSnapshot = BTreeMap<String, String>;

pub fn snapshot(state: &BState) -> BSnapshot {
    state
        .iter_keys()
        .filter(|key| is_document(key))
        .filter_map(|key| state.get_encoded(key).map(|value| (key.clone(), value.to_string())))
        .collect()
}

fn is_document(key: &str) -> bool {
    let (exact, domains, ui) = document_keys();
    if ui.iter().any(|u| key == u || key.starts_with(&format!("{u}."))) {
        return false;
    }
    exact.iter().any(|k| k == key) || domains.iter().any(|d| key.starts_with(d.as_str()))
}

/// Put the document back as `snapshot` had it: what it lacks is removed.
pub fn restore(state: &mut BState, snapshot: &BSnapshot) {
    let stale: Vec<String> = state.iter_keys().filter(|key| is_document(key) && !snapshot.contains_key(*key)).cloned().collect();
    for key in stale {
        state.remove_encoded(&key);
    }
    for (key, value) in snapshot {
        state.set_encoded(key, value.clone());
    }
}

/// The steps behind and ahead of now, each with the name of the change
/// that led on from it (`undo_step.name`, what Undo History lists).
#[derive(Default)]
pub struct BHistory {
    past: Vec<(BSnapshot, String)>,
    future: Vec<(BSnapshot, String)>,
    /// The document from before a gesture under way (a marker grab), held
    /// so the whole gesture is one step.
    held: Option<BSnapshot>,
}

impl BHistory {
    /// Record `before` as a step named `name` when `after` differs from it:
    /// `true` when it did (the document changed — `WM_file_tag_modified`).
    pub fn record(&mut self, before: BSnapshot, after: &BSnapshot, name: &str) -> bool {
        if before == *after {
            return false;
        }
        self.past.push((before, name.to_string()));
        if self.past.len() > STEPS {
            self.past.remove(0);
        }
        self.future.clear();
        true
    }

    /// `record`, with a gesture that makes one step of many actions: while
    /// `holding`, the document from before the first is kept and nothing is
    /// recorded; the action that ends it records from there.
    pub fn record_held(&mut self, before: BSnapshot, after: &BSnapshot, name: &str, holding: bool) -> bool {
        if holding {
            self.held.get_or_insert(before);
            return false;
        }
        let before = self.held.take().unwrap_or(before);
        self.record(before, after, name)
    }

    /// Step back: the document before the last change. `None` at the start.
    pub fn undo(&mut self, now: BSnapshot) -> Option<BSnapshot> {
        let (back, name) = self.past.pop()?;
        self.future.push((now, name));
        Some(back)
    }

    /// Step forward again. `None` when nothing was undone.
    pub fn redo(&mut self, now: BSnapshot) -> Option<BSnapshot> {
        let (ahead, name) = self.future.pop()?;
        self.past.push((now, name));
        Some(ahead)
    }

    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
        self.held = None;
    }

    pub fn depth(&self) -> (usize, usize) {
        (self.past.len(), self.future.len())
    }

    /// Undo History as Blender lists it (`TOPBAR_MT_undo_history`):
    /// "Original", then every step's name, oldest first — the ones undone
    /// included — and which entry the document stands at.
    pub fn listing(&self) -> BUndoListing {
        let mut names = vec!["Original".to_string()];
        names.extend(self.past.iter().map(|(_, name)| name.clone()));
        names.extend(self.future.iter().rev().map(|(_, name)| name.clone()));
        BUndoListing { names, at: self.past.len() }
    }
}

/// What Undo History shows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BUndoListing {
    pub names: Vec<String>,
    pub at: usize,
}

thread_local! {
    /// The listing, put here by the app for the render pass, because the
    /// menu is built from the state alone and the history is the app's.
    static LISTING: std::cell::RefCell<BUndoListing> = std::cell::RefCell::new(BUndoListing::default());
}

/// The app's history, for this render pass (`ed.undo_history`'s menu).
pub fn publish(history: &BHistory) {
    LISTING.with(|cell| *cell.borrow_mut() = history.listing());
}

pub fn listing() -> BUndoListing {
    LISTING.with(|cell| cell.borrow().clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::{BKey as _, CtxKey, MenuKey};

    #[test]
    fn the_document_is_the_scene_keys_and_undo_walks_them_back_and_forward() {
        let mut state = BState::new();
        let mut history = BHistory::default();
        state.set(CtxKey::Object, "Cube");
        state.set(&MenuKey::Nested("topbar", &["File"]).path(), "true");
        let first = snapshot(&state);
        let object = CtxKey::Object.path().encode();
        assert!(first.contains_key(&object) && first.len() == 1, "menus are UI, not document: {first:?}");

        state.set(CtxKey::Object, "Sphere");
        history.record(first, &snapshot(&state), "Select Sphere");
        state.set(CtxKey::Frame, "40");
        let second = snapshot(&state);
        history.record(snapshot_before(&state, &CtxKey::Frame.path().encode()), &second, "Frame 40");
        assert_eq!(history.depth(), (2, 0));
        assert_eq!(history.listing(), BUndoListing { names: ["Original", "Select Sphere", "Frame 40"].map(String::from).to_vec(), at: 2 });

        let back = history.undo(snapshot(&state)).unwrap();
        restore(&mut state, &back);
        assert!(state.get(CtxKey::Frame).is_none(), "the frame edit is undone");
        assert_eq!(state.get(CtxKey::Object), Some("Sphere"));
        let back = history.undo(snapshot(&state)).unwrap();
        restore(&mut state, &back);
        assert_eq!(state.get(CtxKey::Object), Some("Cube"));
        assert!(history.undo(snapshot(&state)).is_none(), "the start");
        let ahead = history.redo(snapshot(&state)).unwrap();
        restore(&mut state, &ahead);
        assert_eq!(state.get(CtxKey::Object), Some("Sphere"));
        assert_eq!(history.depth(), (1, 1));
        assert_eq!(history.listing(), BUndoListing { names: ["Original", "Select Sphere", "Frame 40"].map(String::from).to_vec(), at: 1 }, "the undone step is still listed, ahead");
        // A same-document action records nothing.
        assert!(!history.record(snapshot(&state), &snapshot(&state), "Nothing"));
        assert_eq!(history.depth(), (1, 1));
    }

    fn snapshot_before(state: &BState, without: &str) -> BSnapshot {
        let mut s = snapshot(state);
        s.remove(without);
        s
    }
}
