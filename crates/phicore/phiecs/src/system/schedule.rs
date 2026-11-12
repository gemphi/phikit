//! Adding systems and running a step.
//!
//! A step is SceneKit's frame (SCN `SCNSceneRenderer.h:343-397`): the clock
//! advances first, then each [`Stage`] runs in [`Stage::ORDER`], and the
//! event queue swaps last.
use super::event::EventQueue;
use super::{Stage, System};
use crate::comp::time::Clock;
use crate::world::World;

impl World {
    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }

    /// Runs one step of `dt` seconds:
    /// 1. the [`Clock`] advances: `tick + 1`, `time + dt`;
    /// 2. every stage in [`Stage::ORDER`] runs its systems in the order they
    ///    were added;
    /// 3. the [`EventQueue`] swaps: this step's events stay readable through
    ///    the next step, last step's are dropped.
    ///
    /// A system added during the step first runs on the next one.
    pub fn step(&mut self, dt: f32) {
        if let Some(clock) = self.resources.get_mut::<Clock>() {
            clock.advance(dt);
        }
        let mut systems = std::mem::take(&mut self.systems);
        for stage in Stage::ORDER {
            for s in systems.iter_mut().filter(|s| s.stage() == *stage) {
                s.run(self, dt);
            }
        }
        let added = std::mem::take(&mut self.systems);
        systems.extend(added);
        self.systems = systems;
        if let Some(events) = self.resources.get_mut::<EventQueue>() {
            events.swap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::FnSystem;
    use crate::system::event::{DidAdd, WillRemove};
    use std::sync::{Arc, Mutex};

    #[test]
    fn the_clock_advances_before_any_stage() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = seen.clone();
        let mut w = World::new();
        w.add_system(FnSystem::new("look", move |w, _| log.lock().unwrap().push(w.clock().tick)).at(Stage::Ingest));
        w.step(0.5);
        w.step(0.25);
        assert_eq!(*seen.lock().unwrap(), [1, 2]);
        let c = w.clock();
        assert_eq!((c.tick, c.time, c.dt), (2, 0.75, 0.25));
    }

    #[test]
    fn an_event_is_read_by_a_later_stage_and_the_next_step_then_gone() {
        #[derive(Debug, PartialEq)]
        struct Ping(u32);
        let heard = Arc::new(Mutex::new(Vec::new()));
        let log = heard.clone();
        let mut w = World::new();
        w.add_system(FnSystem::new("emit", |w, _| w.emit(Ping(w.clock().tick as u32))).at(Stage::Ingest));
        w.add_system(FnSystem::new("hear", move |w, _| log.lock().unwrap().push(w.events::<Ping>().map(|p| p.0).collect::<Vec<_>>())).at(Stage::Readout));
        w.step(1.0);
        w.step(1.0);
        w.step(1.0);
        assert_eq!(*heard.lock().unwrap(), [vec![1], vec![1, 2], vec![2, 3]]);
    }

    #[test]
    fn coining_and_forgetting_are_announced() {
        let mut w = World::new();
        let a = w.coin_bare("a");
        w.coin_bare("a");
        assert_eq!(w.events::<DidAdd>().map(|e| e.0).collect::<Vec<_>>(), [a], "once, not for a word known");
        w.forget(a);
        assert_eq!(w.events::<WillRemove>().map(|e| e.0).collect::<Vec<_>>(), [a]);
    }
}
