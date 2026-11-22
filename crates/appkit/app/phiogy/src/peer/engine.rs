//! The engine: what makes a connection to another caller, which is
//! WebRTC's part. The rules of who offers and who gives way (`mesh.rs`)
//! speak to it through these two traits and know nothing else of it, so
//! they are tested with a stand-in (`stand.rs`) and run on the real
//! library (`rtc/`) unchanged.
//!
//! A link works by itself, on threads of its own: what it is asked is
//! begun and not waited for, and what it made is taken later.

use serde_json::Value;

/// The two kinds of description one caller gives another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Offer,
    Answer,
}

/// How far a connection has come.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Stage {
    #[default]
    New,
    Connecting,
    Connected,
    Failed,
}

/// What a link made since it was last asked.
#[derive(Debug, Clone, PartialEq)]
pub enum Made {
    /// What it sends changed: the two ends must agree again.
    Need,
    /// A description of its own, to send to the other caller.
    Local(Kind, String),
    /// A way to reach it, to send to the other caller.
    Candidate(Value),
    /// How far the connection has come.
    Stage(Stage),
    /// How loud what it hears is, from nothing to 1.
    Level(f32),
    /// A picture came under the stream of that name: how many so far
    /// under it, and the size of this one.
    Seen(String, u64, u32, u32),
    /// No picture comes under that stream any more.
    Gone(String),
}

/// This caller's own side, as the engine has it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Own {
    /// Whether its camera is on, and whether it shares a screen.
    pub camera: bool,
    pub screen: bool,
    /// Whether it plays nothing of what it hears.
    pub deaf: bool,
    /// How loud its microphone hears it, from nothing to 1.
    pub level: f32,
    /// What it asked the system for and was not given, by name: the
    /// rows `call.fault.NAME` of the text table say each in words.
    pub fault: Vec<String>,
    /// Why each was not given, in the same order: the system's own
    /// words, or the peer table's where the system said nothing.
    pub why: Vec<String>,
}

/// One connection to one other caller.
pub trait Link: Send {
    /// Begin an offer. It arrives as `Made::Local`.
    fn offer(&mut self);

    /// Take the other caller's description. An offer is answered: the
    /// answer arrives as `Made::Local`. One that arrives while an offer
    /// of this link's own is out takes that offer back first.
    fn accept(&mut self, kind: Kind, sdp: &str);

    /// Take a way to reach the other caller, as it sent it.
    fn candidate(&mut self, body: &Value);

    /// What it made since this was last asked, in order.
    fn take(&mut self) -> Vec<Made>;

    /// Let the connection go.
    fn close(&mut self);
}

/// What makes links, and holds what every link sends: this caller's
/// sound and picture.
pub trait Engine: Send {
    /// A connection to that caller, told which servers help find a way.
    fn link(&mut self, far: &str, ice: &[Value]) -> Box<dyn Link>;

    /// Send no sound, or send it again.
    fn hush(&mut self, _quiet: bool) {}

    /// Turn the camera on, or off.
    fn camera(&mut self, _on: bool) {}

    /// Share a screen, or stop.
    fn share(&mut self, _on: bool) {}

    /// Play nothing of what is heard, or play it again.
    fn deaf(&mut self, _on: bool) {}

    /// How this caller's own side stands.
    fn own(&self) -> Own {
        Own::default()
    }

    /// What it is called where a person reads what carries the call.
    fn name(&self) -> &str;
}

/// What makes an engine, on the thread that will use it.
pub type Maker = Box<dyn FnOnce() -> Box<dyn Engine> + Send>;
