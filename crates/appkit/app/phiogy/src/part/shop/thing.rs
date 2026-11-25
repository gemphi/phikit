//! One thing of a shop as the kit's form `thing` (data/form.tsv), as
//! Phiegg's item page stands (`puikit/app/phiegg/src/components/Phiegg/
//! ItemSheet.tsx`): its name and what it is at the head; in its bar the
//! way back to its shop and its page of the menu, and the bag at the
//! end; at the left its picture, held in view, its views under it and
//! what can be done with it (like it, see it large); at the right its
//! tabs (data/thing.tsv), each of folding sections: what it is and its
//! facts; its figures from Phiegg's ledger of it; its trail of sales;
//! who has held it; its other colours and more like it. This file only
//! says what each list holds and what a press does.

use std::cell::RefCell;
use std::rc::Rc;

use appkit::pui::form::{PForm, PListed};
use appkit::pui::state::PState;
use appkit::service::PTable;
use serde_json::Value;

use super::swatch;
use crate::data;
use crate::part::Desk;
use crate::part::now::Shot;
use crate::part::store::dollar;
use crate::part::tool::grouped;
use crate::site::{Product, Shop};

/// The theme's token for how many more like it are shown.
pub const MORE: &str = "shop.more";

/// The paths the form keeps and is told.
const TAB: &str = "thing/tab";
const VIEW: &str = "thing/view";
const BAG: &str = "thing/bag";
const DO: &str = "thing/do";
const GO: &str = "thing/go";
const OPEN: &str = "thing/open";
const FULL: &str = "thing/full";

type Row = Vec<String>;

/// The rows of data/thing.tsv that begin with a word, without it.
fn table(word: &str) -> Vec<Vec<&'static str>> {
    data::row(data::THING).into_iter().filter(|row| row.first() == Some(&word)).map(|row| row[1..].to_vec()).collect()
}

/// The first cell of the row of data/thing.tsv that begins with a word.
fn first(word: &str) -> &'static str {
    table(word).first().and_then(|row| row.first().copied()).unwrap_or_default()
}

/// One sale of a thing, as Phiegg's ledger has it.
#[derive(Debug, Clone)]
struct Sale {
    id: String,
    from: String,
    to: String,
    price: f64,
    at: String,
    event: String,
}

/// What a thing is without its colour, and its colour: its name with
/// the value of the fact the site calls the colour taken out of it.
/// Nothing for a thing with no such fact.
fn hue_of(one: &Product) -> Option<(String, String)> {
    let colour = PTable::read(data::SITE).get("shop.colour").to_string();
    let hue = one.field.iter().find(|(name, _)| *name == colour).map(|(_, value)| value.clone())?;
    let rest = one.title.replace(&hue, "").split_whitespace().collect::<Vec<_>>().join(" ");
    Some((rest, hue)).filter(|(rest, _)| !rest.is_empty() && *rest != one.title)
}

/// A day, as the ledger writes a moment.
fn day(at: &str) -> String {
    at.split('T').next().unwrap_or_default().to_string()
}

/// What the thing's form holds, and what a press on it does.
struct Thing {
    desk: Desk,
    /// The view shown, of the thing of that place.
    view: RefCell<(String, usize)>,
    /// The sales of the thing of that place, read once.
    sale: RefCell<(String, Vec<Sale>)>,
}

impl Thing {
    /// The place of the thing shown: its shop and its id.
    fn place(&self) -> String {
        self.desk.now.product.borrow().clone().unwrap_or_default()
    }

    /// The thing shown: its shop, and its place in the shop's list.
    fn shown(&self) -> Option<(Rc<Shop>, usize)> {
        let place = self.place();
        let (shop, id) = place.split_once('/')?;
        let shop = self.desk.shop(shop)?;
        let at = shop.product.iter().position(|one| one.id == id)?;
        Some((shop, at))
    }

    /// Which of its views is shown.
    fn view(&self) -> usize {
        let (place, view) = self.view.borrow().clone();
        match place == self.place() {
            true => view,
            false => 0,
        }
    }

