//! One thing's own page, as Phiegg's item page stands
//! (`puikit/app/phiegg/src/components/Phiegg/ItemSheet.tsx`): the way
//! back to its shop and to the page of the menu it is on, as tabs on a
//! ground; then, wide, three columns. At the left a narrow bar of its
//! variants, the thing in each of its colours (itself first), each a
//! picture with the colour's name, a press on one going to it. Beside it
//! the picture, standing at the top, with its views as tabs on a ground
//! under it. At the right every section, each on a ground: what kind of
//! thing it is, what it is called, its price, how it stands and whether
//! the person likes it, what is said of it; its facts, a row each; and
//! more like it from the same page of the menu. Narrow, they stand one
//! under another. Whatever is under the pointer is lit.

use appkit::pui::block::{PBand, PFacts, PPill};
use appkit::pui::core::{PFit, PLight, PPaint, PPoint, PRect, light};
use appkit::pui::layout::{PDry, wrap};
use appkit::service::PTable;

use super::{Act, Shopping};
use crate::data;
use crate::part::{DIM, GAP, INK, LINE, RADIUS, fit};
use crate::site::{Product, Shop};

/// The theme's tokens: the widest a thing's picture is where its words
/// stand under it, how many more like it are shown, the width under
/// which its sections stand under its picture, and how wide the bar of
/// its variants is.
pub const PICTURE: &str = "shop.picture";
pub const MORE: &str = "shop.more";
pub const SPLIT: &str = "room.split";
pub const VARIANT: &str = "shop.variant";

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
        let (gap, radius) = (paint.size(GAP), paint.size(RADIUS));
        let Some(one) = shop.product.get(place) else { return at.origin.y };
        let colour = PTable::read(data::SITE).get("shop.colour").to_string();
        let (left, wide) = (at.origin.x, at.size.width);
        let on = shop.shelf.iter().find(|shelf| shelf.at.contains(&place));
        // The way back, as tabs on a ground: the shop, then its page.
        let crumb: Vec<(&str, String)> = [Some((shop.label.as_str(), shop.id.clone())), on.map(|shelf| (shelf.label.as_str(), shelf.id.clone()))].into_iter().flatten().collect();
        let pill: Vec<(&str, &str, bool)> = crumb.iter().map(|(label, _)| ("", *label, false)).collect();
        spot.extend(pill.draw(paint, PPoint::new(left, at.origin.y)).into_iter().zip(crumb.into_iter().map(|(_, page)| Act::Go(page))));
        let top = at.origin.y + pill.high(paint) + gap;
        // Its variants: itself first, then the same thing in each other
        // colour, in the shop's order.
        let own = place_of(one, &colour);
        let hue = own.as_ref().map(|(_, hue)| hue.clone()).unwrap_or_default();
        let variant: Vec<(usize, String)> = std::iter::once((place, hue)).chain(self.kin(shop, own)).collect();
        // Wide: the variants' bar, the picture, the sections at the right.
        let beside = wide >= paint.size(SPLIT);
        let bar = match variant.len() > 1 {
            true => paint.size(VARIANT),
            false => 0.0,
        };
        let after = match bar > 0.0 {
            true => bar + gap,
            false => 0.0,
        };
        let side = match beside {
            true => ((wide - after - gap) / 2.0).min(paint.size(PICTURE) * 2.0),
            false => paint.size(PICTURE).min(wide - after),
        };
        let reach = self.variants(paint, PRect::new(left, top, bar, side), shop, &variant, place, spot);
        // The picture, standing at the top of its frame.
        let frame = PRect::new(left + after, top, side, side);
        light::surface(paint, frame, radius, PLight::Soft, "");
        match one.view.get(self.view.get()) {
            Some((_, image)) => paint.picture(image, frame.inset(gap / 2.0), radius, PFit::Top),
            None => self.portrait(paint, frame.inset(gap / 2.0), one, &shop.icon),
        }
        // Its views, as tabs on a ground under it; the one shown is lit.
        let view: Vec<(&str, &str, bool)> = one.view.iter().enumerate().map(|(index, (label, _))| ("", label.as_str(), index == self.view.get())).collect();
        let mut under = frame.origin.y + side + gap / 2.0;
        if view.len() > 1 {
            let pressed = view.draw_in(paint, PPoint::new(frame.origin.x, under), side);
            spot.extend(pressed.into_iter().enumerate().map(|(index, rect)| (rect, Act::View(index))));
            under += view.high(paint) + gap / 2.0;
        }
        let under = under.max(reach);
        // Every section, at the right; narrow, under the picture.
        let column = match beside {
            true => PRect::new(frame.origin.x + side + gap, top, left + wide - frame.origin.x - side - gap, 0.0),
            false => PRect::new(left, under + gap, wide, 0.0),
        };
        let down = self.sections(paint, column, shop, place, sight, spot);
        down.max(under)
    }

    /// A thing's sections down a column, each on a ground: what it is
    /// (its kind, name, price and standing, its like, what is said of
    /// it), its facts, and more like it. Where they end.
    fn sections(&self, paint: &mut dyn PPaint, column: PRect, shop: &Shop, place: usize, sight: PRect, spot: &mut Vec<(PRect, Act)>) -> f32 {
        let (gap, line, ink, dim, radius) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.color(DIM), paint.size(RADIUS));
        let Some(one) = shop.product.get(place) else { return column.origin.y };
        let text = &self.desk.text;
        let (room, x) = (column.size.width - gap, column.origin.x + gap / 2.0);
        let ground = |paint: &mut dyn PPaint, from: f32, to: f32| light::surface(paint, PRect::new(column.origin.x, from, column.size.width, to - from), radius, PLight::Soft, "");
        // What it is.
        let body: Vec<String> = wrap(&one.body, room, &|said| paint.measure(said)).into_iter().filter(|said| !said.is_empty()).collect();
        let first = column.origin.y;
        let end = first + gap / 2.0 + line + paint.size("head.line") + gap / 4.0 + line + gap / 2.0 + line + gap + body.len() as f32 * line + gap / 2.0;
        ground(paint, first, end);
        let mut down = first + gap / 2.0;
        paint.text(PPoint::new(x, down), &fit(&one.kind, room, paint), dim);
        down += line;
        paint.head(PPoint::new(x, down), &fit(&one.title, room, paint), ink);
        down += paint.size("head.line") + gap / 4.0;
        paint.text(PPoint::new(x, down), &one.sub, ink);
        let kind = [one.meta.as_str(), one.tag.as_str()].into_iter().filter(|said| !said.is_empty()).collect::<Vec<_>>().join(&format!(" {} ", text.get("between")));
        paint.text(PPoint::new(x + paint.measure(&one.sub) + gap, down), &fit(&kind, room - paint.measure(&one.sub) - gap, paint), dim);
        down += line + gap / 2.0;
        let stand = self.stand(paint, PPoint::new(x, down), &shop.id, one, spot);
        let own = format!("{}/{}", shop.id, one.id);
        let liked = self.desk.liked(&own);
        let said = match liked {
            true => text.get("shop.liked"),
            false => text.get("shop.like"),
        };
        let like = ("", said, liked).draw(paint, PPoint::new(stand.origin.x + stand.size.width + gap / 4.0, stand.origin.y), false);
        spot.push((like, Act::Like(own)));
        down += line + gap;
        for said in &body {
            paint.text(PPoint::new(x, down), said, ink);
            down += line;
        }
        // Its facts, a row each, on a ground of their own.
        let facts = end + gap / 2.0;
        let last = one.field.table(&mut PDry(&*paint), PPoint::new(x, facts + gap / 2.0), room);
        ground(paint, facts, last + gap / 2.0);
        one.field.table(paint, PPoint::new(x, facts + gap / 2.0), room);
        let mut down = last + gap;
        // More like it: the others of its page of the menu.
        let on = shop.shelf.iter().find(|shelf| shelf.at.contains(&place));
        let near: Vec<usize> = on.map(|shelf| shelf.at.clone()).unwrap_or_default().into_iter().filter(|other| *other != place).take(paint.size(MORE) as usize).collect();
        if !near.is_empty() {
            paint.text(PPoint::new(x, down), text.get("shop.more"), dim);
            down = self.grid(paint, PRect::new(column.origin.x, down + line + gap / 2.0, column.size.width, 0.0), shop, &near, sight, spot);
        }
        down
    }

    /// The bar of a thing's variants, down a column: each its picture
    /// with its colour's name under it, the one shown lit; a press on
    /// another goes to it. Where the bar ends.
    fn variants(&self, paint: &mut dyn PPaint, bar: PRect, shop: &Shop, variant: &[(usize, String)], shown: usize, spot: &mut Vec<(PRect, Act)>) -> f32 {
        let (gap, line, ink, radius) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.size(RADIUS));
        let cell = bar.size.width;
        let mut down = bar.origin.y;
        for (other, hue) in variant.iter().filter(|_| cell > 0.0) {
            let rect = PRect::new(bar.origin.x, down, cell, cell + line);
            let lit = match (*other == shown, self.over(rect)) {
                (true, _) => PLight::Strong,
                (false, true) => PLight::Rest,
                (false, false) => PLight::Soft,
            };
            light::surface(paint, rect, radius, lit, "");
            self.portrait(paint, PRect::new(rect.origin.x + gap / 4.0, rect.origin.y + gap / 4.0, cell - gap / 2.0, cell - gap / 2.0), &shop.product[*other], &shop.icon);
            let hue = fit(hue, cell - gap / 2.0, paint);
            paint.text(PPoint::new(rect.center().x - paint.measure(&hue) / 2.0, rect.origin.y + cell - gap / 4.0), &hue, ink);
            spot.push((rect, Act::Thing(format!("{}/{}", shop.id, shop.product[*other].id))));
            down += cell + line + gap / 2.0;
        }
        down
    }
}
