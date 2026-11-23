//! The shops a person has open, their bag, and paying, as the web
//! keeps them (`puikit/app/phi-kit/src/shops.tsx`, `bag.tsx`,
//! `wallet.ts`). A shop of the index is open from the start when the
//! index says so, until the person says otherwise; an open shop is a
//! place of the rail: its landing, its own menu, and the bag. What is
//! in the bag and what was bought are the person's own documents.
//!
//! Paying takes the whole bag's total from the wallet's dollars, and
//! nothing when the wallet holds less. The wallet is Phiora's to
//! write: with Phiora answering it is asked to move the money, as a
//! page asks (looks through `/shop/buy`, where Phiora prices them
//! itself; anything else a plain spend); without it the wallet is the
//! one a person starts with (`data/wallet.tsv`), kept in their folder.
//! It is play money either way: nothing real is paid.

use std::rc::Rc;

use serde_json::{Value, json};

use appkit::service::PTable;

use super::{Desk, Rung, page};
use crate::data;
use crate::hour;
use crate::site::{Product, Shop};

/// The person's own documents the shops are kept in.
pub const OPEN: &str = "shop";
pub const BAG: &str = "bag";
pub const OWNED: &str = "owned";
pub const LIKE: &str = "like";
pub const WALLET: &str = "wallet";
pub const HISTORY: &str = "history";
pub const WORN: &str = "wear";
/// What Phiora is asked to move the money by, and how the row of a
/// coin a person starts with begins in `data/wallet.tsv`.
pub const BUY: &str = "buy";
pub const SPEND: &str = "spend";
pub const COIN: &str = "coin.";

/// A thing in the bag, or bought: enough to show it and to pay for it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Line {
    pub id: String,
    pub shop: String,
    pub title: String,
    pub cost: f64,
    pub image: String,
    pub look: bool,
}

/// Dollars as a person reads them: a sign, and the thousands apart.
pub fn dollar(amount: f64) -> String {
    format!("${}", super::list::grouped(amount.round().max(0.0) as u64))
}

impl Desk {
    /// A shop by its id, read from its own file the first time.
    pub fn shop(&self, id: &str) -> Option<Rc<Shop>> {
        if !self.stock.borrow().contains_key(id) {
            let listed = self.site.extra.store.iter().any(|store| store.id == id);
            let text = Some(format!("{}/{id}.json", self.site.extra.stock)).filter(|_| listed).and_then(|name| self.site.load(&name));
            let shop = text.and_then(|text| Shop::read(&text, &PTable::read(data::SITE))).map(Rc::new);
            self.stock.borrow_mut().insert(id.to_string(), shop);
        }
        self.stock.borrow().get(id).cloned().flatten()
    }

    /// Whether a shop is open: as the person last said, or as the
    /// index says.
    pub fn shop_open(&self, id: &str) -> bool {
        self.kept.flag(OPEN, &self.key(id)).unwrap_or_else(|| self.site.extra.stocked.iter().any(|known| known == id))
    }

    /// Open a shop, or close it.
    pub fn choose(&self, id: &str, on: bool) {
        self.kept.put(OPEN, &self.key(id), json!(on));
    }

    /// The shop a page is of: its landing, or a page of its menu.
    pub fn shop_of(&self, id: &str) -> Option<String> {
        let own = |store: &&crate::site::Store| id == store.id || id.strip_prefix(store.id.as_str()).is_some_and(|rest| rest.starts_with('.'));
        self.site.extra.store.iter().find(own).map(|store| store.id.clone())
    }

