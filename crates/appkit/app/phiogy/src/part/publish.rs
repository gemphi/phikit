//! Publishing a club, and a PFT of one of the person's clubs, on Phiora
//! from the window, as the kit's forms (`data/form.tsv`): the drawers
//! `newclub` and `newpft`, opened from the hosts' bar. What is made is what
//! `--make` makes (`src/make.rs`): a record on Phiora that Phiegg's web
//! reads, made or, where the same name was published before, changed. A
//! guest cannot publish, as Phiora says, so the drawer says so. This file
//! only says what each list holds and what a press does.
use std::rc::Rc;

use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};
use appkit::pui::form::{PForm, PListed};
use appkit::pui::state::PState;
use appkit::service::PTable;
use serde_json::{Map, Value, json};

use super::Desk;
use crate::data;
use crate::make::{self, Row};

/// The drawers, as `data/modal.tsv` names them.
pub const CLUB: &str = "newclub";
pub const PFT: &str = "newpft";
/// What Phiora says of a publish, kept where the drawer reads it, and
/// the clubs the person has, as a list of rows.
const SAID: &str = "said";
const CLUBS: &str = "newpft/clubs";

type Item = Vec<String>;

/// What a name is called as a ref: letters and digits, the rest one `_`.
fn slug(name: &str) -> String {
    let mut out = String::new();
    for one in name.trim().to_lowercase().chars() {
        match one.is_alphanumeric() {
            true => out.push(one),
            false if !out.ends_with('_') && !out.is_empty() => out.push('_'),
            false => {}
        }
    }
    out.trim_end_matches('_').to_string()
}

/// What the publish drawers hold, and what a press on them does.
struct Publish {
    desk: Desk,
}

impl Publish {
    fn get(&self, path: &str) -> String {
        self.desk.shelf.get(path)
    }

    fn say(&self, kind: &str, words: &str) {
        self.desk.shelf.set(&format!("{kind}/{SAID}"), words);
    }

    /// The clubs the person made on Phiora, as `[id, name, icon, ticker]`
    /// rows, asked of Phiora and kept for the drawer to read.
    fn ask(&self) {
        let (kept, word) = (&self.desk.kept, PTable::read(data::MAKE));
        let me = kept.me();
        let mine = me[kept.word("me.id")].as_str().unwrap_or_default().to_string();
        let tree = kept.read(&word.fill("list", "kind", "club")).and_then(|text| serde_json::from_str::<Value>(&text).ok());
        let rows = tree.iter().filter_map(|tree| tree[word.get("answer.sources")].as_array()).flatten().filter(|source| source[word.get("answer.kind")] == "club").filter_map(|source| source[word.get("answer.rows")].as_array()).flatten();
        let text = |row: &Value, name: &str| row[name].as_str().unwrap_or_default().to_string();
        let own: Vec<Item> = rows.filter(|row| !mine.is_empty() && row["ownerId"] == mine.as_str()).map(|row| vec![text(row, word.get("id")), text(row, "name"), self.desk.icon("users").to_string(), text(row, "ticker")]).collect();
        self.desk.shelf.set(CLUBS, &json!(own).to_string());
    }

    fn clubs(&self) -> Vec<Item> {
        serde_json::from_str(&self.get(CLUBS)).unwrap_or_default()
    }

    fn rows(&self, name: &str) -> Vec<Item> {
        let say = |kind: &str| Some(self.get(&format!("{kind}/{SAID}"))).filter(|said| !said.is_empty()).map(|said| vec![vec!["say".to_string(), said]]).unwrap_or_default();
        match name {
            "newclub.say" => say(CLUB),
            "newpft.say" => say(PFT),
            "newpft.club" => self.clubs(),
            _ => data::row(data::FILTER).into_iter().filter(|row| row.first() == Some(&name)).map(|row| row[1..].iter().map(|cell| cell.to_string()).collect()).collect(),
        }
    }

