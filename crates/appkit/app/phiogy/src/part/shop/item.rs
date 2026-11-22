//! One thing's own page: the way back to its shop and to the page of
//! the menu it is on; its picture, with its other views under it, a
//! press on one showing it; what it is called, its price, what kind of
//! thing it is and how it stands; what is said of it; its facts; and
//! more like it, from the same page of the menu.

use appkit::pui::core::{PFit, PLight, PPaint, PPoint, PRect, light};
use appkit::pui::layout::wrap;

use super::{Act, Shopping};
use crate::part::{DIM, GAP, INK, LINE, RADIUS, fit};
use crate::site::Shop;

/// The theme's tokens: the widest a thing's picture is, a picture of
/// its views' row, how many more like it are shown, and the width
/// under which its words stand under its picture.
pub const PICTURE: &str = "shop.picture";
pub const THUMB: &str = "view.thumb";
pub const MARK: &str = "view.mark";
pub const MORE: &str = "shop.more";
pub const SPLIT: &str = "room.split";

impl Shopping {
    /// The page of the thing at a place of a shop's list, from `at`
    /// down; how far it goes.
    pub(super) fn thing(&self, paint: &mut dyn PPaint, at: PRect, shop: &Shop, place: usize, sight: PRect, spot: &mut Vec<(PRect, Act)>) -> f32 {
        let (gap, line, ink, dim, radius) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.color(DIM), paint.size(RADIUS));
        let Some(one) = shop.product.get(place) else { return at.origin.y };
        let (left, wide, text) = (at.origin.x, at.size.width, &self.desk.text);
        let on = shop.shelf.iter().find(|shelf| shelf.at.contains(&place));
        // The way back: the shop, then the page of its menu it is on.
        let mut across = left;
        for (label, page) in [Some((shop.label.as_str(), shop.id.clone())), on.map(|shelf| (shelf.label.as_str(), shelf.id.clone()))].into_iter().flatten() {
            let rect = PRect::new(across, at.origin.y, paint.measure(label) + gap, line + gap / 2.0);
            light::label(paint, rect, radius, PLight::Rest, false, "");
            paint.text(PPoint::new(across + gap / 2.0, at.origin.y + gap / 4.0), label, ink);
            spot.push((rect, Act::Go(page)));
            across += rect.size.width + gap / 4.0;
        }
        let mut top = at.origin.y + line + gap * 1.5;
        // Its picture: the view chosen, or its own. Wide, its words
        // stand beside the picture; narrow, under it.
        let beside = wide >= paint.size(SPLIT);
        let side = match beside {
            true => paint.size(PICTURE).min(wide / 2.0),
            false => paint.size(PICTURE).min(wide),
        };
        let frame = PRect::new(left, top, side, side);
        light::surface(paint, frame, radius, PLight::Soft, "");
        match one.view.get(self.view.get()) {
            Some((_, image)) => paint.picture(image, frame.inset(gap / 2.0), radius, PFit::Whole),
            None => self.portrait(paint, frame.inset(gap / 2.0), one, &shop.icon),
        }
        let thumb = paint.size(THUMB);
        for (index, (_, image)) in one.view.iter().enumerate() {
            let rect = PRect::new(left + index as f32 * (thumb + gap / 4.0), top + side + gap / 2.0, thumb, thumb);
            if index == self.view.get() {
                let mark = paint.size(MARK);
                paint.round(rect.inset(-mark), radius, ink);
            }
            paint.picture(image, rect, radius, PFit::Cover);
            spot.push((rect, Act::View(index)));
        }
        let under = top + side + gap / 2.0 + match one.view.is_empty() {
            true => 0.0,
            false => thumb + gap / 2.0,
        };
        // Its words.
        let (words, room) = match beside {
            true => (PPoint::new(left + side + gap, top), wide - side - gap),
            false => (PPoint::new(left, under), wide),
        };
        let mut down = words.y;
        paint.head(PPoint::new(words.x, down), &fit(&one.title, room, paint), ink);
        down += paint.size("head.line") + gap / 4.0;
        paint.text(PPoint::new(words.x, down), &one.sub, ink);
        let kind = [one.kind.as_str(), one.meta.as_str(), one.tag.as_str()].into_iter().filter(|said| !said.is_empty()).collect::<Vec<_>>().join(&format!(" {} ", text.get("between")));
        paint.text(PPoint::new(words.x, down + line), &fit(&kind, room, paint), dim);
        down += line * 2.0 + gap / 2.0;
        down = self.stand(paint, PPoint::new(words.x, down), &shop.id, one, spot).origin.y + line + gap;
        for said in wrap(&one.body, room, &|text| paint.measure(text)).into_iter().filter(|said| !said.is_empty()) {
            paint.text(PPoint::new(words.x, down), &said, ink);
            down += line;
        }
        down += gap / 2.0;
        for (name, value) in &one.field {
            paint.text(PPoint::new(words.x, down), name, dim);
            let after = words.x + (room / 3.0).max(paint.measure(name) + gap / 2.0);
            paint.text(PPoint::new(after, down), &fit(value, (words.x + room - after).max(0.0), paint), ink);
            down += line;
        }
        top = down.max(under) + gap;
        // More like it: the others of its page of the menu.
        let near: Vec<usize> = on.map(|shelf| shelf.at.clone()).unwrap_or_default().into_iter().filter(|other| *other != place).take(paint.size(MORE) as usize).collect();
        if !near.is_empty() {
            paint.text(PPoint::new(left, top), text.get("shop.more"), dim);
            top = self.grid(paint, PRect::new(left, top + line + gap / 2.0, wide, 0.0), shop, &near, sight, spot);
        }
        top
    }
}
