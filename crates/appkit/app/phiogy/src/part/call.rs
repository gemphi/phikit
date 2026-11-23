//! A call in a room, over its stream, as the web's call stands there.
//! At its head the room called and how long. Then the tiles (`tile`),
//! a row that folds where the part is narrow: a camera at four to
//! three, a screen wider, a face round. Under them how loud this
//! window's microphone hears its person, its buttons as the web's call
//! has them (mute, the camera, a screen, end the call), in rows that
//! fold, and what the call does with them, or what the system did not
//! give.
//!
//! Every size is the theme's (`call.NAME`), every word the text
//! table's (`call.NAME`), and the pictures are the film's.

mod lay;
mod tile;

use std::cell::{Cell, RefCell};

use appkit::pui::core::{PDraw, PEvent, PFit, PLight, PPaint, PPoint, PRect, PView, light};
use appkit::pui::layout::fold;
use appkit::service::PTable;

use super::join::ROUND;
use super::present::BAR;
use super::{DIM, Desk, GAP, INK, LINE, RADIUS, fit};
use crate::data;
pub use tile::{Shown, Tile, face};

/// The theme's tokens: how wide a camera's tile and a screen's are at
/// most, how high each is for its width, how large a face is, the most
/// of the room's height a call takes, and how loud counts as heard.
pub const WIDE: &str = "call.tile";
pub const SCENE: &str = "call.scene";
pub const HIGH: &str = "call.tile.high";
pub const LOW: &str = "call.scene.high";
pub const FACE: &str = "call.face";
pub const SHARE: &str = "call.share";
pub const SPEAK: &str = "call.speak";

/// What a press on one of its pills does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Press {
    Hush(bool),
    Camera(bool),
    Share(bool),
    End,
}

/// The call this window is in, as the open room shows it.
pub struct Calling {
    frame: PRect,
    desk: Desk,
    /// `data/peer.tsv`: what the pictures of a call are named.
    table: PTable,
    /// Where it was drawn last, and each pill, in the window.
    origin: Cell<PPoint>,
    /// How many lines what it says took when it was drawn last.
    line: Cell<f32>,
    /// How many rows its pills took then.
    row: Cell<f32>,
    /// The share of their full size its tiles are drawn at.
    share: Cell<f32>,
    spot: RefCell<Vec<(PRect, Press)>>,
}

impl Calling {
    pub fn new(desk: Desk) -> Calling {
        Calling { frame: PRect::default(), desk, table: PTable::read(data::PEER), origin: Cell::default(), line: Cell::new(1.0), row: Cell::new(1.0), share: Cell::new(1.0), spot: RefCell::default() }
    }

    /// The tiles there are now.
    pub fn tile(&self) -> Vec<Tile> {
        self.desk.tile(&self.table, self.desk.size(SPEAK))
    }

    /// The pills: what each reads, what a press does, whether it says
    /// what is so.
    fn pill(&self) -> Vec<(String, Press, bool)> {
        let text = |name: &str| self.desk.text.get(name).to_string();
        let Some(seen) = self.desk.calling() else { return Vec::new() };
        let hush = match seen.quiet {
            true => (text("call.unmute"), Press::Hush(false), true),
            false => (text("call.mute"), Press::Hush(true), false),
        };
        let camera = match seen.own.camera {
            true => (text("call.cam.off"), Press::Camera(false), true),
            false => (text("call.cam.on"), Press::Camera(true), false),
        };
        let share = match seen.own.screen {
            true => (text("call.unshare"), Press::Share(false), true),
            false => (text("call.share"), Press::Share(true), false),
        };
        vec![hush, camera, share, (text("call.end"), Press::End, false)]
    }

    /// What the call does with what this window sends; or, first, what
    /// the system did not give it.
    fn said(&self) -> String {
        let fault = self.desk.calling().and_then(|seen| seen.own.fault.first().cloned().zip(seen.own.why.first().cloned()));
        let named = fault.map(|(name, why)| self.desk.text.fill(&format!("call.fault.{name}"), "why", &why)).filter(|said| !said.is_empty());
        named.unwrap_or_else(|| self.desk.text.get("call.live").to_string())
    }
}

