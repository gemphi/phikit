//! An item's attributes as a person sets them: a colour and a material
//! (`look/<item>/<part>/material`) for each part of its model and a finish, kept with what they chose (`look/<item>/<part>`
//! and `look/<item>/finish`), so it is so again when Phiogy opens; and
//! the comments they wrote on it (`note/<item>`). The colours are
//! `data/paint.tsv`, the finishes `data/finish.tsv`.

use std::rc::Rc;

use appkit::pui::core::PSun;
use appkit::pui::state::PState;
use appkit::service::PTable;

use crate::choice::Choice;
use crate::data;

/// What is kept of the items' attributes, under the person's choices.
pub struct Attribute {
    choice: Rc<Choice>,
}

impl Attribute {
    pub fn new(choice: Rc<Choice>) -> Attribute {
        Attribute { choice }
    }

    /// The colour chosen for a part of a model; nothing where none was.
    pub fn colour(&self, item: &str, part: usize) -> Option<[f32; 3]> {
        paint(&self.get(&format!("look/{item}/{part}")))
    }

    /// The material chosen for a part of a model, by its name; empty
    /// where none was.
    pub fn material(&self, item: &str, part: usize) -> String {
        self.get(&format!("look/{item}/{part}/material"))
    }

    /// The light a model is shown in: the table's, shining back as its
    /// finish does (the one chosen, or the table's own).
    pub fn sun(&self, item: &str, set: &PTable) -> PSun {
        let three = |key: &str| super::three(set, key);
        let chosen = self.get(&format!("look/{item}/finish"));
        let finish = match chosen.is_empty() {
            true => set.get("finish").to_string(),
            false => chosen,
        };
        let row = data::row(data::FINISH).into_iter().find(|cell| cell.first() == Some(&finish.as_str())).unwrap_or_default();
        let number = |at: usize| row.get(at).and_then(|v| v.parse().ok()).unwrap_or_default();
        PSun { way: three("sun.way"), key: set.get("sun.key").parse().unwrap_or(1.0), sky: three("light.sky"), ground: three("light.ground"), shine: number(2), gloss: number(3) }
    }

    /// The comments written on an item, the first first.
    pub fn note(&self, item: &str) -> Vec<String> {
        (0..).map(|at| self.choice.get(&format!("note/{item}/{at}"))).take_while(|said| !said.is_empty()).collect()
    }

    /// What sets it, as one word: it changes when its attributes do.
    pub fn mark(&self, item: &str, parts: usize) -> String {
        let tuned = |part: usize| super::tune::control_names().iter().chain(["second".to_string()].iter()).map(|name| self.get(&format!("look/{item}/{part}/{name}"))).collect::<Vec<_>>().join(";");
        let colour: Vec<String> = (0..parts).map(|part| format!("{}:{}:{}", self.get(&format!("look/{item}/{part}")), self.material(item, part), tuned(part))).collect();
        format!("{}|{}", colour.join(","), self.get(&format!("look/{item}/finish")))
    }
}

impl PState for Attribute {
    fn get(&self, path: &str) -> String {
        self.choice.get(path)
    }

    /// A comment (`note/<item>`) is added after the others; anything
    /// else is kept at its path.
    fn set(&self, path: &str, value: &str) {
        match path.strip_prefix("note/") {
            Some(item) => {
                let at = self.note(item).len();
                self.choice.put(&format!("note/{item}/{at}"), value);
            }
            None => self.choice.put(path, value),
        }
    }
}

/// A colour of `data/paint.tsv` by its key, 0 to 1.
pub fn paint(key: &str) -> Option<[f32; 3]> {
    // Any colour from the wheel, as `#rrggbb`.
    if let Some(colour) = appkit::pui::block::wheel::read(key) {
        return Some([colour.r, colour.g, colour.b]);
    }
    let row = data::row(data::PAINT).into_iter().find(|cell| cell.first() == Some(&key))?;
    let value: Vec<f32> = row.iter().skip(2).take(3).filter_map(|v| v.parse::<f32>().ok()).map(|v| v / 255.0).collect();
    (value.len() == 3).then(|| [value[0], value[1], value[2]])
}
