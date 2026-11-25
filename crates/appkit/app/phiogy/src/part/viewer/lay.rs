//! How the viewer is laid out and drawn, as Google Photos' and the web's
//! are (`puikit/src/components/overlays/Lightbox`): the picture's own
//! haze behind everything, under the window's ground; one row across the
//! top, the way back and who and when at its left, the tools and Save at
//! its right; the picture in the room the rest leaves, its marks on it;
//! an arrow at each side of the window on the picture's middle line; at
//! the foot the inks and the pens, and the strip (`foot.rs`). Narrow, who
//! and when stand over the foot, and the arrows lie over the picture's
//! edges.

use appkit::pui::block::{PPill, PTip};
use appkit::pui::core::{PDraw, PFit, PLight, PPaint, PPoint, PRect, haze, light};

use super::{AFTER, Act, BEFORE, CLOSE, CONTROL, FACE, KEEP, LARGE, PEN, SPLIT, Spot, TOOL, VEIL, Viewer};
use crate::part::{DIM, GAP, INK, LINE, RADIUS};

/// The theme's token for the window's own ground, and whether the
/// picture's haze lies under it.
const GROUND: &str = "window.fill";
const HAZE: &str = "view.haze";

impl Viewer {
    /// A control: its icon, on a ground when the pointer is over it,
    /// when it is on, and always when it stands alone (`round`). What
    /// comes back is the place to press; nothing for a control the
    /// table leaves out, which is not drawn.
    pub(super) fn control(&self, paint: &mut dyn PPaint, rect: PRect, name: &str, act: Act, on: bool, round: bool) -> Option<Spot> {
        let (icon, word) = self.tool.of(name, on, &self.desk.text)?;
        let pill = (icon, "", on);
        let size = pill.size(paint);
        pill.draw(paint, PPoint::new(rect.center().x - size.width / 2.0, rect.center().y - size.height / 2.0), !(on || round));
        Some(Spot { rect, act, word: word.to_string() })
    }

    /// The viewer's ground: the look's own pane, as every page stands on
    /// (the user, 25 November 2026: not the picture's colours). Where the
    /// theme asks for it (`view.haze` more than nothing), the picture's
    /// haze lies under it, asked for once and shown when it is made, and
    /// the window's ground over that.
    fn haze(&self, paint: &mut dyn PPaint, key: &str, frame: PRect) {
        if paint.theme().size(HAZE).unwrap_or_default() <= 0.0 {
            light::pane(paint, frame, 0.0, GROUND);
            return;
        }
        let name = format!("haze/{key}");
        if self.hazed.borrow_mut().insert(key.to_string()) {
            let film = self.desk.film();
            self.path().into_iter().for_each(|path| haze::show(&film, &name, path));
        }
        if paint.natural(&name).is_some() {
            paint.picture(&name, frame, 0.0, PFit::Cover);
        }
        let mut veil = paint.color(GROUND);
        veil.a = paint.size(VEIL).clamp(0.0, 1.0);
        paint.fill(frame, veil);
    }

    /// The picture, in the room it has: whole, or larger and moved.
    /// Where it lies is kept, where the whole of it is, and how far a
    /// larger one can be moved.
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
                let whole = PRect::new(room.origin.x - shift.x, top, wide, high);
                paint.clip(room);
                paint.picture(key, whole, radius, PFit::Cover);
                let pen = paint.size(PEN);
                self.marks().draw(paint, whole, pen);
                paint.unclip();
                self.reach.set(most);
                self.shift.set(shift);
                self.whole.set(whole);
                self.shown.set(PRect::new(room.origin.x, top.max(room.origin.y), room.size.width, high.min(room.size.height)));
            }
            (natural, _) => {
                paint.picture(key, room, radius, PFit::Whole);
                // Until it is known how large it is, its room stands for it.
                let shown = natural.map_or(room, |natural| PFit::Whole.place(room, natural).0);
                let pen = paint.size(PEN);
                self.marks().draw(paint, shown, pen);
                self.shown.set(shown);
                self.whole.set(shown);
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

    /// The row at the top's right end: the tools on one surface, from
    /// the left in the table's order, then Save, lit while there are
    /// marks not kept, its words beside its icon.
    fn tools(&self, paint: &mut dyn PPaint, right: f32, top: f32) -> Vec<Spot> {
        let (control, tool, gap, radius) = (paint.size(CONTROL), paint.size(TOOL), paint.size(GAP), paint.size(RADIUS));
        let pad = (control - tool) / 2.0;
        let mut spot = Vec::new();
        let kept = self.marks().kept;
        let (icon, word) = self.tool.of(KEEP, kept, &self.desk.text).unwrap_or_default();
        let pill = (icon, word, !kept);
        let size = pill.size(paint);
        let at = PPoint::new(right - size.width, top + (control - size.height) / 2.0);
        let rect = pill.draw(paint, at, kept);
        spot.push(Spot { rect, act: Act::Keep, word: word.to_string() });
        let tools: Vec<(&str, Act)> = self.tool.name().filter_map(|name| Some((name, Act::named(name)?))).filter(|(_, act)| matches!(act, Act::Zoom | Act::Share | Act::Save | Act::Open | Act::Undo | Act::Redo)).collect();
        let wide = tools.len() as f32 * (tool + pad / 2.0) - pad / 2.0 + pad * 2.0;
        let bar = PRect::new(at.x - gap * 0.75 - wide, top, wide, control);
        light::surface(paint, bar, radius, PLight::Rest, "");
        for (index, (name, act)) in tools.iter().enumerate() {
            let on = match act {
                Act::Zoom => self.large.get(),
                Act::Share => self.told.get(),
                _ => false,
            };
            spot.extend(self.control(paint, PRect::new(bar.origin.x + pad + index as f32 * (tool + pad / 2.0), top + pad, tool, tool), name, *act, on, false));
        }
        spot
    }
}

