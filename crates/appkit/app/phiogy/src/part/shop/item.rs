//! One thing's own page, as the web's stands
//! (`puikit/app/phi-kit/src/product`): the way back to its shop and to
//! the page of the menu it is on; then two equal columns. In one its
//! picture, with its views under it two across, a press on one showing
//! it. In the other what kind of thing it is, what it is called, its
//! price, how it stands and whether the person likes it, what is said
//! of it, and its facts, a row each. Under them the same thing in its
//! other colours, every one, and more like it from the same page of
//! the menu. Whatever is under the pointer is lit.

use appkit::pui::core::{PFit, PLight, PPaint, PPoint, PRect, light};
use appkit::pui::layout::wrap;
use appkit::service::PTable;

use super::{Act, Shopping};
use crate::data;
use crate::part::title::{RULE, WIDTH};
use crate::part::{DIM, GAP, INK, LINE, RADIUS, fit};
use crate::site::{Product, Shop};

/// The theme's tokens: the widest a thing's picture is where its words
/// stand under it, how many more like it are shown, and the width
/// under which its words stand under its picture.
pub const PICTURE: &str = "shop.picture";
pub const MORE: &str = "shop.more";
/// How many of a thing's other colours stand in a row.
pub const KIN: &str = "shop.kin";
pub const SPLIT: &str = "room.split";

/// What a thing is without its colour, and its colour: its name with
/// the value of the fact the site calls the colour taken out of it.
/// Nothing for a thing with no such fact.
fn place_of(one: &Product, colour: &str) -> Option<(String, String)> {
    let hue = one.field.iter().find(|(name, _)| name == colour).map(|(_, value)| value.clone())?;
    let rest = one.title.replace(&hue, "");
    Some((rest.split_whitespace().collect::<Vec<_>>().join(" "), hue)).filter(|(rest, _)| !rest.is_empty() && *rest != one.title)
}

impl Shopping {
    /// The other colours of a thing: the places of every other thing of
    /// the shop that is the same without its colour, each with its
    /// colour, in the shop's own order.
    fn kin(&self, shop: &Shop, own: Option<(String, String)>) -> Vec<(usize, String)> {
        let colour = PTable::read(data::SITE).get("shop.colour").to_string();
        let Some((family, hue)) = own else { return Vec::new() };
        let other = shop.product.iter().enumerate().filter_map(|(index, other)| Some((index, place_of(other, &colour)?)));
        other.filter(|(_, (rest, other))| *rest == family && *other != hue).map(|(index, (_, hue))| (index, hue)).collect()
    }

