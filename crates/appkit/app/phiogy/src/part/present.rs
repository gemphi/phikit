//! The column beside a room, the kit's form `aside` (data/form.tsv,
//! data/aside.tsv): the host the room is of, as Phiegg's host page has
//! it, and who is present. At its head the host's name, and under it
//! what it is (its role, genre, how many members, how many here now);
//! Join or Leave at its bar's end for a host of the catalogue; then its
//! face (its mark), and who is on the room's line: in its call, with the
//! room open, or nobody and where a page reaches it. Then its tabs:
//! Info (its world's picture, what its club says of it, its facts from
//! Phiegg's records),
//! Rooms, Members (a press puts a person beside the room), Rules, and
//! Calls: every call kept in the room (history.rs), each a player card
//! with Join to call again, or Leave while it is on.
//!
//! The line runs on other threads, so the board is read each time this
//! is drawn, and another frame is asked for while the line is open. A
//! room of another host begins at the column's top, on its Info.

use std::cell::RefCell;
use std::rc::Rc;

use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};
use appkit::pui::form::{PForm, PListed};
use appkit::pui::state::PState;

use super::search::{Found, said, values};
use super::tabbed::Tabbed;
use super::{Desk, crowd};
use crate::comp::Roster;
use crate::data;
use crate::peer::Stage;
use crate::site::Venue;

/// The icons of one who is in the call, and of one who only has the
/// room open.
pub const CALL: &str = "mic";
pub const WATCH: &str = "eye";
/// The theme's token for how high the bar is that shows how loud a
/// caller is (a call's tiles and a voice room have it too).
pub const BAR: &str = "present.bar";

/// The paths the form keeps and is told.
const TAB: &str = "aside/tab";
const DO: &str = "aside/do";
const ROOM: &str = "aside/room";
const WHO: &str = "aside/who";
/// A call's card and its controls: its room, Join, Leave.
const CALL_AT: &str = "aside/call";
const JOIN: &str = "aside/join";
const LEAVE: &str = "aside/leave";
/// What Profile keeps at `aside/do`: the person's whole profile opens.
const PROFILE: &str = "profile";

/// Its tabs, as data/aside.tsv has them.
const TABS: Tabbed = Tabbed { table: data::ASIDE, path: TAB, prefix: "aside" };

type Row = Vec<String>;

/// The rows of data/aside.tsv that begin with a word, without it.
fn table(word: &str) -> Vec<Vec<&'static str>> {
    data::row(data::ASIDE).into_iter().filter(|row| row.first() == Some(&word)).map(|row| row[1..].to_vec()).collect()
}

/// What the column holds, and what a press on it does.
struct Aside {
    desk: Desk,
}

impl Aside {
    /// The host of the open room: the site's, or one the person made.
    fn venue(&self) -> Option<Venue> {
        let room = self.desk.room();
        let made = self.desk.made();
        self.desk.site.venue.iter().chain(&made).find(|venue| venue.plate.iter().any(|plate| plate.id == room)).cloned()
    }

    /// The person a direct room is with: the column is about them.
    fn person(&self) -> Option<crate::site::Member> {
        let room = self.desk.room();
        let with = self.desk.plate(&room).map(|plate| plate.with).filter(|with| !with.is_empty()).or_else(|| self.desk.direct_with(&room))?;
        self.desk.site.member(&with).cloned()
    }

    /// A direct room's column: the person it is with, their face, what
    /// they are, what they say of themself, and Profile for all of them.
    fn of_person(&self, name: &str, one: &crate::site::Member) -> Vec<Row> {
        let text = &self.desk.text;
        let between = format!(" {} ", text.get("between"));
        match name {
            "aside.head" => {
                let line = [format!("@{}", one.handle), one.role.clone(), one.state.clone()].into_iter().filter(|said| said.len() > 1).collect::<Vec<_>>().join(&between);
                vec![vec![one.id.clone(), one.name.clone(), line]]
            }
            "aside.do" => vec![vec![PROFILE.to_string(), "person.title".to_string(), "user".to_string()]],
            "aside.face" => self.desk.site.face(&one.id).map(|cell| vec![one.id.clone(), String::new(), format!("{} {} {}", cell.sheet, cell.column, cell.index)]).into_iter().collect(),
            "aside.picture" => Some(vec![one.id.clone(), String::new(), one.banner.clone()]).filter(|row| !row[2].is_empty()).into_iter().collect(),
            "aside.about" => Some(vec!["bio".to_string(), one.bio.clone()]).filter(|row| !row[1].is_empty()).into_iter().collect(),
            "aside.fact" => [("person.fact.handle", one.handle.clone()), ("person.fact.role", one.role.clone()), ("person.fact.state", one.state.clone()), ("person.since", one.since.clone())]
                .into_iter().filter(|(_, value)| !value.is_empty()).map(|(word, value)| vec![word.to_string(), text.get(word).to_string(), value]).collect(),
            _ => Vec::new(),
        }
    }

