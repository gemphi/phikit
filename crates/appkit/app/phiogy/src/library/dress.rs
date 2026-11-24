//! How a person dressed a model: a colour for each part and a finish,
//! kept with what they chose (`look/<item>/<part>` and
//! `look/<item>/finish`), so it is dressed so again when Phiogy opens.
//! The colours are `data/paint.tsv`, the finishes `data/finish.tsv`.

use std::rc::Rc;

use appkit::pui::core::PSun;
use appkit::pui::state::PState;
use appkit::service::PTable;

use crate::choice::Choice;
use crate::data;

/// What is kept of the looks, under the person's choices.
pub struct Dress {
    choice: Rc<Choice>,
}

impl Dress {
    pub fn new(choice: Rc<Choice>) -> Dress {
        Dress { choice }
    }

    /// The colour chosen for a part of a model; nothing where none was.
    pub fn colour(&self, item: &str, part: usize) -> Option<[f32; 3]> {
        let chosen = self.get(&format!("look/{item}/{part}"));
        let row = data::row(data::PAINT).into_iter().find(|cell| cell.first() == Some(&chosen.as_str()))?;
        let value: Vec<f32> = row.iter().skip(2).filter_map(|v| v.parse::<f32>().ok()).map(|v| v / 255.0).collect();
        (value.len() == 3).then(|| [value[0], value[1], value[2]])
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

    /// What dresses it, as one word: it changes when the dress does.
    pub fn mark(&self, item: &str, parts: usize) -> String {
        let colour: Vec<String> = (0..parts).map(|part| self.get(&format!("look/{item}/{part}"))).collect();
        format!("{}|{}", colour.join(","), self.get(&format!("look/{item}/finish")))
    }
}

impl PState for Dress {
    fn get(&self, path: &str) -> String {
        self.choice.get(path)
    }

    fn set(&self, path: &str, value: &str) {
        self.choice.put(path, value);
    }
}
