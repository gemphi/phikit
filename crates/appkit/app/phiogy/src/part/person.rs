//! A person, beside the room: the one profile the web's room shows on
//! a press of a face (`profileOf` in
//! `puikit/app/phi-kit/src/blocks/Room.tsx`). The picture over it,
//! their face, name, handle, what they are there and whether they are
//! about; what they say of themself and their tags; what to do (write
//! to them, follow them, mute them, which keeps what they said out of
//! the streams); since when they are a member; and three
//! panes: who they are linked to, the spaces they are in, and the
//! pictures they posted, a press on which opens it in the viewer among
//! their others. The way back to who is here is at its head.
//!
//! Its controls are rows of `data/act.tsv`; its words `person.*` in
//! `data/text.tsv`.

mod pane;

use std::cell::{Cell, RefCell};

use appkit::pui::block::{PBand, PFace, PPill, PText};
use appkit::pui::core::{PDraw, PEvent, PFit, PPaint, PPoint, PRect, PView};
use appkit::pui::layout::{PScroll, section, wrap};

use super::{DIM, Desk, GAP, INK, LINE, RADIUS, Tool};
use crate::data;
use crate::site::Member;

/// The theme's tokens: how large the face is, how many times as wide
/// as high the picture over the profile is, and how many pictures a
/// row of the pictures' pane has.
pub const FACE: &str = "person.face";
pub const BANNER: &str = "person.banner";
pub const ACROSS: &str = "person.across";
/// The rows of `data/act.tsv` it draws.
pub const BACK: &str = "back";
pub const MESSAGE: &str = "message";
pub const FOLLOW: &str = "follow";
pub const QUIET: &str = "quiet";
/// The panes, in order: each is `person.` and this in the words.
pub const PANE: [&str; 3] = ["linked", "space", "media"];

/// What a press on the profile does.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Act {
    Back,
    Message(String),
    Follow,
    Mute,
    Pane(usize),
    Open(String),
    Shot(usize),
}

/// A person, beside the room.
pub struct Person {
    frame: PRect,
    desk: Desk,
    tool: Tool,
    scroll: Cell<PScroll>,
    origin: Cell<PPoint>,
    spot: RefCell<Vec<(PRect, Act)>>,
    /// The pane shown, and the person whose links were read last with
    /// who they are linked to.
    pane: Cell<usize>,
    linked: RefCell<(String, Vec<String>)>,
}

impl Person {
    pub fn new(desk: Desk) -> Person {
        Person { frame: PRect::default(), desk, tool: Tool::read(data::ACT), scroll: Cell::default(), origin: Cell::default(), spot: RefCell::default(), pane: Cell::default(), linked: RefCell::default() }
    }

    /// Who it is about.
    fn who(&self) -> Option<Member> {
        let id = self.desk.now.who.borrow().clone()?;
        self.desk.site.member(&id).cloned()
    }

    /// Who a person is linked to, read once for each person shown.
    pub(super) fn linked(&self, id: &str) -> Vec<String> {
        if self.linked.borrow().0 != id {
            *self.linked.borrow_mut() = (id.to_string(), self.desk.site.linked(id));
        }
        self.linked.borrow().1.clone()
    }
}