    /// The open shops, each a place of the rail: its landing first,
    /// then its own menu, then the bag.
    pub fn shop_rail(&self) -> Vec<(String, Vec<Rung>)> {
        let open = self.site.extra.store.iter().filter(|store| self.shop_open(&store.id)).filter_map(|store| self.shop(&store.id));
        let rail = open.map(|shop| {
            let home = Rung { id: shop.id.clone(), label: self.text.get("shop.home").to_string(), icon: self.page.icon(page::HOME).to_string(), ..Rung::default() };
            let shelf = shop.shelf.iter().map(|shelf| Rung { id: shelf.id.clone(), label: shelf.label.clone(), icon: shelf.icon.clone(), head: shelf.head.clone(), ..Rung::default() });
            let bag = self.page.rung(page::BAG, &self.text, self.bag().len() as u64);
            (shop.label.clone(), std::iter::once(home).chain(shelf).chain(std::iter::once(bag)).collect())
        });
        rail.collect()
    }

    /// The lines of one of the two documents, for this app.
    fn lines(&self, doc: &str) -> Vec<Line> {
        let name = PTable::read(data::WALLET);
        let all = self.kept.get(doc)[&self.site.app].as_array().cloned().unwrap_or_default();
        let text = |one: &Value, row: &str| one[name.get(row)].as_str().unwrap_or_default().to_string();
        all.iter().map(|one| Line { id: text(one, "line.id"), shop: text(one, "line.shop"), title: text(one, "line.title"), cost: one[name.get("line.cost")].as_f64().unwrap_or_default(), image: text(one, "line.image"), look: one[name.get("line.look")].as_bool().unwrap_or(false) }).collect()
    }

    fn keep_lines(&self, doc: &str, list: &[Line]) {
        let name = PTable::read(data::WALLET);
        let one = |line: &Line| json!({ name.get("line.id"): line.id, name.get("line.shop"): line.shop, name.get("line.title"): line.title, name.get("line.cost"): line.cost, name.get("line.image"): line.image, name.get("line.look"): line.look });
        self.kept.put(doc, &self.site.app, Value::Array(list.iter().map(one).collect()));
    }

    /// What the person picked and has not paid for; and what they bought.
    pub fn bag(&self) -> Vec<Line> {
        self.lines(BAG)
    }

    pub fn bought(&self) -> Vec<Line> {
        self.lines(OWNED)
    }

    /// Put a thing in the bag, or take it out again; one that is
    /// bought stays bought.
    pub fn pick(&self, shop: &str, product: &Product) {
        let mut bag = self.bag();
        // What is bought stays bought: one that can be worn is put on,
        // or taken off, by the same press.
        if self.bought().iter().any(|line| line.id == product.id) {
            return Some(format!("{shop}/{}", product.id)).filter(|_| product.wear).into_iter().for_each(|place| self.wear(&place));
        }
        match bag.iter().position(|line| line.id == product.id) {
            Some(at) => _ = bag.remove(at),
            None => bag.push(Line { id: product.id.clone(), shop: shop.to_string(), title: product.title.clone(), cost: product.cost, image: product.image.clone(), look: product.look }),
        }
        self.keep_lines(BAG, &bag);
    }

    /// Take a thing out of the bag.
    pub fn unpick(&self, id: &str) {
        let bag: Vec<Line> = self.bag().into_iter().filter(|line| line.id != id).collect();
        self.keep_lines(BAG, &bag);
    }

    /// How a thing stands, as the row of the words that says so: to
    /// add, in the bag, owned; and one that is owned and can be worn:
    /// to put on, worn.
    /// Whether the person likes a thing of a shop, by its place; and
    /// like it, or no more.
    pub fn liked(&self, place: &str) -> bool {
        appkit::pui::state::PState::has(self, &format!("{LIKE}/{}", self.key(place)))
    }

    pub fn fond(&self, place: &str, on: bool) {
        appkit::pui::state::PState::set(self, &format!("{LIKE}/{}", self.key(place)), appkit::pui::state::flag(on));
    }

