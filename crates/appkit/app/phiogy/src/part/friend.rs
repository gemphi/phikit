//! Friends, and finding people: two pages of the direct list, as the
//! web's are (`Friends` in `puikit/app/phi-kit/src/friends.tsx`,
//! `FindPeople` in `direct.tsx`).
//!
//! Friends has four tabs: who of the person's friends is here now;
//! all of them; the requests that wait, theirs to answer and the
//! person's own to take back; and a search of everyone, to ask
//! somebody new. A press on a line writes to that person; at its end
//! is what can be done: write, part, accept, decline, ask, take back.
//!
//! Finding people is a field and everyone it finds: a press on one
//! begins a conversation with them, which stands in the rail from
//! then on, and goes to it.

use std::cell::Cell;

use appkit::pui::control::{PControl, PField};
use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};

use super::bond::{self, ASKED, NONE, SENT, WITH};
use super::list::{Deed, Item, Lister, Seat, field, grouped};
use super::{Desk, Tool};
use crate::data;
use crate::site::Member;

/// The theme's token for how many people a search lists at most.
pub const MOST: &str = "people.most";
/// The word of the rooms' file for somebody who is not about.
pub const OFF: &str = "off";
/// The tabs of friends, in order: each is `friend.` and this.
pub const TAB: [&str; 4] = ["here", "all", "pending", "add"];

/// What a press on either page means.
#[derive(Debug, Clone, PartialEq)]
enum Act {
    Tab(usize),
    Field,
    /// Write to the person of that id.
    Write(String),
    /// Stand so with the person of that id from now on.
    Bind(String, String),
}

/// Friends, or finding people: which page it is, is how it was made.
pub struct Friend {
    frame: PRect,
    desk: Desk,
    tool: Tool,
    list: Lister<Act>,
    find: PField,
    tab: Cell<usize>,
    /// Whether it is the page a conversation is found from.
    seeking: bool,
}

impl Friend {
    pub fn new(desk: Desk, seeking: bool) -> Friend {
        let mut find = PField::new(PRect::default());
        find.focused = true;
        Friend { frame: PRect::default(), desk, tool: Tool::read(data::ACT), list: Lister::default(), find, tab: Cell::default(), seeking }
    }

    /// The face of somebody, or the icon that stands for one.
    fn seat(&self, one: &Member) -> Seat {
        self.desk.site.face(&one.id).cloned().map_or_else(|| Seat::Icon(self.desk.icon(self.desk.page.face()).to_string()), Seat::Face)
    }

    /// What can be done with one press from where the person and
    /// somebody stand: an icon, what it is called, what it leads to.
    fn step(&self, one: &Member, standing: &str, name: &str) -> Deed<Act> {
        let (to, word) = bond::next(standing);
        let icon = self.tool.of(name, false, &self.desk.text).map_or("", |(icon, _)| icon);
        Deed::icon(icon, self.desk.text.get(word), Act::Bind(one.id.clone(), to.to_string()))
    }

    /// A line of somebody: a press writes to them.
    fn line(&self, one: &Member, sub: &str, deed: Vec<Deed<Act>>) -> Item<Act> {
        Item::Line(self.seat(one), one.name.clone(), sub.to_string(), Some(Act::Write(one.id.clone())), deed)
    }

    /// Everyone a search finds, each with what is so of them.
    fn found(&self, with: impl Fn(&Member) -> (String, Vec<Deed<Act>>)) -> Vec<Item<Act>> {
        let found = self.desk.seek(&self.find.value(), self.desk.size(MOST) as usize);
        found.iter().map(|one| { let (sub, deed) = with(one); self.line(one, &sub, deed) }).collect()
    }

    /// The page a conversation is found or begun from.
    fn people(&self) -> Vec<Item<Act>> {
        let text = &self.desk.text;
        let mut item = vec![Item::Head(text.get("direct.find").to_string()), Item::Field(0, text.get("direct.search").to_string(), Act::Field)];
        let found = self.found(|one| (Some(format!("@{}", one.handle)).filter(|_| !one.handle.is_empty()).unwrap_or_default(), Vec::new()));
        if found.is_empty() {
            item.push(Item::Note(text.get("direct.none").to_string()));
        }
        item.extend(found);
        item.push(Item::Note(text.fill("direct.count", "n", &grouped(self.desk.site.member.len().saturating_sub(1) as u64))));
        item
    }

