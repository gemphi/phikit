//! How the robot looks, and how it is built: a head, a body and a
//! backpack, each a shape from its USD scene (`scenes/<part><n>.usda`),
//! and the material, colour, light and eye shape the customiser chose.
//! The names and numbers are the tables `part`, `material`, `light`,
//! `face` and `default`.

use phiecs::comp::{Parent, Pose, adopt};
use phiecs::prelude::*;
use phiecs::usd::{Stage, intake};

use super::Data;

/// The customiser's choice for one part: its shape, its material and
/// that material's colour.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Choice {
    pub shape: String,
    pub material: String,
    pub colour: usize,
}

/// What the robot looks like: each part's choice, in the order of
/// `part.tsv`, its lights' colour and its eyes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Look {
    pub part: Vec<(String, Choice)>,
    pub light: usize,
    pub face: usize,
}

impl Aspect for Look {}

impl Look {
    /// The robot a session begins with.
    pub fn first(data: &Data) -> Look {
        let d = &data.default;
        let index = |table: &crate::table::Table, name: &str| table.row.iter().position(|row| row.get(1).is_some_and(|n| n == name)).unwrap_or_default();
        let part = data.part.keys().into_iter().map(|name| (name.to_string(), Choice { shape: d.get(name).to_string(), material: d.get("material").to_string(), colour: d.num("colour") as usize })).collect();
        Look { part, light: index(&data.light, d.get("light")), face: index(&data.face, d.get("face")) }
    }

    /// One part's choice.
    pub fn choice(&mut self, part: &str) -> Option<&mut Choice> {
        self.part.iter_mut().find(|(name, _)| name == part).map(|(_, choice)| choice)
    }

    /// The colour a part is drawn in, as the material table gives it.
    pub fn colour(&self, part: &str, data: &Data) -> [u8; 3] {
        let Some((_, choice)) = self.part.iter().find(|(name, _)| name == part) else { return [0; 3] };
        let row = data.material.rows(&choice.material).find(|row| row.get(1).and_then(|i| i.parse::<usize>().ok()) == Some(choice.colour));
        let cell = |i: usize| row.and_then(|row| row.get(i)).and_then(|v| v.parse().ok()).unwrap_or_default();
        [cell(3), cell(4), cell(5)]
    }

    /// The name of its light colour and of its eyes.
    pub fn light_name<'a>(&self, data: &'a Data) -> &'a str {
        data.light.row.get(self.light).and_then(|row| row.get(1)).map_or("", String::as_str)
    }

    pub fn face_name<'a>(&self, data: &'a Data) -> &'a str {
        data.face.row.get(self.face).and_then(|row| row.get(1)).map_or("", String::as_str)
    }
}

/// The word of the robot's model, which holds its parts.
pub const MODEL: &str = "robot/model";

/// Build the robot's parts under its model from their USD scenes: each
/// part a word (`robot/model/head`) holding its scene's prims, as the
/// sample's RobotProvider loads them. A part whose scene is not on this
/// disk is a word with no prims. The words of the parts.
pub fn build(world: &mut World, model: Word, look: &Look, data: &Data) -> Vec<Word> {
    look.part.iter().map(|(part, choice)| {
        let word = world.coin_bare(&format!("{MODEL}/{part}"));
        // A part chosen again is built anew: what it held goes.
        let held: Vec<Word> = world.each::<Parent>().filter(|(_, p)| p.0 == word).map(|(w, _)| w).collect();
        held.into_iter().for_each(|w| _ = world.unbind::<Parent>(w));
        world.bind(word, Pose::default());
        adopt(world, word, model);
        let scene = data.game.get("scene.part").replace("{part}{n}", &choice.shape);
        Stage::open(&data.asset().join(scene)).into_iter().for_each(|stage| _ = intake(world, &stage, Some(word)));
        word
    }).collect()
}
