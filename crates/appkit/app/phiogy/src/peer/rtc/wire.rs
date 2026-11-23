//! A connection as the library makes it, with what this window sends
//! on it. What is sent changes during a call: a camera is turned on or
//! off, a screen shared or not. `wire` has a connection send exactly
//! what is on now, adding and removing tracks, after which the two
//! ends agree again by themselves (the library says so, `Made::Need`).

use std::sync::{Arc, Mutex};

use libwebrtc::media_stream_track::MediaStreamTrack;
use libwebrtc::peer_connection::{PeerConnection, PeerConnectionState};
use libwebrtc::peer_connection_factory::{IceServer, RtcConfiguration};
use libwebrtc::rtp_sender::RtpSender;
use serde_json::{Map, Value, json};
use tokio::task::JoinHandle;

use super::ear::{hear, push, see};
use super::Set;
use super::link::Kit;
use super::sent::{Slot, film};
use crate::board::lock;
use crate::peer::engine::{Made, Stage};

/// One connection: the library's, what it sends now, and the tasks
/// that take in what arrives on it.
pub struct Wired {
    pub pc: PeerConnection,
    /// What it sends: which of the three, at which turn, through what.
    sender: Vec<(Slot, u64, RtpSender)>,
    pub meter: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl Wired {
    /// Send exactly what is on now: what was taken off is removed, and
    /// what was put on, or put on anew, is added.
    pub fn wire(&mut self, kit: &Kit) {
        let want = lock(&kit.sent).want();
        let (kept, gone): (Vec<_>, Vec<_>) = std::mem::take(&mut self.sender).into_iter().partition(|(slot, turn, _)| want.iter().any(|(wanted, at, _)| wanted == slot && at == turn));
        gone.into_iter().for_each(|(.., sender)| _ = self.pc.remove_track(sender));
        self.sender = kept;
        for (slot, turn, track) in want {
            if self.sender.iter().all(|(held, ..)| *held != slot)
                && let Ok(sender) = self.pc.add_track(track, &[kit.set.word(slot.stream())])
            {
                self.sender.push((slot, turn, sender));
            }
        }
    }

    /// Let the connection go, and stop taking in what arrived on it.
    pub fn close(&self) {
        lock(&self.meter).drain(..).for_each(|task| task.abort());
        self.pc.close();
    }
}

/// A connection to that caller with what this window sends on it,
/// saying what it makes.
pub fn connect(kit: &Arc<Kit>, config: &RtcConfiguration, far: &str, made: &Arc<Mutex<Vec<Made>>>) -> Option<Wired> {
    let pc = kit.factory.create_peer_connection(config.clone()).ok()?;
    let told = Arc::clone(made);
    pc.on_negotiation_needed(Some(Box::new(move |_| push(&told, Made::Need))));
    let (told, word) = (Arc::clone(made), kit.set.signal.clone());
    pc.on_ice_candidate(Some(Box::new(move |way| {
        let body = Map::from_iter([(word.get("candidate").to_string(), json!(way.candidate())), (word.get("mid").to_string(), json!(way.sdp_mid())), (word.get("index").to_string(), json!(way.sdp_mline_index()))]);
        push(&told, Made::Candidate(Value::Object(body)));
    })));
    let told = Arc::clone(made);
    pc.on_connection_state_change(Some(Box::new(move |state| {
        let stage = match state {
            PeerConnectionState::Connected => Stage::Connected,
            PeerConnectionState::Failed => Stage::Failed,
            PeerConnectionState::New | PeerConnectionState::Closed => Stage::New,
            PeerConnectionState::Connecting | PeerConnectionState::Disconnected => Stage::Connecting,
        };
        push(&told, Made::Stage(stage));
    })));
    let meter = Arc::new(Mutex::new(Vec::new()));
    let (told, kept, inside, whose) = (Arc::clone(made), Arc::clone(&meter), Arc::clone(kit), far.to_string());
    pc.on_track(Some(Box::new(move |event| {
        // A picture is known by the stream it comes under: that is how
        // a camera is told from a shared screen.
        let stream = event.streams.first().map(libwebrtc::media_stream::MediaStream::id).unwrap_or_default();
        let task = match event.track {
            MediaStreamTrack::Audio(track) => inside.handle.spawn(hear(track, Arc::clone(&inside), Arc::clone(&told))),
            MediaStreamTrack::Video(track) => {
                let name = film(&inside.set, &whose, &stream);
                inside.handle.spawn(see(track, stream, name, Arc::clone(&inside), Arc::clone(&told)))
            }
        };
        lock(&kept).push(task);
    })));
    let mut wired = Wired { pc, sender: Vec::new(), meter };
    wired.wire(kit);
    Some(wired)
}

/// The servers of the line as the library takes them: each one's urls,
/// one or several, and what it wants to be told.
pub(super) fn server(ice: &[Value], set: &Set) -> Vec<IceServer> {
    let text = |value: &Value, name: &str| value.get(set.word(name)).and_then(Value::as_str).unwrap_or_default().to_string();
    let urls = |value: &Value| match value.get(set.word("ice.urls")) {
        Some(Value::String(one)) => vec![one.clone()],
        Some(Value::Array(list)) => list.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        _ => Vec::new(),
    };
    ice.iter().map(|one| IceServer { urls: urls(one), username: text(one, "ice.username"), password: text(one, "ice.credential") }).collect()
}
