//! What is the person's own in the account: their wallet, each coin
//! and all of it in dollars; the sessions signed in as them; and a
//! copy of everything Phiora keeps about them.

use appkit::pui::window::PDeed;
use appkit::service::PTable;

use super::{Account, Act, LABEL, RATE};
use crate::data;
use crate::part::list::{Deed, Item, Seat, grouped};
use crate::part::store::{COIN, WALLET};

impl Account {
    /// The wallet: what all of it is worth, and each coin, every sum
    /// on a ground of its own.
    pub(super) fn wallet(&self) -> Vec<Item<Act>> {
        let (text, start) = (&self.desk.text, PTable::read(data::WALLET));
        let held = self.desk.kept.get(WALLET);
        let coin: Vec<(String, f64)> = start.list().iter().filter_map(|(name, value)| {
            let coin = name.strip_prefix(COIN)?;
            Some((coin.to_string(), held[coin].as_f64().unwrap_or_else(|| value.parse().unwrap_or_default())))
        }).collect();
        let worth: f64 = coin.iter().map(|(coin, amount)| amount * start.get(&format!("{RATE}{coin}")).parse::<f64>().unwrap_or_default()).sum();
        let whole = Item::Sum(text.get("account.total").to_string(), format!("${}", grouped(worth.round().max(0.0) as u64)));
        let line = |(coin, amount): &(String, f64)| {
            let said = match amount.fract() == 0.0 {
                true => grouped(*amount as u64),
                false => format!("{amount}"),
            };
            Item::Sum(start.get(&format!("{LABEL}{coin}")).to_string(), said)
        };
        std::iter::once(whole).chain(coin.iter().map(line)).collect()
    }

    /// For somebody signed in: the sessions signed in as them, with
    /// the way to sign the others out; and taking their data out.
    pub(super) fn signed(&self) -> Vec<Item<Act>> {
        let text = &self.desk.text;
        let word = |name: &str| text.get(name).to_string();
        let session = self.desk.kept.session();
        let line = |(on, this): &(String, bool)| {
            let under = match this {
                true => word("account.this"),
                false => word("account.other"),
            };
            Item::Line(Seat::None, on.clone(), under, None, Vec::new())
        };
        let mut item = vec![Item::Head(word("account.session"))];
        item.extend(session.iter().map(line));
        let alone = Some(Deed::word(text.get("account.alone"), false, Act::Alone)).filter(|_| session.len() > 1);
        item.extend(alone.map(|alone| Item::Line(Seat::None, String::new(), String::new(), None, vec![alone])));
        item.push(Item::Line(Seat::None, word("account.data"), String::new(), None, vec![Deed::word(text.get("account.export"), false, Act::Export)]));
        item
    }

    /// Write what Phiora keeps about the person to a file of the
    /// system's own folder for such, and ask where a copy is to be kept.
    pub(super) fn export(&self) {
        let Some(all) = self.desk.kept.export() else { return };
        let file = std::env::temp_dir().join(self.desk.kept.word("export.file"));
        if std::fs::write(&file, all).is_ok() {
            self.desk.wish.borrow_mut().push(PDeed::Save(file.to_string_lossy().to_string()));
        }
    }
}
