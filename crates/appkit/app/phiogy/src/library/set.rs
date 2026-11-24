//! A set made from a model, as a 10,000-piece collection is made from
//! its traits: as many variants as the set asks for, each part of the
//! model in a colour drawn by its weight (`data/paint.tsv`: a rarer
//! colour has less), seeded by the set's name and the variant's number,
//! so a set is the same every time it is made. A set is kept with what
//! the person chose: `set/<name>` holds how many and the model's key.

use crate::data;

/// A set: its name, how many variants, and the key of its model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Set {
    pub name: String,
    pub count: usize,
    pub base: String,
}

impl Set {
    /// A set as it is kept: its name, and `count key`.
    pub fn kept(name: &str, value: &str) -> Option<Set> {
        let (count, base) = value.split_once(' ')?;
        Some(Set { name: name.to_string(), count: count.parse().ok()?, base: base.to_string() })
    }

    /// What it is kept as.
    pub fn value(&self) -> String {
        format!("{} {}", self.count, self.base)
    }

    /// The colour each part of a variant takes, by its key in the table.
    pub fn variant(&self, index: usize, parts: usize) -> Vec<String> {
        let paint: Vec<(String, u64)> = data::row(data::PAINT).into_iter().filter_map(|cell| Some((cell.first()?.to_string(), cell.get(5)?.parse().ok()?))).collect();
        let total: u64 = paint.iter().map(|(_, weight)| weight).sum::<u64>().max(1);
        (0..parts).map(|part| {
            let mut roll = mark(&format!("{}/{index}/{part}", self.name)) % total;
            let pick = paint.iter().find(|(_, weight)| match roll < *weight {
                true => true,
                false => {
                    roll -= weight;
                    false
                }
            });
            pick.map(|(key, _)| key.clone()).unwrap_or_default()
        }).collect()
    }
}

/// A word's mark: the FNV hash the data steps deal by.
fn mark(text: &str) -> u64 {
    text.bytes().fold(0xcbf29ce484222325, |hash, byte| (hash ^ byte as u64).wrapping_mul(0x100000001b3))
}
