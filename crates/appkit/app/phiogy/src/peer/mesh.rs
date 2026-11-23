//! The mesh: this caller's connections, one to each other caller in the
//! room's call. It is what the browser's side does in
//! `puikit/app/phi-kit/src/peer.ts`, so the two meet.
//!
//! Every caller connects to every other. Both ends may offer at once,
//! as they do when both come with a microphone: then the end with the
//! lesser name gives way, takes its own offer back and answers the
//! other's, and the end with the greater name lets the other's offer
//! pass. That is WebRTC's own "perfect negotiation". What a link sends
//! and takes is the engine's; who offers, who gives way and what is
//! said through the line is here, and has no WebRTC in it.

use appkit::service::PTable;
use serde_json::{Map, Value, json};

use super::engine::{Engine, Kind, Link, Made, Stage};
use crate::data;

/// Another caller, as this one has them.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Far {
    pub peer: String,
    /// Whether they say they are muted.
    pub quiet: bool,
    pub stage: Stage,
    /// How loud what is heard of them is, from nothing to 1.
    pub level: f32,
    /// How many pictures they sent, and how large the last was.
    pub seen: u64,
    pub size: (u32, u32),
    /// The stream they say a screen they share comes under.
    pub screen: String,
    /// The pictures arriving of them now, by stream.
    pub picture: Vec<Picture>,
}

/// Pictures arriving under one stream of another caller's.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Picture {
    pub stream: String,
    pub seen: u64,
    pub size: (u32, u32),
}

impl Far {
    /// Their camera, while it arrives: a picture under any stream that
    /// is not their screen's.
    pub fn camera(&self) -> Option<&Picture> {
        self.picture.iter().find(|picture| picture.stream != self.screen)
    }

    /// A screen they share, while it arrives.
    pub fn scene(&self) -> Option<&Picture> {
        self.picture.iter().find(|picture| picture.stream == self.screen)
    }
}

/// One connection, and where its two ends stand.
struct Bond {
    link: Box<dyn Link>,
    /// Whether this end gives way when both offer at once.
    polite: bool,
    /// Whether an offer of this end's is out and not answered.
    open: bool,
    /// Whether the other end's last offer was let pass.
    ignore: bool,
    /// Whether the two must agree again once the offer out is answered.
    again: bool,
    far: Far,
}

/// What is to be sent through the line: to whom, and what.
pub type Out = Vec<(String, Value)>;

/// This caller's connections.
pub struct Mesh {
    me: String,
    /// The words callers say to each other: `data/signal.tsv`.
    word: PTable,
    bond: Vec<Bond>,
    /// The servers that help find a way, as the line gave them.
    pub ice: Vec<Value>,
    quiet: bool,
    /// The stream a screen this caller shares is sent under.
    scene: String,
}

impl Mesh {
    /// The connections of a caller of that name: none yet.
    pub fn new(me: &str) -> Mesh {
        let scene = PTable::read(data::PEER).get("scene").to_string();
        Mesh { me: me.to_string(), word: PTable::read(data::SIGNAL), bond: Vec::new(), ice: Vec::new(), quiet: false, scene }
    }

    fn said(&self, name: &str, body: Value) -> Value {
        Value::Object(Map::from_iter([(self.word.get(name).to_string(), body)]))
    }

    /// How this caller is: muted or not, and the stream a screen it
    /// shares comes under, so the other end can tell it from a camera.
    fn state(&self) -> Value {
        let state = Map::from_iter([(self.word.get("quiet").to_string(), json!(self.quiet)), (self.word.get("screen").to_string(), json!(self.scene))]);
        self.said("state", Value::Object(state))
    }

    /// Who is in the call changed. While this caller is in it, it has a
    /// connection to each of the others: one who came is connected to
    /// and told how this caller is, one who left is let go. Out of the
    /// call it has none.
    pub fn sync(&mut self, roster: &[String], engine: &mut dyn Engine) -> Out {
        let inside = roster.contains(&self.me);
        let there: Vec<&String> = roster.iter().filter(|peer| inside && **peer != self.me).collect();
        self.bond.retain_mut(|bond| {
            let stays = there.contains(&&bond.far.peer);
            if !stays {
                bond.link.close();
            }
            stays
        });
        let mut out = Out::new();
        for peer in there {
            if self.bond.iter().all(|bond| bond.far.peer != *peer) {
                let far = Far { peer: peer.clone(), ..Far::default() };
                self.bond.push(Bond { link: engine.link(peer, &self.ice), polite: self.me < *peer, open: false, ignore: false, again: false, far });
                out.push((peer.clone(), self.state()));
            }
        }
        out
    }

