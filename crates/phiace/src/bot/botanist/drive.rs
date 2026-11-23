//! Walking the robot (RobotCharacter+Movement.swift, the keyboard
//! controls): keys held add to a move; the robot goes along it at its
//! speed and faces where it goes; letting go ends the walk after a
//! moment. The keys and speeds are the tables `key.tsv` and `game.tsv`.

use phiecs::comp::Pose;
use phiecs::prelude::*;

use super::Data;
use super::machine::{CELEBRATE, Machine, STOP, WALK, WAVE};

/// The keys held, the move they make, and how long ago the last was let
/// go while none is held.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Drive {
    pub held: Vec<String>,
    pub settle: Option<f32>,
}

impl Aspect for Drive {}

impl Drive {
    /// The move the held keys make, on the ground: x and z.
    pub fn vector(&self, data: &Data) -> [f32; 2] {
        let step = data.game.num("key.step");
        self.held.iter().filter_map(|key| data.key.rows(key).next()).fold([0.0, 0.0], |[x, z], row| {
            let cell = |i: usize| row.get(i).and_then(|v| v.parse::<f32>().ok()).unwrap_or_default() * step;
            [x + cell(1), z + cell(2)]
        })
    }
}

/// A key goes down: it is held, and the robot walks.
pub fn press(world: &mut World, robot: Word, key: &str) {
    let Some(data) = world.resources.get::<Data>().cloned() else { return };
    let known = data.key.rows(key).next().is_some();
    if let Some(drive) = world.read_mut::<Drive>(robot).filter(|_| known) {
        (!drive.held.iter().any(|k| k == key)).then(|| drive.held.push(key.to_string()));
        drive.settle = None;
    }
    if let Some(machine) = world.read_mut::<Machine>(robot).filter(|_| known) {
        machine.go(WALK, &data);
    }
}

/// A key comes up; with none held, the walk ends once the wait passes.
pub fn lift(world: &mut World, robot: Word, key: &str) {
    if let Some(drive) = world.read_mut::<Drive>(robot) {
        drive.held.retain(|k| k != key);
        drive.settle = drive.held.is_empty().then_some(0.0);
    }
}

/// Each step: move every robot that walks, and end a walk whose keys
/// were let go long enough ago.
pub fn run(world: &mut World, dt: f32) {
    let Some(data) = world.resources.get::<Data>().cloned() else { return };
    let robot: Vec<Word> = world.each::<Drive>().map(|(word, _)| word).collect();
    for word in robot {
        let (Some(drive), Some(machine)) = (world.read::<Drive>(word).cloned(), world.read::<Machine>(word).cloned()) else { continue };
        // A wait after the last key: past it, with nothing held, the walk ends.
        let settled = drive.settle.map(|t| t + dt);
        let ended = settled.is_some_and(|t| t >= data.game.num("key.settle"));
        if let Some(d) = world.read_mut::<Drive>(word) {
            d.settle = settled.filter(|_| !ended);
        }
        if ended {
            world.read_mut::<Machine>(word).into_iter().for_each(|m| _ = m.go(STOP, &data));
        }
        // Celebrating or waving, it stands.
        let still = matches!(machine.state.as_str(), CELEBRATE | WAVE);
        let v = drive.vector(&data);
        let length = (v[0] * v[0] + v[1] * v[1]).sqrt();
        if still || length == 0.0 {
            continue;
        }
        let dir = v.map(|c| c / length.max(data.game.num("move.floor")));
        let speed = match machine.state.as_str() {
            STOP => data.game.num("speed.end"),
            _ => data.game.num("speed.base"),
        };
        if let Some(pose) = world.read_mut::<Pose>(word) {
            pose.translate[0] += dir[0] * speed * dt;
            pose.translate[2] += dir[1] * speed * dt;
            pose.orient = Pose::facing(dir);
        }
    }
}
