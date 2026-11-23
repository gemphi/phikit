//! A stand-in engine: links that agree as real ones do and carry
//! nothing. It makes an offer when asked, answers one it is given, and
//! is connected once it holds a description of each end. The rules are
//! tested with it, and it is what a window with no WebRTC is given: it
//! joins a call and is seen in it, and sends and hears nothing.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serde_json::Value;

use super::engine::{Engine, Kind, Link, Made, Own, Stage};

/// What a stand-in engine is called.
pub const NAME: &str = "stand-in";

/// A link that agrees and carries nothing.
#[derive(Debug, Default)]
struct StandLink {
    made: Vec<Made>,
    /// Whether it holds an offer of its own that was not answered.
    out: bool,
    count: usize,
    /// How often an offer of its own was taken back, kept for whoever
    /// made the engine.
    back: Arc<AtomicUsize>,
}

impl Link for StandLink {
    fn offer(&mut self) {
        self.count += 1;
        self.out = true;
        self.made.push(Made::Local(Kind::Offer, format!("offer {}", self.count)));
        self.made.push(Made::Stage(Stage::Connecting));
    }

    fn accept(&mut self, kind: Kind, sdp: &str) {
        match (kind, self.out) {
            (Kind::Offer, out) => {
                // An offer of its own is taken back, and wanted again
                // once this one is answered.
                self.back.fetch_add(out as usize, Ordering::SeqCst);
                self.out = false;
                self.made.push(Made::Local(Kind::Answer, format!("answer to {sdp}")));
                self.made.push(Made::Stage(Stage::Connected));
                self.made.extend(out.then_some(Made::Need));
            }
            (Kind::Answer, true) => {
                self.out = false;
                self.made.push(Made::Stage(Stage::Connected));
            }
            (Kind::Answer, false) => {}
        }
    }

    fn candidate(&mut self, _body: &Value) {}

    fn take(&mut self) -> Vec<Made> {
        std::mem::take(&mut self.made)
    }

    fn close(&mut self) {}
}

/// An engine of stand-in links.
#[derive(Debug, Default)]
pub struct Stand {
    /// How often a link took an offer of its own back.
    pub back: Arc<AtomicUsize>,
    /// What it was asked to turn on: it has neither, and remembers.
    own: Own,
}

impl Engine for Stand {
    /// A new link has something to send from the start, as a real one
    /// with a microphone does: it needs to agree.
    fn link(&mut self, _far: &str, _ice: &[Value]) -> Box<dyn Link> {
        Box::new(StandLink { made: vec![Made::Need], back: Arc::clone(&self.back), ..StandLink::default() })
    }

    fn camera(&mut self, on: bool) {
        self.own.camera = on;
    }

    fn share(&mut self, on: bool) {
        self.own.screen = on;
    }

    fn deaf(&mut self, on: bool) {
        self.own.deaf = on;
    }

    fn own(&self) -> Own {
        self.own.clone()
    }

    fn name(&self) -> &str {
        NAME
    }
}