    /// Another caller said something through the line: how they are, a
    /// description, or a way to reach them.
    pub fn hear(&mut self, from: &str, body: &Value) {
        let word = |name: &str| self.word.get(name);
        let Some(bond) = self.bond.iter_mut().find(|bond| bond.far.peer == from) else { return };
        if let Some(state) = body.get(word("state")) {
            bond.far.quiet = state.get(word("quiet")).and_then(Value::as_bool).unwrap_or(false);
            bond.far.screen = state.get(word("screen")).and_then(Value::as_str).unwrap_or_default().to_string();
        }
        let description = body.get(word("description")).map(|given| {
            let text = |name: &str| given.get(word(name)).and_then(Value::as_str).unwrap_or_default();
            (text("type") == word("offer"), text("type") == word("answer"), text("sdp").to_string())
        });
        match (description, bond.open, bond.polite) {
            // Both offered: the greater name keeps its own offer.
            (Some((true, _, _)), true, false) => bond.ignore = true,
            // An offer is taken; one of this end's own gives way to it.
            (Some((true, _, sdp)), ..) => {
                (bond.ignore, bond.open) = (false, false);
                bond.link.accept(Kind::Offer, &sdp);
            }
            (Some((_, true, sdp)), true, _) => {
                (bond.ignore, bond.open) = (false, false);
                bond.link.accept(Kind::Answer, &sdp);
            }
            // An answer to an offer that was taken back, or neither kind.
            (Some(_), ..) => {}
            (None, ..) => {
                // A way that belongs to an offer let pass leads nowhere.
                let way = body.get(word("candidate")).filter(|_| !bond.ignore);
                way.into_iter().for_each(|way| bond.link.candidate(way));
            }
        }
    }

    /// Take what every link made, and say what is to be sent for it. A
    /// link that needs to agree offers, unless an offer is out; its
    /// descriptions and its ways go to the other caller.
    pub fn turn(&mut self) -> Out {
        let mut out = Out::new();
        let word = self.word.clone();
        let said = |name: &str, body: Value| Value::Object(Map::from_iter([(word.get(name).to_string(), body)]));
        for bond in &mut self.bond {
            for made in bond.link.take() {
                match made {
                    Made::Need => bond.again = true,
                    // An offer that gave way meanwhile is not sent.
                    Made::Local(Kind::Offer, _) if !bond.open => {}
                    Made::Local(kind, sdp) => {
                        let name = match kind {
                            Kind::Offer => "offer",
                            Kind::Answer => "answer",
                        };
                        let description = Map::from_iter([(word.get("type").to_string(), json!(word.get(name))), (word.get("sdp").to_string(), json!(sdp))]);
                        out.push((bond.far.peer.clone(), said("description", Value::Object(description))));
                    }
                    Made::Candidate(way) => out.push((bond.far.peer.clone(), said("candidate", way))),
                    Made::Stage(stage) => bond.far.stage = stage,
                    Made::Level(level) => bond.far.level = level,
                    Made::Seen(stream, count, width, height) => {
                        (bond.far.seen, bond.far.size) = (bond.far.seen + 1, (width, height));
                        let shown = Picture { stream, seen: count, size: (width, height) };
                        match bond.far.picture.iter_mut().find(|picture| picture.stream == shown.stream) {
                            Some(picture) => *picture = shown,
                            None => bond.far.picture.push(shown),
                        }
                    }
                    Made::Gone(stream) => bond.far.picture.retain(|picture| picture.stream != stream),
                }
            }
            if bond.again && !bond.open {
                (bond.again, bond.open) = (false, true);
                bond.link.offer();
            }
        }
        out
    }

    /// Mute or unmute, and tell every other caller.
    pub fn hush(&mut self, quiet: bool, engine: &mut dyn Engine) -> Out {
        self.quiet = quiet;
        engine.hush(quiet);
        self.bond.iter().map(|bond| (bond.far.peer.clone(), self.state())).collect()
    }

    /// Whether this caller is muted.
    pub fn quiet(&self) -> bool {
        self.quiet
    }

    /// The other callers, as this one has them.
    pub fn far(&self) -> Vec<Far> {
        self.bond.iter().map(|bond| bond.far.clone()).collect()
    }

    /// Let every connection go.
    pub fn close(&mut self) {
        self.bond.iter_mut().for_each(|bond| bond.link.close());
        self.bond.clear();
    }
}
