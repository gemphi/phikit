//! The real engine: WebRTC itself, the library a browser has, through
//! `libwebrtc`. It is built in only with the `rtc` feature, since it
//! needs the prebuilt library on disk.
//!
//! One engine serves a call: a runtime of its own, what this window
//! sends (`sent`: its sound, its camera and a screen it shares, given
//! to every link), and a link for each other caller (`link`, `wire`).
//!
//! Where the sound and the pictures come from is `data/peer.tsv`'s
//! `device`. `system` is the system's own: its microphone (`mic`), its
//! camera (`camera`), a screen of its (`scene`), and what arrives is
//! played on its sound out (`speaker`). `stand` is a tone and a grey
//! ramp (`tone`) and plays nothing: the tests and every picture in
//! `doc/` are made with it, so nothing of a person's room is kept.
//! What arrives is measured and shown (`ear`): each other caller's
//! pictures go to the window's film, under their names.

mod camera;
mod ear;
mod link;
mod mic;
mod scene;
mod sent;
mod speaker;
mod tone;
mod wire;

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use appkit::pui::core::PFilm;
use appkit::service::PTable;
use libwebrtc::peer_connection_factory::{ContinualGatheringPolicy, IceServer, IceTransportsType, PeerConnectionFactory, RtcConfiguration};
use serde_json::Value;
use tokio::runtime::Runtime;

use super::engine::{Engine, Link, Maker, Own};
use crate::board::lock;
use crate::data;
use link::Kit;
use sent::{Hold, Sent, Slot};

/// The numbers and names the engine works by: `data/peer.tsv`, and the
/// words of a candidate from `data/signal.tsv`. A row may be given
/// another value for one run (`--set NAME=VALUE`).
#[derive(Debug, Clone)]
pub struct Set {
    peer: PTable,
    pub signal: PTable,
    over: Vec<(String, String)>,
}

impl Set {
    pub fn own(over: &[(String, String)]) -> Set {
        Set { peer: PTable::read(data::PEER), signal: PTable::read(data::SIGNAL), over: over.to_vec() }
    }

    /// A number of the peer's table; nothing for one it does not have.
    pub fn count(&self, name: &str) -> f32 {
        self.word(name).parse().unwrap_or_default()
    }

    pub fn word(&self, name: &str) -> &str {
        let given = self.over.iter().rev().find(|(row, _)| row == name);
        given.map_or_else(|| self.peer.get(name), |(_, value)| value.as_str())
    }

    /// Whether the sound and the pictures are the system's own.
    pub fn real(&self) -> bool {
        self.word("device") == self.word("device.system")
    }
}

/// How this caller's own side stands, for every thread that feeds it.
#[derive(Debug, Default)]
pub struct State {
    pub quiet: AtomicBool,
    pub deaf: AtomicBool,
    /// How loud the microphone is, as the bits of a share of full.
    level: AtomicU32,
    /// What the system did not give, each with why.
    fault: Mutex<Vec<(String, String)>>,
}

impl State {
    pub fn level(&self, level: f32) {
        self.level.store(level.to_bits(), Ordering::Relaxed);
    }

    /// The system did not give what was asked for under that name, and
    /// why; or, with no why, it now has.
    pub fn fault(&self, name: &str, why: Option<&String>) {
        let mut fault = lock(&self.fault);
        fault.retain(|(one, _)| one != name);
        fault.extend(why.map(|why| (name.to_string(), why.clone())));
    }
}

/// WebRTC, for one call.
pub struct Rtc {
    kit: Arc<Kit>,
    name: String,
    /// The system's microphone, camera and screen, while each is on,
    /// and its sound out: letting one go closes it.
    mic: Option<Hold>,
    camera: Option<Hold>,
    scene: Option<Hold>,
    _speaker: Option<speaker::Speaker>,
    /// Dropped last, so everything it runs is over before it goes.
    runtime: Runtime,
}

impl Rtc {
    /// The engine, its sound begun. `None` when the system gives it no
    /// threads to run on.
    pub fn new(set: Set, film: PFilm) -> Option<Rtc> {
        let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().ok()?;
        let state = Arc::new(State::default());
        let factory = PeerConnectionFactory::default();
        let real = set.real();
        // What arrives is played where the system has a sound out.
        let played = real.then(|| speaker::Speaker::open(&set));
        state.fault(set.word("fault.speaker"), played.as_ref().and_then(|played| played.as_ref().err()));
        let (speaker, mixer) = played.and_then(Result::ok).unzip();
        // The sound is sent as the microphone gives it, where there is one.
        let rate = real.then(mic::rate).flatten().unwrap_or(set.count("rate") as u32);
        let (voice, sent) = Sent::begin(&factory, &set, rate);
        let name = set.word("engine").to_string();
        let kit = Arc::new(Kit { factory, sent: Mutex::new(sent), voice, set, handle: runtime.handle().clone(), film, mixer, state, link: Mutex::default() });
        let mut engine = Rtc { kit, name, mic: None, camera: None, scene: None, _speaker: speaker, runtime };
        engine.listen(true);
        Some(engine)
    }

