//! The status: at the foot of a room, where the line is and what it
//! holds. On one surface: at the left that the line is open and where,
//! or why it is not; at the right its limits, as the board has them.

use appkit::pui::core::{PDraw, PLight, PPaint, PPoint, PRect, PView, light};

use super::{DIM, Desk, GAP, INK, LINE, RADIUS, fit};
use crate::board::lock;

/// Where the line is.
pub struct Status {
    frame: PRect,
    desk: Desk,
}

impl Status {
    pub fn new(desk: Desk) -> Status {
        Status { frame: PRect::default(), desk }
    }

    /// The line's limits, as a person reads them.
    fn limit(&self) -> String {
        let text = &self.desk.text;
        let (caller, watch, beat) = {
            let board = lock(&self.desk.board);
            (board.limit.caller, board.limit.watch, board.limit.beat / 1000)
        };
        let said = [("status.caller", caller as u64), ("status.watch", watch as u64), ("status.beat", beat)];
        let part: Vec<String> = said.iter().map(|(name, count)| text.fill(name, "count", &count.to_string())).collect();
        part.join(&format!(" {} ", text.get("between")))
    }
}

impl PDraw for Status {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, line, radius) = (paint.size(GAP), paint.size(LINE), paint.size(RADIUS));
        let (ink, dim) = (paint.color(INK), paint.color(DIM));
        let (start, wide) = self.desk.column(self.frame.size.width);
        let rect = PRect::new(origin.x + start, origin.y + gap / 2.0, wide, (self.frame.size.height - gap).max(0.0));
        light::surface(paint, rect, radius, PLight::Soft, "");
        let text = &self.desk.text;
        let said = match self.desk.line.as_ref() {
            Ok(at) => text.fill("status.open", "at", at),
            Err(why) => text.fill("status.shut", "why", why),
        };
        let top = rect.center().y - line / 2.0;
        let inside = wide - gap * 2.0;
        let said = fit(&said, inside, paint);
        paint.text(PPoint::new(rect.origin.x + gap, top), &said, ink);
        // The limits stand at the right when there is room for both.
        let limit = self.limit();
        let room = inside - paint.measure(&said) - gap;
        if paint.measure(&limit) <= room {
            paint.text(PPoint::new(rect.origin.x + gap + inside - paint.measure(&limit), top), &limit, dim);
        }
    }
}

impl PView for Status {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }
}
