//! A voice room: one people are in, not one they write in, as the
//! web's stands (`RoomVoice`). Who is in it, each a card with their
//! face, their name and how they are: the people its file names, this
//! window's person once they joined, and whoever is in it on its line.
//! Under them how loud this window's microphone hears its person; the
//! way in and out, the microphone, and the sound of the others; and
//! what the room does with them.
//!
//! Being in it is a call of the room's line with no camera. It goes on
//! while other rooms are looked at, and ends when the person leaves, or
//! begins or joins a call somewhere else.

use std::cell::{Cell, RefCell};

use appkit::pui::block::{PBand, PDeck};
use appkit::pui::core::{PDraw, PEvent, PLight, PPaint, PPoint, PRect, PView, light};
use appkit::pui::layout::fold;

use super::crowd;
use super::present::BAR;
use super::{DIM, Desk, GAP, LINE, fit};

pub use super::crowd::Seat;

/// What a press does.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Press {
    Join,
    Leave,
    Hush(bool),
    Deaf(bool),
    Who(String),
}

/// The open room, when it is a voice room.
pub struct Voice {
    frame: PRect,
    desk: Desk,
    /// Where it was drawn last, and each thing to press, in the window.
    origin: Cell<PPoint>,
    spot: RefCell<Vec<(PRect, Press)>>,
}

impl Voice {
    pub fn new(desk: Desk) -> Voice {
        Voice { frame: PRect::default(), desk, origin: Cell::default(), spot: RefCell::default() }
    }

    /// Who is in the room now. One the file names who is also in it on
    /// the line is there once, as the line has them.
    pub fn seat(&self) -> Vec<Seat> {
        let desk = &self.desk;
        let text = |name: &str| desk.text.get(name).to_string();
        let (seen, me) = (desk.calling(), &desk.site.me);
        let live = desk.other().into_iter().map(|(peer, name)| {
            let member = desk.site.member.iter().find(|member| member.id != *me && member.name == name);
            let quiet = seen.as_ref().is_some_and(|seen| seen.far.iter().any(|far| far.peer == peer && far.quiet));
            let said = match quiet {
                true => text("call.tile.muted"),
                false => text("voice.inroom"),
            };
            desk.seat(member.map_or("", |member| member.id.as_str()), member.map_or(name.as_str(), |member| member.name.as_str()), &said)
        });
        let live: Vec<Seat> = live.collect();
        // The room's people are kept by their names.
        let named = desk.plate(&desk.room()).map(|plate| plate.member).unwrap_or_default();
        let quiet = seen.as_ref().is_some_and(|seen| seen.quiet);
        let filed = named.iter().filter_map(|name| desk.site.member.iter().find(|member| member.name == *name)).filter(|member| member.id != *me);
        let crowd = filed.chain(desk.site.member(me).filter(|_| seen.is_some())).filter(|member| !live.iter().any(|seat| seat.id == member.id));
        let crowd = crowd.map(|member| {
            let said = match member.id == *me && quiet {
                true => text("call.tile.muted"),
                false => member.state.clone(),
            };
            desk.seat(&member.id, &member.name, &said)
        });
        crowd.chain(live.clone()).collect()
    }

    /// The pills: what each reads, what a press does, whether it says
    /// what is so.
    fn pill(&self) -> Vec<(String, Press, bool)> {
        let text = |name: &str| self.desk.text.get(name).to_string();
        let Some(seen) = self.desk.calling() else { return vec![(text("call.join"), Press::Join, false)] };
        let hush = match seen.quiet {
            true => (text("call.unmute"), Press::Hush(false), true),
            false => (text("call.mute"), Press::Hush(true), false),
        };
        let deaf = match seen.own.deaf {
            true => (text("voice.undeaf"), Press::Deaf(false), true),
            false => (text("voice.deaf"), Press::Deaf(true), false),
        };
        vec![(text("call.leave"), Press::Leave, true), hush, deaf]
    }

