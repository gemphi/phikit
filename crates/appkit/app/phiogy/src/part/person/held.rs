//! A person's tabs as the kit's form `profile` (data/form.tsv), as
//! Phiegg's profile has them: what they are (info), what they posted,
//! Phiegg's records that are theirs (their PFTs, the items they own, the
//! drops of their clubs, their trail of sales), their figures (stats),
//! their clubs, spaces and links. Which tabs, what each holds and how it
//! is theirs is `data/profile.tsv`; the grids switch their layout and
//! filter and order as the kit's do. This file only answers what a list
//! holds and what a press opens.

use std::cell::RefCell;
use std::rc::Rc;

use appkit::pui::form::{PForm, PListed};
use appkit::pui::state::PState;
use serde_json::Value;

use crate::data;
use crate::part::Desk;
use crate::part::search::{said, values};
use crate::part::tool::grouped;

/// A row of words, as data/listing.tsv reads it.
type Row = Vec<String>;

/// The paths the form keeps: the tab on, the filter, the order, and the
/// one a press on a card is told.
const TAB: &str = "who/tab";
const FACET: &str = "who/kind";
const SORT: &str = "who/sort";
const OPEN: &str = "who/open";
/// What a list's name begins with, and a filter's ends with.
const LIST: &str = "who.";
const FACETS: &str = ".facet";

/// A tab of data/profile.tsv.
#[derive(Debug, Clone, Default)]
struct Tab {
    key: String,
    show: String,
    icon: String,
    kind: String,
    place: Vec<String>,
    sort: String,
    facet: String,
    name: String,
}

/// The rows of data/profile.tsv that begin with a word, without it.
fn table(word: &str) -> Vec<Vec<&'static str>> {
    data::row(data::PROFILE).into_iter().filter(|row| row.first() == Some(&word)).map(|row| row[1..].to_vec()).collect()
}

fn tabs() -> Vec<Tab> {
    table("tab").into_iter().map(|row| {
        let at = |i: usize| row.get(i).map(|cell| cell.to_string()).unwrap_or_default();
        Tab { key: at(0), show: at(1), icon: at(2), kind: at(3), place: at(4).split_whitespace().map(str::to_string).collect(), sort: at(5), facet: at(6), name: at(7) }
    }).collect()
}

/// Whether a record is somebody's by a tab's `where`: every `field=value`
/// holds, and any of the rest (a field holding their id, or one among
/// their own `field:own`).
fn theirs(tab: &Tab, row: &Value, id: &str, me: Option<&Value>) -> bool {
    let (must, any): (Vec<&String>, Vec<&String>) = tab.place.iter().partition(|term| term.contains('='));
    let held = |field: &str, wanted: &str| values(row.get(field)).iter().any(|one| one == wanted);
    must.iter().all(|term| term.split_once('=').is_some_and(|(field, value)| held(field, value)))
        && any.iter().any(|term| match term.split_once(':') {
            Some((field, own)) => values(me.and_then(|me| me.get(own))).iter().any(|value| held(field, value)),
            None => held(term, id),
        })
}

/// A card of a grid: what a press opens, its words, its picture or face,
/// what its filter and order read.
#[derive(Debug, Clone, Default)]
struct Card {
    open: String,
    title: String,
    picture: String,
    line: String,
    end: String,
    face: String,
    facet: String,
    rank: f64,
}

/// What a person's profile holds, read once for each person shown:
/// whose, every tab's cards, the facts and the figures.
#[derive(Default)]
struct Kept {
    id: String,
    cards: Vec<(String, Vec<Card>)>,
    facts: Vec<Row>,
    stats: Vec<Row>,
}

pub struct Held {
    desk: Desk,
    kept: RefCell<Kept>,
}

impl Held {
    fn who(&self) -> String {
        self.desk.now.who.borrow().clone().unwrap_or_default()
    }

    /// What the person shown holds, read again when another is shown.
    fn fresh(&self) {
        let id = self.who();
        if self.kept.borrow().id == id {
            return;
        }
        let cards = tabs().iter().map(|tab| (tab.key.clone(), self.read(tab, &id))).collect();
        *self.kept.borrow_mut() = Kept { id: id.clone(), cards, ..Kept::default() };
        let (facts, stats) = (self.facts(&id), self.stats(&id));
        let mut kept = self.kept.borrow_mut();
        (kept.facts, kept.stats) = (facts, stats);
    }

