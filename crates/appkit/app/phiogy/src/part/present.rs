//! Who is present: everyone on the open room's line, as the board has
//! them now. First who is in the room's call, then who has the room
//! open and is not in it; and under them the room's own people, as the
//! site names them. With nobody on the line it says so, and where a
//! page reaches it.
//!
//! The line runs on other threads, so the board is read each time this
//! is drawn, and another frame is asked for while the line is open.

use std::cell::{Cell, RefCell};

use appkit::pui::core::{PDraw, PEvent, PLight, PPaint, PPoint, PRect, PView, light};
use appkit::pui::layout::{PScroll, fold, section};

use super::{DIM, Desk, GAP, ICON, INK, LINE, crowd, fit};
use crate::comp::Roster;
use crate::peer::Stage;

/// The theme's tokens for one who is on the line: how high their row
/// is, how round its light, and how wide the mark before their name.
pub const ROW: &str = "present.row";
pub const RADIUS: &str = "present.radius";
pub const MARK: &str = "present.mark";
/// How wide the bar that shows how loud a caller is at its loudest, and
/// how high.
pub const METER: &str = "present.meter";
pub const BAR: &str = "present.bar";
/// The icons of one who is in the call, and of one who only has the
/// room open.
pub const CALL: &str = "mic";
pub const WATCH: &str = "eye";

/// Everyone on the open room's line.
pub struct Present {
    frame: PRect,
    desk: Desk,
    scroll: Cell<PScroll>,
    /// Where each member's card was drawn last, in the window, and
    /// where the column was.
    spot: RefCell<Vec<(PRect, String)>>,
    origin: Cell<PPoint>,
}

impl Present {
    pub fn new(desk: Desk) -> Present {
        Present { frame: PRect::default(), desk, scroll: Cell::default(), spot: RefCell::default(), origin: Cell::default() }
    }

    /// A heading over some rows; how far down what follows begins.
    fn head(&self, paint: &mut dyn PPaint, at: PPoint, name: &str) -> f32 {
        let (gap, line, dim) = (paint.size(GAP), paint.size(LINE), paint.color(DIM));
        paint.text(at, self.desk.text.get(name), dim);
        at.y + line + gap / 2.0
    }

    /// The rows of some who are on the line, each a label lit from the
    /// left; how far down what follows begins.
    fn rows(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32, roster: &Roster, icon: &str, strength: PLight) -> f32 {
        let (gap, line, row, radius, mark, glyph) = (paint.size(GAP), paint.size(LINE), paint.size(ROW), paint.size(RADIUS), paint.size(MARK), paint.size(ICON));
        let (ink, dim) = (paint.color(INK), paint.color(DIM));
        for (index, (peer, caller)) in roster.list.iter().enumerate() {
            let rect = PRect::new(at.x, at.y + index as f32 * (row + gap / 2.0), wide, row);
            light::label(paint, rect, radius, strength, false, "");
            let seat = PRect::new(rect.origin.x + gap / 2.0, rect.center().y - mark / 2.0, mark, mark);
            paint.icon(icon, PRect::new(seat.center().x - glyph / 2.0, seat.center().y - glyph / 2.0, glyph, glyph), ink);
            let left = seat.origin.x + mark + gap / 2.0;
            let name = caller.handle.as_deref().map_or(caller.name.clone(), |handle| format!("{} {} {handle}", caller.name, self.desk.text.get("between")));
            // What this window's own call knows of them, at the right.
            let (said, bar) = self.known(paint, rect, peer);
            let room = rect.origin.x + wide - left - gap;
            paint.text(PPoint::new(left, rect.center().y - line), &fit(&name, room - said, paint), ink);
            paint.text(PPoint::new(left, rect.center().y), &fit(peer, room - bar, paint), dim);
        }
        at.y + roster.len() as f32 * (row + gap / 2.0) + gap / 2.0
    }

    /// What this window's call knows of a caller, set at the right of
    /// their row: that it is this window; or how far the connection to
    /// them has come, whether they are muted, how many pictures they
    /// sent, and under that a bar as long as they are loud. How much of
    /// the row's first line it takes, and how much of its second.
    fn known(&self, paint: &mut dyn PPaint, rect: PRect, peer: &str) -> (f32, f32) {
        let (gap, line, dim) = (paint.size(GAP), paint.size(LINE), paint.color(DIM));
        let (text, between) = (&self.desk.text, format!(" {} ", self.desk.text.get("between")));
        let Some(seen) = self.desk.calling() else { return (0.0, 0.0) };
        let far = seen.far.iter().find(|far| far.peer == peer);
        let said = match (far, self.desk.me().as_deref() == Some(peer)) {
            (_, true) => text.get("call.here").to_string(),
            (Some(far), _) => {
                let stage = match far.stage {
                    Stage::New => "call.new",
                    Stage::Connecting => "call.connecting",
                    Stage::Connected => "call.connected",
                    Stage::Failed => "call.failed",
                };
                let muted = Some(text.get("call.muted").to_string()).filter(|_| far.quiet);
                let pictures = Some(text.fill("call.seen", "count", &far.seen.to_string())).filter(|_| far.seen > 0);
                std::iter::once(text.get(stage).to_string()).chain(muted).chain(pictures).collect::<Vec<_>>().join(&between)
            }
            (None, false) => return (0.0, 0.0),
        };
        let wide = paint.measure(&said);
        let right = rect.origin.x + rect.size.width - gap;
        paint.text(PPoint::new(right - wide, rect.center().y - line), &said, dim);
        // The bar: under what is said, as long as they are loud.
        let (most, high) = (paint.size(METER), paint.size(BAR));
        let loud = far.map_or(0.0, |far| far.level.clamp(0.0, 1.0)) * most;
        if loud >= 1.0 {
            light::label(paint, PRect::new(right - most, rect.center().y + (line - high) / 2.0, loud, high), high / 2.0, PLight::Strong, false, "");
        }
        (wide + gap, far.map_or(0.0, |_| most + gap))
    }

