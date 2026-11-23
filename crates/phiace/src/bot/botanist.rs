//! The botanist: Apple's BOT-anist sample played out on phiecs. A robot
//! is built from a head, a body and a backpack, each its USD scene; the
//! customiser chooses their shapes, materials, colours, lights and eyes;
//! then it walks a greenhouse read from Apple's own USD scene, and a
//! planter it comes to is propagated. Every name and number is a table
//! under `data/botanist`; the scenes are Apple's, where `game.tsv` says
//! they are. The screens that show it are the app's; this is the world
//! they show.
//!
//! The phases are the sample's (AppState.swift:15-22): the robot is
//! made (`playing`), then explores (`exploration`); Replay begins the
//! greenhouse again and Exit goes back to making.

pub mod drive;
pub mod garden;
pub mod look;
pub mod machine;

use std::path::PathBuf;

use phiecs::comp::{Parent, Pose, adopt, place};
use phiecs::prelude::*;

use crate::table::Table;
use garden::{Garden, enter, open};
use look::{Look, MODEL, build};
use machine::Machine;

/// The botanist's tables, read once.
#[derive(Debug, Clone, Default)]
pub struct Data {
    pub part: Table,
    pub material: Table,
    pub light: Table,
    pub face: Table,
    pub default: Table,
    pub state: Table,
    pub transition: Table,
    pub key: Table,
    pub game: Table,
    /// The folder the tables are in.
    pub dir: PathBuf,
}

impl Data {
    /// The tables in `data/botanist` of this crate.
    pub fn read() -> Data {
        let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "data", "botanist"].iter().collect();
        let table = |name: &str| Table::read(&dir.join(format!("{name}.tsv")));
        Data { part: table("part"), material: table("material"), light: table("light"), face: table("face"), default: table("default"), state: table("state"), transition: table("transition"), key: table("key"), game: table("game"), dir }
    }

    /// Where Apple's assets are.
    pub fn asset(&self) -> PathBuf {
        [env!("CARGO_MANIFEST_DIR"), self.game.get("asset")].iter().collect()
    }
}

/// Where the session is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// The robot is made.
    Playing,
    /// The robot walks the greenhouse.
    Exploration,
}

/// A session: the world, its robot and where it is.
pub struct Botanist {
    pub world: World,
    pub robot: Word,
    pub phase: Phase,
}

impl Botanist {
    /// A session with the robot the tables begin with, being made.
    pub fn new(data: Data) -> Botanist {
        let mut world = World::new();
        let robot = world.coin_bare("robot");
        let model = world.coin_bare(MODEL);
        let look = Look::first(&data);
        let s = data.game.num("model.scale");
        world.bind(robot, Pose::default());
        world.bind(model, Pose { translate: [0.0, -s, 0.0], scale: [s; 3], ..Pose::default() });
        adopt(&mut world, model, robot);
        build(&mut world, model, &look, &data);
        world.bind(robot, look);
        world.bind(robot, Machine::default());
        world.resources.insert(data);
        Botanist { world, robot, phase: Phase::Playing }
    }

    pub fn data(&self) -> &Data {
        self.world.resources.get::<Data>().expect("the tables are put in when the session begins")
    }

    pub fn look(&self) -> Look {
        self.world.read::<Look>(self.robot).cloned().unwrap_or_default()
    }

    /// Change the look and build the robot again.
    pub fn change(&mut self, edit: impl FnOnce(&mut Look, &Data)) {
        let data = self.data().clone();
        let mut look = self.look();
        edit(&mut look, &data);
        let model = self.world.coin_bare(MODEL);
        build(&mut self.world, model, &look, &data);
        self.world.bind(self.robot, look);
    }

    /// Back to the robot the tables begin with.
    pub fn reset(&mut self) {
        self.change(|look, data| *look = Look::first(data));
    }

    /// "Start Planting": into the greenhouse, where its scene is on this
    /// disk. `false` where it is not.
    pub fn start_planting(&mut self) -> bool {
        let stage = garden::read(self.data());
        stage.is_some_and(|stage| self.plant_in(&stage))
    }

    /// Into the greenhouse of a scene read before (`garden::read`). A
    /// second time, the greenhouse is already in the world: Replay.
    pub fn plant_in(&mut self, stage: &phiecs::usd::Stage) -> bool {
        let data = self.data().clone();
        if self.world.resources.get::<Garden>().is_some() {
            let root = self.world.coin_bare(garden::GREENHOUSE);
            enter(&mut self.world, self.robot, root, &data);
            self.replay();
            self.phase = Phase::Exploration;
            return true;
        }
        let (root, garden) = open(&mut self.world, stage, &data);
        enter(&mut self.world, self.robot, root, &data);
        self.world.bind(self.robot, drive::Drive::default());
        self.world.resources.insert(garden);
        self.world.add_system(FnSystem::new("drive", drive::run).at(Stage::PreCouple));
        self.world.add_system(FnSystem::new("garden", garden::run).at(Stage::Couple));
        self.world.add_system(FnSystem::new("machine", machine::run).at(Stage::PostCouple));
        self.world.add_system(FnSystem::new("place", |world: &mut World, _| place(world)).at(Stage::Readout));
        self.phase = Phase::Exploration;
        true
    }

    /// "Replay": every plant to propagate again, the robot back where it
    /// began, standing.
    pub fn replay(&mut self) {
        let data = self.data().clone();
        if let Some(garden) = self.world.resources.get_mut::<Garden>() {
            garden.planter.iter_mut().for_each(|p| p.grown = false);
        }
        let at = ["start.x", "start.y", "start.z"].map(|key| data.game.num(key));
        self.world.bind(self.robot, Pose::at(at));
        self.world.bind(self.robot, Machine::default());
        self.world.bind(self.robot, drive::Drive::default());
    }

    /// "Exit Game": back to making the robot, out of the greenhouse.
    pub fn exit(&mut self) {
        self.world.unbind::<Parent>(self.robot);
        self.world.unbind::<drive::Drive>(self.robot);
        self.world.bind(self.robot, Pose::default());
        self.world.bind(self.robot, Machine::default());
        self.phase = Phase::Playing;
    }

    /// A key goes down, or comes up.
    pub fn press(&mut self, key: &str) {
        drive::press(&mut self.world, self.robot, key);
    }

    pub fn lift(&mut self, key: &str) {
        drive::lift(&mut self.world, self.robot, key);
    }

    /// Time passes.
    pub fn step(&mut self, dt: f32) {
        self.world.step(dt);
    }

    /// The game, while exploring.
    pub fn garden(&self) -> Option<&Garden> {
        self.world.resources.get::<Garden>()
    }

    /// The state the robot is in.
    pub fn state(&self) -> String {
        self.world.read::<Machine>(self.robot).map(|m| m.state.clone()).unwrap_or_default()
    }
}
