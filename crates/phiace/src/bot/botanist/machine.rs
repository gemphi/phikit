//! The robot's animation states (AnimationStateMachine.swift): which may
//! follow which, which loop, and where a clip that plays once goes when
//! it ends. All of it is the tables `state.tsv` and `transition.tsv`.

use phiecs::prelude::*;

use super::Data;

/// The state a robot is in, and how long it has been in it.
#[derive(Debug, Clone, PartialEq)]
pub struct Machine {
    pub state: String,
    pub time: f32,
}

impl Aspect for Machine {}

impl Default for Machine {
    fn default() -> Self {
        Machine { state: IDLE.to_string(), time: 0.0 }
    }
}

/// The state a robot begins in, and the states the game asks for.
pub const IDLE: &str = "idle";
pub const WALK: &str = "walkLoop";
pub const STOP: &str = "walkEnd";
pub const CELEBRATE: &str = "celebrate";
pub const WAVE: &str = "wave";

impl Machine {
    /// Go to another state, where the table lets this one go there.
    /// `true` when it went.
    pub fn go(&mut self, to: &str, data: &Data) -> bool {
        let allowed = data.transition.get(&self.state).split_whitespace().any(|state| state == to);
        if allowed {
            self.state = to.to_string();
            self.time = 0.0;
        }
        allowed
    }

    /// Whether its state's clip loops.
    pub fn loops(&self, data: &Data) -> bool {
        data.state.get(&self.state) == "loop"
    }
}

/// Time passes: a clip that plays once and has played its length goes
/// to the state the table names after it.
pub fn run(world: &mut World, dt: f32) {
    let Some(data) = world.resources.get::<Data>().cloned() else { return };
    let length = data.game.num(CLIP);
    let robot: Vec<Word> = world.each::<Machine>().map(|(word, _)| word).collect();
    for word in robot {
        let Some(machine) = world.read_mut::<Machine>(word) else { continue };
        machine.time += dt;
        let done = !machine.loops(&data) && machine.time >= length;
        let next = data.state.rows(&machine.state).next().and_then(|row| row.get(2)).cloned();
        if let (true, Some(next)) = (done, next) {
            machine.state = next;
            machine.time = 0.0;
        }
    }
}

/// The row of `game.tsv` for how long a clip that plays once lasts. The
/// sample's clips are binary USD the stage does not open yet: a mock.
pub const CLIP: &str = "clip.length";