    /// Lines of text broken to fit; how far down what follows begins.
    fn lines(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32, text: &str, ink: &str) -> f32 {
        let (line, color) = (paint.size(LINE), paint.color(ink));
        // A word too long for a line, as an address is, is cut to fit.
        let set = fold(text, wide, &|text| paint.measure(text));
        for (index, text) in set.iter().enumerate() {
            paint.text(PPoint::new(at.x, at.y + index as f32 * line), text, color);
        }
        at.y + set.len() as f32 * line
    }
}

impl PDraw for Present {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, line, ink) = (paint.size(GAP), paint.size("head.line"), paint.color(INK));
        let (start, wide) = self.desk.column(self.frame.size.width);
        let open = self.desk.room();
        let text = &self.desk.text;
        let scroll = self.scroll.get();
        self.origin.set(origin);
        paint.clip(PRect { origin, size: self.frame.size });
        let left = origin.x + start;
        let top = origin.y + gap - scroll.at();
        let Some(plate) = self.desk.plate(&open) else {
            let end = self.lines(paint, PPoint::new(left, top), wide, text.get("present.nowhere"), DIM);
            paint.unclip();
            return self.fitted(end - top + gap);
        };
        paint.head(PPoint::new(left, top), text.get("present.head"), ink);
        let mut next = top + line + gap;
        let (call, watch) = self.desk.on(&plate.id);
        // Every section of the column stands on a ground of its own;
        // its heading is over it.
        // Nobody: say so, and where a page reaches the line. And why
        // this window is not in the call though it asked to be, and
        // what carries the call it is in.
        let nobody = call.is_empty() && watch.is_empty();
        let how = match self.desk.line.as_ref() {
            Ok(at) => text.fill("present.how", "at", &format!("http://{at}")).replace("{key}", &self.desk.key(&plate.id)),
            Err(_) => text.get("present.shut").to_string(),
        };
        let carried = self.desk.calling().map(|seen| text.fill("call.carry", "engine", &seen.engine));
        let said: Vec<String> = [Some(self.desk.why()).filter(|why| !why.is_empty()), carried].into_iter().flatten().collect();
        let from = next;
        next = section(paint, PPoint::new(left, from), wide, &mut |paint| {
            let mut next = from;
            if nobody {
                next = self.lines(paint, PPoint::new(left, next), wide, text.get("present.none"), INK);
                next = self.lines(paint, PPoint::new(left, next), wide, &how, DIM);
            }
            said.iter().fold(next, |next, said| self.lines(paint, PPoint::new(left, next), wide, said, DIM))
        }) + gap;
        let group = [(&call, "present.call", CALL, PLight::Strong), (&watch, "present.watch", WATCH, PLight::Soft)];
        for (roster, name, icon, strength) in group.into_iter().filter(|(roster, ..)| !roster.is_empty()) {
            let from = self.head(paint, PPoint::new(left, next), name);
            next = section(paint, PPoint::new(left, from), wide, &mut |paint| self.rows(paint, PPoint::new(left, from), wide, roster, icon, strength)) + gap;
        }
        // The room's own people, as the site names them.
        // Each a card with their face, as people in a voice room are.
        self.spot.borrow_mut().clear();
        if !plate.member.is_empty() {
            let from = self.head(paint, PPoint::new(left, next), "present.member");
            let seat = crowd::named(&self.desk, &plate.member);
            let (spot, end) = crowd::cards(&self.desk, paint, PPoint::new(left, from), wide, &seat);
            (*self.spot.borrow_mut(), next) = (spot, end);
        }
        paint.unclip();
        self.fitted(next - top + gap);
    }
}

impl Present {
    /// Keep how long what was drawn is, for the wheel.
    fn fitted(&self, content: f32) {
        let mut scroll = self.scroll.get();
        scroll.fit(content, self.frame.size.height);
        self.scroll.set(scroll);
    }
}

impl PView for Present {
    /// Its members' cards.
    fn hand(&self) -> Vec<PRect> {
        self.spot.borrow().iter().map(|(rect, _)| *rect).collect()
    }

    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    /// While the line is open, somebody may come or go at any time.
    fn tick(&mut self) -> bool {
        self.desk.line.is_ok()
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        let mut scroll = self.scroll.get();
        match *event {
            // The wheel turned away from the hand goes up.
            PEvent::Scroll(_, by) => {
                let any = scroll.turn(-by.y);
                self.scroll.set(scroll);
                any
            }
            // A press on a member's card opens them beside the room.
            PEvent::PointerDown(at) => {
                let who = self.spot.borrow().iter().find(|(rect, _)| rect.contains(at.plus(self.origin.get()))).map(|(_, id)| id.clone());
                let known = who.is_some();
                who.into_iter().for_each(|id| *self.desk.now.who.borrow_mut() = Some(id));
                known
            }
            _ => false,
        }
    }
}
