//! The world the vehicles live in, and how one is put together.
//!
//! [`Sim`] is the world's one resource — phiplay's `BGame`, for a
//! simulation: the clock, the environment, the list of vehicles and the
//! [`ControlLink`] both doors come through. [`build`] coins the words,
//! binds the aspects and adds the systems in stage order; after that a
//! host's whole loop is `world.step(dt)`.

use super::aspects::{Autopilot, Craft, Helm, Home, Motion, Placement, Senses};
use super::drive::Drive;
use super::fly::Fly;
use super::intake::Intake;
use super::link::ControlLink;
use super::readout::Readout;
use crate::common::GeoPoint;
use crate::input::{CarDemand, ControlState, RotorDemand};
use crate::physics::Environment;
use glam::Vec3;
use phiecs::prelude::{Word, World};

/// Gravity, m/s², down. z is up, so it pulls negative.
pub const GRAVITY: f32 = 9.81;

/// The simulation itself: the world's one resource.
pub struct Sim {
    /// Which frame it is on.
    pub frame: usize,
    /// Simulation time, seconds.
    pub time: f64,
    /// How many times [`Intake`] has run, which is how
    /// many steps the world has taken.
    pub taken: usize,
    /// Wind, temperature and pressure, as the sensors read them.
    pub environment: Environment,
    /// Every vehicle in the world, in the order it was coined.
    pub vehicles: Vec<Word>,
    /// The way in and the way out.
    pub link: ControlLink,
    /// A paused world takes orders in and reports out but does not move.
    pub paused: bool,
}

impl Sim {
    /// A simulation on `link`, with `home` as the world origin's
    /// geographic point.
    pub fn new(link: ControlLink, home: GeoPoint) -> Self {
        Self {
            frame: 0,
            time: 0.0,
            taken: 0,
            environment: Environment::new(home),
            vehicles: Vec::new(),
            link,
            paused: false,
        }
    }
}

/// The simulation resource, read.
///
/// # Panics
/// If the world was not built by [`build`], which is the only way to make
/// one — a world without a `Sim` is not a simkit world.
pub fn sim(world: &World) -> &Sim {
    world.resources.get::<Sim>().expect("a simkit world has a Sim")
}

/// The simulation resource, to write.
///
/// # Panics
/// As [`sim`].
pub fn sim_mut(world: &mut World) -> &mut Sim {
    world.resources.get_mut::<Sim>().expect("a simkit world has a Sim")
}

/// How a world is to be set up.
pub struct Plan {
    /// The link the world takes its orders from.
    pub link: ControlLink,
    /// The geographic point the world's origin sits at.
    pub home: GeoPoint,
    /// The vehicles to coin: a spelling, what kind of craft, and where it
    /// starts.
    pub vehicles: Vec<(String, Craft, Vec3)>,
}

impl Default for Plan {
    fn default() -> Self {
        Self::new()
    }
}

impl Plan {
    /// An empty plan on a fresh link, homed where AirSim homes its own
    /// examples.
    pub fn new() -> Self {
        Self {
            link: ControlLink::new(),
            home: GeoPoint::new(47.641468, -122.140165, 122.0),
            vehicles: Vec::new(),
        }
    }

    /// The same plan on `link`, so a server already holding one can share
    /// it.
    pub fn on(mut self, link: ControlLink) -> Self {
        self.link = link;
        self
    }

    /// The same plan homed at `home`.
    pub fn homed(mut self, home: GeoPoint) -> Self {
        self.home = home;
        self
    }

    /// With a stock multirotor called `name`, on the ground at the origin.
    pub fn with_drone(self, name: &str) -> Self {
        self.with_vehicle(name, Craft::rotor(), Vec3::ZERO)
    }

    /// With a stock car called `name`, at the origin.
    pub fn with_car(self, name: &str) -> Self {
        self.with_vehicle(name, Craft::car(), Vec3::ZERO)
    }

    /// With `craft` called `name`, starting at `at`.
    pub fn with_vehicle(mut self, name: &str, craft: Craft, at: Vec3) -> Self {
        self.vehicles.push((name.to_string(), craft, at));
        self
    }
}

/// A world built to `plan`: its vehicles coined and bound, its systems
/// added in stage order.
///
/// The stages are the whole of the bridge, and they run in this order
/// every step:
///
/// | stage | system | what it does |
/// |---|---|---|
/// | `Ingest` | [`Intake`] | the link drained into [`Helm`] aspects |
/// | `PreCouple` | [`Fly`], [`Drive`] | demand plus state into [`Wanted`](super::aspects::Wanted) |
/// | `Couple` | [`Fly`], [`Drive`] | the physics step: [`Placement`] and [`Motion`] |
/// | `PostCouple` | [`Readout`] | the sensors read off the new state |
/// | `Readout` | [`Readout`] | the frame written back onto the link |
pub fn build(plan: Plan) -> World {
    let mut world = World::new();
    world.register::<Placement>();
    world.register::<Motion>();
    world.register::<Helm>();
    world.register::<Autopilot>();
    world.register::<Senses>();
    world.register::<Craft>();
    world.register::<Home>();

    world.resources.insert(Sim::new(plan.link, plan.home));

    for (name, craft, at) in plan.vehicles {
        let idle = match craft.kind() {
            crate::input::VehicleKind::Rotor => ControlState::Rotor(RotorDemand::idle()),
            crate::input::VehicleKind::Car => ControlState::Car(CarDemand::idle()),
        };
        let word = world
            .coin(&name)
            .bind(Placement::at(at))
            .bind(Motion::still())
            .bind(Helm::new(idle, super::link::Origin::Device))
            .bind(Autopilot::new())
            .bind(Senses::default())
            .bind(Home { geo: plan.home })
            .bind(craft)
            .word();
        if let Some(pilot) = world.read_mut::<Autopilot>(word) {
            pilot.home = at;
            pilot.target = at;
        }
        sim_mut(&mut world).vehicles.push(word);
    }

    world.add_system(Intake);
    world.add_system(Fly::command());
    world.add_system(Fly::integrate());
    world.add_system(Drive::command());
    world.add_system(Drive::integrate());
    world.add_system(Readout::sense());
    world.add_system(Readout::report());
    world
}

/// The link a built world takes its orders from and publishes its reports
/// to. Cloning it is sharing it, so this is what a host hands to an RPC
/// server or a device loop.
pub fn link_of(world: &World) -> ControlLink {
    sim(world).link.clone()
}

/// The vehicles in the world, with their spellings, for a `listVehicles`.
pub fn names(world: &World) -> Vec<String> {
    sim(world).vehicles.iter().map(|w| world.spelling(*w).to_string()).collect()
}
