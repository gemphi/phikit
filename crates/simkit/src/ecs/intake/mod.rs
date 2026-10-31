//! `Stage::Ingest`: the link drained into the vehicles' aspects.
//!
//! The first thing that happens in a step, and the only thing in simkit
//! that writes a [`Helm`](super::Helm). Whatever a stick or a client asked for between
//! the last step and this one arrives here, is checked against who holds
//! the vehicle, and becomes an aspect. After this system the world is
//! self-contained: the controller and the physics read aspects and nothing
//! else.

use super::aspects::Autopilot;
use super::link::{Act, Order};
use super::world::{sim, sim_mut};
use phiecs::prelude::{Stage, System, Word, World};

mod orders;

use orders::apply;

/// `Stage::Ingest`: takes in everything on the link.
pub struct Intake;

impl System for Intake {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let orders = sim(world).link.drain();
        for order in orders {
            take(world, order);
        }
        sim_mut(world).taken += 1;
    }

    fn stage(&self) -> Stage {
        Stage::Ingest
    }

    fn name(&self) -> &str {
        "intake"
    }
}

/// The word a vehicle of that spelling is, if the world has one.
pub fn vehicle(world: &World, spelling: &str) -> Option<Word> {
    let word = world.lexicon.get(spelling)?;
    match sim(world).vehicles.contains(&word) {
        true => Some(word),
        false => None,
    }
}

/// Takes one order in, or counts it refused.
fn take(world: &mut World, order: Order) {
    let Some(word) = vehicle(world, &order.vehicle) else {
        log::debug!("order for unknown vehicle {:?}", order.vehicle);
        return;
    };
    // `ApiControl` is how control changes hands, so it is never itself
    // subject to who holds it — as in AirSim, where a client may call
    // `enableApiControl` at any time.
    match &order.act {
        Act::ApiControl(_) => {}
        _ => {
            let held = world.read::<Autopilot>(word).is_some_and(|a| a.holds_control(order.origin));
            match held {
                true => {}
                false => {
                    if let Some(pilot) = world.read_mut::<Autopilot>(word) {
                        pilot.refused = pilot.refused.saturating_add(1);
                    }
                    return;
                }
            }
        }
    }
    apply(world, word, order);
}