    /// Its sales, the newest first, read from the book once for each
    /// thing shown.
    fn sales(&self, one: &Product) -> Vec<Sale> {
        let place = self.place();
        if self.sale.borrow().0 != place {
            let row = table("sale").into_iter().next().filter(|row| row.len() >= 7).unwrap_or_default();
            let book = self.desk.book();
            // Its id as the book has it: Phiora's key where it gave the records.
            let key = book.key(first("item"), &one.id);
            let word = |found: &Value, at: usize| row.get(at).and_then(|field| found.get(*field)).and_then(Value::as_str).unwrap_or_default().to_string();
            let mut sale: Vec<Sale> = row.first().map(|kind| book.of(kind)).unwrap_or_default().iter().filter(|found| [one.id.as_str(), key.as_str()].contains(&word(&found.row, 1).as_str())).map(|found| Sale {
                id: found.id.clone(),
                from: word(&found.row, 2),
                to: word(&found.row, 3),
                price: row.get(4).and_then(|field| found.row.get(*field)).and_then(Value::as_f64).unwrap_or_default(),
                at: word(&found.row, 5),
                event: word(&found.row, 6),
            }).collect();
            sale.sort_by(|a, b| b.at.cmp(&a.at));
            *self.sale.borrow_mut() = (place, sale);
        }
        self.sale.borrow().1.clone()
    }

    /// Who held it, each once, the newest first: their id, when it came
    /// to them, at what price.
    fn holders(&self, one: &Product) -> Vec<Sale> {
        let none: Vec<&str> = table("none").into_iter().flatten().collect();
        let mut seen: Vec<String> = Vec::new();
        self.sales(one).into_iter().filter(|sale| !sale.to.is_empty() && !none.contains(&sale.event.as_str())).filter(|sale| {
            let first = !seen.contains(&sale.to);
            seen.push(sale.to.clone());
            first
        }).collect()
    }

    /// Somebody's name, as the book or the site has it.
    fn name(&self, id: &str) -> String {
        let book = self.desk.book();
        let found = book.named(id).map(|found| found.name.clone()).filter(|name| !name.is_empty());
        found.or_else(|| self.desk.site.member(id).map(|one| one.name.clone())).unwrap_or_else(|| id.to_string())
    }

    /// A card of a thing of the shop: what a press opens, its name, its
    /// icon where it has no picture, its picture, its price, its tag, its
    /// face.
    fn card(&self, shop: &Shop, one: &Product, line: &str) -> Row {
        let face = one.sprite.as_ref().map(|cell| format!("{} {} {}", cell.sheet, cell.column, cell.index)).unwrap_or_default();
        let icon = match one.image.is_empty() && face.is_empty() {
            true => self.desk.icon(&shop.icon).to_string(),
            false => String::new(),
        };
        vec![format!("{}/{}", shop.id, one.id), one.title.clone(), icon, one.image.clone(), line.to_string(), one.sub.clone(), face]
    }

    /// What it is, under its name: its kind, its price, its shop, its tag.
    fn said(&self, shop: &Shop, one: &Product) -> String {
        let between = format!(" {} ", self.desk.text.get("between"));
        [one.kind.as_str(), one.sub.as_str(), shop.label.as_str(), one.tag.as_str()].into_iter().filter(|said| !said.is_empty()).collect::<Vec<_>>().join(&between)
    }

    /// Its figures, as data/thing.tsv names them: the word and the value.
    fn stats(&self, one: &Product) -> Vec<Row> {
        let text = &self.desk.text;
        let sales = self.sales(one);
        let priced: Vec<f64> = sales.iter().map(|sale| sale.price).filter(|price| *price > 0.0).collect();
        let most = priced.iter().copied().fold(None, |most: Option<f64>, price| Some(most.map_or(price, |most| most.max(price))));
        let least = priced.iter().copied().fold(None, |least: Option<f64>, price| Some(least.map_or(price, |least| least.min(price))));
        table("stat").into_iter().filter_map(|row| {
            let (key, how) = (row.first()?, row.get(1)?);
            let value = match *how {
                "price" => Some(one.sub.clone()).filter(|sub| !sub.is_empty()),
                "count" => Some(sales.iter().filter(|sale| Some(&sale.event.as_str()) == row.get(2)).count()).filter(|n| *n > 0).map(|n| grouped(n as u64)),
                "last" => priced.first().map(|price| dollar(*price)),
                "high" => most.map(dollar),
                "low" => least.map(dollar),
                "mean" => Some(priced.len()).filter(|n| *n > 0).map(|n| dollar(priced.iter().sum::<f64>() / n as f64)),
                "holders" => Some(self.holders(one).len()).filter(|n| *n > 0).map(|n| grouped(n as u64)),
                "first" => sales.last().map(|sale| day(&sale.at)),
                "latest" => sales.first().map(|sale| day(&sale.at)),
                _ => None,
            }?;
            Some(vec![key.to_string(), text.get(&format!("thing.stat.{key}")).to_string(), value])
        }).collect()
    }

