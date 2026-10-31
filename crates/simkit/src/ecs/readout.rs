//! The last two stages: what the sensors read, and what gets said about
//! it.
//!
//! `Stage::PostCouple` fills each vehicle's [`Senses`] from the state the
//! physics just produced — a sensor must never read a frame the physics
//! has not finished. `Stage::Readout` writes a [`Report`] per vehicle back
//! onto the [`ControlLink`](super::ControlLink) and advances the clock, so
//! an AirSim client asking `simGetVehiclePose` between steps is answered
//! from the same frame the pilot just flew.

use super::aspects::{Autopilot, Home, Motion, Placement, Senses};
use super::report::Report;
use super::world::{sim, sim_mut, GRAVITY};
use crate::common::GeoPoint;
use glam::Vec3;
use phiecs::prelude::{Stage, System, Word, World};

/// One of the two readout systems, told apart by its stage.
pub struct Readout {
    stage: Stage,
}

impl Readout {
    /// `Stage::PostCouple`: the sensors read off the new state.
    pub fn sense() -> Self {
        Self { stage: Stage::PostCouple }
    }

    /// `Stage::Readout`: the frame written back onto the link.
    pub fn report() -> Self {
        Self { stage: Stage::Readout }
    }
}

impl System for Readout {
    fn run(&mut self, world: &mut World, dt: f32) {
        match self.stage {
            Stage::PostCouple => sense_all(world, dt),
            _ => report_all(world, dt),
        }
    }

    fn stage(&self) -> Stage {
        self.stage
    }

    fn name(&self) -> &str {
        match self.stage {
            Stage::PostCouple => "readout: sense",
            _ => "readout: report",
        }
    }
}

fn sense_all(world: &mut World, _dt: f32) {
    let vehicles = sim(world).vehicles.clone();
    let at = stamp(sim(world).time);
    let (temperature, pressure) = {
        let state = sim(world).environment.state();
        (state.temperature, state.pressure)
    };
    for word in vehicles {
        sense(world, word, at, temperature, pressure);
    }
}

/// What one vehicle's sensors read.
///
/// Ground truth, with no noise added: the sensor models in
/// [`sensors`](crate::sensors) are what adds noise, and a world that is
/// also the truth a test asserts against should not be guessing. A host
/// that wants noise runs the readings through those models.
fn sense(world: &mut World, word: Word, at: u64, temperature: f32, pressure: f32) {
    let Some(place) = world.read::<Placement>(word).copied() else { return };
    let Some(motion) = world.read::<Motion>(word).copied() else { return };
    let home = world.read::<Home>(word).map(|h| h.geo).unwrap_or_default();
    let Some(senses) = world.read_mut::<Senses>(word) else { return };

    // An accelerometer reads specific force, not acceleration: at rest it
    // reads one gravity upward, and in free fall it reads nothing. So
    // gravity is added back before turning into the body frame.
    let specific = motion.accel.linear + Vec3::new(0.0, 0.0, GRAVITY);
    senses.imu.time_stamp = at;
    senses.imu.orientation = place.pose.orientation;
    senses.imu.angular_velocity = place.pose.orientation.inverse() * motion.twist.angular;
    senses.imu.linear_acceleration = place.pose.orientation.inverse() * specific;

    senses.baro.time_stamp = at;
    senses.baro.altitude = home.altitude + place.altitude();
    senses.baro.temperature = temperature;
    senses.baro.pressure = pressure * barometric(place.altitude());

    senses.gps.time_stamp = at;
    senses.gps.geo_point = offset_geo(home, place.pose.position);
    // A GPS reports velocity in NED, so the climb is reported as a
    // descent.
    senses.gps.velocity = super::link::flip_ned(motion.twist.linear);
    senses.gps.has_fix = true;

    senses.agl = place.altitude().max(0.0);
}

fn report_all(world: &mut World, dt: f32) {
    let vehicles = sim(world).vehicles.clone();
    let frame = sim(world).frame;
    let time = sim(world).time;
    let at = stamp(time);
    let mut reports = Vec::with_capacity(vehicles.len());
    for word in &vehicles {
        let place = world.read::<Placement>(*word).copied().unwrap_or_default();
        let motion = world.read::<Motion>(*word).copied().unwrap_or_default();
        let senses = world.read::<Senses>(*word).copied().unwrap_or_default();
        let pilot = match world.read::<Autopilot>(*word) {
            Some(p) => p.clone(),
            None => Autopilot::new(),
        };
        reports.push((
            world.spelling(*word).to_string(),
            Report::of(frame, time, at, &place, &motion, &pilot, &senses),
        ));
    }
    let state = sim_mut(world);
    for (name, report) in &reports {
        state.link.publish(name, report.clone());
    }
    state.frame += 1;
    match state.paused {
        true => {}
        false => state.time += f64::from(dt),
    }
}

/// Seconds of simulation time as the nanosecond stamp every AirSim
/// reading carries.
fn stamp(time: f64) -> u64 {
    (time * 1e9).max(0.0) as u64
}

/// The fraction of sea-level pressure at `altitude` metres, by the
/// barometric formula the troposphere is usually modelled with — the same
/// one AirSim's `BarometerSimple` uses.
fn barometric(altitude: f32) -> f32 {
    const LAPSE: f32 = 0.0065;
    const SEA_TEMP: f32 = 288.15;
    const EXPONENT: f32 = 5.2558;
    (1.0 - LAPSE * altitude.max(-500.0) / SEA_TEMP).max(0.1).powf(EXPONENT)
}

/// `home` moved by `offset` metres, x north and y east. Flat enough for a
/// city, which is what this is for.
fn offset_geo(home: GeoPoint, offset: Vec3) -> GeoPoint {
    const METRES_PER_DEGREE: f64 = 111_320.0;
    let lat = home.latitude + f64::from(offset.x) / METRES_PER_DEGREE;
    let scale = home.latitude.to_radians().cos().abs().max(1e-6);
    let lon = home.longitude + f64::from(offset.y) / (METRES_PER_DEGREE * scale);
    GeoPoint::new(lat, lon, home.altitude + offset.z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressure_falls_with_height() {
        assert!((barometric(0.0) - 1.0).abs() < 1e-5);
        assert!(barometric(100.0) < 1.0);
        assert!(barometric(1000.0) < barometric(100.0));
    }

    #[test]
    fn moving_north_raises_the_latitude_and_moving_east_the_longitude() {
        let home = GeoPoint::new(47.6, -122.3, 10.0);
        let north = offset_geo(home, Vec3::new(1113.2, 0.0, 0.0));
        assert!((north.latitude - 47.61).abs() < 1e-4, "got {}", north.latitude);
        assert_eq!(north.longitude, home.longitude);

        let east = offset_geo(home, Vec3::new(0.0, 1000.0, 5.0));
        assert!(east.longitude > home.longitude);
        assert_eq!(east.altitude, 15.0, "and a climb raises the altitude");
    }
}
