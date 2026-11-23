//! The stream: a room's talk, as the web's room has it
//! (`puikit/app/phi-kit/src/blocks/RoomStream.tsx`). What was said, the
//! oldest at the top and the newest at the foot, where it opens: a line
//! where each day begins; one person's messages in a row under one
//! face and name; over a message that answers another, whom it answers
//! and how that began; its pictures; the marks it was given.
//!
//! At its foot is the field a message is written in: Enter, or the
//! control beside it, says it, and it joins the stream at once. It is
//! kept as the web keeps it (`part/talk.rs`). Before that control is
//! the one that posts a picture: the system asks which file of this
//! disk, and it is said in the room as a message of its own.
//!
//! The message under the pointer, or the one pressed last, shows its
//! controls: answer it, like it, pin it. A press on a mark gives it or
//! takes it back; on a face, opens that person beside the room; on a
//! picture, opens it in the viewer; on what a message answers, brings
//! that message into sight. While the room is searched a field stands
//! over the stream and only the messages that hold what is typed show;
//! its pinned messages are listed there when they are asked for.
//!
//! `stream/lay.rs` lays it out, `stream/one.rs` draws one message. Its
//! controls, their icons and their words are `data/act.tsv`.

mod act;
mod lay;
mod one;

pub use act::{Act, Spot};

use std::cell::{Cell, RefCell};

use appkit::pui::control::{PControl, PField};
use appkit::pui::core::{PEvent, PPoint, PRect, PView, send};
use appkit::pui::layout::PScroll;
use appkit::pui::window::PDeed;

use super::{Desk, GAP, ROW, Tool};
use crate::data;

/// The theme's tokens: how large a face is, and how wide and how high
/// a picture posted in the room is drawn.
/// The modal a press on a picture opens.
pub const VIEW: &str = "view";
/// The rows of `data/act.tsv` the stream draws, by what each does.
pub const REPLY: &str = "reply";
pub const LIKE: &str = "like";
pub const PIN: &str = "pin";
pub const SEND: &str = "send";
pub const POST: &str = "post";
pub const UNANSWER: &str = "unanswer";

/// What was said in the open room, and where more is said.
pub struct Stream {
    frame: PRect,
    desk: Desk,
    tool: Tool,
    scroll: Cell<PScroll>,
    /// The room it was drawn for last, and how many messages it had:
    /// another room, and one more message, are shown from the foot.
    seen: RefCell<(String, usize)>,
    /// Where it was drawn last; its places to press; where its messages
    /// are, between its head and its foot; and each message's id with
    /// its top and its foot.
    origin: Cell<PPoint>,
    spot: RefCell<Vec<Spot>>,
    body: Cell<PRect>,
    place: RefCell<Vec<(String, f32, f32)>>,
    /// The message the pointer is over, or that was pressed last, and
    /// the control the pointer is over.
    over: RefCell<String>,
    hand: RefCell<Option<Act>>,
    /// Where the pointer was last over the stream, in the window: what
    /// is under it is found again as the stream is drawn, so a turn of
    /// the wheel brings the controls of the message it brought there.
    point: Cell<Option<PPoint>>,
    /// The field a message is written in, and the one the room is
    /// searched with; and whether the keys are the first's while the
    /// second is shown.
    write: PField,
    find: PField,
    aim: Cell<bool>,
}

impl Stream {
    pub fn new(desk: Desk) -> Stream {
        let mut write = PField::new(PRect::default());
        write.focused = true;
        Stream { frame: PRect::default(), desk, tool: Tool::read(data::ACT), scroll: Cell::default(), seen: RefCell::default(), origin: Cell::default(), spot: RefCell::default(), body: Cell::default(), place: RefCell::default(), over: RefCell::default(), hand: RefCell::default(), point: Cell::default(), write, find: PField::new(PRect::default()), aim: Cell::default() }
    }

    /// Whether the keys are the search field's.
    fn seeking(&self) -> bool {
        self.desk.now.finding.get() && !self.aim.get()
    }

    /// Say what the field holds, and empty it.
    fn say(&mut self) -> bool {
        let said = self.desk.write(&self.write.value());
        if said {
            self.write.store(String::new());
        }
        said
    }

    /// Do what a place does.
    fn act(&mut self, act: Act) {
        let now = &self.desk.now;
        match act {
            Act::Shot(number) => self.desk.look(self.desk.shot(), number),
            Act::Face(id) => *now.who.borrow_mut() = Some(id),
            Act::Seek(id) => {
                now.pins.set(false);
                *now.seek.borrow_mut() = Some(id);
            }
            Act::Mark(id, icon) => self.desk.give(&id, &icon),
            Act::Reply(id) => {
                let answered = *now.answer.borrow() == id;
                *now.answer.borrow_mut() = Some(id).filter(|_| !answered).unwrap_or_default();
            }
            Act::Pin(id) => self.desk.said().iter().filter(|one| one.id == id).for_each(|one| self.desk.pin(one)),
            Act::Unanswer => now.answer.borrow_mut().clear(),
            Act::Send => _ = self.say(),
            // The system asks which file; its answer comes as a word.
            Act::Post => self.desk.wish.borrow_mut().push(PDeed::Choose(POST.to_string())),
        }
    }