    /// What a list holds, whichever tab is on.
    fn list(&self, name: &str) -> Vec<Row> {
        let Some((shop, at)) = self.shown() else { return Vec::new() };
        let (one, text) = (&shop.product[at], &self.desk.text);
        match name {
            "thing.head" => vec![vec![one.id.clone(), one.title.clone(), self.said(&shop, one)]],
            // The way back: its shop, its page of the menu, and itself.
            "thing.path" => {
                let on = shop.shelf.iter().find(|shelf| shelf.at.contains(&at));
                let way = [Some((shop.id.clone(), shop.label.clone())), on.map(|shelf| (shelf.id.clone(), shelf.label.clone())), Some((String::new(), one.title.clone()))];
                way.into_iter().flatten().map(|(page, label)| vec![page, label]).collect()
            }
            "thing.buy" => vec![vec!["pick".to_string(), self.desk.stands(&shop.id, one).to_string(), "bag".to_string()]],
            // Its picture: the view shown, its own, or the cell of a sheet.
            "thing.picture" => {
                let face = one.sprite.as_ref().map(|cell| format!("{} {} {}", cell.sheet, cell.column, cell.index)).unwrap_or_default();
                let picture = one.view.get(self.view()).map(|(_, image)| image.clone()).unwrap_or_else(|| one.image.clone());
                match picture.is_empty() && face.is_empty() {
                    true => Vec::new(),
                    false => vec![vec!["full".to_string(), one.title.clone(), picture, face]],
                }
            }
            "thing.view" if one.view.len() > 1 => one.view.iter().enumerate().map(|(index, (label, _))| vec![index.to_string(), label.clone()]).collect(),
            // A theme is shown by its own colours.
            "thing.swatch" => swatch::colour(&one.swatch).into_iter().enumerate().map(|(index, colour)| {
                let byte = |part: f32| ((part * 255.0).round() as u8).to_string();
                vec![index.to_string(), String::new(), byte(colour.r), byte(colour.g), byte(colour.b)]
            }).collect(),
            "thing.deed" => {
                let like = match self.desk.liked(&self.place()) {
                    true => "shop.liked",
                    false => "shop.like",
                };
                let full = Some(vec!["full".to_string(), "shop.full".to_string(), "maximize".to_string()]).filter(|_| !one.view.is_empty() || !one.image.is_empty());
                std::iter::once(vec!["like".to_string(), like.to_string(), "heart".to_string()]).chain(full).collect()
            }
            "thing.tab" => self.tabs().into_iter().map(|(key, icon, _)| vec![key.clone(), text.get(&format!("thing.{key}")).to_string(), icon]).collect(),
            "thing.about" => {
                let body = Some(vec!["body".to_string(), one.body.clone(), String::new()]).filter(|row| !row[1].is_empty());
                body.into_iter().chain(one.point.iter().enumerate().map(|(index, point)| vec![index.to_string(), point.clone(), String::new()])).collect()
            }
            "thing.field" => one.field.iter().map(|(name, value)| vec![name.clone(), name.clone(), value.clone()]).collect(),
            "thing.stat" => self.stats(one),
            "thing.trail" => self.sales(one).into_iter().map(|sale| {
                let went = text.get("thing.went").replace("{from}", &self.name(&sale.from)).replace("{to}", &self.name(&sale.to));
                let event = match text.get(&format!("thing.event.{}", sale.event)) {
                    "" => sale.event.clone(),
                    word => word.to_string(),
                };
                let line = [event, day(&sale.at)].join(&format!(" {} ", text.get("between")));
                vec![format!("rec:{}:{}", first("sale"), sale.id), went, "workflow".to_string(), line, dollar(sale.price)]
            }).collect(),
            "thing.owner" => self.holders(one).into_iter().map(|sale| {
                let kind = first("person");
                let picture = self.desk.book().record(kind, &sale.to).map(|found| found.picture.clone()).unwrap_or_default();
                let face = self.desk.site.face(&sale.to).map(|cell| format!("{} {} {}", cell.sheet, cell.column, cell.index)).unwrap_or_default();
                let icon = match picture.is_empty() && face.is_empty() {
                    true => "user".to_string(),
                    false => String::new(),
                };
                vec![format!("rec:{kind}:{}", sale.to), self.name(&sale.to), icon, picture, text.get("thing.since").replace("{d}", &day(&sale.at)), dollar(sale.price), face]
            }).collect(),
            // The same thing in its other colours, in the shop's order.
            "thing.kin" => {
                let Some((family, hue)) = hue_of(one) else { return Vec::new() };
                shop.product.iter().filter_map(|other| Some((other, hue_of(other)?))).filter(|(_, (rest, other))| *rest == family && *other != hue).map(|(other, (_, hue))| self.card(&shop, other, &hue)).collect()
            }
            // More like it: the others of its page of the menu.
            "thing.more" => {
                let on = shop.shelf.iter().find(|shelf| shelf.at.contains(&at));
                let near = on.map(|shelf| shelf.at.clone()).unwrap_or_default().into_iter().filter(|other| *other != at).take(self.desk.size(MORE) as usize);
                near.filter_map(|other| shop.product.get(other)).map(|other| self.card(&shop, other, &other.kind)).collect()
            }
            _ => Vec::new(),
        }
    }