    /// The page of the thing at a place of a shop's list, from `at`
    /// down; how far it goes.
    pub(super) fn thing(&self, paint: &mut dyn PPaint, at: PRect, shop: &Shop, place: usize, sight: PRect, spot: &mut Vec<(PRect, Act)>) -> f32 {
        let (gap, line, ink, dim, radius) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.color(DIM), paint.size(RADIUS));
        let Some(one) = shop.product.get(place) else { return at.origin.y };
        let colour = PTable::read(data::SITE).get("shop.colour").to_string();
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
        // Wide, the page is two equal columns, as the web's is: the
        // picture and its views, and beside them the words. Narrow,
        // the words stand under the picture.
        let beside = wide >= paint.size(SPLIT);
        let side = match beside {
            true => (wide - gap) / 2.0,
            false => paint.size(PICTURE).min(wide),
        };
        let frame = PRect::new(left, top, side, side);
        light::surface(paint, frame, radius, PLight::Soft, "");
        match one.view.get(self.view.get()) {
            Some((_, image)) => paint.picture(image, frame.inset(gap / 2.0), radius, PFit::Whole),
            None => self.portrait(paint, frame.inset(gap / 2.0), one, &shop.icon),
        }
        // Its views, two across, each on a ground with what it is
        // called; the one shown is lit.
        let cell = (side - gap / 2.0) / 2.0;
        for (index, (label, image)) in one.view.iter().enumerate() {
            let rect = PRect::new(left + (index % 2) as f32 * (cell + gap / 2.0), top + side + gap / 2.0 + (index / 2) as f32 * (cell + gap / 2.0), cell, cell);
            let lit = match (index == self.view.get(), self.over(rect)) {
                (true, _) => PLight::Strong,
                (false, true) => PLight::Rest,
                (false, false) => PLight::Soft,
            };
            light::surface(paint, rect, radius, lit, "");
            paint.picture(image, rect.inset(gap / 2.0), radius, PFit::Whole);
            paint.text(PPoint::new(rect.origin.x + gap / 2.0, rect.origin.y + gap / 4.0), &fit(label, cell - gap, paint), dim);
            spot.push((rect, Act::View(index)));
        }
        let under = top + side + gap / 2.0 + one.view.len().div_ceil(2) as f32 * (cell + gap / 2.0);
        // Its words.
        let (words, room) = match beside {
            true => (PPoint::new(left + side + gap, top), wide - side - gap),
            false => (PPoint::new(left, under), wide),
        };
        let mut down = words.y;
        paint.text(PPoint::new(words.x, down), &fit(&one.kind, room, paint), dim);
        down += line;
        paint.head(PPoint::new(words.x, down), &fit(&one.title, room, paint), ink);
        down += paint.size("head.line") + gap / 4.0;
        paint.text(PPoint::new(words.x, down), &one.sub, ink);
        let kind = [one.meta.as_str(), one.tag.as_str()].into_iter().filter(|said| !said.is_empty()).collect::<Vec<_>>().join(&format!(" {} ", text.get("between")));
        paint.text(PPoint::new(words.x + paint.measure(&one.sub) + gap, down), &fit(&kind, room - paint.measure(&one.sub) - gap, paint), dim);
        down += line + gap / 2.0;
        // How it stands, and whether the person likes it.
        let stand = self.stand(paint, PPoint::new(words.x, down), &shop.id, one, spot);
        let own = format!("{}/{}", shop.id, one.id);
        let liked = self.desk.liked(&own);
        let said = match liked {
            true => text.get("shop.liked"),
            false => text.get("shop.like"),
        };
        let like = PRect::new(stand.origin.x + stand.size.width + gap / 4.0, stand.origin.y, paint.measure(said) + gap, stand.size.height);
        let lit = match (liked, self.over(like)) {
            (true, _) => PLight::Strong,
            (false, true) => PLight::Rest,
            (false, false) => PLight::Soft,
        };
        light::label(paint, like, radius, lit, false, "");
        paint.text(PPoint::new(like.origin.x + gap / 2.0, like.origin.y + gap / 4.0), said, ink);
        spot.push((like, Act::Like(own)));
        down = stand.origin.y + line + gap;
        for said in wrap(&one.body, room, &|text| paint.measure(text)).into_iter().filter(|said| !said.is_empty()) {
            paint.text(PPoint::new(words.x, down), &said, ink);
            down += line;
        }
        down += gap / 2.0;
        // Its facts, a row each: what it is at the left, its value at
        // the right, a line under it.
        let (rule, ruled) = (paint.size(WIDTH), paint.color(RULE));
        for (name, value) in &one.field {
            paint.text(PPoint::new(words.x, down + gap / 4.0), name, dim);
            let value = fit(value, (room - paint.measure(name) - gap).max(0.0), paint);
            paint.text(PPoint::new(words.x + room - paint.measure(&value), down + gap / 4.0), &value, ink);
            down += line + gap / 2.0;
            paint.fill(PRect::new(words.x, down - rule, room, rule), ruled);
        }
        top = down.max(under) + gap;
        // The same thing in its other colours: every one of them, in
        // rows, a press on one going to it.
        let kin = self.kin(shop, place_of(one, &colour));
        if !kin.is_empty() {
            paint.text(PPoint::new(left, top), text.get("shop.colours"), dim);
            top += line + gap / 2.0;
            let across = paint.size(KIN).max(1.0);
            let cell = (wide - (across - 1.0) * gap / 2.0) / across;
            for (index, (other, hue)) in kin.iter().enumerate() {
                let rect = PRect::new(left + (index % across as usize) as f32 * (cell + gap / 2.0), top + (index / across as usize) as f32 * (cell + line + gap), cell, cell + line + gap / 2.0);
                let lit = match self.over(rect) {
                    true => PLight::Rest,
                    false => PLight::Soft,
                };
                light::surface(paint, rect, radius, lit, "");
                self.portrait(paint, PRect::new(rect.origin.x + gap / 4.0, rect.origin.y + gap / 4.0, cell - gap / 2.0, cell - gap / 2.0), &shop.product[*other], &shop.icon);
                let hue = fit(hue, cell - gap / 2.0, paint);
                paint.text(PPoint::new(rect.center().x - paint.measure(&hue) / 2.0, rect.origin.y + cell), &hue, ink);
                spot.push((rect, Act::Thing(format!("{}/{}", shop.id, shop.product[*other].id))));
            }
            top += kin.len().div_ceil(across as usize) as f32 * (cell + line + gap) + gap / 2.0;
        }
        // More like it: the others of its page of the menu.
        let near: Vec<usize> = on.map(|shelf| shelf.at.clone()).unwrap_or_default().into_iter().filter(|other| *other != place).take(paint.size(MORE) as usize).collect();
        if !near.is_empty() {
            paint.text(PPoint::new(left, top), text.get("shop.more"), dim);
            top = self.grid(paint, PRect::new(left, top + line + gap / 2.0, wide, 0.0), shop, &near, sight, spot);
        }
        top
    }
}