    /// The message at a height of the window, where one is.
    fn at(&self, y: f32) -> String {
        let inside = self.body.get().origin.y <= y && y < self.body.get().origin.y + self.body.get().size.height;
        let found = self.place.borrow().iter().find(|(_, top, foot)| inside && *top <= y && y < *foot).map(|(id, ..)| id.clone());
        found.unwrap_or_default()
    }

    /// A key: Enter says what was written, Escape leaves the search or
    /// what is being answered, and any other is the field's.
    fn key(&mut self, event: &PEvent, key: char) -> bool {
        let (finding, answering) = (self.desk.now.finding.get(), !self.desk.now.answer.borrow().is_empty());
        match (key, self.seeking(), finding, answering) {
            (PEvent::ENTER, false, ..) => self.say(),
            (PEvent::ENTER, true, ..) => false,
            (PEvent::ESCAPE, _, true, _) => {
                self.desk.now.finding.set(false);
                true
            }
            (PEvent::ESCAPE, _, false, true) => {
                self.desk.now.answer.borrow_mut().clear();
                true
            }
            (PEvent::ESCAPE, ..) => false,
            (_, true, ..) => self.find.respond(event),
            (_, false, ..) => self.write.respond(event),
        }
    }
}

impl PView for Stream {
    /// Where a press on it does something.
    fn hand(&self) -> Vec<PRect> {
        self.spot.borrow().iter().map(|spot| spot.rect).collect()
    }

    fn frame(&self) -> PRect {
        self.frame
    }

    /// The two fields keep their places: the search at the top of the
    /// column, the writing at its foot, a control's width left of its
    /// end for the one that says it.
    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
        let (gap, row) = (self.desk.size(GAP), self.desk.size(ROW));
        let (start, wide) = self.desk.column(frame.size.width);
        self.find.set_frame(PRect::new(start, gap / 2.0, wide, row));
        self.write.set_frame(PRect::new(start, frame.size.height - row - gap / 2.0, (wide - (row + gap / 2.0) * 2.0).max(0.0), row));
    }

    /// A search that was left is forgotten, and the keys are the
    /// writing's again.
    fn tick(&mut self) -> bool {
        if !self.desk.now.finding.get() {
            self.find.store(String::new());
            self.aim.set(false);
        }
        (self.find.focused, self.write.focused) = (self.seeking(), !self.seeking());
        false
    }

    /// The file the system was asked for, as a word with its path
    /// after it: the picture is posted. The word with no path asks the
    /// system, as a press on the control does.
    fn tell(&mut self, word: &str) -> bool {
        match word.strip_prefix(POST) {
            // The word alone is a press on the control: the system is asked.
            Some("") => {
                self.act(Act::Post);
                true
            }
            Some(rest) => rest.strip_prefix(' ').is_some_and(|path| self.desk.post(path)),
            None => false,
        }
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        self.tick();
        // What was drawn is in the window's coordinates.
        let window = |at: PPoint| at.plus(self.origin.get());
        let under = |at: PPoint| self.spot.borrow().iter().rev().find(|spot| spot.rect.contains(window(at))).map(|spot| spot.act.clone());
        match *event {
            PEvent::KeyDown(key) => self.key(event, key),
            // The wheel turned away from the hand goes up.
            PEvent::Scroll(_, by) => {
                let mut scroll = self.scroll.get();
                let any = scroll.turn(-by.y);
                self.scroll.set(scroll);
                any
            }
            // The window is drawn again when the pointer comes over
            // another message or another control.
            PEvent::PointerMove(at) => {
                self.point.set(Some(window(at)));
                let (message, hand) = (self.at(window(at).y), under(at));
                let moved = *self.over.borrow() != message || *self.hand.borrow() != hand;
                (*self.over.borrow_mut(), *self.hand.borrow_mut()) = (message, hand);
                moved
            }
            PEvent::PointerDown(at) => {
                let (found, message) = (under(at), self.at(window(at).y));
                // A press in a field gives it the keys.
                let searched = self.desk.now.finding.get() && self.find.frame().contains(at);
                self.aim.set(self.desk.now.finding.get() && !searched && (self.aim.get() || self.write.frame().contains(at)));
                let typed = send(&mut self.write, event) | (searched && send(&mut self.find, event));
                let known = found.is_some() || !message.is_empty();
                found.into_iter().for_each(|act| self.act(act));
                if !message.is_empty() {
                    *self.over.borrow_mut() = message;
                }
                typed || known
            }
            _ => false,
        }
    }
}
