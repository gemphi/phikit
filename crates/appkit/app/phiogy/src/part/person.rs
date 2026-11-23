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

use appkit::pui::core::{PDraw, PEvent, PFit, PLight, PPaint, PPoint, PRect, PView, light};
use appkit::pui::layout::{PScroll, wrap};

use super::{DIM, Desk, GAP, INK, LINE, RADIUS, Tool, fit, tool};
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

    /// A pill with words in it, at `at`; how wide it is.
    fn pill(&self, paint: &mut dyn PPaint, at: PPoint, said: &str, light: PLight) -> PRect {
        let (gap, line, ink) = (paint.size(GAP), paint.size(LINE), paint.color(INK));
        let (rect, radius) = (PRect::new(at.x, at.y, paint.measure(said) + gap, line + gap / 4.0), paint.size(RADIUS));
        light::label(paint, rect, radius, light, false, "");
        paint.text(PPoint::new(at.x + gap / 2.0, at.y + gap / 8.0), said, ink);
        rect
    }
}

impl PDraw for Person {
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
        let back = PRect::new(left, begin, line + gap / 2.0, line + gap / 2.0);
        if let Some((icon, _)) = self.tool.of(BACK, false, text) {
            tool::button(paint, back, icon, Some(PLight::Rest));
            spot.push((back, Act::Back));
        }
        paint.text(PPoint::new(back.origin.x + back.size.width + gap / 2.0, begin + gap / 4.0), text.get("person.title"), dim);
        let mut top = begin + back.size.height + gap / 2.0;
        // The picture over the profile, then the face and who they are.
        if !one.banner.is_empty() {
            let high = wide / paint.size(BANNER).max(1.0);
            paint.picture(&one.banner, PRect::new(left, top, wide, high), radius, PFit::Cover);
            top += high + gap / 2.0;
        }
        if let Some(cell) = self.desk.site.face(&one.id) {
            paint.picture(&cell.sheet, PRect::new(left, top, face, face), face / 2.0, PFit::Cell { column: cell.column, index: cell.index });
        }
        let beside = left + face + gap / 2.0;
        paint.head(PPoint::new(beside, top), &fit(&one.name, wide - face - gap / 2.0, paint), ink);
        let under = [one.handle.clone(), one.role.clone(), one.state.clone()];
        let said = under.iter().filter(|said| !said.is_empty()).cloned().collect::<Vec<_>>().join(&format!(" {} ", text.get("between")));
        paint.text(PPoint::new(beside, top + face - line), &fit(&said, wide - face - gap / 2.0, paint), dim);
        top += face + gap / 2.0;
        for words in wrap(&one.bio, wide, &|text| paint.measure(text)).into_iter().filter(|words| !words.is_empty()) {
            paint.text(PPoint::new(left, top), &words, ink);
            top += line;
        }
        // Their tags, then what to do: write to them, follow them.
        let mut across = left;
        for tag in &one.tag {
            across += self.pill(paint, PPoint::new(across, top + gap / 4.0), tag, PLight::Soft).size.width + gap / 4.0;
        }
        top += match one.tag.is_empty() {
            true => 0.0,
            false => line + gap,
        };
        let direct = Some(self.desk.direct_room(&one.id)).filter(|_| one.id != self.desk.site.me);
        let follows = self.desk.follows(&one.id);
        let other = one.id != self.desk.site.me;
        let deed = [(MESSAGE, direct.map(Act::Message), false), (FOLLOW, Some(Act::Follow).filter(|_| other), follows), (QUIET, Some(Act::Mute).filter(|_| other), self.desk.muted(&one.id))];
        let mut across = left;
        for (name, act, on) in deed {
            let (Some(act), Some((_, word))) = (act, self.tool.of(name, on, text)) else { continue };
            let light = match on {
                true => PLight::Strong,
                false => PLight::Rest,
            };
            let rect = self.pill(paint, PPoint::new(across, top), word, light);
            spot.push((rect, act));
            across += rect.size.width + gap / 2.0;
        }
        top += line + gap;
        if !one.since.is_empty() {
            paint.text(PPoint::new(left, top), &format!("{} {} {}", text.get("person.since"), text.get("between"), one.since), dim);
            top += line + gap / 2.0;
        }
        // The panes, each with how much it holds; then the one shown.
        let shot = self.desk.shot_of(&one.id);
        let count = [one.link as usize, one.space.len(), shot.len()];
        let mut across = left;
        for (index, name) in PANE.iter().enumerate().filter(|(index, _)| *index > 0 || one.link > 0) {
            let said = format!("{} {}", text.get(&format!("person.{name}")), count[index]);
            let light = match index == self.pane.get() {
                true => PLight::Strong,
                false => PLight::Soft,
            };
            let rect = self.pill(paint, PPoint::new(across, top), &said, light);
            spot.push((rect, Act::Pane(index)));
            across += rect.size.width + gap / 4.0;
        }
        top += line + gap;
        let end = self.body(paint, PPoint::new(left, top), wide, &one, &shot, &mut spot);
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
