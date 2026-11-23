//! The talk: the open chat, each turn in a band of the label's light,
//! the person's on the right of the body's column, lit from its right,
//! and Phigen's on its left, lit from its left. The newest turn is at the bottom. A
//! chat longer than the talk is scrolled back with the wheel, and comes
//! back to its newest turn when something is said or another chat opens.
//! Once a frame it takes in what has arrived of an answer.

use std::cell::Cell;

use appkit::pui::core::{PDraw, PEvent, PLight, PPaint, PPoint, PRect, PSize, PView, light};
use appkit::pui::layout::{PScroll, wrap};

use super::{Desk, GAP, INK, LINE};
use crate::chat::Who;

/// The theme's tokens for a turn: the room around its text, how round its
/// band is, and how much of the width it may take.
pub const INSET: &str = "turn.inset";
pub const RADIUS: &str = "turn.radius";
pub const SHARE: &str = "turn.share";

/// The open chat, turn by turn.
pub struct Talk {
    frame: PRect,
    desk: Desk,
    /// How far back from the newest turn it is scrolled.
    scroll: Cell<PScroll>,
    /// Which chat was drawn last, and how many turns it had.
    seen: Cell<(usize, usize)>,
}

impl Talk {
    pub fn new(desk: Desk) -> Talk {
        Talk { frame: PRect::default(), desk, scroll: Cell::default(), seen: Cell::new((0, 0)) }
    }
}

impl PDraw for Talk {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, inset, line, radius) = (paint.size(GAP), paint.size(INSET), paint.size(LINE), paint.size(RADIUS));
        let ((start, wide), high) = (self.desk.column(self.frame.size.width), self.frame.size.height);
        let limit = wide * paint.size(SHARE) - inset * 2.0;
        let history = self.desk.history.borrow();
        let turn = &history.current().turn;
        // Every turn's lines and the size of its bubble.
        let set = |text: &str| {
            let row = wrap(text, limit, &|text| paint.measure(text));
            let widest = row.iter().map(|text| paint.measure(text)).fold(0.0, f32::max);
            let size = PSize { width: widest + inset * 2.0, height: row.len() as f32 * line + inset * 2.0 };
            (row, size)
        };
        // An answer that has not begun to arrive shows that it is awaited.
        let said = |text: &'_ String| match text.is_empty() && history.waiting() {
            true => self.desk.text.get("reply.wait").to_string(),
            false => text.clone(),
        };
        let laid: Vec<(Vec<String>, PSize)> = turn.iter().map(|turn| set(&said(&turn.text))).collect();
        let tall = laid.iter().map(|(_, size)| size.height + gap).sum::<f32>() + gap;
        // A new turn, or another chat, is shown from its newest turn.
        let now = (history.index(), turn.len());
        let mut scroll = self.scroll.get();
        if self.seen.replace(now) != now {
            scroll.home();
        }
        scroll.fit(tall, high);
        self.scroll.set(scroll);
        let mut bottom = origin.y + high - gap + scroll.at();
        for (turn, (row, size)) in turn.iter().zip(&laid).rev() {
            let top = bottom - size.height;
            bottom = top - gap;
            // A turn wholly above or below the talk is not drawn.
            if top >= origin.y + high || top + size.height <= origin.y {
                continue;
            }
            // What sits at the right is lit the other way.
            let (left, end) = match turn.who {
                Who::Person => (origin.x + start + wide - size.width, true),
                Who::Phigen => (origin.x + start, false),
            };
            let ink = paint.color(INK);
            light::label(paint, PRect { origin: PPoint::new(left, top), size: *size }, radius, PLight::Strong, end, "");
            for (index, text) in row.iter().enumerate() {
                paint.text(PPoint::new(left + inset, top + inset + index as f32 * line), text, ink);
            }
        }
    }
}

impl PView for Talk {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    fn tick(&mut self) -> bool {
        self.desk.history.borrow_mut().poll()
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        let PEvent::Scroll(_, by) = *event else { return false };
        // The wheel turned away from the hand goes back through the chat.
        let mut scroll = self.scroll.get();
        let any = scroll.turn(by.y);
        self.scroll.set(scroll);
        any
    }
}