    pub fn stands(&self, shop: &str, product: &Product) -> &'static str {
        let (id, worn) = (&product.id, self.worn() == format!("{shop}/{}", product.id));
        match (self.bought().iter().any(|line| line.id == *id), self.bag().iter().any(|line| line.id == *id), product.wear, worn) {
            (true, _, true, true) => "shop.worn",
            (true, _, true, false) => "shop.wear",
            (true, ..) => "shop.owned",
            (false, true, ..) => "shop.inbag",
            (false, false, ..) => "shop.put",
        }
    }

    /// The theme that is worn, as its shop and its id; nothing for none.
    pub fn worn(&self) -> String {
        self.kept.get(WORN)[&self.site.app].as_str().unwrap_or_default().to_string()
    }

    /// Put a theme on, or take it off again when it is the one worn:
    /// the window takes its colours, as `data/wear.tsv` says.
    pub fn wear(&self, place: &str) {
        let next = Some(place).filter(|place| *place != self.worn()).unwrap_or_default();
        self.kept.put(WORN, &self.site.app, json!(next));
        self.dressed();
    }

    /// Wear what is kept as worn: called when the window opens too.
    pub fn dressed(&self) {
        let worn = self.worn();
        let thing = worn.split_once('/').and_then(|(shop, id)| self.shop(shop)?.product.iter().find(|product| product.id == id).cloned());
        self.shade.wear(&thing.map(|thing| super::shop::worn(&thing.swatch)).unwrap_or_default());
    }

    /// The dollars in the wallet: Phiora's count or the folder's, and
    /// what a person starts with where neither has one.
    pub fn wallet(&self) -> f64 {
        let start = PTable::read(data::WALLET);
        let coin = start.get("pay");
        self.kept.get(WALLET)[coin].as_f64().unwrap_or_else(|| start.get(&format!("{COIN}{coin}")).parse().unwrap_or_default())
    }

    /// Pay for the bag from the wallet. `false`, and nothing is taken,
    /// when the bag is empty or the wallet holds less than all of it.
    pub fn pay(&self) -> bool {
        let (bag, start) = (self.bag(), PTable::read(data::WALLET));
        let total: f64 = bag.iter().map(|line| line.cost).sum();
        if bag.is_empty() || self.wallet() < total {
            return false;
        }
        let (coin, what) = (start.get("pay"), self.text.fill("shop.paid", "n", &bag.len().to_string()));
        // The wallet as it is, each coin, with the dollars taken.
        let mut wallet = self.kept.get(WALLET).as_object().cloned().unwrap_or_default();
        let coins = start.list().iter().filter_map(|(name, value)| Some((name.strip_prefix(COIN)?, value.parse::<f64>().unwrap_or_default())));
        coins.for_each(|(name, value)| _ = wallet.entry(name.to_string()).or_insert(json!(value)));
        wallet.insert(coin.to_string(), json!(self.wallet() - total));
        self.kept.set(WALLET, Value::Object(wallet));
        let mut history = self.kept.get(HISTORY).as_array().cloned().unwrap_or_default();
        history.insert(0, json!({ start.get("log.when"): hour::moment() as u64, start.get("log.coin"): coin, start.get("log.amount"): -total, start.get("log.what"): what }));
        self.kept.set(HISTORY, Value::Array(history));
        // Phiora moves the money itself, where it answers.
        let (look, rest): (Vec<&Line>, Vec<&Line>) = bag.iter().partition(|line| line.look);
        if !look.is_empty() {
            self.kept.ask(BUY, json!({ start.get("buy.look"): look.iter().map(|line| line.id.clone()).collect::<Vec<_>>(), start.get("buy.label"): what }));
        }
        if !rest.is_empty() {
            self.kept.ask(SPEND, json!({ start.get("spend.coin"): coin, start.get("spend.amount"): rest.iter().map(|line| line.cost).sum::<f64>(), start.get("spend.what"): what }));
        }
        let owned: Vec<Line> = bag.into_iter().chain(self.bought()).collect();
        self.keep_lines(OWNED, &owned);
        self.keep_lines(BAG, &[]);
        true
    }
}