    /// The record a drawer asks for, or what is wrong with what was
    /// written.
    fn row(&self, kind: &str) -> Result<Row, String> {
        let text = &self.desk.text;
        let name = self.get(&format!("{kind}/name")).trim().to_string();
        if name.is_empty() {
            return Err(text.get("publish.name").to_string());
        }
        let mut fields = Map::new();
        fields.insert("name".to_string(), json!(name));
        let reference = match kind {
            CLUB => {
                for field in ["ticker", "tagline"] {
                    Some(self.get(&format!("newclub/{field}")).trim().to_string()).filter(|value| !value.is_empty()).into_iter().for_each(|value| _ = fields.insert(field.to_string(), json!(value)));
                }
                let supply = self.get("newclub/supply").trim().to_string();
                if !supply.is_empty() {
                    let number = supply.parse::<u64>().map_err(|_| text.get("publish.number").to_string())?;
                    fields.insert("supply".to_string(), json!(number));
                }
                format!("club_{}", slug(&name))
            }
            _ => {
                let number = self.get("newpft/number").trim().parse::<u64>().map_err(|_| text.get("publish.number").to_string())?;
                let club = self.clubs().into_iter().find(|club| club[0] == self.get("newpft/club")).ok_or_else(|| text.get("publish.club").to_string())?;
                let tag = format!("{}_{number:04}", slug(&club[1]));
                fields.insert("n".to_string(), json!(number));
                fields.insert("clubId".to_string(), json!(club[0]));
                fields.insert("set".to_string(), json!(club[1]));
                fields.insert("tokenId".to_string(), json!(tag.replace('_', "-")));
                format!("pft_{tag}")
            }
        };
        fields.insert("ref".to_string(), json!(reference));
        Ok(Row { line: 0, kind: if kind == CLUB { "club" } else { "profilePft" }.to_string(), name: reference, fields })
    }

    /// Publish what the drawer holds, and say what Phiora did: its own
    /// words when it refused.
    fn publish(&self, kind: &str) {
        let word = PTable::read(data::MAKE);
        let said = match self.row(kind) {
            Err(why) => why,
            Ok(row) => match self.desk.kept.records().and_then(|records| records.set(&row.kind, &row.name, Value::Object(row.fields.clone()))) {
                Err(why) => why,
                Ok(set) => set.said(&row.kind, &row.name),
            },
        };
        self.say(kind, &said);
        // A club that was just made is one to put a PFT in.
        self.ask();
    }
}

impl PState for Publish {
    fn get(&self, path: &str) -> String {
        self.desk.shelf.get(path)
    }

    fn set(&self, path: &str, value: &str) {
        match path {
            "newclub/do" => self.publish(CLUB),
            "newpft/do" => self.publish(PFT),
            _ => self.desk.shelf.set(path, value),
        }
    }
}

/// Publish what a drawer would hold, as pressing its Publish does, and what
/// it says after. For the command line (--fill, --publish).
pub fn publish_from(desk: &Desk, kind: &str) -> String {
    let publish = Publish { desk: desk.clone() };
    publish.ask();
    publish.publish(kind);
    publish.get(&format!("{kind}/{SAID}"))
}

/// A drawer to publish in: the club's or the PFT's.
pub struct Publishing {
    form: PForm,
}

impl Publishing {
    pub fn new(name: &str, desk: Desk) -> Publishing {
        let publish = Rc::new(Publish { desk: desk.clone() });
        // The clubs to choose from are asked of Phiora when the drawer is made.
        publish.ask();
        let rows = publish.clone();
        let state: Vec<(String, Rc<dyn PState>)> = vec![(CLUB.to_string(), publish.clone()), (PFT.to_string(), publish), ("ui".to_string(), desk.shelf.clone())];
        let source = Rc::new(PListed::new(data::LISTING, Rc::new(move |list: &str| rows.rows(list)), desk.text.clone(), state));
        Publishing { form: PForm::new(name, data::FORM, source, desk.text.clone(), desk.shade.clone()) }
    }
}

impl PDraw for Publishing {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        self.form.draw(paint, origin);
    }
}

impl PView for Publishing {
    fn frame(&self) -> PRect {
        self.form.frame()
    }
    fn set_frame(&mut self, frame: PRect) {
        self.form.set_frame(frame);
    }
    fn respond(&mut self, event: &PEvent) -> bool {
        self.form.respond(event)
    }
    fn tick(&mut self) -> bool {
        self.form.tick()
    }
    fn hand(&self) -> Vec<PRect> {
        self.form.hand()
    }
}