    /// What the room does with the person's sound: that no line carries
    /// it, how to begin, what the system did not give, or that it goes
    /// to everyone in the room.
    fn said(&self) -> String {
        let desk = &self.desk;
        let text = |name: &str| desk.text.get(name).to_string();
        let fault = |seen: crate::peer::Seen| {
            let named = seen.own.fault.first().cloned().zip(seen.own.why.first().cloned());
            named.map(|(name, why)| desk.text.fill(&format!("call.fault.{name}"), "why", &why)).filter(|said| !said.is_empty())
        };
        match (desk.line.is_ok(), desk.calling()) {
            (false, _) => text("voice.soundless"),
            (true, None) => Some(desk.why()).filter(|why| !why.is_empty()).unwrap_or_else(|| text("voice.wait")),
            (true, Some(seen)) => fault(seen).unwrap_or_else(|| text("voice.live")),
        }
    }
}

impl PDraw for Voice {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let Some(plate) = self.desk.plate(&self.desk.room()).filter(|plate| plate.voice && self.frame.size.height > 0.0) else { return self.spot.borrow_mut().clear() };
        let (gap, line, dim) = (paint.size(GAP), paint.size(LINE), paint.color(DIM));
        let (start, full) = self.desk.column(self.frame.size.width);
        let (left, mut top, mut spot) = (origin.x + start, origin.y + gap, Vec::new());
        self.origin.set(origin);
        paint.clip(PRect { origin, size: self.frame.size });
        if !plate.topic.is_empty() {
            paint.text(PPoint::new(left, top), &fit(&plate.topic, full, paint), dim);
            top += line + gap / 2.0;
        }
        // Who is in it: people as cards, the part every crowd is shown by.
        let seat = self.seat();
        let (cards, end) = seat.draw(paint, PPoint::new(left, top), full);
        spot.extend(crowd::whose(cards, &seat).into_iter().map(|(rect, id)| (rect, Press::Who(id))));
        top = match seat.is_empty() {
            true => {
                paint.text(PPoint::new(left, top), self.desk.text.get("voice.empty"), dim);
                top + line + gap
            }
            false => end + gap,
        };
        // In it: how loud the person is, as far along as they are loud.
        if let Some(seen) = self.desk.calling() {
            let bar = paint.size(BAR);
            light::label(paint, PRect::new(left, top, full, bar), bar / 2.0, PLight::Soft, false, "");
            let loud = seen.own.level.clamp(0.0, 1.0) * full;
            if loud >= 1.0 && !seen.quiet {
                light::label(paint, PRect::new(left, top, loud, bar), bar / 2.0, PLight::Strong, false, "");
            }
            top += bar + gap / 2.0;
        }
        let list = self.pill();
        let pill: Vec<(&str, &str, bool)> = list.iter().map(|(label, _, so)| ("", label.as_str(), *so)).collect();
        spot.extend(pill.draw(paint, PPoint::new(left, top)).into_iter().zip(list.iter().map(|(_, press, _)| press.clone())));
        top += line + gap + gap / 2.0;
        for (index, text) in fold(&self.said(), full, &|text| paint.measure(text)).iter().enumerate() {
            paint.text(PPoint::new(left, top + index as f32 * line), text, dim);
        }
        *self.spot.borrow_mut() = spot;
        paint.unclip();
    }
}

impl PView for Voice {
    /// Where a press on it does something.
    fn hand(&self) -> Vec<PRect> {
        self.spot.borrow().iter().map(|(rect, _)| *rect).collect()
    }

    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        let PEvent::PointerDown(at) = *event else { return false };
        // What can be pressed keeps where it was drawn in the window; a
        // press comes in this part's own measure.
        let place = at.plus(self.origin.get());
        let press = self.spot.borrow().iter().find(|(rect, _)| rect.contains(place)).map(|(_, press)| press.clone());
        match press {
            Some(Press::Join) => self.desk.join(),
            Some(Press::Leave) => self.desk.leave(),
            Some(Press::Hush(quiet)) => self.desk.hush(quiet),
            Some(Press::Deaf(on)) => self.desk.deaf(on),
            Some(Press::Who(id)) => *self.desk.now.who.borrow_mut() = Some(id),
            None => return false,
        }
        true
    }
}
