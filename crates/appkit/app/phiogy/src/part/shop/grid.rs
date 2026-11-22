//! A shop's landing and the pages of its menu: what the page is
//! called, on a landing the shop's own picture, and its things in a
//! grid of cards. A card is the thing's picture, whole; what it is
//! called; its price; and how it stands, which a press changes. Only
//! the rows in sight are drawn: a page can hold hundreds of things.

use appkit::pui::core::{PFit, PLight, PPaint, PPoint, PRect, light};

use super::{Act, Shopping, swatch};
use crate::part::{DIM, GAP, ICON, INK, LINE, RADIUS, fit};
use crate::site::Product;

/// The theme's tokens: how wide a card is, and how many times as wide
/// as high a landing's picture is.
pub const CARD: &str = "shop.card";
pub const HERO: &str = "shop.hero";

impl Shopping {
    /// The picture of a thing in a place: its own, whole; a cell of a
    /// sheet; or, where it has neither, the icon of its shop.
    pub(super) fn portrait(&self, paint: &mut dyn PPaint, rect: PRect, product: &Product, icon: &str) {
        let (radius, size, ink) = (paint.size(RADIUS), paint.size(ICON), paint.color(INK));
        match (&product.sprite, product.image.is_empty(), product.swatch.is_empty()) {
            (Some(cell), ..) => paint.picture(&cell.sheet, rect, radius, PFit::Cell { column: cell.column, index: cell.index }),
            (None, false, _) => paint.picture(&product.image, rect, radius, PFit::Whole),
            // A theme is shown by its own colours.
            (None, true, false) => swatch::draw(paint, rect, &product.swatch, radius),
            (None, true, true) => paint.icon(self.desk.icon(icon), PRect::new(rect.center().x - size / 2.0, rect.center().y - size / 2.0, size, size), ink),
        }
    }

