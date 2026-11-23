//! One real connection to one other caller. What it is asked is put in
//! a queue and done by one task, a thing at a time and in order, as the
//! browser's side chains what it does: so an answer is never set before
//! its offer, nor a way added before the description it belongs to.
//!
//! Three things the library insists on are kept here. An offer says
//! outright that this end takes sound and picture: left unsaid, the
//! library offers to send only, and turns off what the other end was
//! already sending. A way to reach the
//! other caller is held back until their description is set: one added
//! before it stops the connection for good. And an offer of this end's
//! own cannot be taken back, so when it must give way the connection is
//! made anew and the other's offer set on the new one.

use std::sync::{Arc, Mutex};

use appkit::pui::core::PFilm;
use libwebrtc::audio_source::native::NativeAudioSource;
use libwebrtc::ice_candidate::IceCandidate;
use libwebrtc::peer_connection::{AnswerOptions, OfferOptions, PeerConnection, SignalingState};
use libwebrtc::peer_connection_factory::{PeerConnectionFactory, RtcConfiguration};
use libwebrtc::session_description::{SdpType, SessionDescription};
use serde_json::Value;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use super::ear::push;
use super::sent::Sent;
use super::speaker::Mixer;
use super::wire::connect;
use super::{Set, State};
use crate::board::lock;
use crate::peer::engine::{Kind, Link, Made, Stage};

/// What every link of an engine shares.
pub struct Kit {
    pub factory: PeerConnectionFactory,
    /// What this window sends now.
    pub sent: Mutex<Sent>,
    /// What its sound is fed to.
    pub voice: NativeAudioSource,
    pub set: Set,
    pub handle: Handle,
    /// Where the pictures of a call are shown.
    pub film: PFilm,
    /// Where what is heard is played; none with no sound out.
    pub mixer: Option<Mixer>,
    pub state: Arc<State>,
    /// Every link's queue, to tell each when what is sent changed.
    pub link: Mutex<Vec<UnboundedSender<Ask>>>,
}

/// What a link is asked.
pub enum Ask {
    Offer,
    Accept(Kind, String),
    Candidate(Value),
    /// What this window sends changed: send what is on now.
    Wire,
    Close,
}

/// A real connection, as the mesh holds it.
pub struct RtcLink {
    ask: UnboundedSender<Ask>,
    made: Arc<Mutex<Vec<Made>>>,
}

impl RtcLink {
    /// A link to that caller, its task begun. Called inside the
    /// engine's runtime.
    pub fn begin(kit: Arc<Kit>, config: RtcConfiguration, far: &str) -> RtcLink {
        let (ask, asked) = unbounded_channel();
        let made = Arc::new(Mutex::new(Vec::new()));
        lock(&kit.link).push(ask.clone());
        tokio::spawn(work(kit, config, far.to_string(), asked, Arc::clone(&made)));
        RtcLink { ask, made }
    }
}

impl Link for RtcLink {
    fn offer(&mut self) {
        _ = self.ask.send(Ask::Offer);
    }

    fn accept(&mut self, kind: Kind, sdp: &str) {
        _ = self.ask.send(Ask::Accept(kind, sdp.to_string()));
    }

    fn candidate(&mut self, body: &Value) {
        _ = self.ask.send(Ask::Candidate(body.clone()));
    }

    fn take(&mut self) -> Vec<Made> {
        std::mem::take(&mut lock(&self.made))
    }

    fn close(&mut self) {
        _ = self.ask.send(Ask::Close);
    }
}

/// Add a way to reach the other caller, as it sent it.
async fn way(pc: &PeerConnection, body: &Value, word: &appkit::service::PTable) {
    let text = body.get(word.get("candidate")).and_then(Value::as_str).unwrap_or_default();
    let mid = body.get(word.get("mid")).and_then(Value::as_str).unwrap_or_default();
    let index = body.get(word.get("index")).and_then(Value::as_i64).unwrap_or_default() as i32;
    // An empty one only says there are no more.
    if let Some(parsed) = Some(text).filter(|text| !text.is_empty()).and_then(|text| IceCandidate::parse(mid, index, text).ok()) {
        _ = pc.add_ice_candidate(parsed).await;
    }
}

/// The link's one task: do what is asked, in order, until it is closed.
async fn work(kit: Arc<Kit>, config: RtcConfiguration, far: String, mut asked: UnboundedReceiver<Ask>, made: Arc<Mutex<Vec<Made>>>) {
    let Some(mut wired) = connect(&kit, &config, &far, &made) else { return push(&made, Made::Stage(Stage::Failed)) };
    // The ways that came before the other caller's description.
    let (mut held, mut told) = (Vec::new(), false);
    while let Some(ask) = asked.recv().await {
        match ask {
            Ask::Offer => {
                // An offer says outright that this end takes sound and
                // picture: left unsaid, this library offers to send only,
                // and turns off what the other end was already sending.
                let wanted = OfferOptions { ice_restart: false, offer_to_receive_audio: true, offer_to_receive_video: true };
                let offer = wired.pc.create_offer(wanted).await.ok();
                if let Some(offer) = offer.filter(|_| wired.pc.signaling_state() == SignalingState::Stable)
                    && wired.pc.set_local_description(offer.clone()).await.is_ok()
                {
                    push(&made, Made::Local(Kind::Offer, offer.to_string()));
                }
            }
            Ask::Accept(Kind::Offer, sdp) => {
                // An offer of this end's own is out: it gives way, on a
                // connection made anew.
                if wired.pc.signaling_state() != SignalingState::Stable {
                    wired.close();
                    let Some(anew) = connect(&kit, &config, &far, &made) else { return push(&made, Made::Stage(Stage::Failed)) };
                    (wired, told) = (anew, false);
                    lock(&made).retain(|one| !matches!(one, Made::Candidate(_) | Made::Local(..)));
                }
                let Ok(offer) = SessionDescription::parse(&sdp, SdpType::Offer) else { continue };
                if wired.pc.set_remote_description(offer).await.is_err() {
                    continue;
                }
                told = true;
                for body in held.drain(..) {
                    way(&wired.pc, &body, &kit.set.signal).await;
                }
                if let Ok(answer) = wired.pc.create_answer(AnswerOptions::default()).await
                    && wired.pc.set_local_description(answer.clone()).await.is_ok()
                {
                    push(&made, Made::Local(Kind::Answer, answer.to_string()));
                }
            }
            Ask::Accept(Kind::Answer, sdp) => {
                let Ok(answer) = SessionDescription::parse(&sdp, SdpType::Answer) else { continue };
                if wired.pc.set_remote_description(answer).await.is_ok() {
                    told = true;
                    for body in held.drain(..) {
                        way(&wired.pc, &body, &kit.set.signal).await;
                    }
                }
            }
            Ask::Candidate(body) => match told {
                true => way(&wired.pc, &body, &kit.set.signal).await,
                false => held.push(body),
            },
            Ask::Wire => wired.wire(&kit),
            Ask::Close => break,
        }
    }
    wired.close();
}
