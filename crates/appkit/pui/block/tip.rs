//! A tip: what something is called, said by it while the pointer is
//! over it. It floats: it stands on the window's own ground (glass
//! where the theme has glass), so what is behind it does not show
//! through its words.

use super::super::core::light::pane;
use super::super::core::{PPaint, PPoint, PRect};
use super::{GAP, INK, LINE, RADIUS};

/// Something that can be said by a place. Any text is one.
pub trait PTip {
    /// What is said.
    fn word(&self) -> &str;

    /// Say it by a place: under one in the upper half of `frame`, over
    /// one in the lower, and never outside the frame. Where it stands;
    /// nothing is drawn for no word.
    fn tip(&self, paint: &mut dyn PPaint, at: PRect, frame: PRect) -> Option<PRect> {
        let (gap, line, ink, radius) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.size(RADIUS));
        let word = Some(self.word()).filter(|word| !word.is_empty())?;
        let (wide, high) = (paint.measure(word) + gap, line + gap / 2.0);
        let most = (frame.origin.x + frame.size.width - gap / 2.0 - wide).max(frame.origin.x);
        let left = (at.center().x - wide / 2.0).clamp((frame.origin.x + gap / 2.0).min(most), most);
        let top = match at.center().y < frame.center().y {
            true => at.origin.y + at.size.height + gap / 4.0,
            false => at.origin.y - gap / 4.0 - high,
        };
        let rect = PRect::new(left, top, wide, high);
        pane(paint, rect, radius, "");
        paint.text(PPoint::new(left + gap / 2.0, top + gap / 4.0), word, ink);
        Some(rect)
    }
}

impl PTip for str {
    fn word(&self) -> &str {
        self
    }
}
