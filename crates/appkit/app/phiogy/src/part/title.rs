//! The title: which room is open. Its icon, what it is called and,
//! after that, its topic, over a line. At its right its tools: a call
//! begun, with the camera or without, the room searched, what is pinned
//! in it. While somebody else is calling there, the two that begin a
//! call give way to Join, and who is calling is said after the name.

use appkit::pui::core::{PDraw, PEvent, PFit, PPaint, PPoint, PRect, PSize, PView};

use std::cell::{Cell, RefCell};

use appkit::pui::core::{PLight, light};

use super::{DIM, Desk, GAP, ICON, INK, Join, LINE, RADIUS, Tool, fit, tool};
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
}

impl Title {
    pub fn new(desk: Desk) -> Title {
        Title { frame: PRect::default(), join: Join::new(desk.clone()), tool: Tool::read(data::ACT), spot: RefCell::default(), over: Cell::default(), face: Cell::default(), desk }
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
            PINS => now.pins.set(!now.pins.get()),
            FAVE => self.desk.star(&self.desk.room(), !self.desk.starred(&self.desk.room())),
            CALL => self.desk.dial(false),
            _ => self.desk.dial(true),
        }
    }

    /// Draw the tools so that the last ends at `end`, their middle at
    /// its height. How wide they are together, and what the one under
    /// the pointer is called.
    fn tools(&self, paint: &mut dyn PPaint, end: PPoint) -> (f32, String) {
        let (gap, line) = (paint.size(GAP), paint.size(LINE));
        let (mut spot, mut right, mut said) = (Vec::new(), end.x, String::new());
        // From the right; a call is begun only where one can be.
        let call = [VIDEO, CALL].into_iter().filter(|_| self.desk.callable());
        let list: Vec<&'static str> = [FAVE, PINS, FIND].into_iter().chain(call).filter(|name| self.tool.of(name, false, &self.desk.text).is_some()).collect();
        // The tools belong together: one ground under them all.
        let (count, radius) = (list.len() as f32, paint.size(RADIUS));
        let wide = count * (line + gap) + (count - 1.0).max(0.0) * gap / 4.0;
        light::label(paint, PRect::new(end.x - wide - gap / 4.0, end.y - (line + gap) / 2.0 - gap / 8.0, wide + gap / 2.0, line + gap * 1.25), radius, PLight::Rest, true, "");
        for name in list {
            let on = self.on(name);
            let Some((icon, word)) = self.tool.of(name, on, &self.desk.text) else { continue };
            let rect = PRect::new(right - line - gap, end.y - (line + gap) / 2.0, line + gap, line + gap);
            let over = self.over.get() == Some(name);
            let light = match (on, over) {
                (true, _) => Some(PLight::Strong),
                (false, true) => Some(PLight::Rest),
                // At rest a tool stands on the ground of them all.
                (false, false) => None,
            };
            tool::button(paint, rect, icon, light);
            if over {
                said = word.to_string();
            }
            spot.push((rect, name));
            right = rect.origin.x - gap / 4.0;
        }
        *self.spot.borrow_mut() = spot;
        (end.x - right, said)
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
        let (tools, said) = self.tools(paint, PPoint::new(origin.x + start + full - pill - gap / 2.0, middle));
        let wide = full - pill - tools - gap / 2.0;
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
        self.spot.borrow().iter().map(|(rect, _)| *rect).chain(self.join.hands()).collect()
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
                tool.is_some() || faced || self.join.press(place)
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
