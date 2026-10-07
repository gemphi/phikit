//! The collaboration service: the pluggable transport and the bookkeeping that
//! turns its traffic into local state. No wire protocol ships here — a real
//! transport (a server client) is installed with [`BCollab::arm`]; unarmed, the
//! session is offline and the whole service is inert.
//!
//! Two directions:
//! - **Inbound** ([`BCollab::drain`]): poll the transport, apply each peer's
//!   `BAction` through [`apply_action`] — the same pure transition a local edit
//!   takes — and write presence into `collab.*` (status, server, peers,
//!   channels), the keys the Activity panel and the Collaboration modal already
//!   read.
//! - **Outbound** ([`BCollab::broadcast`]): send a local `BAction` to peers.
//!   Remote actions are applied directly (never through `handle_action`), so
//!   they are never echoed back out; [`broadcasts`] keeps local-only keys (the
//!   caret, open menus, the viewport camera, the file browser, the workspace,
//!   `collab.*` itself) off the wire.

use std::sync::Mutex;

use crate::v2::state::action::apply_action;
use crate::v2::state::{AppKey, BState};
use crate::v2::view::prim::BAction;

/// What a transport hands the service each poll.
pub enum BCollabEvent {
    /// A peer's edit, to apply locally.
    Action(BAction),
    /// The session status for `collab.status` ("online", "connecting", …).
    Status(String),
    /// The peers now present — `(name, state)` per slot (`collab.peer.<n>`).
    Peers(Vec<(String, String)>),
    /// The channels available — `(name, state)` per slot (`collab.channel.<n>`).
    Channels(Vec<(String, String)>),
}

/// A collaboration transport — the seam a real server client plugs into.
/// `Send` so the armed service can hold it behind a mutex.
pub trait BCollabTransport: Send {
    /// Send a local action to peers.
    fn send(&self, action: &BAction);
    /// Events arrived since the last poll (empty when nothing has).
    fn poll(&self) -> Vec<BCollabEvent>;
    /// The server this is connected to, for `collab.server`.
    fn server(&self) -> String {
        String::new()
    }
}

static TRANSPORT: Mutex<Option<Box<dyn BCollabTransport>>> = Mutex::new(None);

/// Peer / channel slot names (`collab.peer.0` … `.15`), matching the UI's slots
/// (`part::activity::option`, `modal::collab`). `&'static` because the keys are.
const SLOTS: [&str; 16] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15",
];

pub struct BCollab;

impl BCollab {
    /// Install a transport — a session connecting. Unarmed, the service is
    /// inert and the session shows offline.
    pub fn arm(transport: Box<dyn BCollabTransport>) {
        *TRANSPORT.lock().unwrap() = Some(transport);
    }

    /// Drop the transport (disconnect). The service goes inert; the caller
    /// clears `collab.*` if it wants the UI to show offline at once.
    pub fn disarm() {
        *TRANSPORT.lock().unwrap() = None;
    }

    /// Send a local action to peers — when connected and the action is one
    /// worth sharing ([`broadcasts`]). A no-op unarmed.
    pub fn broadcast(action: &BAction) {
        if !broadcasts(action) {
            return;
        }
        if let Some(t) = TRANSPORT.lock().unwrap().as_ref() {
            t.send(action);
        }
    }

    /// Apply everything the transport has: remote actions through
    /// [`apply_action`], presence into `collab.*`. Returns whether anything
    /// changed (so the caller re-renders). A no-op unarmed.
    ///
    /// The transport lock is released before applying, so a remote action can
    /// never deadlock against an outbound [`Self::broadcast`] it might trigger.
    pub fn drain(state: &mut BState) -> bool {
        let (events, server) = {
            let guard = TRANSPORT.lock().unwrap();
            let Some(t) = guard.as_ref() else { return false };
            (t.poll(), t.server())
        };
        let mut changed = false;
        if !server.is_empty() && state.get(AppKey::CollabServer) != Some(server.as_str()) {
            state.set(AppKey::CollabServer, server);
            changed = true;
        }
        for event in events {
            changed = true;
            match event {
                BCollabEvent::Action(action) => apply_action(state, &action),
                BCollabEvent::Status(status) => state.set(AppKey::CollabStatus, status),
                BCollabEvent::Peers(list) => write_slots(state, &list, AppKey::CollabPeer),
                BCollabEvent::Channels(list) => write_slots(state, &list, AppKey::CollabChannel),
            }
        }
        changed
    }
}