    /// The tabs that hold something: each its key, its icon, its lists.
    fn tabs(&self) -> Vec<(String, String, Vec<String>)> {
        table("tab").into_iter().filter(|row| row.len() >= 3).map(|row| (row[0].to_string(), row[1].to_string(), row[2].split_whitespace().map(str::to_string).collect::<Vec<_>>()))
            .filter(|(_, _, lists)| lists.iter().any(|list| !self.list(list).is_empty())).collect()
    }

    /// The tab on: the one chosen, where it holds something; else the
    /// first that does.
    fn on(&self) -> String {
        let (tabs, kept) = (self.tabs(), self.desk.shelf.get(TAB));
        tabs.iter().find(|(key, ..)| *key == kept).or(tabs.first()).map(|(key, ..)| key.clone()).unwrap_or_default()
    }

    /// What a list holds now: a tab's lists only while it is on.
    fn rows(&self, name: &str) -> Vec<Row> {
        let tab = table("tab").into_iter().find(|row| row.get(2).is_some_and(|lists| lists.split_whitespace().any(|list| list == name)));
        match tab {
            Some(tab) if tab[0] != self.on() => Vec::new(),
            _ => self.list(name),
        }
    }

    /// The viewer, on its views, the one shown first.
    fn full(&self) {
        let Some((shop, at)) = self.shown() else { return };
        let one = &shop.product[at];
        let shot = |(label, key): (String, String)| Shot { key, who: String::new(), by: one.title.clone(), when: label, file: None };
        let album: Vec<Shot> = match one.view.is_empty() {
            true => vec![shot((String::new(), one.image.clone()))],
            false => one.view.iter().cloned().map(shot).collect(),
        };
        self.desk.look(album, self.view());
    }
}

impl PState for Thing {
    fn get(&self, path: &str) -> String {
        let one = self.shown();
        match path {
            TAB => self.on(),
            VIEW => self.view().to_string(),
            // Lit while it is in the bag or theirs.
            BAG => one.filter(|(shop, at)| !matches!(self.desk.stands(&shop.id, &shop.product[*at]), "shop.put" | "shop.wear")).map(|_| "pick".to_string()).unwrap_or_default(),
            DO => Some("like".to_string()).filter(|_| self.desk.liked(&self.place())).unwrap_or_default(),
            _ => self.desk.shelf.get(path),
        }
    }

    fn set(&self, path: &str, value: &str) {
        let desk = &self.desk;
        match (path, value) {
            (VIEW, view) => *self.view.borrow_mut() = (self.place(), view.parse().unwrap_or_default()),
            (BAG, _) => self.shown().into_iter().for_each(|(shop, at)| desk.pick(&shop.id, &shop.product[at])),
            (DO, "like") => desk.fond(&self.place(), !desk.liked(&self.place())),
            (DO | FULL, "full") => self.full(),
            // Another thing of the shop, or a page of it; itself is no way.
            (GO, "") => {}
            (GO, place) if place.contains('/') => *desk.now.product.borrow_mut() = Some(place.to_string()),
            (GO, page) => _ = desk.show(page),
            (OPEN, what) => {
                let (kind, id) = what.strip_prefix("rec:").and_then(|rest| rest.split_once(':')).unwrap_or_default();
                desk.open_record(kind, id);
            }
            _ => desk.shelf.set(path, value),
        }
    }
}

/// The form of a thing's page, its lists and values from the thing shown.
pub fn form(desk: &Desk) -> PForm {
    let thing = Rc::new(Thing { desk: desk.clone(), view: RefCell::default(), sale: RefCell::default() });
    let rows = thing.clone();
    let state: Vec<(String, Rc<dyn PState>)> = vec![("thing".to_string(), thing), ("ui".to_string(), desk.shelf.clone())];
    let source = Rc::new(PListed::new(data::LISTING, Rc::new(move |name: &str| rows.rows(name)), desk.text.clone(), state));
    PForm::new("thing", data::FORM, source, desk.text.clone(), desk.shade.clone())
}