    /// Open what feeds the sound, or let it go. The stand-in is a tone
    /// that falls silent while muted.
    fn listen(&mut self, on: bool) {
        let kit = &self.kit;
        match (kit.set.real(), on) {
            (true, true) => {
                let opened = mic::open(kit);
                kit.state.fault(kit.set.word("fault.mic"), opened.as_ref().err());
                self.mic = opened.ok();
            }
            (true, false) => {
                self.mic = None;
                kit.state.level(0.0);
            }
            // The stand-in runs for the whole call.
            (false, _) if self.mic.is_none() => {
                let _inside = self.runtime.enter();
                self.mic = Some(Hold::task(tokio::spawn(tone::tone(Arc::clone(kit)))));
            }
            (false, _) => {}
        }
    }

    /// Put a picture on what is sent, or take it off, and have every
    /// link send what is on now.
    fn put(&self, slot: Slot, track: Option<libwebrtc::video_track::RtcVideoTrack>) {
        lock(&self.kit.sent).put(slot, track);
        lock(&self.kit.link).retain(|link| link.send(link::Ask::Wire).is_ok());
    }
}

/// The servers of the line as the library takes them: each one's urls,
/// one or several, and what it wants to be told.
fn server(ice: &[Value], set: &Set) -> Vec<IceServer> {
    let text = |value: &Value, name: &str| value.get(set.word(name)).and_then(Value::as_str).unwrap_or_default().to_string();
    let urls = |value: &Value| match value.get(set.word("ice.urls")) {
        Some(Value::String(one)) => vec![one.clone()],
        Some(Value::Array(list)) => list.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        _ => Vec::new(),
    };
    ice.iter().map(|one| IceServer { urls: urls(one), username: text(one, "ice.username"), password: text(one, "ice.credential") }).collect()
}

impl Engine for Rtc {
    fn link(&mut self, far: &str, ice: &[Value]) -> Box<dyn Link> {
        let config = RtcConfiguration { ice_servers: server(ice, &self.kit.set), continual_gathering_policy: ContinualGatheringPolicy::GatherContinually, ice_transport_type: IceTransportsType::All };
        let _inside = self.runtime.enter();
        Box::new(link::RtcLink::begin(Arc::clone(&self.kit), config, far))
    }

    /// Muted, the system's microphone is closed: nothing of the room is
    /// taken, and nothing is sent.
    fn hush(&mut self, quiet: bool) {
        self.kit.state.quiet.store(quiet, Ordering::SeqCst);
        if self.kit.set.real() {
            self.listen(!quiet);
        }
    }

    /// Off, the camera is closed and its picture taken off every link.
    fn camera(&mut self, on: bool) {
        self.camera = None;
        let _inside = self.runtime.enter();
        let opened = on.then(|| match self.kit.set.real() {
            true => camera::open(&self.kit),
            false => Ok(tone::ramp(&self.kit, Slot::Picture)),
        });
        self.kit.state.fault(self.kit.set.word("fault.camera"), opened.as_ref().and_then(|opened| opened.as_ref().err()));
        let (hold, track) = opened.and_then(Result::ok).unzip();
        self.camera = hold;
        self.put(Slot::Picture, track);
    }

    fn share(&mut self, on: bool) {
        self.scene = None;
        let _inside = self.runtime.enter();
        let opened = on.then(|| match self.kit.set.real() {
            true => scene::open(&self.kit),
            false => Ok(tone::ramp(&self.kit, Slot::Scene)),
        });
        self.kit.state.fault(self.kit.set.word("fault.scene"), opened.as_ref().and_then(|opened| opened.as_ref().err()));
        let (hold, track) = opened.and_then(Result::ok).unzip();
        self.scene = hold;
        self.put(Slot::Scene, track);
    }

    fn deaf(&mut self, on: bool) {
        self.kit.state.deaf.store(on, Ordering::SeqCst);
        self.kit.mixer.iter().for_each(|mixer| mixer.deaf.store(on, Ordering::SeqCst));
    }

    fn own(&self) -> Own {
        let state = &self.kit.state;
        let sent = lock(&self.kit.sent);
        let (fault, why) = lock(&state.fault).iter().cloned().unzip();
        Own { camera: sent.has(Slot::Picture), screen: sent.has(Slot::Scene), deaf: state.deaf.load(Ordering::SeqCst), level: f32::from_bits(state.level.load(Ordering::Relaxed)), fault, why }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for Rtc {
    /// The call is over: every picture of it is.
    fn drop(&mut self) {
        self.kit.film.clear(self.kit.set.word("film"));
    }
}

/// What makes the real engine on a call's own thread, its pictures
/// shown on that film; the stand-in where it cannot be made.
pub fn maker(over: Vec<(String, String)>, film: PFilm) -> Maker {
    Box::new(move || match Rtc::new(Set::own(&over), film) {
        Some(engine) => Box::new(engine) as Box<dyn Engine>,
        None => Box::new(super::Stand::default()),
    })
}