impl PDraw for Person {
    /// The pane is made of the kit's blocks, in sections, each on a
    /// ground of its own: the way back; who they are; what can be done
    /// with them; the panes; and the pane that is shown.
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let Some(one) = self.who() else { return };
        // Somebody linked to nobody has no pane of links, as on the web.
        if one.link == 0 && self.pane.get() == 0 {
            self.pane.set(1);
        }
        let (gap, line, radius, face) = (paint.size(GAP), paint.size(LINE), paint.size(RADIUS), paint.size(FACE));
        let (ink, dim, text) = (paint.color(INK), paint.color(DIM), &self.desk.text);
        let (left, wide) = (origin.x + gap, (self.frame.size.width - gap * 2.0).max(0.0));
        let mut spot = Vec::new();
        self.origin.set(origin);
        paint.clip(PRect { origin, size: self.frame.size });
        let begin = origin.y + gap / 2.0 - self.scroll.get().at();
        // The way back, and what this is.
        let back: Vec<(&str, &str, bool)> = self.tool.of(BACK, false, text).map(|(icon, _)| (icon, text.get("person.title"), false)).into_iter().collect();
        spot.extend(back.draw(paint, PPoint::new(left, begin)).into_iter().map(|rect| (rect, Act::Back)));
        let from = begin + back.high(paint) + gap * 0.75;
        // Who they are: the picture over the profile, their face, their
        // name and how they stand, what they say of themself, their tags.
        let cell = self.desk.site.face(&one.id).map(|cell| (cell.sheet.as_str(), cell.column, cell.index));
        let under = [one.handle.clone(), one.role.clone(), one.state.clone()];
        let said = under.iter().filter(|said| !said.is_empty()).cloned().collect::<Vec<_>>().join(&format!(" {} ", text.get("between")));
        let since = Some(format!("{} {} {}", text.get("person.since"), text.get("between"), one.since)).filter(|_| !one.since.is_empty());
        let mut top = section(paint, PPoint::new(left, from), wide, &mut |paint| {
            let mut top = from;
            if !one.banner.is_empty() {
                let high = wide / paint.size(BANNER).max(1.0);
                paint.picture(&one.banner, PRect::new(left, top, wide, high), radius, PFit::Cover);
                top += high + gap / 2.0;
            }
            (cell, one.name.as_str()).face(paint, PRect::new(left, top, face, face));
            let beside = left + face + gap / 2.0;
            paint.head(PPoint::new(beside, top), &one.name.fit(wide - face - gap / 2.0, paint), ink);
            paint.text(PPoint::new(beside, top + face - line), &said.fit(wide - face - gap / 2.0, paint), dim);
            top += face + gap / 2.0;
            for words in wrap(&one.bio, wide, &|text| paint.measure(text)).into_iter().filter(|words| !words.is_empty()) {
                paint.text(PPoint::new(left, top), &words, ink);
                top += line;
            }
            let mut across = left;
            for tag in &one.tag {
                across += ("", tag.as_str(), false).draw(paint, PPoint::new(across, top + gap / 4.0), false).size.width + gap / 4.0;
            }
            top += match one.tag.is_empty() {
                true => 0.0,
                false => line + gap,
            };
            if let Some(said) = &since {
                paint.text(PPoint::new(left, top), said, dim);
                top += line;
            }
            top
        }) + gap * 0.75;
        // What can be done: write to them, follow them, mute them.
        let direct = Some(self.desk.direct_room(&one.id)).filter(|_| one.id != self.desk.site.me);
        let other = one.id != self.desk.site.me;
        let deed = [(MESSAGE, direct.map(Act::Message), false), (FOLLOW, Some(Act::Follow).filter(|_| other), self.desk.follows(&one.id)), (QUIET, Some(Act::Mute).filter(|_| other), self.desk.muted(&one.id))];
        let deed: Vec<((&str, &str, bool), Act)> = deed.into_iter().filter_map(|(name, act, on)| Some(((self.tool.of(name, on, text)?.0, self.tool.of(name, on, text)?.1, on), act?))).collect();
        let pill: Vec<(&str, &str, bool)> = deed.iter().map(|(pill, _)| *pill).collect();
        spot.extend(pill.draw(paint, PPoint::new(left, top)).into_iter().zip(deed.into_iter().map(|(_, act)| act)));
        top += match pill.is_empty() {
            true => 0.0,
            false => pill.high(paint) + gap / 2.0,
        };
        // The panes, each with how much it holds; then the one shown.
        let shot = self.desk.shot_of(&one.id);
        let count = [one.link as usize, one.space.len(), shot.len()];
        let pane: Vec<(usize, String)> = PANE.iter().enumerate().filter(|(index, _)| *index > 0 || one.link > 0).map(|(index, name)| (index, format!("{} {}", text.get(&format!("person.{name}")), count[index]))).collect();
        let tab: Vec<(&str, &str, bool)> = pane.iter().map(|(index, said)| ("", said.as_str(), *index == self.pane.get())).collect();
        spot.extend(tab.draw(paint, PPoint::new(left, top)).into_iter().zip(pane.iter().map(|(index, _)| Act::Pane(*index))));
        top += tab.high(paint) + gap * 0.75;
        let mut got = Vec::new();
        let end = section(paint, PPoint::new(left, top), wide, &mut |paint| {
            got.clear();
            self.body(paint, PPoint::new(left, top), wide, &one, &shot, &mut got)
        });
        spot.extend(got);
        paint.unclip();
        let mut scroll = self.scroll.get();
        scroll.fit(end - begin + gap, self.frame.size.height);
        self.scroll.set(scroll);
        *self.spot.borrow_mut() = spot;
    }
}

impl PView for Person {
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
        match *event {
            PEvent::Scroll(_, by) => {
                let mut scroll = self.scroll.get();
                let any = scroll.turn(-by.y);
                self.scroll.set(scroll);
                any
            }
            PEvent::PointerDown(at) => {
                let place = at.plus(self.origin.get());
                let Some(act) = self.spot.borrow().iter().find(|(rect, _)| rect.contains(place)).map(|(_, act)| act.clone()) else { return false };
                let who = self.desk.now.who.borrow().clone().unwrap_or_default();
                match act {
                    Act::Back => *self.desk.now.who.borrow_mut() = None,
                    Act::Message(room) => _ = self.desk.show(&room),
                    Act::Follow => self.desk.follow(&who),
                    Act::Mute => self.desk.mute(&who),
                    Act::Pane(index) => self.pane.set(index),
                    Act::Open(id) => *self.desk.now.who.borrow_mut() = Some(id),
                    Act::Shot(number) => self.desk.look(self.desk.shot_of(&who), number),
                }
                true
            }
            _ => false,
        }
    }
}
