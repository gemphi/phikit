//! The find: the Search window. In one column down the middle: a heading,
//! a field that searches, a row of pills that narrow what is listed and
//! one at its end that turns the order, and every chat that is left, its
//! title over the day it began and how much was said in it. A press on a
//! chat opens it and shows the chat's space.
//!
//! The pills are `data/find.tsv`. What is drawn is what can be pressed:
//! each pill and each row leaves its rectangle behind when it is drawn.

use std::cell::{Cell, RefCell};

use appkit::pui::control::{PControl, PField, inside};
use appkit::pui::core::{PDraw, PEvent, PLight, PPaint, PPoint, PRect, PView, light, send, show};
use appkit::pui::layout::PScroll;

use super::{DIM, Desk, GAP, INK, LINE, fit};
use crate::date;

/// The theme's tokens for the window: how far down its heading is, how
/// high its field and a chat's row.
pub const TOP: &str = "find.top";
pub const FIELD: &str = "find.field";
pub const ROW: &str = "find.row";

/// What a press on something drawn does.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Press {
    /// Narrow to the pill of that number.
    Pill(usize),
    /// Turn the order.
    Turn,
    /// Open the chat of that number.
    Chat(usize),
}

/// The Search window.
pub struct Find {
    frame: PRect,
    desk: Desk,
    icon: String,
    field: PField,
    /// The pills: what each is called in the table, and what it reads.
    pill: Vec<(String, String)>,
    chosen: usize,
    /// Whether the newest chat is first.
    newest: bool,
    scroll: Cell<PScroll>,
    /// Where each pill and each chat was drawn last, in the window.
    spot: RefCell<Vec<(PRect, Press)>>,
}

impl Find {
    /// The window, its rows marked with that icon, its pills from that
    /// table (a pill a line: its name, then what it reads), its field
    /// taking the keys whenever it is shown.
    pub fn new(desk: Desk, icon: &str, table: &str) -> Find {
        let mut field = PField::new(PRect::default());
        // It stands in the box the window draws for it and its icon.
        (field.focused, field.bare) = (true, true);
        let line = table.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#'));
        let pill = line.filter_map(|line| line.split_once('\t')).map(|(name, label)| (name.trim().to_string(), label.trim().to_string())).collect();
        Find { frame: PRect::default(), desk, icon: icon.to_string(), field, pill, chosen: 0, newest: true, scroll: Cell::default(), spot: RefCell::default() }
    }

    /// The column everything is in: the body's.
    fn column(&self) -> (f32, f32) {
        self.desk.column(self.frame.size.width)
    }

    /// The numbers of the chats that are listed, in the order shown:
    /// those with what the field holds in their title, whatever its case,
    /// that the chosen pill lets through. A chat nothing was said in is
    /// not one to find.
    fn found(&self) -> Vec<usize> {
        let (wanted, history, now) = (self.field.value().to_lowercase(), self.desk.history.borrow(), self.desk.history.borrow().now());
        let pill = self.pill.get(self.chosen).map_or("", |(name, _)| name.as_str());
        let let_through = |when: u64| match pill {
            "today" => when > 0 && date::day(when) == date::day(now),
            "earlier" => when > 0 && date::day(when) != date::day(now),
            _ => true,
        };
        let chat = history.list().iter().enumerate();
        let mut found: Vec<usize> = chat.filter(|(_, chat)| chat.title().is_some_and(|title| title.to_lowercase().contains(&wanted)) && let_through(chat.when)).map(|(index, _)| index).collect();
        // The history keeps the newest first.
        if !self.newest {
            found.reverse();
        }
        found
    }
}

