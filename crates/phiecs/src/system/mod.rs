//! `System` — something that runs over the world each step.
//!
//! The trait and the staged loop are Apple's (`reality-foundation/system.rs`).
//! The stages are not: `Input → PreUpdate → Update → PostUpdate → Render`
//! assumes a 16 ms frame, and a language engine advances per observation, not
//! per frame. See [`Stage`].

mod set;
pub use set::Stage;

use crate::world::World;

/// A step of the world. `dt` is the step's size — usually 1.0, one
/// observation — not a frame time.
pub trait System: Send + Sync {
    fn run(&mut self, world: &mut World, dt: f32);

    fn stage(&self) -> Stage {
        Stage::Couple
    }

    /// For logs and the ECS Wand's node list.
    fn name(&self) -> &str {
        "system"
    }
}

/// A system written as a closure.
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
    use std::sync::Arc;

    #[test]
    fn stages_run_in_order() {
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut w = World::new();
        for (name, stage) in
            [("readout", Stage::Readout), ("ingest", Stage::Ingest), ("couple", Stage::Couple)]
        {
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