/// Write `(name, state)` lines into contiguous `collab.*` slots and clear the
/// rest, so the UI (which reads slots until the first absent one) shows exactly
/// the current set. Each value is `"name\tstate"`, as the views parse it.
fn write_slots(state: &mut BState, list: &[(String, String)], key: fn(&'static str) -> AppKey) {
    for (slot, entry) in SLOTS.iter().zip(list.iter()) {
        state.set(key(slot), format!("{}\t{}", entry.0, entry.1));
    }
    for slot in SLOTS.iter().skip(list.len()) {
        state.remove(key(slot));
    }
}

/// Whether a local action is shared with peers. The document edits go out;
/// each user's own view and session chrome stay home. This is the policy knob
/// for what "collaboration" means here — widen or narrow it freely.
pub fn broadcasts(action: &BAction) -> bool {
    let path = match action {
        BAction::Set(p, _) | BAction::Adjust(p, _) | BAction::Toggle(p) | BAction::Open(p) | BAction::Reset(p) => p,
    };
    !is_local_only(&path.encode())
}

/// Keys that are each session's own, never sent: presence itself, open menus
/// and dialogs, layout drags, the text caret, the viewport camera, the import
/// machinery, the file browser, the status log and the chosen workspace.
fn is_local_only(encoded: &str) -> bool {
    encoded.starts_with("ctx.collab.")
        || encoded.starts_with("ctx.menu.")
        || encoded.starts_with("ctx.header.")
        || encoded == "ctx.modal"
        || encoded.starts_with("ctx.area_resize.")
        || encoded == "ctx.edit"
        || encoded == "ctx.edit_undo"
        || encoded.starts_with("ctx.view3d.")
        || encoded.starts_with("ctx.scene.load")
        || encoded == "ctx.scene.loading"
        || encoded.starts_with("ctx.file.")
        || encoded.starts_with("ctx.status.")
        || encoded == "ctx.space"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::{BKey as _, CtxKey};
    use std::sync::Arc;

    /// An in-memory transport: a queue of inbound events to feed, and a record
    /// of what was sent out.
    struct Fake {
        inbound: Mutex<Vec<BCollabEvent>>,
        sent: Arc<Mutex<Vec<BAction>>>,
        server: String,
    }
    impl BCollabTransport for Fake {
        fn send(&self, action: &BAction) {
            self.sent.lock().unwrap().push(action.clone());
        }
        fn poll(&self) -> Vec<BCollabEvent> {
            std::mem::take(&mut self.inbound.lock().unwrap())
        }
        fn server(&self) -> String {
            self.server.clone()
        }
    }

    // One test, run start to finish, because the transport is a process global:
    // splitting it would let parallel tests arm over each other.
    #[test]
    fn applies_inbound_broadcasts_outbound_and_is_inert_unarmed() {
        // Unarmed: inert.
        BCollab::disarm();
        let mut s = BState::new();
        assert!(!BCollab::drain(&mut s), "nothing to drain with no transport");
        BCollab::broadcast(&BAction::Set(CtxKey::Object.path(), "X".into())); // no panic

        // Armed: inbound presence + a remote edit.
        let sent = Arc::new(Mutex::new(Vec::new()));
        BCollab::arm(Box::new(Fake {
            inbound: Mutex::new(vec![
                BCollabEvent::Status("online".into()),
                BCollabEvent::Peers(vec![("Ada".into(), "editing".into()), ("Lin".into(), "viewing".into())]),
                BCollabEvent::Action(BAction::Set(CtxKey::Object.path(), "Cube".into())),
            ]),
            sent: sent.clone(),
            server: "wss://studio.local".into(),
        }));

        let mut s = BState::new();
        assert!(BCollab::drain(&mut s));
        assert_eq!(s.get(AppKey::CollabStatus), Some("online"));
        assert_eq!(s.get(AppKey::CollabServer), Some("wss://studio.local"));
        assert_eq!(s.get(AppKey::CollabPeer("0")), Some("Ada\tediting"));
        assert_eq!(s.get(AppKey::CollabPeer("1")), Some("Lin\tviewing"));
        assert_eq!(s.get(AppKey::CollabPeer("2")), None, "slots past the peer count are cleared");
        assert_eq!(s.get(CtxKey::Object), Some("Cube"), "the remote edit applied via apply_action");

        // A shorter presence list later clears the trailing slot.
        BCollab::arm(Box::new(Fake {
            inbound: Mutex::new(vec![BCollabEvent::Peers(vec![("Ada".into(), "editing".into())])]),
            sent: sent.clone(),
            server: String::new(),
        }));
        BCollab::drain(&mut s);
        assert_eq!(s.get(AppKey::CollabPeer("1")), None, "a peer who left frees its slot");

        // Outbound: a document edit goes out; local-only edits do not. Assert
        // by membership (not an exact count) so it stays correct if anything
        // else shares this process's transport.
        sent.lock().unwrap().clear();
        BCollab::broadcast(&BAction::Set(CtxKey::Object.path(), "Sphere".into())); // shared
        BCollab::broadcast(&BAction::Set(AppKey::Edit.path(), "typing".into())); // the caret — local
        BCollab::broadcast(&BAction::Set(AppKey::CollabStatus.path(), "online".into())); // never echo presence
        BCollab::broadcast(&BAction::Set(CtxKey::Space.path(), "Shading".into())); // the user's workspace — local
        let out = sent.lock().unwrap();
        let sent_a = |target: &BAction| out.iter().any(|a| a == target);
        assert!(sent_a(&BAction::Set(CtxKey::Object.path(), "Sphere".into())), "the document edit went out: {out:?}");
        assert!(!sent_a(&BAction::Set(AppKey::Edit.path(), "typing".into())), "the caret stayed home");
        assert!(!sent_a(&BAction::Set(AppKey::CollabStatus.path(), "online".into())), "presence is never echoed");
        assert!(!sent_a(&BAction::Set(CtxKey::Space.path(), "Shading".into())), "the workspace stayed home");
        drop(out);

        BCollab::disarm();
    }
}
