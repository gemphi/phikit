//! simkit's vehicles as a phiecs world.
//!
//! # The shape of it
//!
//! A vehicle is a `Word`. Its pose, its motion, what it is being asked
//! for, its controller and its sensor readings are [`Aspect`](phiecs::prelude::Aspect)s on that
//! word. The simulation itself — the clock, the environment, the list of
//! vehicles — is one resource, [`Sim`]. The logic is [`System`](phiecs::prelude::System)s run in
//! `Stage` order by `World::step`. This is phiplay's shape, not a second
//! one: `BBody`/`BRole`/`BAct` on a player's word and `BGame` as the
//! resource is `Placement`/`Motion`/`Helm` on a vehicle's word and `Sim`
//! as the resource.
//!
//! ```text
//!  a stick ──▶ ┐                                      ┌──▶ a pose
//!              ├─▶ ControlLink ─▶ World::step(dt) ─────┤
//!  a client ─▶ ┘      orders        Ingest             └──▶ a report
//!   (rpc)                           PreCouple
//!                                   Couple
//!                                   PostCouple
//!                                   Readout ──▶ ControlLink
//! ```
//!
//! | stage | system | what it does |
//! |---|---|---|
//! | `Ingest` | [`Intake`] | drains the link into [`Helm`] aspects |
//! | `PreCouple` | [`Fly`], [`Drive`] | the controller: demand into [`Wanted`] |
//! | `Couple` | [`Fly`], [`Drive`] | the physics: [`Placement`] and [`Motion`] |
//! | `PostCouple` | [`Readout`] | the sensors read off the new state |
//! | `Readout` | [`Readout`] | the frame published, the clock advanced |
//!
//! # Why one link
//!
//! The thing that makes this worth building is that a client's command and
//! a pilot's stick take the *same* path. Both put an [`Order`] on the
//! [`ControlLink`]; [`Intake`] is the only thing that reads it; nothing
//! else writes a [`Helm`]. So neither door can bypass the other, there is
//! no second code path to keep in step, and who is obeyed at any moment is
//! one flag — AirSim's `enableApiControl`, which [`Autopilot`] holds and
//! [`Intake`] honours.
//!
//! # Driving one
//!
//! ```
//! use simkit::ecs::{build, link_of, Act, Plan};
//! use simkit::input::{ControlState, RotorDemand, RotorMode};
//!
//! let mut world = build(Plan::new().with_drone("drone"));
//! let link = link_of(&world);
//!
//! // Arm it and ask for a climb, as a stick would.
//! link.device(
//!     "drone",
//!     Act::Helm(ControlState::Rotor(RotorDemand {
//!         armed: true,
//!         throttle: 0.95,
//!         mode: RotorMode::AltitudeHold,
//!         ..RotorDemand::idle()
//!     })),
//! );
//! for _ in 0..200 {
//!     world.step(1.0 / 60.0);
//! }
//! assert!(link.report("drone").expect("a report").altitude() > 1.0);
//! ```

pub mod aspects;
pub mod drive;
pub mod fly;
pub mod intake;
pub mod link;
pub mod readout;
pub mod report;
pub mod wire;
pub mod world;

#[cfg(test)]
mod tests;

pub use aspects::{
    Autopilot, CarLimits, Craft, Helm, Home, Motion, Placement, RotorLimits, Senses, Wanted,
};
pub use drive::Drive;
pub use fly::Fly;
pub use intake::{vehicle, Intake};
pub use link::{flip_ned, Act, ControlLink, Order, Origin};
pub use readout::Readout;
pub use report::{Report, GROUND_TOUCH};
pub use wire::{Call, Params};
pub use world::{build, link_of, names, sim, sim_mut, Plan, Sim, GRAVITY};

use crate::input::{Device, Pilot};
use phiecs::prelude::World;

/// A device wired to a world: polls the device, reads it through a pilot,
/// puts the demand on the link.
///
/// This is the whole of the local path, and it is deliberately thin —
/// it builds an [`Act::Helm`] and sends it, exactly as the RPC server
/// does for a client's `moveByRC`. A host that wants to drive the world
/// itself can skip it and send orders straight.
pub struct Driver<D> {
    /// The device being read.
    pub device: D,
    /// The bindings and switch state it is read through.
    pub pilot: Pilot,
    /// Which vehicle it is flying.
    pub vehicle: String,
    kind: crate::input::VehicleKind,
    link: ControlLink,
}

impl<D: Device> Driver<D> {
    /// A driver flying `vehicle` in `world` with `device`, read through
    /// `pilot`.
    ///
    /// Which kind of demand to produce is taken from the vehicle's
    /// [`Craft`] now, so the poll loop never has to borrow the world.
    pub fn new(world: &World, vehicle: &str, device: D, pilot: Pilot) -> Self {
        let kind = intake::vehicle(world, vehicle)
            .and_then(|w| world.read::<Craft>(w))
            .map(Craft::kind)
            .unwrap_or_default();
        Self { device, pilot, vehicle: vehicle.to_string(), kind, link: link_of(world) }
    }

    /// Which kind of vehicle this driver is flying.
    pub fn kind(&self) -> crate::input::VehicleKind {
        self.kind
    }

    /// Polls the device and puts this frame's demand on the link. Called
    /// once per step, before `World::step`.
    pub fn poll(&mut self, dt: f32) -> crate::input::ControlState {
        let reading = self.device.poll(dt).clone();
        let control = self.pilot.read(self.kind, &reading);
        self.link.device(&self.vehicle, Act::Helm(control));
        control
    }

    /// What the vehicle last reported.
    pub fn report(&self) -> Option<Report> {
        self.link.report(&self.vehicle)
    }
}
