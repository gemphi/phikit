//! Stages and the system trait.
use crate::world::World;

/// When in a step a system runs. [`World::step`] runs every system of one
/// stage, in the order they were added, before any of the next.
///
/// The five stages are SceneKit's documented frame (SCN
/// `SCNSceneRenderer.h:343-397`: update, animations, physics, constraints,
/// render). Apple's RealityKit orders systems by dependency instead
/// (RF:14757-14761), so the stages are ours.
///
/// | stage | SceneKit phase | what runs |
/// |---|---|---|
/// | `Ingest` | update | input, commands, asset loads |
/// | `PreCouple` | animations | clips sample and write their bound fields |
/// | `Couple` | physics | bodies in, a step, poses out |
/// | `PostCouple` | constraints | constraints, IK, then propagate the tree |
/// | `Readout` | render | hand the changed state to the renderer and the journal |
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Ingest,
    PreCouple,
    Couple,
    PostCouple,
    Readout,
}

impl Stage {
    pub const ORDER: &'static [Stage] = &[Stage::Ingest, Stage::PreCouple, Stage::Couple, Stage::PostCouple, Stage::Readout];

    pub fn label(self) -> &'static str {
        match self {
            Stage::Ingest => "Ingest",
            Stage::PreCouple => "Pre-couple",
            Stage::Couple => "Couple",
            Stage::PostCouple => "Post-couple",
            Stage::Readout => "Readout",
        }
    }
}

pub trait System: Send + Sync {
    fn run(&mut self, world: &mut World, dt: f32);

    fn stage(&self) -> Stage {
        Stage::Couple
    }

    fn name(&self) -> &str {
        "system"
    }
}

/// A function as a system, at [`Stage::Couple`] unless placed [`at`](Self::at)
/// another stage.
pub struct FnSystem<F> {
    f: F,
    stage: Stage,
    name: &'static str,
}

impl<F: FnMut(&mut World, f32) + Send + Sync> FnSystem<F> {
    pub fn new(name: &'static str, f: F) -> Self {
        Self { f, stage: Stage::Couple, name }
    }

    pub fn at(mut self, stage: Stage) -> Self {
        self.stage = stage;
        self
    }
}

impl<F: FnMut(&mut World, f32) + Send + Sync> System for FnSystem<F> {
    fn run(&mut self, world: &mut World, dt: f32) {
        (self.f)(world, dt)
    }

    fn stage(&self) -> Stage {
        self.stage
    }

    fn name(&self) -> &str {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    #[test]
    fn stages_run_in_order() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut w = World::new();
        for (name, stage) in [("readout", Stage::Readout), ("ingest", Stage::Ingest), ("couple", Stage::Couple)] {
            let log = order.clone();
            w.add_system(FnSystem::new(name, move |_, _| log.lock().unwrap().push(name)).at(stage));
        }
        let ticks = Arc::new(AtomicUsize::new(0));
        let t = ticks.clone();
        w.add_system(FnSystem::new("count", move |_, _| {
            t.fetch_add(1, Ordering::Relaxed);
        }));
        w.step(1.0);
        w.step(1.0);
        assert_eq!(&order.lock().unwrap()[..3], &["ingest", "couple", "readout"]);
        assert_eq!(ticks.load(Ordering::Relaxed), 2, "systems survive a step");
    }
}
