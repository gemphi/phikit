//! The tables a collection of things is read from: `thing.tsv` (every
//! thing's parts, a row each), `look.tsv` (the looks parts wear) and
//! `source.tsv` (the font and drawing files letters and outlines come
//! from, each path the folder's own); and how finely each form is made
//! (`data/thing.tsv`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::table::{TsvRow, TsvTable};

/// How a part is made (the `form` column).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThingForm {
    Box,
    Sphere,
    Capsule,
    Cylinder,
    Cone,
    Torus,
    Dome,
    Letter,
    Outline,
    Bloom,
    Thing,
}

impl ThingForm {
    /// The form a table word names.
    pub fn from_word(word: &str) -> Option<Self> {
        match word {
            "box" => Some(Self::Box),
            "sphere" => Some(Self::Sphere),
            "capsule" => Some(Self::Capsule),
            "cylinder" => Some(Self::Cylinder),
            "cone" => Some(Self::Cone),
            "torus" => Some(Self::Torus),
            "dome" => Some(Self::Dome),
            "letter" => Some(Self::Letter),
            "outline" => Some(Self::Outline),
            "bloom" => Some(Self::Bloom),
            "thing" => Some(Self::Thing),
            _ => None,
        }
    }
}

/// A look: its colour as it is seen (sRGB, 0-1), how rough, how metal.
#[derive(Clone, Debug, PartialEq)]
pub struct ThingLook {
    pub name: String,
    pub colour: [f32; 3],
    pub roughness: f32,
    pub metallic: f32,
}

impl ThingLook {
    /// Its colour as USD keeps it (linear).
    pub fn linear(&self) -> [f32; 3] {
        self.colour.map(|c| match c <= 0.04045 {
            true => c / 12.92,
            false => ((c + 0.055) / 1.055).powf(2.4),
        })
    }
}

/// A part of a thing: its name, form, size (metres along x, y and z; a
/// thing's is how many times its own size), where its middle stands,
/// how it is turned (degrees about x, then y, then z), how round its
/// edges are (a share of its smallest side; a letter's or an outline's
/// of its height; a bloom's of its thickness), its look (a thing's: the looks it swaps, `from=to`),
/// what it is made from (a letter: `font:character`; an outline: a
/// drawing; a bloom: its lobes, how deep, its hole; a turned form:
/// lobes and how deep, to scallop it; a thing: its name) and how often it repeats, each copy
/// moved and then turned a step on about the thing's middle.
#[derive(Clone, Debug, PartialEq)]
pub struct ThingRow {
    pub name: String,
    pub form: ThingForm,
    pub size: [f32; 3],
    pub at: [f32; 3],
    pub turn: [f32; 3],
    pub round: f32,
    pub look: String,
    pub from: String,
    pub repeat: usize,
    pub step_at: [f32; 3],
    pub step_turn: [f32; 3],
}

/// A thing: its name and parts, in table order.
#[derive(Clone, Debug, PartialEq)]
pub struct Thing {
    pub name: String,
    pub row: Vec<ThingRow>,
}

/// A collection: its things, its looks, its sources, and how finely
/// each form is made.
#[derive(Clone, Debug, Default)]
pub struct ThingSet {
    pub thing: Vec<Thing>,
    pub look: HashMap<String, ThingLook>,
    pub source: HashMap<String, PathBuf>,
    step: TsvTable,
}

impl ThingSet {
    /// The collection whose tables are in `folder`; a table missing is
    /// empty.
    pub fn read(folder: &Path) -> ThingSet {
        let table = |name: &str| TsvTable::parse(&std::fs::read_to_string(folder.join(name)).unwrap_or_default());
        let mut set = ThingSet { step: TsvTable::parse(include_str!("../../../data/thing.tsv")), ..ThingSet::default() };
        for row in table("thing.tsv").row() {
            let Some(part) = part(row) else { continue };
            let name = row.get("thing");
            match set.thing.iter_mut().find(|t| t.name == name) {
                Some(thing) => thing.row.push(part),
                None => set.thing.push(Thing { name: name.to_string(), row: vec![part] }),
            }
        }
        for row in table("look.tsv").row() {
            let finish = numbers(&row, "finish");
            let look = ThingLook { name: row.get("look").to_string(), colour: hex(row.get("colour")), roughness: finish.first().copied().unwrap_or(0.5), metallic: finish.get(1).copied().unwrap_or(0.0) };
            set.look.insert(look.name.clone(), look);
        }
        for row in table("source.tsv").row() {
            set.source.insert(row.get("source").to_string(), folder.join(row.get("file")));
        }
        set
    }

    /// The thing named `name`.
    pub fn named(&self, name: &str) -> Option<&Thing> {
        self.thing.iter().find(|t| t.name == name)
    }

    /// How finely a form is made: steps round, steps along, steps round
    /// a rounded edge (0 where the table says none).
    pub fn step(&self, form: &str) -> [usize; 3] {
        let row = self.step.row().find(|r| r.get("form") == form);
        ["segment", "ring", "bevel"].map(|c| row.map_or(0, |r| r.count(c)))
    }
}

/// A row as a part; none where its form is not known.
fn part(row: TsvRow<'_>) -> Option<ThingRow> {
    let form = ThingForm::from_word(row.get("form"))?;
    let size = match numbers(&row, "size").as_slice() {
        [one] => [*one; 3],
        [x, y, z, ..] => [*x, *y, *z],
        _ => [1.0; 3],
    };
    let repeat = numbers(&row, "repeat");
    let three = |v: &[f32], from: usize| [0, 1, 2].map(|i| v.get(from + i).copied().unwrap_or(0.0));
    Some(ThingRow {
        name: row.get("part").to_string(),
        form,
        size,
        at: three(&numbers(&row, "at"), 0),
        turn: three(&numbers(&row, "turn"), 0),
        round: row.number("round"),
        look: row.get("look").trim_matches('-').to_string(),
        from: row.get("from").trim_matches('-').to_string(),
        repeat: repeat.first().map_or(1, |n| (*n as usize).max(1)),
        step_at: three(&repeat, 1),
        step_turn: three(&repeat, 4),
    })
}

/// A cell's numbers; `-` and words that are not numbers are left out.
fn numbers(row: &TsvRow<'_>, column: &str) -> Vec<f32> {
    row.word_list(column).iter().filter_map(|w| w.parse().ok()).collect()
}

/// `#rrggbb` as 0-1 (grey where it cannot be read).
fn hex(text: &str) -> [f32; 3] {
    let value = u32::from_str_radix(text.trim_start_matches('#'), 16).unwrap_or(0x808080);
    [16, 8, 0].map(|shift| ((value >> shift) & 0xff) as f32 / 255.0)
}