impl PDraw for Viewer {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let list = self.desk.now.album.borrow().clone();
        let Some((now, shot)) = self.desk.view.get().and_then(|now| Some((now, list.get(now)?.clone()))) else { return };
        let key = &shot.key;
        // Another picture is seen whole first, and was not shared yet.
        self.live.set(true);
        if self.seen.replace(Some(now)) != Some(now) {
            self.large.set(false);
            self.told.set(false);
            self.shift.set(PPoint::default());
        }
        let (size, frame) = (self.frame.size, PRect { origin, size: self.frame.size });
        self.haze(paint, key, frame);
        let narrow = size.width < paint.size(SPLIT);
        let gap = match narrow {
            true => paint.size(GAP) / 2.0,
            false => paint.size(GAP),
        };
        let (control, face, line, ink, dim) = (paint.size(CONTROL), paint.size(FACE), paint.size(LINE), paint.color(INK), paint.color(DIM));
        let (left, top, right, foot) = (origin.x + gap, origin.y + gap, origin.x + size.width - gap, origin.y + size.height - gap);
        let mut spot = Vec::new();
        // The way back, at the top left.
        spot.extend(self.control(paint, PRect::new(left, top, control, control), CLOSE, Act::Close, false, true));
        // The foot: the inks, the pens and the strip; narrow, who and
        // when stand over it.
        let under = self.foot_high(paint, list.len());
        let (who, band) = match narrow {
            true => (foot - under - control, control + gap),
            false => (top, 0.0),
        };
        {
            let beside = match narrow {
                true => left,
                false => left + control + gap / 2.0,
            };
            let cell = self.desk.site.face(&shot.who);
            cell.into_iter().for_each(|cell| paint.picture(&cell.sheet, PRect::new(beside, who + (control - face) / 2.0, face, face), face / 2.0, PFit::Cell { column: cell.column, index: cell.index }));
            let beside = beside + cell.map_or(0.0, |_| face + gap / 2.0);
            paint.text(PPoint::new(beside, who + control / 2.0 - line), &shot.by, ink);
            paint.text(PPoint::new(beside, who + control / 2.0), &shot.when, dim);
        }
        // The picture, in the room the rest leaves, its marks on it.
        let side = match narrow {
            true => 0.0,
            false => control + gap,
        };
        let stage = PRect::new(left + side, top + control + gap, (right - left - side * 2.0).max(0.0), (foot - under - band - top - control - gap * 2.0).max(0.0));
        self.picture(paint, key, stage);
        spot.extend(self.tools(paint, right, top));
        // The one before and the one after; none past either end.
        let arrow = [(now > 0, BEFORE, Act::Before, left), (now + 1 < list.len(), AFTER, Act::After, right - control)];
        for (_, name, act, at) in arrow.into_iter().filter(|(there, ..)| *there) {
            spot.extend(self.control(paint, PRect::new(at, stage.center().y - control / 2.0, control, control), name, act, false, false));
        }
        spot.extend(self.foot(paint, &list, now, PRect::new(left, foot - under, right - left, under)));
        self.word(paint, &spot, frame);
        *self.spot.borrow_mut() = spot;
        self.origin.set(origin);
    }
}
