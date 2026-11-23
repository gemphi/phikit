//! The greenhouse (AppState+Exploration.swift, the movement's plant
//! check): Apple's own scene read into the world, its planters found by
//! the plant component Reality Composer Pro authored on them, and a
//! planter the robot comes to propagated. When every plant is, the robot
//! celebrates. The rules are `game.tsv`.

use phiecs::comp::{Pose, adopt, place};
use phiecs::prelude::*;
use phiecs::usd::{ComponentSet, Stage, Value, intake};

use super::Data;
use super::machine::{CELEBRATE, Machine};

/// A planter: its word, its plant, and whether it was propagated.
#[derive(Debug, Clone, PartialEq)]
pub struct Planter {
    pub word: Word,
    pub plant: String,
    pub grown: bool,
}

/// The game in the greenhouse: its planters, and how many there are to
/// propagate.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Garden {
    pub planter: Vec<Planter>,
    pub total: usize,
}

impl Garden {
    /// How many plants were propagated.
    pub fn found(&self) -> usize {
        self.planter.iter().filter(|p| p.grown).count()
    }

    /// Whether every plant was.
    pub fn done(&self) -> bool {
        self.total > 0 && self.found() >= self.total
    }
}

/// The word the greenhouse stands under.
pub const GREENHOUSE: &str = "greenhouse";

/// Read the greenhouse into the world under its word, at its size, and
/// find its planters. `None` where its scene is not on this disk.
pub fn open(world: &mut World, data: &Data) -> Option<(Word, Garden)> {
    let stage = Stage::open(&data.asset().join(data.game.get("scene.greenhouse")))?;
    let root = world.coin_bare(GREENHOUSE);
    let s = data.game.num("explore.scale");
    world.bind(root, Pose { scale: [s; 3], ..Pose::default() });
    intake(world, &stage, Some(root));
    place(world);
    let (id, key) = (data.game.get("plant.component"), data.game.get("plant.type"));
    let planter = world.each::<ComponentSet>().filter_map(|(word, set)| set.find(id).map(|c| Planter { word, plant: c.get(key).map_or("", Value::text).to_string(), grown: false })).collect();
    Some((root, Garden { planter, total: data.game.num("plant.total") as usize }))
}

/// Put the robot in the greenhouse where the game begins.
pub fn enter(world: &mut World, robot: Word, root: Word, data: &Data) {
    let at = ["start.x", "start.y", "start.z"].map(|key| data.game.num(key));
    adopt(world, robot, root);
    world.bind(robot, Pose::at(at));
}

/// Where a thing stands on the ground of the world, and how far its
/// planter's box reaches from its middle there.
fn ground(world: &World, word: Word) -> [f32; 2] {
    let o = phiecs::comp::tree::matrix(world, word).origin();
    [o[0], o[2]]
}

fn reach(world: &World, planter: Word, data: &Data) -> f32 {
    let m = phiecs::comp::tree::matrix(world, planter).0;
    let size = (m[0][0] * m[0][0] + m[1][0] * m[1][0] + m[2][0] * m[2][0]).sqrt();
    let extent = world.read::<ComponentSet>(planter).and_then(|set| set.find("RealityKit.Collider")).and_then(|c| c.get("Shape.extent")).map(Value::numbers).unwrap_or_default();
    extent.first().copied().unwrap_or_default() / 2.0 * size + data.game.num("touch.reach")
}

/// Each step: a planter the robot has come to is propagated, once;
/// with every plant propagated, the robot celebrates.
pub fn run(world: &mut World, _dt: f32) {
    let (Some(data), Some(garden)) = (world.resources.get::<Data>().cloned(), world.resources.get::<Garden>().cloned()) else { return };
    let robot: Vec<Word> = world.each::<super::drive::Drive>().map(|(word, _)| word).collect();
    let mut garden = garden;
    for word in robot {
        let at = ground(world, word);
        for planter in garden.planter.iter_mut().filter(|p| !p.grown) {
            let there = ground(world, planter.word);
            let far = ((at[0] - there[0]).powi(2) + (at[1] - there[1]).powi(2)).sqrt();
            planter.grown = far <= reach(world, planter.word, &data);
        }
        if garden.done() {
            world.read_mut::<Machine>(word).into_iter().for_each(|m| _ = m.go(CELEBRATE, &data));
        }
    }
    world.resources.insert(garden);
}