    /// A tab's cards.
    fn cards(&self, key: &str) -> Vec<Card> {
        self.fresh();
        self.kept.borrow().cards.iter().find(|(name, _)| name == key).map(|(_, cards)| cards.clone()).unwrap_or_default()
    }

    /// The facts, or the figures.
    fn kept(&self, stats: bool) -> Vec<Row> {
        self.fresh();
        let kept = self.kept.borrow();
        match stats {
            true => kept.stats.clone(),
            false => kept.facts.clone(),
        }
    }

    fn read(&self, tab: &Tab, id: &str) -> Vec<Card> {
        let (desk, text) = (&self.desk, &self.desk.text);
        let Some(one) = desk.site.member(id) else { return Vec::new() };
        let face = |cell: Option<&crate::site::Face>| cell.map(|cell| format!("{} {} {}", cell.sheet, cell.column, cell.index)).unwrap_or_default();
        match tab.show.as_str() {
            "picture" => desk.shot_of(id).into_iter().enumerate().map(|(n, shot)| Card { open: format!("shot:{n}"), title: shot.by, picture: shot.key, line: shot.when, ..Card::default() }).collect(),
            "kind" => {
                // Their id as the book has it: Phiora's key where it gave the records.
                let book = desk.book();
                let key = book.key(&own(), id);
                let me = book.record(&own(), &key).map(|found| &found.row);
                book.of(&tab.kind).iter().filter(|found| theirs(tab, &found.row, &key, me)).map(|found| {
                    // A nameless record (a sale) is named by what it is about.
                    let about = book.named(&said(found.row.get(&tab.name)));
                    // Never a bare key: what it was (a sale, an offer) where nothing names it.
                    let title = [found.name.clone(), about.map(|about| about.name.clone()).unwrap_or_default(), said(found.row.get(&tab.facet))].into_iter().find(|one| !one.is_empty()).unwrap_or_default();
                    let picture = [found.picture.clone(), about.map(|about| about.picture.clone()).unwrap_or_default()].into_iter().find(|one| !one.is_empty()).unwrap_or_default();
                    let facet = said(found.row.get(&tab.facet));
                    let end = said(found.row.get(&tab.sort));
                    let line = match found.line.is_empty() {
                        true => facet.clone(),
                        false => found.line.clone(),
                    };
                    Card { open: format!("rec:{}:{}", tab.kind, found.id), title, picture, line, end: end.split('T').next().unwrap_or_default().to_string(), face: found.face.clone(), rank: found.row.get(&tab.sort).and_then(Value::as_f64).unwrap_or_default(), facet }
                }).collect()
            }
            "club" => one.club.iter().map(|club| Card { open: format!("rec:club:{}", desk.book().key("club", &club.id)), title: club.name.clone(), line: text.get("person.holder").replace("{n}", &grouped(club.holder)), face: face(Some(&club.mark)), rank: club.holder as f64, ..Card::default() }).collect(),
            "space" => one.space.iter().filter_map(|space| desk.site.venue.iter().find(|venue| venue.label == *space)).map(|venue| Card {
                open: venue.plate.first().map(|plate| format!("room:{}", plate.id)).unwrap_or_default(),
                title: venue.label.clone(),
                line: text.get("person.inside").replace("{n}", &grouped(venue.count)),
                face: face(venue.mark.as_ref()),
                rank: venue.count as f64,
                ..Card::default()
            }).collect(),
            "linked" => desk.site.linked(id).iter().filter_map(|other| desk.site.member(other)).map(|other| Card { open: format!("who:{}", other.id), title: other.name.clone(), line: format!("@{}", other.handle), face: face(desk.site.face(&other.id)), ..Card::default() }).collect(),
            _ => Vec::new(),
        }
    }