    /// Phiegg's records of the host and of its club, where the book has
    /// them.
    fn record(&self, venue: &Venue) -> (Option<Found>, Option<Found>) {
        let book = self.desk.book();
        let Some(row) = table("host").into_iter().next().filter(|row| row.len() >= 3) else { return (None, None) };
        let host = book.record(row[0], &book.key(row[0], &venue.id)).cloned();
        let club = host.as_ref().and_then(|host| values(host.row.get(row[1])).into_iter().next()).and_then(|id| book.record(row[2], &id).or_else(|| book.named(&id)).cloned());
        (host, club)
    }

    /// A field of the host's record or its club's, as words: an id named
    /// by what it names.
    fn field(&self, host: &Option<Found>, club: &Option<Found>, of: &str, field: &str) -> String {
        let row = match of {
            "club" => club.as_ref(),
            _ => host.as_ref(),
        };
        let value = row.and_then(|found| found.row.get(field));
        let book = self.desk.book();
        let named: Vec<String> = values(value).iter().filter_map(|id| book.named(id).map(|found| found.name.clone())).filter(|name| !name.is_empty()).collect();
        match named.is_empty() {
            true => said(value),
            false => named.join(", "),
        }
    }

    /// Who is on a line: their name (and handle), their place on the
    /// line, what this window's call knows of them, how loud they are.
    fn line(&self, roster: &Roster, icon: &str) -> Vec<Row> {
        let text = &self.desk.text;
        let between = format!(" {} ", text.get("between"));
        let seen = self.desk.calling();
        roster.list.iter().map(|(peer, caller)| {
            let name = caller.handle.as_deref().map_or(caller.name.clone(), |handle| format!("{}{between}{handle}", caller.name));
            let far = seen.as_ref().and_then(|seen| seen.far.iter().find(|far| far.peer == *peer));
            let known = match (far, seen.is_some() && self.desk.me().as_deref() == Some(peer)) {
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
                (None, false) => String::new(),
            };
            let level = far.map(|far| far.level.clamp(0.0, 1.0).to_string()).unwrap_or_default();
            vec![peer.clone(), name, icon.to_string(), peer.clone(), known, level]
        }).collect()
    }

