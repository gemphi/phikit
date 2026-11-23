//! How the viewer is laid out and drawn, as the web's is
//! (`puikit/src/components/overlays/Lightbox/styles.module.scss`): one
//! row across the top, who and when at its left and the tools and the
//! way out at its right; the picture in the room the controls leave;
//! an arrow at each side on the picture's middle line; the strip at
//! the foot. Narrow, the top row is the tools' alone, who and when
//! stand over the strip, and the arrows lie over the picture's edges.

use appkit::pui::block::{PPill, PTip};
use appkit::pui::core::{PDraw, PFit, PLight, PPaint, PPoint, PRect, light};


use super::{AFTER, Act, BEFORE, CLOSE, CONTROL, FACE, LARGE, MARK, SPLIT, Spot, THUMB, TOOL, Viewer};
use crate::part::{DIM, GAP, INK, LINE, RADIUS};

impl Viewer {
    /// A control: its icon, on a ground when the pointer is over it,
    /// when it is on, and always when it stands alone (`round`). What
    /// comes back is the place to press; nothing for a control the
    /// table leaves out, which is not drawn.
    fn control(&self, paint: &mut dyn PPaint, rect: PRect, name: &str, act: Act, on: bool, round: bool) -> Option<Spot> {
        let (icon, word) = self.tool.of(name, on, &self.desk.text)?;
        // The kit's pill, in the middle of its place: on a ground of
        // its own when it is on, and always when it stands alone.
        let pill = (icon, "", on);
        let size = pill.size(paint);
        pill.draw(paint, PPoint::new(rect.center().x - size.width / 2.0, rect.center().y - size.height / 2.0), !(on || round));
        Some(Spot { rect, act, word: word.to_string() })
    }

    /// The picture, in the room it has: whole, or larger and moved.
    /// Where it lies is kept, and how far a larger one can be moved.
    fn picture(&self, paint: &mut dyn PPaint, key: &str, room: PRect) {
        let radius = paint.size(RADIUS);
        let natural = paint.natural(key).filter(|size| size.width > 0.0 && size.height > 0.0);
        match (natural, self.large.get()) {
            (Some(natural), true) => {
                let wide = room.size.width * paint.size(LARGE).max(1.0);
                let high = wide * natural.height / natural.width;
                let most = PPoint::new(wide - room.size.width, (high - room.size.height).max(0.0));
                let shift = self.shift.get();
                let shift = PPoint::new(shift.x.clamp(0.0, most.x), shift.y.clamp(0.0, most.y));
                // One lower than its room lies in its middle.
                let top = room.origin.y + ((room.size.height - high) / 2.0).max(0.0) - shift.y;
                paint.clip(room);
                paint.picture(key, PRect::new(room.origin.x - shift.x, top, wide, high), radius, PFit::Cover);
                paint.unclip();
                self.reach.set(most);
                self.shift.set(shift);
                self.shown.set(PRect::new(room.origin.x, top.max(room.origin.y), room.size.width, high.min(room.size.height)));
            }
            (natural, _) => {
                paint.picture(key, room, radius, PFit::Whole);
                // Until it is known how large it is, its room stands for it.
                self.shown.set(natural.map_or(room, |natural| PFit::Whole.place(room, natural).0));
            }
        }
    }

    /// What the control under the pointer is called, by it: under one
    /// in the upper half, over one in the lower. Where the file is was
    /// copied, the share tool says so until another picture is shown.
    fn word(&self, paint: &mut dyn PPaint, spot: &[Spot], frame: PRect) {
        let told = || spot.iter().find(|spot| spot.act == Act::Share).filter(|_| self.told.get());
        let Some(spot) = self.over.get().and_then(|act| spot.iter().find(|spot| spot.act == act)).or_else(told) else { return };
        _ = spot.word.tip(paint, spot.rect, frame);
    }
}

