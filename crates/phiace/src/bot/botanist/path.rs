//! The session by path: what anything outside the world asks of it and
//! tells it, as text, with no knowledge of its types. A window's settings
//! read and write these and nothing else.
//!
//! * `look/<part>/shape`, `look/<part>/material`, `look/<part>/colour`:
//!   a part's choice (a shape's name, a material's name, a colour's
//!   number in its material).
//! * `look/face`, `look/light`: the eyes' and the lights' numbers.
//! * `do`: `reset` (the robot the tables begin with) or `random` (every
//!   choice at random, the sample's Randomize).

use super::look::Look;
use super::{Botanist, Data};

impl Botanist {
    /// What is kept at a path; nothing for a path with nothing.
    pub fn get(&self, path: &str) -> String {
        let look = self.look();
        let step: Vec<&str> = path.split('/').collect();
        let choice = |part: &str| look.part.iter().find(|(name, _)| name == part).map(|(_, c)| c.clone()).unwrap_or_default();
        match step.as_slice() {
            ["look", "face"] => look.face.to_string(),
            ["look", "light"] => look.light.to_string(),
            ["look", part, "shape"] => choice(part).shape,
            ["look", part, "material"] => choice(part).material,
            ["look", part, "colour"] => choice(part).colour.to_string(),
            _ => String::new(),
        }
    }

    /// Keep something at a path, and build the robot again.
    pub fn set(&mut self, path: &str, value: &str) {
        let step: Vec<&str> = path.split('/').collect();
        let number = value.parse::<usize>().unwrap_or_default();
        match (step.as_slice(), value) {
            (["do"], "reset") => self.reset(),
            (["do"], "random") => self.change(random),
            (["look", "face"], _) => self.change(|look, _| look.face = number),
            (["look", "light"], _) => self.change(|look, _| look.light = number),
            (["look", part, field], _) => {
                let (part, field) = (part.to_string(), field.to_string());
                self.change(|look, _| {
                    match (field.as_str(), look.choice(&part)) {
                        ("shape", Some(c)) => c.shape = value.to_string(),
                        ("material", Some(c)) => c.material = value.to_string(),
                        ("colour", Some(c)) => c.colour = number,
                        _ => {}
                    }
                })
            }
            _ => {}
        }
    }
}

/// Every choice at random, from the clock: each part's shape from the
/// shape table, its material and colour from the material table, and
/// the eyes and the lights from theirs.
fn random(look: &mut Look, data: &Data) {
    let mut seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(7, |d| d.subsec_nanos() as u64 | 1);
    let mut roll = |n: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % n.max(1) as u64) as usize
    };
    let mut material: Vec<&str> = data.material.keys();
    material.dedup();
    for (name, choice) in look.part.iter_mut() {
        let shape: Vec<&Vec<String>> = data.shape.rows(name).collect();
        choice.shape = shape.get(roll(shape.len())).and_then(|row| row.get(1)).cloned().unwrap_or_default();
        choice.material = material.get(roll(material.len())).map(|m| m.to_string()).unwrap_or_default();
        choice.colour = roll(data.material.rows(&choice.material).count());
    }
    (look.face, look.light) = (roll(data.face.row.len()), roll(data.light.row.len()));
}
