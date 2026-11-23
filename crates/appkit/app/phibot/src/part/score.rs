//! The greenhouse's score (OrnamentView): a ring of dots lit as far as
//! the plants propagated go, how many in its middle, what to do or how
//! well it went, and Replay and Exit Game.

use std::cell::RefCell;

use appkit::pui::block::PBand;
use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};

use super::{DIM, Desk, GAP, INK, LINE};

/// The theme's tokens: how many dots the ring has, how wide it is, and
/// the colours of a dot lit and not.
pub const DOT: &str = "score.dot";
pub const RING: &str = "score.ring";
pub const ON: &str = "score.on";
pub const OFF: &str = "score.off";

#[derive(Debug, Clone, Copy, PartialEq)]
enum Act {
    Replay,
    Exit,
}

pub struct Score {
    frame: PRect,
    desk: Desk,
    spot: RefCell<Vec<(PRect, Act)>>,
}

impl Score {
    pub fn new(desk: Desk) -> Score {
        Score { frame: PRect::default(), desk, spot: RefCell::default() }
    }
}

impl PDraw for Score {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, line, ink, dim) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.color(DIM));
        let (found, total) = self.desk.bot.borrow().garden().map_or((0, 0), |g| (g.found(), g.total));
        let text = &self.desk.text;
        let left = origin.x + gap;
        let mut top = origin.y + gap;
        // What to do, or how well it went.
        let done = total > 0 && found >= total;
        let (head, lead) = match done {
            true => (text.get("score.done").to_string(), text.get("score.won").replace("{n}", &found.to_string())),
            false => (text.get("score.title").to_string(), text.get("score.lead").to_string()),
        };
        paint.head(PPoint::new(left, top), &head, ink);
        top += paint.size("head.line") + gap / 4.0;
        paint.text(PPoint::new(left, top), &lead, dim);
        top += line + gap;
        // The ring: as many dots as the theme says, lit as far as the plants found go.
        let (count, ring) = (paint.size(DOT).max(1.0) as usize, paint.size(RING));
        let middle = PPoint::new(origin.x + self.frame.size.width / 2.0, top + ring / 2.0);
        let lit = (found as f32 / total.max(1) as f32 * count as f32).round() as usize;
        let dot = ring * std::f32::consts::PI / count as f32 * 0.6;
        for i in 0..count {
            let angle = i as f32 / count as f32 * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
            let at = PPoint::new(middle.x + angle.cos() * ring / 2.0, middle.y + angle.sin() * ring / 2.0);
            let colour = match i < lit {
                true => paint.color(ON),
                false => paint.color(OFF),
            };
            paint.round(PRect::new(at.x - dot / 2.0, at.y - dot / 2.0, dot, dot), dot / 2.0, colour);
        }
        let said = found.to_string();
        paint.head(PPoint::new(middle.x - paint.measure_head(&said) / 2.0, middle.y - paint.size("head.line") / 2.0), &said, ink);
        top += ring + gap;
        // Replay and Exit Game.
        let pill = [("rotate", text.get("replay"), false), ("log-out", text.get("exit"), false)];
        let rect = pill.draw(paint, PPoint::new(left, top));
        *self.spot.borrow_mut() = rect.into_iter().zip([Act::Replay, Act::Exit]).collect();
    }
}

impl PView for Score {
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
        let act = self.spot.borrow().iter().find(|(rect, _)| rect.contains(at.plus(self.frame.origin))).map(|(_, act)| *act);
        match act {
            Some(Act::Replay) => self.desk.bot.borrow_mut().replay(),
            Some(Act::Exit) => self.desk.exit(),
            None => return false,
        }
        true
    }
}