    /// The facts of the info tab: their Phiegg record's fields, else their
    /// membership's; ids named by what they name.
    fn facts(&self, id: &str) -> Vec<Row> {
        let (book, text) = (self.desk.book(), &self.desk.text);
        let Some(one) = self.desk.site.member(id) else { return Vec::new() };
        let me = book.record(&own(), &book.key(&own(), id)).map(|found| &found.row);
        table("fact").into_iter().filter(|row| row.len() >= 2).filter_map(|row| {
            let field = me.and_then(|me| me.get(row[0]));
            let named: Vec<String> = values(field).iter().filter_map(|value| book.named(value).map(|found| found.name.clone())).collect();
            let value = match (named.is_empty(), said(field).is_empty()) {
                (false, _) => named.join(", "),
                (true, false) => said(field),
                (true, true) => match row[0] {
                    "handle" => one.handle.clone(),
                    "role" => one.role.clone(),
                    "state" => one.state.clone(),
                    "since" => one.since.clone(),
                    _ => String::new(),
                },
            };
            Some(vec![row[0].to_string(), text.get(row[1]).to_string(), value]).filter(|row| !row[2].is_empty())
        }).collect()
    }

    /// The figures of the stats tab: how much every other tab holds, then
    /// the sums data/profile.tsv names.
    fn stats(&self, id: &str) -> Vec<Row> {
        let (book, text) = (self.desk.book(), &self.desk.text);
        let every = tabs();
        let mut row: Vec<Row> = every.iter().filter(|tab| !matches!(tab.show.as_str(), "info" | "stat")).map(|tab| (tab, self.cards(&tab.key).len())).filter(|(_, n)| *n > 0)
            .map(|(tab, n)| vec![tab.key.clone(), text.get(&format!("person.{}", tab.key)).to_string(), grouped(n as u64)]).collect();
        for sum in table("sum").into_iter().filter(|sum| sum.len() >= 4) {
            let Some(tab) = every.iter().find(|tab| tab.key == sum[0]) else { continue };
            let key = book.key(&own(), id);
            let total: f64 = book.of(&tab.kind).iter().filter(|found| values(found.row.get(sum[1])).contains(&key)).filter_map(|found| found.row.get(sum[2])?.as_f64()).sum();
            if total > 0.0 {
                row.push(vec![sum[3].to_string(), text.get(sum[3]).to_string(), format!("{}{}", sum.get(4).copied().unwrap_or_default(), grouped(total.round() as u64))]);
            }
        }
        row
    }

    /// The tabs that hold something, and the one on.
    fn shown(&self) -> (Vec<Tab>, String) {
        let shown: Vec<Tab> = tabs().into_iter().filter(|tab| match tab.show.as_str() {
            "info" => !self.kept(false).is_empty(),
            "stat" => !self.kept(true).is_empty(),
            _ => !self.cards(&tab.key).is_empty(),
        }).collect();
        let kept = self.desk.shelf.get(TAB);
        let on = shown.iter().find(|tab| tab.key == kept).or(shown.first()).map(|tab| tab.key.clone()).unwrap_or_default();
        (shown, on)
    }

    fn rows(&self, name: &str) -> Vec<Row> {
        let text = &self.desk.text;
        let Some(name) = name.strip_prefix(LIST) else { return Vec::new() };
        match name {
            "tab" => {
                let (shown, _) = self.shown();
                shown.iter().map(|tab| {
                    let word = text.get(&format!("person.{}", tab.key));
                    let said = match tab.show.as_str() {
                        "info" | "stat" => word.to_string(),
                        _ => format!("{word} {}", grouped(self.cards(&tab.key).len() as u64)),
                    };
                    vec![tab.key.clone(), said, tab.icon.clone()]
                }).collect()
            }
            "sort" => data::row(data::FILTER).into_iter().filter(|row| row.first() == Some(&"who.sort")).map(|row| row[1..].iter().map(|cell| cell.to_string()).collect()).collect(),
            _ => {
                let (key, facet) = match name.strip_suffix(FACETS) {
                    Some(key) => (key, true),
                    None => (name, false),
                };
                if self.shown().1 != key {
                    return Vec::new();
                }
                let show = tabs().into_iter().find(|tab| tab.key == key).map(|tab| tab.show).unwrap_or_default();
                match (show.as_str(), facet) {
                    ("info", _) => self.kept(false),
                    ("stat", _) => self.kept(true),
                    (_, true) => self.facets(key),
                    (_, false) => self.grid(key),
                }
            }
        }
    }

