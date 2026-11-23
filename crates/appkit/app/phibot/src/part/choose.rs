//! What the customiser (the kit's form `choose`, `data/form.tsv`) asks of
//! the botanist: each list's choices, from phiace's tables, and each
//! path's value, from the robot's look. No drawing: the form is the
//! kit's.
//!
//! The lists: `tool` (Reset, Randomize), `part` (Head, Body, Backpack),
//! and for the part chosen `shape`, `face` (the head's only), `material`,
//! `colour`; and `light`, the robot's own. The paths are the same words,
//! and `do` for a tool.

use appkit::pui::form::{PChoice, PSource};
use appkit::pui::state::PState;
use phiace::bot::botanist::Data;
use phiace::bot::botanist::look::Look;

use super::{Desk, byte};

/// The botanist, as the customiser's source.
pub struct Choose(pub Desk);

impl Choose {
    /// The part chosen, by its name in phiace's part table.
    fn part(&self) -> String {
        let data = self.0.data();
        data.part.row.get(self.0.chosen.get()).and_then(|row| row.first()).cloned().unwrap_or_default()
    }
}

/// Choices from values and words alike.
fn plain(value: impl IntoIterator<Item = String>) -> Vec<PChoice> {
    value.into_iter().map(|v| PChoice { label: v.clone(), value: v, ..PChoice::default() }).collect()
}

impl PSource for Choose {
    fn list(&self, name: &str) -> Vec<PChoice> {
        let (data, look, part) = (self.0.data(), self.0.bot.borrow().look(), self.part());
        let choice = look.part.iter().find(|(n, _)| *n == part).map(|(_, c)| c.clone()).unwrap_or_default();
        let count = data.part.rows(&part).next().and_then(|row| row.get(2)).and_then(|n| n.parse::<usize>().ok()).unwrap_or_default();
        let cell = |row: &Vec<String>, i: usize| row.get(i).cloned().unwrap_or_default();
        match name {
            "tool" => crate::data::TOOL.lines().filter(|l| !l.starts_with('#')).filter_map(|l| l.split_once('\t')).map(|(key, icon)| PChoice { value: key.to_string(), label: self.0.text.get(key).to_string(), icon: icon.to_string(), colour: None }).collect(),
            "part" => data.part.row.iter().enumerate().map(|(i, row)| PChoice { value: i.to_string(), label: cell(row, 3), ..PChoice::default() }).collect(),
            "shape" => plain((1..=count).map(|n| format!("{part}{n}"))),
            "face" => match part.as_str() {
                "head" => data.face.row.iter().enumerate().map(|(i, row)| PChoice { value: i.to_string(), icon: cell(row, 1), ..PChoice::default() }).collect(),
                _ => Vec::new(),
            },
            "material" => plain(materials(&data)),
            "colour" => data.material.rows(&choice.material).enumerate().map(|(i, row)| PChoice { value: i.to_string(), colour: Some(byte([3, 4, 5].map(|c| row.get(c).and_then(|v| v.parse().ok()).unwrap_or_default()))), ..PChoice::default() }).collect(),
            "light" => (0..data.light.row.len()).map(|i| PChoice { value: i.to_string(), colour: Some(super::light(&data, i)), ..PChoice::default() }).collect(),
            _ => Vec::new(),
        }
    }
}

impl PState for Choose {
    fn get(&self, path: &str) -> String {
        let (look, part) = (self.0.bot.borrow().look(), self.part());
        let choice = look.part.iter().find(|(n, _)| *n == part).map(|(_, c)| c.clone()).unwrap_or_default();
        match path {
            "part" => self.0.chosen.get().to_string(),
            "shape" => choice.shape,
            "material" => choice.material,
            "colour" => choice.colour.to_string(),
            "face" => look.face.to_string(),
            "light" => look.light.to_string(),
            _ => String::new(),
        }
    }

    fn set(&self, path: &str, value: &str) {
        let (part, number) = (self.part(), value.parse::<usize>().unwrap_or_default());
        match (path, value) {
            ("part", _) => self.0.chosen.set(number),
            ("do", "reset") => self.0.bot.borrow_mut().reset(),
            ("do", "random") => self.0.bot.borrow_mut().change(random),
            (path, value) => self.0.bot.borrow_mut().change(|look, _| {
                match (path, look.choice(&part)) {
                    ("shape", Some(c)) => c.shape = value.to_string(),
                    ("material", Some(c)) => c.material = value.to_string(),
                    ("colour", Some(c)) => c.colour = number,
                    ("face", _) => look.face = number,
                    ("light", _) => look.light = number,
                    _ => {}
                }
            }),
        }
    }
}

/// The materials, in the order the material table first names each.
fn materials(data: &Data) -> Vec<String> {
    let mut out: Vec<String> = data.material.keys().into_iter().map(str::to_string).collect();
    out.dedup();
    out
}

/// Every choice at random (the sample's Randomize), from the clock.
fn random(look: &mut Look, data: &Data) {
    let mut seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(7, |d| d.subsec_nanos() as u64 | 1);
    let mut roll = |n: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % n.max(1) as u64) as usize
    };
    let material = materials(data);
    for (name, choice) in look.part.iter_mut() {
        let count = data.part.rows(name).next().and_then(|r| r.get(2)).and_then(|n| n.parse::<usize>().ok()).unwrap_or(1);
        choice.shape = format!("{name}{}", roll(count) + 1);
        choice.material = material.get(roll(material.len())).cloned().unwrap_or_default();
        choice.colour = roll(4);
    }
    (look.face, look.light) = (roll(data.face.row.len()), roll(data.light.row.len()));
}