impl PDraw for Viewer {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let list = self.desk.now.album.borrow();
        let Some((now, shot)) = self.desk.view.get().and_then(|now| Some((now, list.get(now)?))) else { return };
        let key = &shot.key;
        // Another picture is seen whole first, and was not shared yet.
        self.live.set(true);
        if self.seen.replace(Some(now)) != Some(now) {
            self.large.set(false);
            self.told.set(false);
            self.shift.set(PPoint::default());
        }
        let (size, frame) = (self.frame.size, PRect { origin, size: self.frame.size });
        let narrow = size.width < paint.size(SPLIT);
        let gap = match narrow {
            true => paint.size(GAP) / 2.0,
            false => paint.size(GAP),
        };
        let (control, thumb, tool, face) = (paint.size(CONTROL), paint.size(THUMB), paint.size(TOOL), paint.size(FACE));
        let (line, radius, ink, dim) = (paint.size(LINE), paint.size(RADIUS), paint.color(INK), paint.color(DIM));
        let pad = (control - tool) / 2.0;
        let (left, top, right, foot) = (origin.x + gap, origin.y + gap, origin.x + size.width - gap, origin.y + size.height - gap);
        let mut spot = Vec::new();
        // The strip has the foot, where there is more than one picture;
        // narrow, who and when stand over it.
        let under = match list.len() > 1 {
            true => thumb + pad * 2.0 + gap,
            false => 0.0,
        };
        let (who, band) = match narrow {
            true => (foot - under - control, control + gap),
            false => (top, 0.0),
        };
        {
            let cell = self.desk.site.face(&shot.who);
            cell.into_iter().for_each(|cell| paint.picture(&cell.sheet, PRect::new(left, who + (control - face) / 2.0, face, face), face / 2.0, PFit::Cell { column: cell.column, index: cell.index }));
            let beside = left + cell.map_or(0.0, |_| face + gap / 2.0);
            paint.text(PPoint::new(beside, who + control / 2.0 - line), &shot.by, ink);
            paint.text(PPoint::new(beside, who + control / 2.0), &shot.when, dim);
        }
        // The picture, in the room the controls leave.
        let side = match narrow {
            true => 0.0,
            false => control + gap,
        };
        let stage = PRect::new(left + side, top + control + gap, (right - left - side * 2.0).max(0.0), (foot - under - band - top - control - gap).max(0.0));
        self.picture(paint, key, stage);
        // The way out at the top right; the tools on one surface by it.
        let close = PRect::new(right - control, top, control, control);
        spot.extend(self.control(paint, close, CLOSE, Act::Close, false, true));
        let tools: Vec<(&str, Act)> = self.tool.name().filter_map(|name| Some((name, Act::named(name).filter(|act| *act != Act::Close)?))).collect();
        let wide = tools.len() as f32 * (tool + pad / 2.0) - pad / 2.0 + pad * 2.0;
        let bar = PRect::new(close.origin.x - gap * 0.75 - wide, top, wide, control);
        light::surface(paint, bar, radius, PLight::Rest, "");
        for (index, (name, act)) in tools.iter().enumerate() {
            let on = (*act == Act::Zoom && self.large.get()) || (*act == Act::Share && self.told.get());
            spot.extend(self.control(paint, PRect::new(bar.origin.x + pad + index as f32 * (tool + pad / 2.0), top + pad, tool, tool), name, *act, on, false));
        }
        // The one before and the one after; none past either end.
        let arrow = [(now > 0, BEFORE, Act::Before, left), (now + 1 < list.len(), AFTER, Act::After, right - control)];
        for (_, name, act, at) in arrow.into_iter().filter(|(there, ..)| *there) {
            spot.extend(self.control(paint, PRect::new(at, stage.center().y - control / 2.0, control, control), name, act, false, true));
        }
        // The strip: every picture of the room, the one shown marked
        // and, where they do not all fit, kept in sight.
        let all = list.len() as f32 * (thumb + pad) + pad;
        let seen = all.min(right - left);
        let strip = PRect::new(origin.x + (size.width - seen) / 2.0, foot - thumb - pad * 2.0, seen, thumb + pad * 2.0);
        let moved = ((now as f32 + 0.5) * (thumb + pad) + pad / 2.0 - seen / 2.0).clamp(0.0, all - seen);
        if list.len() > 1 {
            light::surface(paint, strip, radius, PLight::Rest, "");
            paint.clip(strip);
            for (index, other) in list.iter().enumerate() {
                let rect = PRect::new(strip.origin.x + pad + index as f32 * (thumb + pad) - moved, strip.origin.y + pad, thumb, thumb);
                if index == now {
                    paint.round(rect.inset(-paint.size(MARK)), radius, ink);
                }
                paint.picture(&other.key, rect, radius, PFit::Cover);
                if strip.contains(rect.center()) {
                    spot.push(Spot { rect, act: Act::Show(index), word: format!("{}, {}", other.by, other.when) });
                }
            }
            paint.unclip();
        }
        self.word(paint, &spot, frame);
        *self.spot.borrow_mut() = spot;
        self.origin.set(origin);
    }
}
