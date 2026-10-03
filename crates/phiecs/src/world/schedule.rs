#[allow(unused_imports)]
use super::*;

use crate::system::Stage;

impl World {
    /// Add a system to this world's own schedule.
    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }

    /// Run one step: every system, in stage order.
    ///
    /// Apple writes this loop twice — once in `World::update` and once in
    /// `Scheduler::run` (`world.rs:67`, `system.rs:93`). Here the scheduler
    /// calls this, so there is one implementation of what "a step" means.
    pub fn step(&mut self, dt: f32) {
        let mut systems = std::mem::take(&mut self.systems);
        for stage in Stage::ORDER {
            for s in systems.iter_mut().filter(|s| s.stage() == *stage) {
                s.run(self, dt);
            }
        }
        // A system may have added another; keep those, and put ours back
        // underneath them so order is stable across steps.
        let added = std::mem::take(&mut self.systems);
        systems.extend(added);
        self.systems = systems;
    }
}
