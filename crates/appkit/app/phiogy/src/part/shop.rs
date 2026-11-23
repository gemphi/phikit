//! The shops: the page a shop is picked from, a shop's own pages, one
//! thing's page, and the bag, as the web's are
//! (`puikit/app/phi-kit/src/shops.tsx`, `bag.tsx`, the `product`
//! module).
//!
//! * Shops: every shop of the index, searched by name; one is opened,
//!   which puts it in the rail, and closed again.
//! * A shop's landing and the pages of its menu: its things in a grid
//!   (`shop/grid.rs`), each with its picture, what it is called, its
//!   price, and how it stands: to add, in the bag, owned. A press on
//!   that does the next thing; a press on the thing opens its page.
//! * A thing's page (`shop/item.rs`): its views, what is said of it,
//!   its facts, and more like it.
//! * The bag: its lines, the total, paying from the wallet, and under
//!   it what was bought.

mod grid;
mod hover;
mod item;
mod swatch;

pub use swatch::{colour, hex, worn};

use std::cell::{Cell, RefCell};

use appkit::pui::control::{PControl, PField};
use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};
use appkit::pui::layout::PScroll;

use super::list::{Deed, Item, Lister, Seat, field, grouped};
use super::store::{Line, dollar};
use super::{Desk, Tool, page};
use crate::data;

/// The theme's token for how many shops a search lists at most.
pub const MOST: &str = "shop.most";
/// The row of `data/act.tsv` for taking a thing out of the bag.
pub const OUT: &str = "out";

/// What a press on any of the shops' pages means.
#[derive(Debug, Clone, PartialEq)]
pub enum Act {
    Field,
    /// Open a shop, or close it.
    Choose(String, bool),
    /// Go to a page.
    Go(String),
    /// Open a thing's own page; and show the view of that number there.
    Thing(String),
    View(usize),
    /// Like a thing, or no more.
    Like(String),
    /// Put a thing in the bag, or take it out.
    Pick(String),
    Remove(String),
    Pay,
}

/// The shops.
pub struct Shopping {
    frame: PRect,
    desk: Desk,
    list: Lister<Act>,
    tool: Tool,
    find: PField,
    /// A grid's own scroll, where it was drawn, and its places to press.
    scroll: Cell<PScroll>,
    origin: Cell<PPoint>,
    spot: RefCell<Vec<(PRect, Act)>>,
    /// The page it was drawn for last: another begins at its top.
    seen: RefCell<String>,
    /// The view of a thing that is shown, and whether the wallet was
    /// too short for the bag the last time paying was tried.
    view: Cell<usize>,
    short: Cell<bool>,
    /// Where the pointer is, in the window.
    point: Cell<Option<PPoint>>,
}

impl Shopping {
    pub fn new(desk: Desk) -> Shopping {
        let mut find = PField::new(PRect::default());
        find.focused = true;
        Shopping { frame: PRect::default(), desk, list: Lister::default(), tool: Tool::read(data::ACT), find, scroll: Cell::default(), origin: Cell::default(), spot: RefCell::default(), seen: RefCell::default(), view: Cell::default(), short: Cell::default(), point: Cell::default() }
    }

    /// The page every shop is picked from.
    fn index(&self) -> Vec<Item<Act>> {
        let (text, extra) = (&self.desk.text, &self.desk.site.extra);
        let word = |name: &str| text.get(name).to_string();
        let mut item = vec![Item::Head(word("shop.title")), Item::Note(word("shop.lead")), Item::Field(0, word("shop.find"), Act::Field)];
        let wanted = self.find.value().trim().to_lowercase();
        let found: Vec<_> = extra.store.iter().filter(|store| wanted.is_empty() || store.label.to_lowercase().contains(&wanted)).take(self.desk.size(MOST) as usize).collect();
        for store in &found {
            let count = text.fill("shop.items", "n", &grouped(store.count));
            let sub = [store.sub.as_str(), count.as_str()].into_iter().filter(|said| !said.is_empty()).collect::<Vec<_>>().join(&format!(" {} ", text.get("between")));
            let close = Some(Deed::word(text.get("shop.close"), false, Act::Choose(store.id.clone(), false))).filter(|_| self.desk.shop_open(&store.id));
            let deed = close.into_iter().chain([Deed::word(text.get("shop.open"), false, Act::Choose(store.id.clone(), true))]).collect();
            item.push(Item::Line(Seat::Icon(self.desk.icon(&store.icon).to_string()), store.label.clone(), sub, None, deed));
        }
        if found.is_empty() {
            item.push(Item::Note(word("shop.none")));
        }
        item.push(Item::Note(text.fill("shop.count", "n", &extra.store.len().to_string())));
        item
    }