    /// What a list holds, whichever tab is on.
    fn list(&self, name: &str) -> Vec<Row> {
        let (desk, text) = (&self.desk, &self.desk.text);
        let room = desk.room();
        let Some(plate) = desk.plate(&room) else {
            return match name {
                "aside.head" => vec![vec![String::new(), text.get("present.nowhere").to_string(), String::new()]],
                _ => Vec::new(),
            };
        };
        // A direct room is about the person it is with; its line stays.
        if let Some(one) = self.person().filter(|_| !matches!(name, "aside.line" | "aside.call" | "aside.watch" | "aside.tab" | "aside.calls")) {
            return self.of_person(name, &one);
        }
        let venue = self.venue();
        let (host, club) = venue.as_ref().map(|venue| self.record(venue)).unwrap_or_default();
        let face = |cell: Option<&crate::site::Face>| cell.map(|cell| format!("{} {} {}", cell.sheet, cell.column, cell.index)).unwrap_or_default();
        match name {
            // Its name, and what it is under it.
            "aside.head" => {
                let label = venue.as_ref().map_or(plate.label.clone(), |venue| venue.label.clone());
                let line: Vec<String> = table("line").into_iter().filter(|row| row.len() >= 3).filter_map(|row| {
                    let value = self.field(&host, &club, row[0], row[1]);
                    let value = match (value.is_empty(), row[1], venue.as_ref()) {
                        (true, "members", Some(venue)) if venue.count > 0 => super::tool::grouped(venue.count),
                        (true, "online", Some(venue)) if venue.online > 0 => super::tool::grouped(venue.online),
                        _ => value,
                    };
                    Some(match row[2] {
                        "-" => value.clone(),
                        word => text.get(word).replace("{n}", &value),
                    }).filter(|_| !value.is_empty())
                }).collect();
                vec![vec![plate.id.clone(), label, line.join(&format!(" {} ", text.get("between")))]]
            }
            // Join or leave a host of the catalogue.
            "aside.do" => {
                let Some(venue) = venue.as_ref() else { return Vec::new() };
                let own = desk.site.venue.iter().position(|one| one.id == venue.id).is_none_or(|at| at < desk.site.extra.nav);
                let deed = match desk.joined(venue) {
                    true => ["leave", "host.leave", "log-out"],
                    false => ["join", "host.enter", "log-in"],
                };
                match own {
                    true => Vec::new(),
                    false => vec![deed.iter().map(|cell| cell.to_string()).collect()],
                }
            }
            // Its face, its mark, as Phiegg's host page puts it by its name.
            "aside.face" => {
                let mark = face(venue.as_ref().and_then(|venue| venue.mark.as_ref()));
                Some(vec![plate.id.clone(), String::new(), mark]).filter(|row| !row[2].is_empty()).into_iter().collect()
            }
            // Its world's picture, or its club's.
            "aside.picture" => {
                let picture = [&host, &club].into_iter().flatten().map(|found| found.picture.clone()).find(|picture| !picture.is_empty()).unwrap_or_default();
                Some(vec![plate.id.clone(), String::new(), picture]).filter(|row| !row[2].is_empty()).into_iter().collect()
            }
            // Nobody on the line: say so, and where a page reaches it; and
            // why this window is not in the call, and what carries it.
            "aside.line" => {
                let (call, watch) = desk.on(&plate.id);
                let how = match desk.line.as_ref() {
                    Ok(at) => text.fill("present.how", "at", &format!("http://{at}")).replace("{key}", &desk.key(&plate.id)),
                    Err(_) => text.get("present.shut").to_string(),
                };
                let nobody = Some([text.get("present.none").to_string(), how]).filter(|_| call.is_empty() && watch.is_empty()).into_iter().flatten();
                let carried = desk.calling().map(|seen| text.fill("call.carry", "engine", &seen.engine));
                let said = nobody.chain(Some(desk.why()).filter(|why| !why.is_empty())).chain(carried);
                said.enumerate().map(|(index, said)| vec![index.to_string(), said]).collect()
            }
            "aside.call" => self.line(&desk.on(&plate.id).0, CALL),
            "aside.watch" => self.line(&desk.on(&plate.id).1, WATCH),
            "aside.tab" => TABS.list(desk, &|name| self.list(name)),
            "aside.about" => table("about").into_iter().filter(|row| row.len() >= 2).map(|row| self.field(&host, &club, row[0], row[1])).filter(|said| !said.is_empty()).enumerate().map(|(index, said)| vec![index.to_string(), said]).collect(),
            "aside.fact" => {
                let fact: Vec<Row> = table("fact").into_iter().filter(|row| row.len() >= 2).map(|row| (row[1], self.field(&host, &club, row[0], row[1]))).filter(|(_, value)| !value.is_empty())
                    .map(|(field, value)| vec![field.to_string(), text.get(&format!("aside.fact.{field}")).to_string(), value]).collect();
                // A host Phiegg does not keep: what this app knows of it.
                let own = venue.as_ref().map(|venue| [("members", venue.count), ("online", venue.online), ("rooms", venue.plate.len() as u64)]).into_iter().flatten().filter(|(_, n)| *n > 0)
                    .map(|(field, n)| vec![field.to_string(), text.get(&format!("aside.fact.{field}")).to_string(), super::tool::grouped(n)]).collect();
                match fact.is_empty() {
                    true => own,
                    false => fact,
                }
            }
            // Its rooms: a press goes to one.
            "aside.room" => venue.as_ref().map(|venue| venue.plate.iter().map(|one| {
                let unread = Some(text.get("aside.unread").replace("{n}", &one.unread.to_string())).filter(|_| one.unread > 0).unwrap_or_default();
                vec![one.id.clone(), one.label.clone(), desk.icon(&one.icon).to_string(), String::new(), one.topic.clone(), unread, String::new()]
            }).collect()).unwrap_or_default(),
            // The room's people: a press puts one beside the room.
            "aside.member" => crowd::named(desk, &plate.member).into_iter().map(|seat| {
                let face = seat.cell.as_ref().map(|(sheet, column, index)| format!("{sheet} {column} {index}")).unwrap_or_default();
                let icon = match face.is_empty() {
                    true => "user".to_string(),
                    false => String::new(),
                };
                vec![seat.id.clone(), seat.name.clone(), icon, String::new(), seat.said.clone(), String::new(), face]
            }).collect(),
            // Every call kept in this room, the newest first, each a player
            // card with the room's picture and face.
            "aside.calls" => {
                let first = |name: &str| self.list(name).first().map(|row| row[2].clone()).unwrap_or_default();
                let (picture, face, live) = (first("aside.picture"), first("aside.face"), desk.calling().is_some());
                desk.history().into_iter().filter(|one| one.room == plate.id).map(|one| one.card(text, picture.clone(), face.clone(), live)).collect()
            }
            "aside.rule" => table("rule").into_iter().filter(|row| row.len() >= 2).flat_map(|row| {
                let source = match row[0] {
                    "club" => club.as_ref(),
                    _ => host.as_ref(),
                };
                values(source.and_then(|found| found.row.get(row[1])))
            }).enumerate().map(|(index, rule)| vec![index.to_string(), rule]).collect(),
            _ => Vec::new(),
        }
    }

