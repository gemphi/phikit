//! The pointer over the shops' pages: the card or the pill it is over
//! is lit, and wherever a press does something the pointer is a hand.

use appkit::pui::core::{PPoint, PRect};

use super::Shopping;

impl Shopping {
    /// Whether the pointer is over a place of the window.
    pub(super) fn over(&self, rect: PRect) -> bool {
        self.point.get().is_some_and(|at| rect.contains(at))
    }

    /// The pointer moved to a place of this part. `true` when what it
    /// is over is another thing than before.
    pub(super) fn moved(&self, at: PPoint) -> bool {
        let place = at.plus(self.origin.get());
        let found = |point: Option<PPoint>| point.and_then(|point| self.spot.borrow().iter().rposition(|(rect, _)| rect.contains(point)));
        let turned = found(self.point.get()) != found(Some(place));
        self.point.set(Some(place));
        turned
    }

    /// Where a press does something, as it was drawn last: what of it
    /// is inside the part.
    pub(super) fn hands(&self) -> Vec<PRect> {
        let frame = PRect { origin: self.origin.get(), size: self.frame.size };
        self.spot.borrow().iter().map(|(rect, _)| *rect).filter(|rect| frame.contains(rect.center())).collect()
    }
}
