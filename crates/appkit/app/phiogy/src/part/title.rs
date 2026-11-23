//! The title: which room is open. Its icon, what it is called and,
//! after that, its topic, over a line. At its right its tools: a call
//! begun, with the camera or without, the room searched, what is pinned
//! in it. While somebody else is calling there, the two that begin a
//! call give way to Join, and who is calling is said after the name.

use appkit::pui::core::{PDraw, PEvent, PFit, PPaint, PPoint, PRect, PSize, PView};

use std::cell::{Cell, RefCell};

use appkit::pui::block::{PBand, PCrumb, PPill};
use appkit::pui::window::PDeed;

use super::{DIM, Desk, GAP, ICON, INK, Join, LINE, Tool, fit};
use crate::data;

/// The theme's tokens for the line under the title.
pub const RULE: &str = "line.ink";
pub const WIDTH: &str = "line.width";
/// How large the face of a direct room's person is.
pub const FACE: &str = "title.face";
/// The rows of `data/act.tsv` the title draws: search the room, and
/// list what is pinned in it.
pub const FIND: &str = "find";
pub const PINS: &str = "pins";
/// Star the room, so that it is among the favorites.
pub const FAVE: &str = "fave";
/// The search field: it opens the Search window in the middle. And
/// how wide it is, where there is room for it.
pub const SEEK: &str = "seek";
pub const SEARCH: &str = "title.search";
/// And the two that begin a call: without the camera, and with it.
pub const CALL: &str = "call";
pub const VIDEO: &str = "video";

/// What is over a room.
pub struct Title {
    frame: PRect,
    desk: Desk,
    /// The pills of the room's call, at its right.
    join: Join,
    /// Its tools, left of the pills: where each was drawn last, in the
    /// window, and the one the pointer is over.
    tool: Tool,
    spot: RefCell<Vec<(PRect, &'static str)>>,
    over: Cell<Option<&'static str>>,
    /// Where the face of a direct room's person was drawn last.
    face: Cell<PRect>,
    /// Where each step of the way to the room was drawn last, and the
    /// host a press on it goes to.
    crumb: RefCell<Vec<(PRect, String)>>,
}

impl Title {
    pub fn new(desk: Desk) -> Title {
        Title { frame: PRect::default(), join: Join::new(desk.clone()), tool: Tool::read(data::ACT), spot: RefCell::default(), over: Cell::default(), crumb: RefCell::default(), face: Cell::default(), desk }
    }

    /// Whether a tool is on: one that begins a call never is.
    fn on(&self, name: &str) -> bool {
        match name {
            FIND => self.desk.now.finding.get(),
            PINS => self.desk.now.pins.get(),
            FAVE => self.desk.starred(&self.desk.room()),
            _ => false,
        }
    }

    /// A press on a tool: the room searched or not, its pins listed or
    /// not, a call begun.
    fn turn(&self, name: &str) {
        let now = &self.desk.now;
        match name {
            FIND => now.finding.set(!now.finding.get()),
            SEEK => self.desk.wish.borrow_mut().push(PDeed::Open(super::SEARCH.to_string())),
            PINS => now.pins.set(!now.pins.get()),
            FAVE => self.desk.star(&self.desk.room(), !self.desk.starred(&self.desk.room())),
            CALL => self.desk.dial(false),
            _ => self.desk.dial(true),
        }
    }

