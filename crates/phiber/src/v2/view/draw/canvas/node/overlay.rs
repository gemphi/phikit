//! `BNodeOverlay` — the shape every node-editor overlay shares.
//!
//! The file is read-only (`link/mod.rs` says why), so each kind of edit the
//! user makes to a node tree — added nodes (`add/`), made and broken links
//! (`link/`), changed socket values (`edit/`) — lives as an overlay in one
//! state value under one `NodeKey`, round-tripped through a string and merged
//! over the file on every frame.
//!
//! The three differ in WHAT they merge — nodes, links, socket defaults — which
//! is each one's own `apply`, and stays inherent. They are identical in HOW they
//! are stored, read and written back, which is this trait: one key, one
//! `encode`/`decode` round trip, and from those a read (`of`) and a write action
//! (`write`) that every caller would otherwise spell by hand.

use super::{add, edit, link};
use crate::v2::host::BHostType;
use crate::v2::state::{BKey as _, BState, NodeKey};
use crate::v2::view::prim::BAction;

pub trait BNodeOverlay: Sized + Default {
    /// The one key this overlay is stored under, for a host's tree.
    fn key(host: BHostType) -> NodeKey;
    /// The overlay as one state string. The spelling is each overlay's own
    /// (`*/store.rs`); the round trip with `decode` is the contract.
    fn encode(&self) -> String;
    /// The overlay back out of that string — an empty overlay for empty or
    /// malformed input, never a panic.
    fn decode(text: &str) -> Self;

    /// This host's overlay, read out of the store: the file's own edits of this
    /// kind, or an empty overlay when nothing is stored yet.
    fn of(state: &BState, host: BHostType) -> Self {
        Self::decode(state.get(&Self::key(host).path()).unwrap_or(""))
    }

    /// The action that writes this overlay back — `Set` the one key to the
    /// encoded value, which is what every writer of an overlay dispatches.
    fn write(&self, host: BHostType) -> BAction {
        BAction::Set(Self::key(host).path(), self.encode())
    }
}

impl BNodeOverlay for edit::BValueEdits {
    fn key(host: BHostType) -> NodeKey {
        NodeKey::Values(host)
    }
    fn encode(&self) -> String {
        edit::store::encode(self)
    }
    fn decode(text: &str) -> Self {
        edit::store::decode(text)
    }
}

impl BNodeOverlay for link::BLinkEdits {
    fn key(host: BHostType) -> NodeKey {
        NodeKey::Wired(host)
    }
    fn encode(&self) -> String {
        link::store::encode(self)
    }
    fn decode(text: &str) -> Self {
        link::store::decode(text)
    }
}

impl BNodeOverlay for add::BNodeAdditions {
    fn key(host: BHostType) -> NodeKey {
        NodeKey::Added(host)
    }
    // Added nodes are spelled as the bare list; the struct wraps it with the
    // induced-link bookkeeping the other two do not have.
    fn encode(&self) -> String {
        add::store::encode(&self.added)
    }
    fn decode(text: &str) -> Self {
        Self { added: add::store::decode(text) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::blend::shade::socket::BSocketDefault;
    use crate::v2::state::action::apply_action;

    /// For any overlay: the action `write` produces, applied to a fresh store,
    /// reads back through `of` as the same overlay. Written once, run for each —
    /// which is the point of the trait: one store contract, three overlays.
    fn round_trips<T: BNodeOverlay + PartialEq + core::fmt::Debug>(overlay: T) {
        let mut s = BState::new();
        apply_action(&mut s, &overlay.write(BHostType::Shader));
        assert_eq!(T::of(&s, BHostType::Shader), overlay, "overlay did not survive write → of");
        // And it is scoped to its host: another editor's store is untouched.
        assert_eq!(T::of(&s, BHostType::Compositor), T::default(), "one host's overlay is not another's");
    }

    #[test]
    fn every_node_overlay_round_trips_through_its_store_via_the_trait() {
        let mut values = edit::BValueEdits::default();
        values.set("Mix", "Fac", BSocketDefault::Float(0.3));
        round_trips(values);

        round_trips(link::BLinkEdits {
            added: vec![(link::BLinkEnd::new("A", "Out"), link::BLinkEnd::new("B", "In"))],
            removed: Vec::new(),
        });

        round_trips(add::BNodeAdditions {
            added: vec![add::BNodeAddition {
                idname: "ShaderNodeValue".into(),
                name: "Value".into(),
                at: (1.0, 2.0),
                link: None,
            }],
        });
    }
}