    /// A tab's filter: all, then each value its records have, by name.
    fn facets(&self, key: &str) -> Vec<Row> {
        let book = self.desk.book();
        let mut seen: Vec<String> = Vec::new();
        self.cards(key).into_iter().map(|card| card.facet).filter(|facet| !facet.is_empty()).for_each(|facet| if !seen.contains(&facet) { seen.push(facet) });
        if seen.len() < 2 {
            return Vec::new();
        }
        let all = vec![String::new(), self.desk.text.get("filter.all").to_string(), "layers".to_string()];
        std::iter::once(all).chain(seen.into_iter().map(|value| vec![value.clone(), book.named(&value).map(|found| found.name.clone()).unwrap_or(value), String::new()])).collect()
    }

    /// A tab's cards as the grid's rows, filtered and in the order chosen.
    fn grid(&self, key: &str) -> Vec<Row> {
        let (facet, sort) = (self.desk.shelf.get(FACET), self.desk.shelf.get(SORT));
        let mut cards: Vec<Card> = self.cards(key).into_iter().filter(|card| facet.is_empty() || card.facet == facet).collect();
        match sort.as_str() {
            "name" => cards.sort_by_key(|card| card.title.to_lowercase()),
            _ => cards.sort_by(|a, b| b.rank.total_cmp(&a.rank).then_with(|| b.end.cmp(&a.end))),
        }
        // A card with neither picture nor face wears its tab's icon.
        let icon = tabs().into_iter().find(|tab| tab.key == key).map(|tab| tab.icon).unwrap_or_default();
        cards.into_iter().map(|card| {
            let mark = match card.picture.is_empty() && card.face.is_empty() {
                true => icon.clone(),
                false => String::new(),
            };
            vec![card.open, card.title, mark, card.picture, card.line, card.end, card.face]
        }).collect()
    }

    /// Open what a card names: a picture in the viewer, a record's page,
    /// a room, or another person.
    fn open(&self, what: &str) {
        let desk = &self.desk;
        match what.split_once(':') {
            Some(("shot", n)) => desk.look(desk.shot_of(&self.who()), n.parse().unwrap_or_default()),
            Some(("rec", rest)) => {
                let (kind, id) = rest.split_once(':').unwrap_or_default();
                desk.open_record(kind, id);
            }
            Some(("room", id)) => _ = desk.show(id),
            Some(("who", id)) => *desk.now.who.borrow_mut() = Some(id.to_string()),
            _ => {}
        }
    }
}

/// The kind of a person's own Phiegg record.
fn own() -> String {
    table("own").first().and_then(|row| row.first()).map(|kind| kind.to_string()).unwrap_or_default()
}

impl PState for Held {
    fn get(&self, path: &str) -> String {
        match path {
            TAB => self.shown().1,
            _ => self.desk.shelf.get(path),
        }
    }

    fn set(&self, path: &str, value: &str) {
        match path {
            OPEN => self.open(value),
            // Another tab begins with no filter.
            TAB => {
                self.desk.shelf.set(TAB, value);
                self.desk.shelf.set(FACET, "");
            }
            _ => self.desk.shelf.set(path, value),
        }
    }
}

/// The profile's form and lists, as the kit's tables write them, made
/// from data/profile.tsv's tabs: its tabs, then a block for each, facts
/// for info and stats, else a grid with its layout, its filter where its
/// records have one, and its order. One tab, one row of that table.
pub fn tables() -> (String, String) {
    let mut form = String::from("profile\ttabs\ttab\t-\twho.tab\twho/tab\t-\t-\n");
    let mut list = String::from("who.tab\twho.tab\t0\t1\t2\nwho.sort\twho.sort\t0\ttext:1\t2\n");
    for tab in tabs() {
        let key = &tab.key;
        match tab.show.as_str() {
            "info" | "stat" => {
                form += &format!("profile\tfacts\twho.{key}\tperson.{key}\twho.{key}\t-\ttab\t-\n");
                list += &format!("who.{key}\twho.{key}\t0\t1\t-\t-\t-\t-\t-\t-\t-\t2\n");
            }
            _ => {
                let filter = match tab.show == "kind" && !tab.facet.is_empty() {
                    true => format!(" drop=filter:who.{key}.facet:who/kind"),
                    false => String::new(),
                };
                form += &format!("profile\tgrid\twho.{key}\tperson.{key}\twho.{key}\twho/open\ttab\tlayout{filter} drop=sort:who.sort:who/sort\n");
                list += &format!("who.{key}\twho.{key}\t0\t1\t2\t-\t-\t-\t-\t3\t4\t5\t-\t6\nwho.{key}.facet\twho.{key}.facet\t0\t1\t2\n");
            }
        }
    }
    (form, list)
}