    /// The bag's page.
    fn bag(&self) -> Vec<Item<Act>> {
        let text = &self.desk.text;
        let word = |name: &str| text.get(name).to_string();
        let (bag, bought) = (self.desk.bag(), self.desk.bought());
        // A line shows the thing itself: its cell of a sheet, its
        // picture, or the bag where it has neither.
        let seat = |line: &Line| {
            let thing = self.desk.shop(&line.shop).and_then(|shop| shop.product.iter().find(|product| product.id == line.id).cloned());
            match (thing.and_then(|thing| thing.sprite), line.image.is_empty()) {
                (Some(cell), _) => Seat::Face(cell),
                (None, false) => Seat::Picture(line.image.clone()),
                (None, true) => Seat::Icon(self.desk.icon(self.desk.page.icon(page::BAG)).to_string()),
            }
        };
        let out = self.tool.of(OUT, false, text).map_or("", |(icon, _)| icon);
        let mut item = vec![Item::Head(word("shop.bag"))];
        for line in &bag {
            item.push(Item::Line(seat(line), line.title.clone(), dollar(line.cost), Some(Act::Thing(format!("{}/{}", line.shop, line.id))), vec![Deed::icon(out, text.get("shop.remove"), Act::Remove(line.id.clone()))]));
        }
        if bag.is_empty() {
            item.extend([Item::Note(word("shop.empty")), Item::Note(word("shop.emptylead"))]);
        } else {
            let total = dollar(bag.iter().map(|line| line.cost).sum());
            item.extend([Item::Sum(word("shop.subtotal"), total.clone()), Item::Sum(word("shop.delivery"), word("shop.free")), Item::Sum(word("shop.total"), total)]);
            item.push(Item::Line(Seat::None, String::new(), String::new(), None, vec![Deed::word(text.get("shop.pay"), true, Act::Pay)]));
        }
        item.push(Item::Sum(word("shop.wallet"), dollar(self.desk.wallet())));
        if self.short.get() {
            item.push(Item::Note(word("shop.short")));
        }
        if !bought.is_empty() {
            item.push(Item::Head(word("shop.yours")));
            item.extend(bought.iter().map(|line| Item::Line(seat(line), line.title.clone(), dollar(line.cost), Some(Act::Thing(format!("{}/{}", line.shop, line.id))), Vec::new())));
        }
        item
    }

    /// The page that is open, and whether it is one of the two lists.
    fn listed(&self) -> Option<Vec<Item<Act>>> {
        let open = self.desk.open.borrow().clone();
        let thing = self.desk.now.product.borrow().is_some();
        // A page that is no shop's is the one every shop is picked from.
        match (thing, open.as_str() == self.desk.page.id(page::BAG), self.desk.shop_of(&open).is_some()) {
            (true, ..) | (false, false, true) => None,
            (false, true, _) => Some(self.bag()),
            (false, false, false) => Some(self.index()),
        }
    }

    /// Do what a press means.
    fn act(&mut self, act: Act) {
        let desk = &self.desk;
        match act {
            Act::Field => {}
            Act::Choose(id, on) => {
                desk.choose(&id, on);
                on.then(|| desk.show(&id));
            }
            Act::Go(page) => _ = desk.show(&page),
            Act::Thing(place) => {
                self.view.set(0);
                *desk.now.product.borrow_mut() = Some(place);
            }
            Act::View(index) => self.view.set(index),
            Act::Like(place) => desk.fond(&place, !desk.liked(&place)),
            Act::Pick(place) => {
                let (shop, id) = place.split_once('/').unwrap_or_default();
                let found = desk.shop(shop).and_then(|shop| shop.product.iter().find(|product| product.id == id).cloned());
                found.into_iter().for_each(|product| desk.pick(shop, &product));
                self.short.set(false);
            }
            Act::Remove(id) => {
                desk.unpick(&id);
                self.short.set(false);
            }
            Act::Pay => self.short.set(!desk.pay()),
        }
    }
}

impl PDraw for Shopping {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        // Another page, and another thing, begin at their top.
        let now = format!("{} {:?}", self.desk.open.borrow(), self.desk.now.product.borrow());
        if *self.seen.borrow() != now {
            self.scroll.set(PScroll::default());
            *self.seen.borrow_mut() = now;
        }
        match self.listed() {
            Some(item) => self.list.draw(paint, origin, self.frame, &self.desk, &item, &[&self.find]),
            None => self.shelf(paint, origin),
        }
    }
}

impl PView for Shopping {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
        self.find.set_frame(field(&self.desk, frame.size.width));
    }

    fn hand(&self) -> Vec<PRect> {
        match self.listed().is_some() {
            true => self.list.hands(),
            false => self.hands(),
        }
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        let listed = self.listed().is_some();
        match (*event, listed) {
            (PEvent::KeyDown(_), true) => self.find.respond(event),
            (PEvent::Scroll(_, by), true) => self.list.wheel(by.y),
            (PEvent::Scroll(_, by), false) => {
                let mut scroll = self.scroll.get();
                let any = scroll.turn(-by.y);
                self.scroll.set(scroll);
                any
            }
            (PEvent::PointerMove(at), false) => self.moved(at),
            (PEvent::PointerDown(at), _) => {
                let place = at.plus(self.origin.get());
                let found = match listed {
                    true => self.list.press(at),
                    false => self.spot.borrow().iter().rev().find(|(rect, _)| rect.contains(place)).map(|(_, act)| act.clone()),
                };
                let known = found.is_some();
                found.into_iter().for_each(|act| self.act(act));
                known
            }
            _ => false,
        }
    }
}
