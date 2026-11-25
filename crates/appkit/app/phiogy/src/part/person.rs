//! A person, beside the room: the one profile the web's room shows on
//! a press of a face (`profileOf` in
//! `puikit/app/phi-kit/src/blocks/Room.tsx`). The picture over it,
//! their face, name, handle, what they are there and whether they are
//! about; what they say of themself and their tags; what to do (write
//! to them, follow them, mute them, which keeps what they said out of
//! the streams); since when they are a member; and under it their tabs
//! as Phiegg's profile has them, the kit's form `profile`
//! (`person/held.rs`, `data/profile.tsv`): info, pictures, PFTs, items,
//! drops, trail, stats, clubs, spaces, links, each a grid with its
//! layout, filter and order. The way back to who is here is at its head.
//!
//! Its controls are rows of `data/act.tsv`; its words `person.*` in
//! `data/text.tsv`.

mod card;
mod held;

use std::cell::{Cell, RefCell};

use appkit::pui::block::PBand;
use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};
use appkit::pui::form::PForm;
use appkit::pui::layout::section;

use super::{Desk, GAP, Tool};
use crate::data;
use crate::site::Member;

/// The theme's tokens: how large the face is, and how many times as
/// wide as high the picture over the profile is.
pub const FACE: &str = "person.face";
pub const BANNER: &str = "person.banner";
/// The rows of `data/act.tsv` it draws.
pub const BACK: &str = "back";
pub const MESSAGE: &str = "message";
pub const FOLLOW: &str = "follow";
pub const QUIET: &str = "quiet";

/// What a press on the profile does.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Act {
    Back,
    Message(String),
    Follow,
    Mute,
}

/// A person, beside the room.
pub struct Person {
    frame: PRect,
    desk: Desk,
    tool: Tool,
    origin: Cell<PPoint>,
    spot: RefCell<Vec<(PRect, Act)>>,
    /// Their tabs, under who they are: where they begin, down the pane,
    /// and how far who they are is scrolled away over them.
    form: RefCell<PForm>,
    under: Cell<f32>,
    lift: Cell<f32>,
}

impl Person {
    pub fn new(desk: Desk) -> Person {
        let form = RefCell::new(held::form(&desk));
        Person { frame: PRect::default(), desk, tool: Tool::read(data::ACT), origin: Cell::default(), spot: RefCell::default(), form, under: Cell::default(), lift: Cell::default() }
    }

    /// Who it is about.
    fn who(&self) -> Option<Member> {
        let id = self.desk.now.who.borrow().clone()?;
        self.desk.site.member(&id).cloned()
    }
}