/// The profile's form, its lists and values from a person's records.
pub fn form(desk: &Desk) -> PForm {
    let held = Rc::new(Held { desk: desk.clone(), kept: RefCell::default() });
    let rows = held.clone();
    let state: Vec<(String, Rc<dyn PState>)> = vec![("who".to_string(), held), ("ui".to_string(), desk.shelf.clone())];
    let (form, list) = tables();
    let source = Rc::new(PListed::new(&format!("{}{list}", data::LISTING), Rc::new(move |name: &str| rows.rows(name)), desk.text.clone(), state));
    PForm::new("profile", &format!("{}{form}", data::FORM), source, desk.text.clone(), desk.shade.clone())
}

#[cfg(test)]
mod tests {
    use super::{own, tabs, theirs};
    use crate::kept::Kept;
    use crate::part::search::Book;

    #[test]
    fn phiegg_s_records_are_a_person_s_by_the_profile_table() {
        let kinds = Book::kinds();
        let book = Book::read(&Kept::none(), &kinds.iter().map(String::as_str).collect::<Vec<_>>());
        let count = |key: &str, id: &str| {
            let tab = tabs().into_iter().find(|tab| tab.key == key).expect("a tab of the table");
            let me = book.record(&own(), id).map(|found| &found.row);
            book.of(&tab.kind).iter().filter(|found| theirs(&tab, &found.row, id, me)).count()
        };
        // The items Ines owns, her trail of sales (to her or from her),
        // and the drops of Noor's clubs.
        assert!(count("item", "u_ines") > 0 && count("trail", "u_ines") >= 68, "{} {}", count("item", "u_ines"), count("trail", "u_ines"));
        assert!(count("drop", "u_noor") > 0);
        // Nobody's records are a stranger's.
        assert_eq!((count("item", "nobody"), count("trail", "nobody")), (0, 0));
        // Every tab the web's profile has: her shots, the hosts she owns or
        // is in, the club she founded.
        assert!(count("feed", "u_ines") >= 5 && count("host", "u_ines") > 0 && count("founded", "u_ines") > 0, "{} {} {}", count("feed", "u_ines"), count("host", "u_ines"), count("founded", "u_ines"));
    }

    #[test]
    fn records_wear_their_pictures_as_phiegg_draws_them() {
        let kinds = Book::kinds();
        let book = Book::read(&Kept::none(), &kinds.iter().map(String::as_str).collect::<Vec<_>>());
        // A PFT is a cell of its portrait sheet; a shot wears its look's.
        let pft = book.of("profilePft").iter().find(|found| found.row.get("holderId").and_then(|id| id.as_str()) == Some("u_ines")).expect("her PFT");
        assert_eq!(pft.face, "phiegg:portraits/sheet-05.webp 8 60");
        let shot = book.of("shot").iter().find(|found| found.row.get("authorId").and_then(|id| id.as_str()) == Some("u_ines")).expect("her shot");
        assert!(shot.picture.ends_with("FRONT.webp") || shot.picture.starts_with("phiegg:look/"), "{}", shot.picture);
    }

    #[test]
    fn the_profile_s_form_is_made_from_its_tabs_one_row_each() {
        let (form, list) = super::tables();
        assert!(tabs().iter().all(|tab| form.contains(&format!("\twho.{}\tperson.{}\t", tab.key, tab.key)) && list.contains(&format!("who.{}\twho.{}\t", tab.key, tab.key))));
        assert!(form.contains("drop=filter:who.feed.facet:who/kind") && !form.contains("drop=filter:who.linked.facet"));
    }
}