    /// How a thing stands, as a pill at a place: lit when it is in the
    /// bag or owned. A press on it does the next thing.
    pub(super) fn stand(&self, paint: &mut dyn PPaint, at: PPoint, shop: &str, product: &Product, spot: &mut Vec<(PRect, Act)>) -> PRect {
        let (gap, line, ink, radius) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.size(RADIUS));
        let (place, stands) = (format!("{shop}/{}", product.id), self.desk.stands(shop, product));
        let said = self.desk.text.get(stands);
        let rect = PRect::new(at.x, at.y, paint.measure(said) + gap, line + gap / 2.0);
        let lit = match stands {
            "shop.put" | "shop.wear" => PLight::Rest,
            _ => PLight::Strong,
        };
        light::label(paint, rect, radius, lit, false, "");
        paint.text(PPoint::new(at.x + gap / 2.0, at.y + gap / 4.0), said, ink);
        spot.push((rect, Act::Pick(place)));
        rect
    }

    /// One card of a grid.
    pub(super) fn card(&self, paint: &mut dyn PPaint, rect: PRect, shop: &str, icon: &str, product: &Product, spot: &mut Vec<(PRect, Act)>) {
        let (gap, line, ink, dim, radius) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.color(DIM), paint.size(RADIUS));
        let place = format!("{shop}/{}", product.id);
        light::surface(paint, rect, radius, PLight::Soft, "");
        spot.push((rect, Act::Thing(place.clone())));
        let side = rect.size.width - gap;
        self.portrait(paint, PRect::new(rect.origin.x + gap / 2.0, rect.origin.y + gap / 2.0, side, side), product, icon);
        let (left, top) = (rect.origin.x + gap / 2.0, rect.origin.y + side + gap * 0.75);
        paint.text(PPoint::new(left, top), &fit(&product.title, side, paint), ink);
        paint.text(PPoint::new(left, top + line), &fit(&product.sub, side, paint), dim);
        self.stand(paint, PPoint::new(left, top + line * 2.0 + gap / 4.0), shop, product, spot);
    }

    /// How high a card that wide is.
    pub(super) fn tall(paint: &dyn PPaint, wide: f32) -> f32 {
        wide + paint.size(LINE) * 3.0 + paint.size(GAP) * 1.5
    }

    /// Some things of a shop in a grid from `top` down, as many across
    /// as fit; how far down it ends.
    pub(super) fn grid(&self, paint: &mut dyn PPaint, at: PRect, shop: &crate::site::Shop, list: &[usize], sight: PRect, spot: &mut Vec<(PRect, Act)>) -> f32 {
        let (gap, card) = (paint.size(GAP), paint.size(CARD).max(1.0));
        let across = (((at.size.width + gap / 2.0) / (card + gap / 2.0)).floor() as usize).max(1);
        let wide = (at.size.width - (across - 1) as f32 * gap / 2.0) / across as f32;
        let high = Self::tall(paint, wide);
        for (index, product) in list.iter().filter_map(|place| shop.product.get(*place)).enumerate() {
            let rect = PRect::new(at.origin.x + (index % across) as f32 * (wide + gap / 2.0), at.origin.y + (index / across) as f32 * (high + gap / 2.0), wide, high);
            // A row out of sight is not drawn.
            if rect.origin.y + high >= sight.origin.y && rect.origin.y <= sight.origin.y + sight.size.height {
                self.card(paint, rect, &shop.id, &shop.icon, product, spot);
            }
        }
        at.origin.y + list.len().div_ceil(across) as f32 * (high + gap / 2.0)
    }

    /// A shop's page: its landing, a page of its menu, or one thing's.
    pub(super) fn shelf(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, line, ink, dim) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.color(DIM));
        let open = self.desk.open.borrow().clone();
        let thing = self.desk.now.product.borrow().clone();
        let id = thing.as_deref().and_then(|place| place.split_once('/')).map(|(shop, _)| shop.to_string()).or_else(|| self.desk.shop_of(&open));
        let sight = PRect { origin, size: self.frame.size };
        let (left, wide) = (origin.x + gap, (self.frame.size.width - gap * 2.0).max(0.0));
        let begin = origin.y + gap - self.scroll.get().at();
        let mut spot = Vec::new();
        self.origin.set(origin);
        let Some(shop) = id.and_then(|id| self.desk.shop(&id)) else {
            return paint.text(PPoint::new(left, begin), self.desk.text.get("shop.none"), dim);
        };
        paint.clip(sight);
        let end = match thing.as_deref().and_then(|place| place.split_once('/')).and_then(|(_, id)| shop.product.iter().position(|product| product.id == id)) {
            Some(at) => self.thing(paint, PRect::new(left, begin, wide, 0.0), &shop, at, sight, &mut spot),
            None => {
                // What the page is called, and under it whose it is.
                let shelf = shop.shelf.iter().find(|shelf| shelf.id == open);
                let (title, under) = match shelf {
                    Some(shelf) => (shelf.label.as_str(), shop.label.as_str()),
                    None => (shop.label.as_str(), shop.sub.as_str()),
                };
                paint.head(PPoint::new(left, begin), title, ink);
                let mut top = begin + paint.size("head.line") + gap / 4.0;
                paint.text(PPoint::new(left, top), &fit(under, wide, paint), dim);
                top += line + gap;
                // A landing has the shop's picture, and names its grid.
                if shelf.is_none() && !shop.image.is_empty() {
                    let high = wide / paint.size(HERO).max(1.0);
                    let radius = paint.size(RADIUS);
                    paint.picture(&shop.image, PRect::new(left, top, wide, high), radius, PFit::Cover);
                    top += high + gap;
                }
                if shelf.is_none() && !shop.lead.is_empty() {
                    paint.text(PPoint::new(left, top), &shop.lead, dim);
                    top += line + gap / 2.0;
                }
                let list = shelf.map_or(shop.first.as_slice(), |shelf| shelf.at.as_slice());
                self.grid(paint, PRect::new(left, top, wide, 0.0), &shop, list, sight, &mut spot)
            }
        };
        paint.unclip();
        let mut scroll = self.scroll.get();
        scroll.fit(end - begin + gap * 2.0, self.frame.size.height);
        self.scroll.set(scroll);
        *self.spot.borrow_mut() = spot;
    }
}
