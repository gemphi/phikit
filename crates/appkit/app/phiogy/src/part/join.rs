//! The join: the pills at the right of a room's title, with which this
//! window is a caller itself. Out of the room's call there is one,
//! Join. In it there are two: Mute, which reads Unmute while muted, and
//! Leave. Each is a label lit from the left, the one that says what is
//! so the strongest. What is drawn is what can be pressed.

use std::cell::RefCell;

use appkit::pui::core::{PLight, PPaint, PPoint, PRect, light};

use super::{Desk, GAP, INK, LINE};
use crate::peer::{self, Call, Seen};

/// The theme's token for how round a pill is.
pub const ROUND: &str = "tab.radius";

/// What a press on a pill does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Press {
    Join,
    Leave,
    Hush(bool),
}

/// The pills of a room's call.
pub struct Join {
    desk: Desk,
    /// Where each pill was drawn last, in the window.
    spot: RefCell<Vec<(PRect, Press)>>,
}

impl Join {
    pub fn new(desk: Desk) -> Join {
        Join { desk, spot: RefCell::default() }
    }

    /// The pills there are now, from the left: what each reads, what a
    /// press does, and whether it says what is so.
    fn pill(&self) -> Vec<(String, Press, bool)> {
        let text = |name: &str| self.desk.text.get(name).to_string();
        match self.desk.calling() {
            Some(seen) => {
                let hush = match seen.quiet {
                    true => (text("call.unmute"), Press::Hush(false), true),
                    false => (text("call.mute"), Press::Hush(true), false),
                };
                vec![hush, (text("call.leave"), Press::Leave, false)]
            }
            None => vec![(text("call.join"), Press::Join, false)],
        }
    }

    /// Draw the pills so that the last ends at `end`, their middle at
    /// its height. How wide they are together; nothing where the line
    /// is not open, since there is no call to join.
    pub fn draw(&self, paint: &mut dyn PPaint, end: PPoint) -> f32 {
        let (gap, line, round, ink) = (paint.size(GAP), paint.size(LINE), paint.size(ROUND), paint.color(INK));
        let mut spot = Vec::new();
        let mut right = end.x;
        for (label, press, so) in self.pill().into_iter().rev().filter(|_| self.desk.line.is_ok()) {
            let rect = PRect::new(right - paint.measure(&label) - gap * 2.0, end.y - (line + gap) / 2.0, paint.measure(&label) + gap * 2.0, line + gap);
            let strength = match so {
                true => PLight::Strong,
                false => PLight::Rest,
            };
            light::label(paint, rect, round, strength, false, "");
            paint.text(PPoint::new(rect.origin.x + gap, rect.center().y - line / 2.0), &label, ink);
            spot.push((rect, press));
            right = rect.origin.x - gap / 2.0;
        }
        *self.spot.borrow_mut() = spot;
        end.x - right
    }

    /// A press at a place in the window. `true` when it was on a pill.
    pub fn press(&self, at: PPoint) -> bool {
        let press = self.spot.borrow().iter().find(|(rect, _)| rect.contains(at)).map(|(_, press)| *press);
        match press {
            Some(Press::Join) => self.desk.join(),
            Some(Press::Leave) => self.desk.leave(),
            Some(Press::Hush(quiet)) => self.desk.hush(quiet),
            None => return false,
        }
        true
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