impl PDraw for Person {
    /// The pane is made of the kit's blocks, in sections, each on a
    /// ground of its own: the way back; who they are; what can be done
    /// with them; and their tabs, the kit's form, in the room left.
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let Some(one) = self.who() else { return };
        let (gap, text) = (paint.size(GAP), &self.desk.text);
        let (left, wide) = (origin.x + gap, (self.frame.size.width - gap * 2.0).max(0.0));
        let mut spot = Vec::new();
        self.origin.set(origin);
        paint.clip(PRect { origin, size: self.frame.size });
        let begin = origin.y + gap / 2.0 - self.lift.get();
        // The way back, and what this is.
        let back: Vec<(&str, &str, bool)> = self.tool.of(BACK, false, text).map(|(icon, _)| (icon, text.get("person.title"), false)).into_iter().collect();
        spot.extend(back.draw(paint, PPoint::new(left, begin)).into_iter().map(|rect| (rect, Act::Back)));
        let from = begin + back.high(paint) + gap * 0.75;
        // Who they are, as Phiegg's card has it (person/card.rs).
        let mut top = section(paint, PPoint::new(left, from), wide, &mut |paint| self.card(paint, PPoint::new(left, from), wide, &one)) + gap * 0.75;
        // What can be done: write to them, follow them, mute them.
        let direct = Some(self.desk.direct_room(&one.id)).filter(|_| one.id != self.desk.site.me);
        let other = one.id != self.desk.site.me;
        let deed = [(MESSAGE, direct.map(Act::Message), false), (FOLLOW, Some(Act::Follow).filter(|_| other), self.desk.follows(&one.id)), (QUIET, Some(Act::Mute).filter(|_| other), self.desk.muted(&one.id))];
        let deed: Vec<((&str, &str, bool), Act)> = deed.into_iter().filter_map(|(name, act, on)| Some(((self.tool.of(name, on, text)?.0, self.tool.of(name, on, text)?.1, on), act?))).collect();
        let pill: Vec<(&str, &str, bool)> = deed.iter().map(|(pill, _)| *pill).collect();
        // As many on a line as the column is wide, the rest under them.
        let (rect, high) = pill.draw_lines(paint, PPoint::new(left, top), wide);
        spot.extend(rect.into_iter().zip(deed.into_iter().map(|(_, act)| act)));
        top += match pill.is_empty() {
            true => 0.0,
            false => high + gap / 2.0,
        };
        paint.unclip();
        // Their tabs, the kit's form, in the room under all that; who they
        // are scrolls away over them first.
        let under = top - origin.y;
        self.under.set(under);
        self.lift.set(self.lift.get().clamp(0.0, (under + self.lift.get()).max(0.0)));
        let mut form = self.form.borrow_mut();
        form.set_frame(PRect::new(self.frame.origin.x, self.frame.origin.y + under, self.frame.size.width, (self.frame.size.height - under).max(0.0)));
        form.draw(paint, PPoint::new(origin.x, origin.y + under));
        *self.spot.borrow_mut() = spot;
    }
}

impl PView for Person {
    /// Where a press on it does something.
    fn hand(&self) -> Vec<PRect> {
        let mut hand: Vec<PRect> = self.spot.borrow().iter().map(|(rect, _)| *rect).collect();
        hand.extend(self.form.borrow().hand());
        hand
    }

    fn tick(&mut self) -> bool {
        self.form.get_mut().tick()
    }

    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    /// What lies under who they are is the form's: told it as it stands
    /// there. A press over it is the profile's own.
    fn respond(&mut self, event: &PEvent) -> bool {
        let under = self.under.get();
        let below = |at: PPoint| (at.y >= under).then(|| at.minus(PPoint::new(0.0, under)));
        // A scroll down lifts who they are away first, then scrolls the
        // tabs; up, the tabs first, then who they are comes back.
        if let PEvent::Scroll(at, by) = *event {
            let (lift, most) = (self.lift.get(), (under + self.lift.get()).max(0.0));
            let form = self.form.get_mut();
            let place = at.minus(PPoint::new(0.0, under));
            return match by.y < 0.0 {
                true if lift < most => {
                    self.lift.set((lift - by.y).min(most));
                    true
                }
                true => form.respond(&PEvent::Scroll(place, by)),
                false => form.respond(&PEvent::Scroll(place, by)) || {
                    self.lift.set((lift - by.y).max(0.0));
                    lift > 0.0
                },
            };
        }
        let told = match *event {
            PEvent::PointerDown(at) => below(at).map(PEvent::PointerDown),
            PEvent::PointerMove(at) => below(at).map(PEvent::PointerMove),
            // It has no line to type: keys are the window's (Escape).
            _ => None,
        };
        if let Some(told) = told {
            return self.form.get_mut().respond(&told);
        }
        match *event {
            PEvent::PointerDown(at) => {
                let place = at.plus(self.origin.get());
                let Some(act) = self.spot.borrow().iter().find(|(rect, _)| rect.contains(place)).map(|(_, act)| act.clone()) else { return false };
                let who = self.desk.now.who.borrow().clone().unwrap_or_default();
                match act {
                    Act::Back => *self.desk.now.who.borrow_mut() = None,
                    Act::Message(room) => _ = self.desk.show(&room),
                    Act::Follow => self.desk.follow(&who),
                    Act::Mute => self.desk.mute(&who),
                }
                true
            }
            _ => false,
        }
    }
}
