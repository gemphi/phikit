//! The foot of the guide while the person is in a voice room, as the
//! web's menu has at its foot (`VoiceBar` in
//! `puikit/app/phi-kit/src/voice.tsx`): the room, which a press goes
//! to; the microphone; the sound of the others; and the way out. It is
//! there on every page the guide is, because the room goes on when the
//! page changes. With nobody in a voice room it is nothing, and takes
//! no room.
//!
//! The menu is narrow, so it is two rows on one ground: the room's
//! mark and name over its controls. The controls are rows of
//! `data/act.tsv`; while the pointer is over one, what it is called
//! stands beside them.

use std::cell::{Cell, RefCell};

use appkit::pui::block::PBand;
use appkit::pui::core::{PLight, PPaint, PPoint, PRect, light};

use super::super::{DIM, Desk, GAP, ICON, INK, LINE, RADIUS, ROW, Tool, fit};
use crate::data;

/// The rows of `data/act.tsv` the foot draws: the room it is of, the
/// microphone, the sound of the others, and the way out.
pub const WITHIN: &str = "within";
pub const HUSH: &str = "hush";
pub const DEAF: &str = "deaf";
pub const LEAVE: &str = "leave";

/// The voice room a person is in, at the guide's foot.
pub struct Foot {
    desk: Desk,
    tool: Tool,
    /// Where each thing to press was drawn last, in the window, and the
    /// one the pointer is over.
    spot: RefCell<Vec<(PRect, &'static str)>>,
    over: Cell<Option<&'static str>>,
}

impl Foot {
    pub fn new(desk: Desk) -> Foot {
        Foot { desk, tool: Tool::read(data::ACT), spot: RefCell::default(), over: Cell::default() }
    }

    /// How high its ground is: the room's row, and its controls with a
    /// little under them.
    fn ground(&self) -> f32 {
        self.desk.size(ROW) + self.desk.size(LINE) + self.desk.size(GAP) * 1.25
    }

    /// How high it is: its ground and the room over it; nothing with
    /// nobody in a voice room.
    pub fn high(&self) -> f32 {
        match self.desk.within() {
            Some(_) => self.ground() + self.desk.size(GAP) / 2.0,
            None => 0.0,
        }
    }

    /// Draw it in the lowest part of `rect`, which is the guide's.
    pub fn draw(&self, paint: &mut dyn PPaint, rect: PRect) {
        let Some((_, label, seen)) = self.desk.within() else { return self.spot.borrow_mut().clear() };
        let (gap, row, line, icon) = (paint.size(GAP), paint.size(ROW), paint.size(LINE), paint.size(ICON));
        let (ink, dim, radius) = (paint.color(INK), paint.color(DIM), paint.size(RADIUS));
        // A ground against the group it stands in, as the open room's row is.
        let high = self.ground();
        let ground = PRect::new(rect.origin.x + gap / 2.0, rect.origin.y + rect.size.height - high, rect.size.width - gap, high);
        light::label(paint, ground, radius, PLight::Rest, true, "");
        // Its tools: the microphone, the others' sound and the way out,
        // the kit's band; and what the one under the pointer is called.
        let room = PRect::new(ground.origin.x, ground.origin.y, ground.size.width, row);
        let list: Vec<(&'static str, &str, &str, bool)> = [(HUSH, seen.quiet), (DEAF, seen.own.deaf), (LEAVE, false)].into_iter().filter_map(|(name, on)| self.tool.of(name, on, &self.desk.text).map(|(icon, word)| (name, icon, word, on))).collect();
        let pill: Vec<(&str, &str, bool)> = list.iter().map(|(_, icon, _, on)| (*icon, "", *on)).collect();
        let said = list.iter().find(|(name, ..)| self.over.get() == Some(*name)).map(|(_, _, word, _)| word.to_string());
        // Over them: the room, its mark and its name, or that word.
        let left = match self.tool.of(WITHIN, false, &self.desk.text) {
            Some((name, _)) => {
                paint.icon(name, PRect::new(room.origin.x + gap / 2.0, room.center().y - icon / 2.0, icon, icon), ink);
                room.origin.x + gap + icon
            }
            None => room.origin.x + gap / 2.0,
        };
        let (words, color) = match said {
            Some(said) => (said, dim),
            None => (label, ink),
        };
        paint.text(PPoint::new(left, room.center().y - line / 2.0), &fit(&words, room.origin.x + room.size.width - left - gap / 2.0, paint), color);
        let mut spot: Vec<(PRect, &'static str)> = pill.draw_end(paint, PPoint::new(ground.origin.x + ground.size.width - gap / 4.0, room.origin.y + row)).into_iter().zip(list.iter().map(|(name, ..)| *name)).collect();
        spot.push((room, WITHIN));
        *self.spot.borrow_mut() = spot;
    }

    /// Where a press on it does something.
    pub fn hands(&self) -> Vec<PRect> {
        self.spot.borrow().iter().map(|(rect, _)| *rect).collect()
    }

    /// What is at a place in the window.
    fn at(&self, place: PPoint) -> Option<&'static str> {
        self.spot.borrow().iter().find(|(rect, _)| rect.contains(place)).map(|(_, name)| *name)
    }

    /// A press at a place in the window. `true` when it was on the foot.
    pub fn press(&self, place: PPoint) -> bool {
        let (name, within) = (self.at(place), self.desk.within());
        let Some(((id, _, seen), name)) = within.zip(name) else { return false };
        match name {
            HUSH => self.desk.hush(!seen.quiet),
            DEAF => self.desk.deaf(!seen.own.deaf),
            LEAVE => self.desk.leave(),
            _ => _ = self.desk.show(&id),
        }
        true
    }

    /// The pointer at a place in the window. `true` when what it is
    /// over changed.
    pub fn hover(&self, place: PPoint) -> bool {
        let over = self.at(place).filter(|name| *name != WITHIN);
        self.over.replace(over) != over
    }
}
