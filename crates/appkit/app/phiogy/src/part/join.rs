//! The join: the pill at the right of a room's title while somebody
//! else is in a call there and this window is not, as the web's room
//! has it beside its name: Join. With nobody calling, the title's
//! tools begin a call (`title`); in a call, its buttons are the call's
//! own (`call`). A voice room has its way in on its own page (`voice`).
//!
//! And what the desk does for a call: begin, join, leave, mute, the
//! camera, a screen, the sound of the others, and who else is calling.

use std::cell::RefCell;

use appkit::pui::block::PPill;
use appkit::pui::core::{PPaint, PPoint, PRect};

use super::{Desk, GAP};
use crate::peer::{self, Call, Ring, Seen};


/// The pill of a room's call.
pub struct Join {
    desk: Desk,
    /// Where it was drawn last, in the window.
    spot: RefCell<Option<PRect>>,
}

impl Join {
    pub fn new(desk: Desk) -> Join {
        Join { desk, spot: RefCell::default() }
    }

    /// Draw the pill so that it ends at `end`, its middle at that
    /// height. How wide it is; nothing where nobody is calling.
    pub fn draw(&self, paint: &mut dyn PPaint, end: PPoint) -> f32 {
        let gap = paint.size(GAP);
        let label = self.desk.text.get("call.join").to_string();
        // The kit's pill, lit: it is what to press now.
        let pill = ("", label.as_str(), true);
        let size = pill.size(paint);
        let rect = PRect::new(end.x - size.width, end.y - size.height / 2.0, size.width, size.height);
        let shown = self.desk.ringing().map(|_| rect);
        if shown.is_some() {
            pill.draw(paint, rect.origin, false);
        }
        *self.spot.borrow_mut() = shown;
        shown.map_or(0.0, |rect| rect.size.width + gap / 2.0)
    }

    /// Where the pill is, while it is drawn.
    pub fn hands(&self) -> Vec<PRect> {
        self.spot.borrow().iter().copied().collect()
    }

    /// A press at a place in the window. `true` when it was on the pill.
    pub fn press(&self, at: PPoint) -> bool {
        let on = self.spot.borrow().is_some_and(|rect| rect.contains(at));
        if on {
            self.desk.join();
        }
        on
    }
}

impl Desk {
    /// The call this window is in, when it is the open room's and is
    /// not over: what it has seen.
    pub fn calling(&self) -> Option<Seen> {
        let call = self.call.borrow();
        let here = call.as_ref().filter(|call| call.key == self.key(&self.room()))?;
        Some(here.seen()).filter(|seen| seen.over.is_none())
    }

    /// What this window calls itself in the call it is in.
    pub fn me(&self) -> Option<String> {
        self.call.borrow().as_ref().map(|call| call.peer.clone())
    }

    /// Whether the open room is one people are in, not one they write in.
    pub fn voice(&self) -> bool {
        self.plate(&self.room()).is_some_and(|plate| plate.voice)
    }

    /// The voice room this window is in, whichever page is open: its
    /// id, what it is called, and what its call has seen.
    pub fn within(&self) -> Option<(String, String, Seen)> {
        let call = self.call.borrow();
        let call = call.as_ref()?;
        let id = call.key.strip_prefix(&self.key(""))?.to_string();
        let plate = self.plate(&id).filter(|plate| plate.voice)?;
        Some((id, plate.label, call.seen())).filter(|(.., seen)| seen.over.is_none())
    }

    /// Who else is in the open room's call: what each is called on the
    /// line, and what they call themself. The board says, where this
    /// window serves the line; the ring, where it is on another's.
    pub fn other(&self) -> Vec<(String, String)> {
        let (room, me) = (self.room(), self.me());
        let (roster, _) = self.on(&room);
        let near = roster.list.into_iter().map(|(peer, caller)| (peer, caller.name));
        let far = self.ring.borrow().as_ref().filter(|ring| ring.key == self.key(&room)).map(Ring::caller).unwrap_or_default();
        near.chain(far).filter(|(peer, _)| Some(peer) != me.as_ref()).collect()
    }

    /// Who is calling in the open room, when somebody is and this
    /// window is not in that call: the first of them. Never in a voice
    /// room, which nobody is rung into.
    pub fn ringing(&self) -> Option<String> {
        let free = self.line.is_ok() && self.calling().is_none() && !self.voice();
        self.other().into_iter().next().map(|(_, name)| name).filter(|_| free)
    }

    /// Whether a call can be begun in the open room: it is a room, the
    /// line is open, and nobody is in a call there.
    pub fn callable(&self) -> bool {
        self.line.is_ok() && self.plate(&self.room()).is_some() && !self.voice() && self.calling().is_none() && self.other().is_empty()
    }

    /// On another's line, listen to the open room's while this window
    /// is not in its call, so a call there is seen; to none otherwise.
    pub fn listen(&self) {
        let room = self.room();
        let key = self.key(&room);
        let wanted = self.away && self.calling().is_none() && self.plate(&room).is_some();
        let held = self.ring.borrow().as_ref().is_some_and(|ring| ring.key == key);
        match (wanted, held) {
            (true, false) => *self.ring.borrow_mut() = self.line.as_ref().as_ref().ok().zip(peer::name()).map(|(at, name)| Ring::open(at, &key, &name)),
            (false, _) => _ = self.ring.borrow_mut().take(),
            (true, true) => {}
        }
    }

    /// Begin a call in the open room, with the camera or without.
    pub fn dial(&self, video: bool) {
        self.join();
        self.camera(video);
    }

    /// Join the open room's call, leaving any other. Where the line is
    /// not open or refuses, the reason is kept for the room to say.
    pub fn join(&self) {
        self.leave();
        let (Ok(at), Some(name)) = (self.line.as_ref(), peer::name()) else { return };
        let key = self.key(&self.room());
        match Call::start(at, &key, &name, (self.engine)()) {
            Ok(call) => *self.call.borrow_mut() = Some(call),
            Err(error) => *self.refused.borrow_mut() = error.to_string(),
        }
    }

    /// Leave the call this window is in.
    pub fn leave(&self) {
        self.call.borrow_mut().take();
        self.refused.borrow_mut().clear();
    }

    /// Mute, or unmute.
    pub fn hush(&self, quiet: bool) {
        self.call.borrow().iter().for_each(|call| call.hush(quiet));
    }

    /// Turn the camera on, or off.
    pub fn camera(&self, on: bool) {
        self.call.borrow().iter().for_each(|call| call.camera(on));
    }

    /// Hear nobody, or hear them again.
    pub fn deaf(&self, on: bool) {
        self.call.borrow().iter().for_each(|call| call.deaf(on));
    }

    /// Share a screen, or stop.
    pub fn share(&self, on: bool) {
        self.call.borrow().iter().for_each(|call| call.share(on));
    }

    /// How many seconds the call this window is in has been on.
    pub fn since(&self) -> Option<u64> {
        self.call.borrow().as_ref().map(|call| call.since.elapsed().as_secs())
    }

    /// Why this window is not in the open room's call though it asked
    /// to be, in a person's words; nothing when there is no reason.
    pub fn why(&self) -> String {
        let refused = self.refused.borrow();
        let call = self.call.borrow();
        let over = call.as_ref().filter(|call| call.key == self.key(&self.room())).and_then(|call| call.seen().over);
        match (refused.is_empty(), over) {
            (false, _) => self.text.fill("call.refused", "why", &refused),
            (true, Some(status)) => match self.text.get(&format!("call.over.{status}")) {
                "" => self.text.get("call.over").to_string(),
                said => said.to_string(),
            },
            (true, None) => String::new(),
        }
    }
}