    /// The friends page.
    fn friends(&self) -> Vec<Item<Act>> {
        let text = &self.desk.text;
        let word = |name: &str| text.get(name).to_string();
        let (with, asked, sent) = (self.desk.bonded(WITH), self.desk.bonded(ASKED), self.desk.bonded(SENT));
        let here: Vec<&Member> = with.iter().filter(|one| !one.presence.is_empty() && one.presence != OFF).collect();
        let count = [Some(here.len()), Some(with.len()), Some(asked.len() + sent.len()).filter(|count| *count > 0), None];
        let tab = TAB.iter().enumerate().map(|(index, name)| {
            let label = count[index].map_or_else(|| word(&format!("friend.{name}")), |count| format!("{} {count}", text.get(&format!("friend.{name}"))));
            Deed::word(&label, index == self.tab.get(), Act::Tab(index))
        });
        let mut item = vec![Item::Head(word("friend.title")), Item::Tabs(tab.collect())];
        let write = |one: &Member| Deed::icon(self.tool.of("message", false, text).map_or("", |(icon, _)| icon), text.get("friend.message"), Act::Write(one.id.clone()));
        let friend = |one: &Member| self.line(one, &one.state, vec![write(one), self.step(one, WITH, "part")]);
        let none = |name: &str| vec![Item::Note(word(name))];
        let rows = match TAB[self.tab.get().min(TAB.len() - 1)] {
            "here" => Some(here.iter().map(|one| friend(one)).collect::<Vec<_>>()).filter(|rows| !rows.is_empty()).unwrap_or_else(|| none("friend.nobody")),
            "all" => Some(with.iter().map(friend).collect::<Vec<_>>()).filter(|rows| !rows.is_empty()).unwrap_or_else(|| none("friend.nobody")),
            "pending" => {
                let theirs = asked.iter().map(|one| self.line(one, text.get("friend.asked"), vec![self.step(one, ASKED, "accept"), Deed::icon(self.tool.of("part", false, text).map_or("", |(icon, _)| icon), text.get("friend.decline"), Act::Bind(one.id.clone(), NONE.to_string()))]));
                let mine = sent.iter().map(|one| self.line(one, text.get("friend.yours"), vec![self.step(one, SENT, "part")]));
                let mut rows: Vec<Item<Act>> = theirs.chain(mine).collect();
                rows.extend(Some(Item::Note(word("friend.nothing"))).filter(|_| asked.is_empty() && sent.is_empty()));
                rows.extend(Some(Item::Note(word("friend.waits"))).filter(|_| !sent.is_empty()));
                rows
            }
            _ => {
                let mut rows = vec![Item::Field(0, word("friend.search"), Act::Field)];
                let found = self.found(|one| {
                    let standing = self.desk.bond(&one.id);
                    let (sub, icon) = match standing.as_str() {
                        NONE => (Some(format!("@{}", one.handle)).filter(|_| !one.handle.is_empty()).unwrap_or_default(), "ask"),
                        ASKED => (word("friend.asked"), "accept"),
                        other => (word(&format!("friend.{other}")), "part"),
                    };
                    (sub, vec![self.step(one, &standing, icon)])
                });
                rows.extend(Some(Item::Note(word("friend.unfound"))).filter(|_| found.is_empty()));
                rows.extend(found);
                rows
            }
        };
        item.extend(rows);
        item
    }

    fn item(&self) -> Vec<Item<Act>> {
        match self.seeking {
            true => self.people(),
            false => self.friends(),
        }
    }
}

impl PDraw for Friend {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        self.list.draw(paint, origin, self.frame, &self.desk, &self.item(), &[&self.find]);
    }
}

impl PView for Friend {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
        self.find.set_frame(field(&self.desk, frame.size.width));
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        match *event {
            PEvent::KeyDown(_) => self.find.respond(event),
            PEvent::Scroll(_, by) => self.list.wheel(by.y),
            PEvent::PointerDown(at) => {
                let Some(act) = self.list.press(at) else { return false };
                match act {
                    Act::Tab(index) => self.tab.set(index),
                    Act::Field => {}
                    Act::Write(id) => _ = self.desk.show(&self.desk.direct_room(&id)),
                    Act::Bind(id, to) => self.desk.bind(&id, &to),
                }
                true
            }
            _ => false,
        }
    }
}