impl PDraw for Find {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, line, row, icon) = (paint.size(GAP), paint.size(LINE), paint.size(ROW), paint.size("icon.size"));
        let (ink, dim, round) = (paint.color(INK), paint.color(DIM), paint.size("tab.radius"));
        let (left, wide) = self.column();
        let left = origin.x + left;
        let mut spot = Vec::new();
        // The heading, in the middle.
        let head = self.desk.text.get("find.head");
        let at = PPoint::new(origin.x + (self.frame.size.width - paint.measure_head(head)) / 2.0, origin.y + paint.size(TOP));
        paint.head(at, head, ink);
        // The field: one long box, its icon a gap in, the text after it.
        let held = self.field.frame().moved(origin);
        let whole = PRect::new(left, held.origin.y, wide, held.size.height);
        light::surface(paint, whole, whole.size.height / 2.0, PLight::Rest, "");
        show(&self.field, paint, origin);
        paint.icon("search", PRect::new(left + gap, whole.center().y - icon / 2.0, icon, icon), dim);
        if self.field.value().is_empty() {
            paint.text(held.origin.plus(inside(paint, held.size.height)), self.desk.text.get("find.hint"), dim);
        }
        // The pills, from the left; the one that turns the order, at the right.
        let high = line + gap;
        let top = whole.origin.y + whole.size.height + gap * 1.5;
        let mut next = left;
        for (index, (_, label)) in self.pill.iter().enumerate() {
            let rect = PRect::new(next, top, paint.measure(label) + gap * 2.0, high);
            // The pill chosen has the strongest light, the others the faintest.
            let strength = match index == self.chosen {
                true => PLight::Strong,
                false => PLight::Soft,
            };
            light::label(paint, rect, round, strength, false, "");
            paint.text(PPoint::new(rect.origin.x + gap, rect.center().y - line / 2.0), label, ink);
            spot.push((rect, Press::Pill(index)));
            next += rect.size.width + gap / 2.0;
        }
        let (order, arrow) = match self.newest {
            true => (self.desk.text.get("find.newest"), "arrow-down"),
            false => (self.desk.text.get("find.oldest"), "arrow-up"),
        };
        let long = paint.measure(order) + icon + gap * 2.5;
        let turn = PRect::new(left + wide - long, top, long, high);
        light::label(paint, turn, round, PLight::Soft, true, "");
        paint.text(PPoint::new(turn.origin.x + gap, turn.center().y - line / 2.0), order, ink);
        paint.icon(arrow, PRect::new(turn.origin.x + turn.size.width - gap - icon, turn.center().y - icon / 2.0, icon, icon), ink);
        spot.push((turn, Press::Turn));
        // The chats scroll under the pills, and are drawn nowhere else.
        let (history, found) = (self.desk.history.borrow(), self.found());
        let under = top + high + gap;
        let room = PRect::new(left, under, wide, origin.y + self.frame.size.height - under);
        let mut scroll = self.scroll.get();
        scroll.fit(found.len() as f32 * row, room.size.height);
        self.scroll.set(scroll);
        paint.clip(room);
        let shown = |place: &usize| (*place + 1) as f32 * row > scroll.at() && *place as f32 * row < scroll.at() + room.size.height;
        for (place, index) in found.into_iter().enumerate().filter(|(place, _)| shown(place)) {
            let chat = &history.list()[index];
            let rect = PRect::new(left, room.origin.y + place as f32 * row - scroll.at(), wide, row);
            paint.icon(&self.icon, PRect::new(left, rect.center().y - icon / 2.0, icon, icon), ink);
            let beside = PPoint::new(left + icon + gap, rect.center().y - line);
            paint.text(beside, &fit(chat.title().unwrap_or_default(), wide - icon - gap, paint), ink);
            let count = self.desk.text.fill("find.count", "count", &chat.turn.len().to_string());
            let under = match date::label(chat.when, history.now(), &self.desk.text) {
                day if day.is_empty() => count,
                day => format!("{day} {} {count}", self.desk.text.get("find.between")),
            };
            paint.text(beside.plus(PPoint::new(0.0, line)), &under, dim);
            spot.push((rect, Press::Chat(index)));
        }
        paint.unclip();
        *self.spot.borrow_mut() = spot;
    }
}

impl PView for Find {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
        let (gap, icon, high) = (self.desk.size(GAP), self.desk.size("icon.size"), self.desk.size(FIELD));
        let (left, wide) = self.column();
        let lead = gap + icon + gap / 2.0;
        let top = self.desk.size(TOP) + self.desk.size("head.line") + gap * 2.0;
        self.field.set_frame(PRect::new(left + lead, top, wide - lead - gap, high));
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        match *event {
            PEvent::KeyDown(_) => self.field.respond(event),
            // The wheel turned away from the hand goes up the chats.
            PEvent::Scroll(_, by) => {
                let mut scroll = self.scroll.get();
                let any = scroll.turn(-by.y);
                self.scroll.set(scroll);
                any
            }
            PEvent::PointerDown(at) => {
                // What was drawn is in the window's coordinates.
                let place = at.plus(self.frame.origin);
                let press = self.spot.borrow().iter().rev().find(|(rect, _)| rect.contains(place)).map(|(_, press)| *press);
                match press {
                    Some(Press::Pill(index)) => {
                        self.chosen = index;
                        true
                    }
                    Some(Press::Turn) => {
                        self.newest = !self.newest;
                        true
                    }
                    Some(Press::Chat(index)) => self.desk.open(index),
                    None => send(&mut self.field, event),
                }
            }
            _ => false,
        }
    }
}