impl PDraw for Calling {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let tile = self.tile();
        let Some(seen) = self.desk.calling().filter(|_| self.frame.size.height > 0.0) else { return self.spot.borrow_mut().clear() };
        let (gap, line, radius, round) = (paint.size(GAP), paint.size(LINE), paint.size(RADIUS), paint.size(ROUND));
        let (ink, dim) = (paint.color(INK), paint.color(DIM));
        let wide = self.frame.size.width;
        self.origin.set(origin);
        paint.clip(PRect { origin, size: self.frame.size });
        light::surface(paint, PRect::new(origin.x + gap / 2.0, origin.y + gap / 2.0, wide - gap, self.frame.size.height - gap), radius, PLight::Soft, "");
        // The head: the room called, and how long the call has been on.
        let room = self.desk.plate(&self.desk.room()).map(|plate| plate.label).unwrap_or_default();
        let clock = self.desk.since().map(|since| format!("{:02}:{:02}", since / 60, since % 60)).unwrap_or_default();
        let top = origin.y + gap;
        paint.text(PPoint::new(origin.x + gap * 1.5, top), &fit(&self.desk.text.fill("call.calling", "room", &room), wide - gap * 4.0 - paint.measure(&clock), paint), ink);
        paint.text(PPoint::new(origin.x + wide - gap * 1.5 - paint.measure(&clock), top), &clock, dim);
        let (rect, tiles) = self.lay(&tile, wide - gap, self.share.get());
        let from = PPoint::new(origin.x + gap / 2.0, top + line + gap / 2.0);
        for (one, rect) in tile.iter().zip(rect) {
            let rect = PRect::new(from.x + rect.origin.x, from.y + rect.origin.y, rect.size.width, rect.size.height);
            // One who is heard has a light around them.
            if one.loud {
                let ring = PRect::new(rect.origin.x - gap / 4.0, rect.origin.y - gap / 4.0, rect.size.width + gap / 2.0, rect.size.height + gap / 2.0);
                light::label(paint, ring, radius.max(ring.size.width / 2.0 * (matches!(one.shown, Shown::Face(..)) as u8 as f32)), PLight::Strong, false, "");
            }
            match &one.shown {
                Shown::Film(name) => {
                    light::label(paint, rect, radius, PLight::Rest, false, "");
                    paint.picture(name, rect, radius, PFit::Cover);
                }
                Shown::Face(id, name) => face(&self.desk, paint, rect, id, name),
            }
            let under = [(&one.name, ink), (&one.said, dim)];
            for (index, (said, color)) in under.into_iter().filter(|(said, _)| !said.is_empty()).enumerate() {
                let said = fit(said, rect.size.width.max(paint.size(lay::NAME)), paint);
                paint.text(PPoint::new(rect.center().x - paint.measure(&said) / 2.0, rect.origin.y + rect.size.height + index as f32 * line), &said, color);
            }
        }
        // How loud this window's person is, as far along as they are loud.
        let (left, inside, bar) = (origin.x + gap * 1.5, wide - gap * 3.0, paint.size(BAR));
        let mut next = from.y + tiles + gap;
        light::label(paint, PRect::new(left, next, inside, bar), bar / 2.0, PLight::Soft, false, "");
        let loud = seen.own.level.clamp(0.0, 1.0) * inside;
        if loud >= 1.0 {
            light::label(paint, PRect::new(left, next, loud, bar), bar / 2.0, PLight::Strong, false, "");
        }
        next += bar + gap / 2.0;
        // Its buttons, from the left, a row folding where the next
        // does not fit.
        let (mut spot, mut at, mut rows) = (Vec::new(), left, 1.0);
        for (label, press, so) in self.pill() {
            let width = paint.measure(&label) + gap * 2.0;
            if at > left && at + width > left + inside {
                (at, next, rows) = (left, next + line + gap + gap / 2.0, rows + 1.0);
            }
            let rect = PRect::new(at, next, width, line + gap);
            let strength = match so {
                true => PLight::Strong,
                false => PLight::Rest,
            };
            light::label(paint, rect, round, strength, false, "");
            paint.text(PPoint::new(rect.origin.x + gap, rect.center().y - line / 2.0), &label, ink);
            spot.push((rect, press));
            at += rect.size.width + gap / 2.0;
        }
        *self.spot.borrow_mut() = spot;
        self.row.set(rows);
        next += line + gap + gap / 2.0;
        // What it says is broken to fit; the next frame makes room for
        // as many lines as it took.
        let said = fold(&self.said(), inside, &|text| paint.measure(text));
        for (index, text) in said.iter().enumerate() {
            paint.text(PPoint::new(left, next + index as f32 * line), text, dim);
        }
        self.line.set(said.len() as f32);
        paint.unclip();
    }
}

impl PView for Calling {
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
        let PEvent::PointerDown(at) = *event else { return false };
        // A pill keeps where it was drawn in the window; a press comes
        // in this part's own measure.
        let place = at.plus(self.origin.get());
        let press = self.spot.borrow().iter().find(|(rect, _)| rect.contains(place)).map(|(_, press)| *press);
        match press {
            Some(Press::Hush(quiet)) => self.desk.hush(quiet),
            Some(Press::End) => self.desk.leave(),
            Some(Press::Camera(on)) => self.desk.camera(on),
            Some(Press::Share(on)) => self.desk.share(on),
            None => return false,
        }
        true
    }
}