    /// What a list holds now: a tab's lists only while it is on.
    fn rows(&self, name: &str) -> Vec<Row> {
        TABS.rows(&self.desk, name, &|name| self.list(name))
    }
}

impl PState for Aside {
    fn get(&self, path: &str) -> String {
        match path {
            TAB => TABS.on(&self.desk, &|name| self.list(name)),
            _ => self.desk.shelf.get(path),
        }
    }

    fn set(&self, path: &str, value: &str) {
        let desk = &self.desk;
        match (path, value) {
            (DO, PROFILE) => self.person().into_iter().for_each(|one| *desk.now.who.borrow_mut() = Some(one.id)),
            (DO, deed) => self.venue().into_iter().for_each(|venue| desk.enter(&venue.id, deed == "join")),
            (ROOM, room) => _ = desk.show(room),
            // The Calls tab holds the open room's calls only: a card is of
            // this room already, its Join calls it again.
            (CALL_AT, _) => {}
            (JOIN, _) => desk.join(),
            (LEAVE, _) => desk.leave(),
            (WHO, id) if desk.site.member(id).is_some() => *desk.now.who.borrow_mut() = Some(id.to_string()),
            (WHO, _) => {}
            _ => desk.shelf.set(path, value),
        }
    }
}

/// The column's form, its lists from the open room's host and line.
fn form(desk: &Desk) -> PForm {
    let aside = Rc::new(Aside { desk: desk.clone() });
    let rows = aside.clone();
    let state: Vec<(String, Rc<dyn PState>)> = vec![("aside".to_string(), aside), ("ui".to_string(), desk.shelf.clone())];
    let source = Rc::new(PListed::new(data::LISTING, Rc::new(move |name: &str| rows.rows(name)), desk.text.clone(), state));
    PForm::new("aside", data::FORM, source, desk.text.clone(), desk.shade.clone())
}

/// The column beside a room.
pub struct Present {
    frame: PRect,
    desk: Desk,
    form: RefCell<PForm>,
    /// The host it was drawn for last: another begins at its top, on
    /// its first tab.
    seen: RefCell<Option<String>>,
}

impl Present {
    pub fn new(desk: Desk) -> Present {
        Present { frame: PRect::default(), form: RefCell::new(form(&desk)), desk, seen: RefCell::default() }
    }

    /// The host of the open room, or the room where it is of none.
    fn host(&self) -> String {
        let room = self.desk.room();
        self.desk.host_of(&room).map_or(room, |(id, _)| id)
    }
}

impl PDraw for Present {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let host = Some(self.host());
        if *self.seen.borrow() != host {
            self.desk.shelf.set(TAB, "");
            let mut fresh = form(&self.desk);
            fresh.set_frame(self.frame);
            *self.form.borrow_mut() = fresh;
            *self.seen.borrow_mut() = host;
        }
        self.form.borrow().draw(paint, origin);
    }
}

impl PView for Present {
    fn hand(&self) -> Vec<PRect> {
        self.form.borrow().hand()
    }

    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
        self.form.get_mut().set_frame(frame);
    }

    /// In a call what is heard and seen changes by itself, and every
    /// frame is drawn; the tabs' light moves.
    fn tick(&mut self) -> bool {
        let moving = self.form.get_mut().tick();
        moving | self.desk.call.borrow().is_some()
    }

    /// While the line is open, somebody may come or go at any time:
    /// what it watches is who is on the open room's line.
    fn beat(&self) -> Option<u64> {
        use std::hash::{Hash, Hasher};
        let mut seen = std::collections::hash_map::DefaultHasher::new();
        format!("{:?}", self.desk.on(&self.desk.room())).hash(&mut seen);
        self.desk.line.is_ok().then(|| seen.finish())
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        self.form.get_mut().respond(event)
    }
}