    /// Draw the tools so that the last ends at `end`, their middle at
    /// its height. How wide they are together, and what the one under
    /// the pointer is called.
    fn tools(&self, paint: &mut dyn PPaint, end: PPoint, from: f32) -> (f32, String) {
        let text = &self.desk.text;
        // In reading order; a call is begun only where one can be.
        let call = [CALL, VIDEO].into_iter().filter(|_| self.desk.callable());
        let list: Vec<(&'static str, &str, &str, bool)> = call.chain([FIND, PINS, FAVE]).filter_map(|name| self.tool.of(name, self.on(name), text).map(|(icon, word)| (name, icon, word, self.on(name)))).collect();
        // The tools belong together: the kit's group, on one ground.
        let pill: Vec<(&str, &str, bool)> = list.iter().map(|(_, icon, _, on)| (*icon, "", *on)).collect();
        let high = pill.high(paint);
        let rect = pill.draw_end(paint, PPoint::new(end.x, end.y - high / 2.0));
        let said = list.iter().find(|(name, ..)| self.over.get() == Some(*name)).map(|(_, _, word, _)| word.to_string()).unwrap_or_default();
        let mut spot: Vec<(PRect, &'static str)> = rect.into_iter().zip(list.iter().map(|(name, ..)| *name)).collect();
        // Left of them the search field: the kit's pill, as wide as the
        // theme says, with its icon and what it finds.
        let gap = paint.size(GAP);
        let seek = self.tool.of(SEEK, false, text).map(|(icon, word)| (icon, word, false));
        // Only where it leaves the room's name as much room as itself.
        let field = seek.and_then(|seek| {
            let room = (paint.size(SEARCH) - seek.sized(paint, 0.0).width).max(gap);
            let size = seek.sized(paint, room);
            let at = PPoint::new(end.x - pill.span(paint) - gap / 2.0 - size.width, end.y - size.height / 2.0);
            (at.x - from >= size.width).then(|| seek.lay(paint, at, false, room))
        });
        spot.extend(field.map(|rect| (rect, SEEK)));
        *self.spot.borrow_mut() = spot;
        (pill.span(paint) + field.map_or(0.0, |rect| rect.size.width + gap / 2.0), said)
    }
}

impl PDraw for Title {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, line, rule, icon, size) = (paint.size(GAP), paint.size(LINE), paint.size(WIDTH), paint.size(ICON), self.frame.size);
        let (ink, dim, ruled) = (paint.color(INK), paint.color(DIM), paint.color(RULE));
        let open = self.desk.room();
        let Some(plate) = self.desk.plate(&open) else { return };
        // It starts where what is under it does.
        let (start, full) = self.desk.column(size.width);
        let middle = origin.y + size.height / 2.0;
        // The pills of its call stand at the right; its name has the rest.
        let pill = self.join.draw(paint, PPoint::new(origin.x + start + full, middle));
        // Its tools stand left of them: search, and what is pinned.
        let (tools, said) = self.tools(paint, PPoint::new(origin.x + start + full - pill - gap / 2.0, middle), origin.x + start);
        let wide = full - pill - tools - gap / 2.0;
        // Before it the way to it: the host it is a room of, as a crumb.
        let host = self.desk.host_of(&open);
        let way: Vec<&str> = host.iter().map(|(_, label)| label.as_str()).collect();
        let (crumb, begun) = way.lead(paint, PPoint::new(origin.x + start, middle - line / 2.0));
        *self.crumb.borrow_mut() = crumb.into_iter().zip(host.iter().map(|(id, _)| id.clone())).collect();
        let (start, wide) = (start + begun - (origin.x + start), wide - (begun - (origin.x + start)));
        // A room with one person has their face; any other its icon.
        let face = paint.size(FACE);
        let (icon, left) = match self.desk.site.face(&plate.with) {
            Some(cell) => {
                let rect = PRect::new(origin.x + start, middle - face / 2.0, face, face);
                paint.picture(&cell.sheet, rect, face / 2.0, PFit::Cell { column: cell.column, index: cell.index });
                self.face.set(rect);
                (face, start + face + gap / 2.0)
            }
            None => {
                self.face.set(PRect::default());
                paint.icon(self.desk.icon(&plate.icon), PRect::new(origin.x + start, middle - icon / 2.0, icon, icon), ink);
                (icon, start + icon + gap / 2.0)
            }
        };
        let label = fit(&plate.label, wide - icon - gap / 2.0, paint);
        paint.text(PPoint::new(origin.x + left, middle - line / 2.0), &label, ink);
        let used = left + paint.measure(&label) + gap;
        let room = start + wide - used;
        // After its name its topic; or who is calling there; or, while
        // the pointer is over a tool, what the tool is called.
        let ring = self.desk.ringing().map(|name| self.desk.text.fill("call.incoming", "name", &name));
        let after = match said.is_empty() {
            true => ring.unwrap_or_else(|| plate.topic.clone()),
            false => said,
        };
        if room > gap * 2.0 && !after.is_empty() {
            paint.text(PPoint::new(origin.x + used, middle - line / 2.0), &fit(&after, room, paint), dim);
        }
        paint.fill(PRect { origin: origin.plus(PPoint::new(0.0, size.height - rule)), size: PSize { width: size.width, height: rule } }, ruled);
    }
}

impl PView for Title {
    /// Where a press on it does something.
    fn hand(&self) -> Vec<PRect> {
        let crumb: Vec<PRect> = self.crumb.borrow().iter().map(|(rect, _)| *rect).collect();
        self.spot.borrow().iter().map(|(rect, _)| *rect).chain(crumb).chain(self.join.hands()).collect()
    }

    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        match *event {
            // The pills and the tools keep where they were drawn in the
            // window. A tool is turned on, or off again.
            PEvent::PointerDown(at) => {
                let place = at.plus(self.frame.origin);
                let tool = self.spot.borrow().iter().find(|(rect, _)| rect.contains(place)).map(|(_, name)| *name);
                tool.into_iter().for_each(|name| self.turn(name));
                // A press on the face of a direct room's person opens
                // them beside the room.
                let with = self.desk.plate(&self.desk.room()).map(|plate| plate.with).filter(|_| self.face.get().contains(place));
                let faced = with.is_some();
                with.into_iter().for_each(|id| *self.desk.now.who.borrow_mut() = Some(id));
                // A press on a step of the way goes to that host.
                let step = self.crumb.borrow().iter().find(|(rect, _)| rect.contains(place)).map(|(_, id)| id.clone());
                let went = step.is_some_and(|id| self.desk.visit(&id));
                tool.is_some() || faced || went || self.join.press(place)
            }
            PEvent::PointerMove(at) => {
                let place = at.plus(self.frame.origin);
                let tool = self.spot.borrow().iter().find(|(rect, _)| rect.contains(place)).map(|(_, name)| *name);
                self.over.replace(tool) != tool
            }
            _ => false,
        }
    }
}
