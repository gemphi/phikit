//! What this window sends: its sound always, its camera while that is
//! on, and a screen while one is shared. The same is given to every
//! link. The sound and the camera go under one stream, as a page's do;
//! the screen under a stream of its own, so the other end can tell it
//! from a camera.
//!
//! A picture put on or taken off is a new turn: a link compares the
//! turn of what it sends with what is on, and sends exactly that.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use appkit::pui::core::PFilm;
use libwebrtc::audio_source::AudioSourceOptions;
use libwebrtc::audio_source::native::NativeAudioSource;
use libwebrtc::audio_track::RtcAudioTrack;
use libwebrtc::media_stream_track::MediaStreamTrack;
use libwebrtc::peer_connection_factory::PeerConnectionFactory;
use libwebrtc::peer_connection_factory::native::PeerConnectionFactoryExt;
use libwebrtc::video_track::RtcVideoTrack;

use super::Set;

/// The three things a caller sends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Sound,
    Picture,
    Scene,
}

impl Slot {
    /// The row of `data/peer.tsv` that names the stream it goes under.
    pub fn stream(self) -> &'static str {
        match self {
            Slot::Sound | Slot::Picture => "stream",
            Slot::Scene => "scene",
        }
    }

    /// The row that names its track.
    pub fn track(self) -> &'static str {
        match self {
            Slot::Sound => "sound",
            Slot::Picture => "picture",
            Slot::Scene => "scene.picture",
        }
    }
}

/// What this window sends now.
pub struct Sent {
    sound: RtcAudioTrack,
    picture: Option<(u64, RtcVideoTrack)>,
    scene: Option<(u64, RtcVideoTrack)>,
    turn: u64,
}

impl Sent {
    /// The sound, at that many samples a second, with nothing feeding
    /// it yet: what feeds it is given the source.
    pub fn begin(factory: &PeerConnectionFactory, set: &Set, rate: u32) -> (NativeAudioSource, Sent) {
        let (channel, piece) = (set.count("channel") as u32, set.count("piece") as u32);
        let source = NativeAudioSource::new(AudioSourceOptions::default(), rate, channel, piece * 10);
        let sound = factory.create_audio_track(set.word(Slot::Sound.track()), source.clone());
        (source, Sent { sound, picture: None, scene: None, turn: 0 })
    }

    /// Put a picture on, or take it off.
    pub fn put(&mut self, slot: Slot, track: Option<RtcVideoTrack>) {
        self.turn += 1;
        let held = track.map(|track| (self.turn, track));
        match slot {
            Slot::Picture => self.picture = held,
            Slot::Scene => self.scene = held,
            Slot::Sound => {}
        }
    }

    pub fn has(&self, slot: Slot) -> bool {
        match slot {
            Slot::Sound => true,
            Slot::Picture => self.picture.is_some(),
            Slot::Scene => self.scene.is_some(),
        }
    }

    /// What is on now: each with its turn.
    pub fn want(&self) -> Vec<(Slot, u64, MediaStreamTrack)> {
        let picture = |slot: Slot, held: &Option<(u64, RtcVideoTrack)>| held.as_ref().map(|(turn, track)| (slot, *turn, MediaStreamTrack::Video(track.clone())));
        let sound = Some((Slot::Sound, 0, MediaStreamTrack::Audio(self.sound.clone())));
        sound.into_iter().chain(picture(Slot::Picture, &self.picture)).chain(picture(Slot::Scene, &self.scene)).collect()
    }
}

/// What keeps a source of sound or of pictures running. Let go, the
/// source stops, the system's device behind it is closed, and the
/// picture it showed of itself in the window is over.
pub struct Hold {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
    task: Option<tokio::task::JoinHandle<()>>,
    shown: Option<(PFilm, String)>,
}

impl Hold {
    /// A source that runs as a task of the engine's runtime.
    pub fn task(task: tokio::task::JoinHandle<()>) -> Hold {
        Hold { stop: Arc::default(), thread: None, task: Some(task), shown: None }
    }

    /// A source on a thread of its own, which ends when `stop` is set.
    pub fn thread(stop: Arc<AtomicBool>, thread: std::thread::JoinHandle<()>) -> Hold {
        Hold { stop, thread: Some(thread), task: None, shown: None }
    }

    /// It shows itself in the window under that name.
    pub fn shown(mut self, film: &PFilm, name: String) -> Hold {
        self.shown = Some((film.clone(), name));
        self
    }

    /// It has a task besides, which ends with it.
    pub fn with(mut self, task: tokio::task::JoinHandle<()>) -> Hold {
        self.task = Some(task);
        self
    }
}

impl Drop for Hold {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.task.iter().for_each(tokio::task::JoinHandle::abort);
        // The thread has the device: it is closed before this returns.
        _ = self.thread.take().map(std::thread::JoinHandle::join);
        self.shown.iter().for_each(|(film, name)| film.end(name));
    }
}

/// The name a picture has in the window's film: whose it is, and the
/// stream it comes under.
pub fn film(set: &Set, who: &str, stream: &str) -> String {
    crate::peer::shown(set.word("film"), who, stream)
}
