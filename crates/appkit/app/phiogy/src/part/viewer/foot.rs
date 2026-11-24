//! The viewer's foot, as Google Photos has it under a picture: the inks
//! a mark can be in, a dot each, the one in hand ringed; under them the
//! pens, Sketch and Text, each round with its words under it, the one in
//! hand on; and at the bottom the strip of every picture it was opened
//! among, the one shown marked and kept in sight.

use appkit::pui::core::{PFit, PPaint, PPoint, PRect};

use super::mark::inks;
use crate::part::Shot;
use super::{Act, CONTROL, DOT, MARK, Pen, SKETCH, Spot, TEXT, THUMB, TOOL, Viewer};
use crate::part::{GAP, INK, LINE, RADIUS};

impl Viewer {
    /// How high the foot is, for so many pictures: the inks, the pens,
    /// and where there is more than one picture, the strip.
    pub(super) fn foot_high(&self, paint: &dyn PPaint, count: usize) -> f32 {
        let (gap, control, line, dot, thumb, tool) = (paint.size(GAP), paint.size(CONTROL), paint.size(LINE), paint.size(DOT), paint.size(THUMB), paint.size(TOOL));
        let pad = (control - tool) / 2.0;
        let strip = match count > 1 {
            true => thumb + pad * 2.0 + gap / 2.0,
            false => 0.0,
        };
        dot + gap / 2.0 + control + line + gap / 2.0 + strip
    }

    /// Draw the foot in its place: where a press on each part does what.
    pub(super) fn foot(&self, paint: &mut dyn PPaint, list: &[Shot], now: usize, room: PRect) -> Vec<Spot> {
        let (gap, control, line, dot, ink, mark) = (paint.size(GAP), paint.size(CONTROL), paint.size(LINE), paint.size(DOT), paint.color(INK), paint.size(MARK));
        let middle = room.center().x;
        let mut spot = Vec::new();
        // The inks, a dot each, the one in hand ringed.
        let colour = inks();
        let across = colour.len() as f32 * (dot + gap / 2.0) - gap / 2.0;
        for (index, one) in colour.iter().enumerate() {
            let rect = PRect::new(middle - across / 2.0 + index as f32 * (dot + gap / 2.0), room.origin.y, dot, dot);
            if index == self.ink.get() {
                paint.round(rect.inset(-mark * 1.5), dot, ink);
            }
            paint.round(rect, dot / 2.0, *one);
            spot.push(Spot { rect, act: Act::Ink(index), word: String::new() });
        }
        // The pens, each its words under it.
        let top = room.origin.y + dot + gap / 2.0;
        let pen = [(SKETCH, Pen::Sketch), (TEXT, Pen::Text)];
        let across = pen.len() as f32 * (control + gap) - gap;
        for (index, (name, pen)) in pen.into_iter().enumerate() {
            let rect = PRect::new(middle - across / 2.0 + index as f32 * (control + gap), top, control, control);
            let on = self.pen.get() == Some(pen);
            let Some(one) = self.control(paint, rect, name, Act::Pen(pen), on, true) else { continue };
            let wide = paint.measure(&one.word);
            paint.text(PPoint::new(rect.center().x - wide / 2.0, rect.origin.y + control), &one.word, ink);
            spot.push(Spot { rect: PRect::new(rect.origin.x, rect.origin.y, control, control + line), ..one });
        }
        spot.extend(self.strip(paint, list, now, room));
        spot
    }

    /// The strip: every picture, the one shown marked and, where they do
    /// not all fit, kept in sight.
    fn strip(&self, paint: &mut dyn PPaint, list: &[Shot], now: usize, room: PRect) -> Vec<Spot> {
        if list.len() < 2 {
            return Vec::new();
        }
        let (control, thumb, tool, radius, ink, mark) = (paint.size(CONTROL), paint.size(THUMB), paint.size(TOOL), paint.size(RADIUS), paint.color(INK), paint.size(MARK));
        let pad = (control - tool) / 2.0;
        let all = list.len() as f32 * (thumb + pad) + pad;
        let seen = all.min(room.size.width);
        let strip = PRect::new(room.center().x - seen / 2.0, room.origin.y + room.size.height - thumb - pad * 2.0, seen, thumb + pad * 2.0);
        let moved = ((now as f32 + 0.5) * (thumb + pad) + pad / 2.0 - seen / 2.0).clamp(0.0, all - seen);
        let mut spot = Vec::new();
        paint.clip(strip);
        for (index, other) in list.iter().enumerate() {
            let rect = PRect::new(strip.origin.x + pad + index as f32 * (thumb + pad) - moved, strip.origin.y + pad, thumb, thumb);
            if index == now {
                paint.round(rect.inset(-mark), radius, ink);
            }
            paint.picture(&other.key, rect, radius, PFit::Cover);
            if strip.contains(rect.center()) {
                spot.push(Spot { rect, act: Act::Show(index), word: format!("{}, {}", other.by, other.when) });
            }
        }
        paint.unclip();
        spot
    }
}
