//! The one place a button code becomes a demand.
//!
//! [`Pilot`] holds a device's [`Bindings`] and the state a switch has to
//! remember between frames — which mode is selected, which gear is in,
//! which buttons were down last frame so a press can be told from a hold.
//! Everything downstream reads the [`ControlState`] it produces and never
//! the reading it came from.

use super::binding::{Bindings, Intent};
use super::control::{CarDemand, ControlState, Gear, Manoeuvre, RotorDemand, RotorMode, VehicleKind};
use super::device::Reading;

/// A device, read through its bindings into a control state.
pub struct Pilot {
    bindings: Bindings,
    mode: RotorMode,
    gear: Gear,
    top_gear: u8,
    /// Last frame's switch states, so a press is a press and not a hold.
    was: u64,
}

impl Pilot {
    /// A pilot on `bindings`, in [`RotorMode::Angle`] and first gear.
    pub fn new(bindings: Bindings) -> Self {
        Self { bindings, mode: RotorMode::Angle, gear: Gear::Forward(1), top_gear: 6, was: 0 }
    }

    /// A pilot on the gamepad preset, which also reads a
    /// [`Keys`](super::Keys) keyboard.
    pub fn gamepad() -> Self {
        Self::new(Bindings::gamepad())
    }

    /// A pilot on the flight-stick preset.
    pub fn flight_stick() -> Self {
        Self::new(Bindings::flight_stick())
    }

    /// A pilot on the wheel-and-pedals preset.
    pub fn wheel() -> Self {
        Self::new(Bindings::wheel())
    }

    /// The table, to read.
    pub fn bindings(&self) -> &Bindings {
        &self.bindings
    }

    /// The table, to remap or retune mid-flight.
    pub fn bindings_mut(&mut self) -> &mut Bindings {
        &mut self.bindings
    }

    /// The mode the switch is on.
    pub fn mode(&self) -> RotorMode {
        self.mode
    }

    /// Sets the mode, as the mode switch or an API call would.
    pub fn set_mode(&mut self, mode: RotorMode) {
        self.mode = mode;
    }

    /// The gear that is in.
    pub fn gear(&self) -> Gear {
        self.gear
    }

    /// How many forward gears the box has.
    pub fn set_top_gear(&mut self, top: u8) {
        self.top_gear = top.max(1);
    }

    /// `reading` as a demand for `kind` of vehicle.
    pub fn read(&mut self, kind: VehicleKind, reading: &Reading) -> ControlState {
        match kind {
            VehicleKind::Rotor => ControlState::Rotor(self.fly(reading)),
            VehicleKind::Car => ControlState::Car(self.drive(reading)),
        }
    }

    /// `reading` as a multirotor demand.
    ///
    /// The mode switch is stepped first, so the demand it returns is
    /// already in the mode the press selected — otherwise the frame a
    /// pilot changes mode in is flown in the old one.
    pub fn fly(&mut self, reading: &Reading) -> RotorDemand {
        if self.pressed(Intent::NextMode, reading) {
            self.mode = self.mode.next();
        }
        let manoeuvre = match (
            self.pressed(Intent::Takeoff, reading),
            self.pressed(Intent::Land, reading),
        ) {
            (true, _) => Some(Manoeuvre::Takeoff),
            (false, true) => Some(Manoeuvre::Land),
            (false, false) => None,
        };
        let demand = RotorDemand {
            roll: self.bindings.value(Intent::Roll, reading),
            pitch: self.bindings.value(Intent::Pitch, reading),
            yaw: self.bindings.value(Intent::Yaw, reading),
            throttle: self.bindings.value(Intent::Throttle, reading),
            mode: self.mode,
            armed: self.bindings.is_held(Intent::Arm, reading),
            manoeuvre,
        };
        self.latch(reading);
        demand
    }

    /// `reading` as a car demand.
    pub fn drive(&mut self, reading: &Reading) -> CarDemand {
        match (self.pressed(Intent::GearUp, reading), self.pressed(Intent::GearDown, reading)) {
            (true, false) => self.gear = self.gear.up(self.top_gear),
            (false, true) => self.gear = self.gear.down(),
            _ => {}
        }
        let demand = CarDemand {
            steer: self.bindings.value(Intent::Steer, reading),
            throttle: self.bindings.value(Intent::Throttle, reading),
            brake: self.bindings.value(Intent::Brake, reading),
            handbrake: self.bindings.is_held(Intent::Handbrake, reading),
            gear: self.gear,
        };
        self.latch(reading);
        demand
    }

    /// Whether `intent`'s switch went down this frame, having been up
    /// last frame.
    fn pressed(&self, intent: Intent, reading: &Reading) -> bool {
        let now = self.bindings.is_held(intent, reading);
        match (now, self.held_last_frame(intent)) {
            (true, false) => true,
            _ => false,
        }
    }

    fn held_last_frame(&self, intent: Intent) -> bool {
        match Self::latch_bit(intent) {
            Some(bit) => self.was & (1 << bit) != 0,
            None => false,
        }
    }

    /// Remembers which switches are down, for the next frame's edges.
    fn latch(&mut self, reading: &Reading) {
        self.was = 0;
        for intent in LATCHED {
            let down = self.bindings.is_held(*intent, reading);
            match (Self::latch_bit(*intent), down) {
                (Some(bit), true) => self.was |= 1 << bit,
                _ => {}
            }
        }
    }

    /// Which bit of [`Pilot::was`] remembers an intent.
    fn latch_bit(intent: Intent) -> Option<u8> {
        LATCHED.iter().position(|i| *i == intent).map(|i| i as u8)
    }
}

/// The switch intents whose presses are told from their holds.
const LATCHED: &[Intent] =
    &[Intent::NextMode, Intent::Takeoff, Intent::Land, Intent::GearUp, Intent::GearDown];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::device::slot;

    fn held(button: u8) -> Reading {
        Reading::resting(slot::AXES).with_held(button)
    }

    #[test]
    fn a_held_mode_button_steps_the_switch_once() {
        let mut pilot = Pilot::gamepad();
        assert_eq!(pilot.mode(), RotorMode::Angle);

        let down = held(slot::BUTTON_MODE);
        assert_eq!(pilot.fly(&down).mode, RotorMode::AltitudeHold, "the press lands this frame");
        assert_eq!(pilot.fly(&down).mode, RotorMode::AltitudeHold, "holding it does not step again");
        pilot.fly(&Reading::resting(slot::AXES));
        assert_eq!(pilot.fly(&down).mode, RotorMode::PositionHold, "a second press steps on");
    }

    #[test]
    fn the_arm_switch_is_read_while_held_not_on_its_edge() {
        let mut pilot = Pilot::gamepad();
        let down = held(slot::BUTTON_ARM);
        assert!(pilot.fly(&down).armed);
        assert!(pilot.fly(&down).armed, "a switch stays on while it is on");
        assert!(!pilot.fly(&Reading::resting(slot::AXES)).armed);
    }

    #[test]
    fn a_manoeuvre_fires_once_on_the_press() {
        let mut pilot = Pilot::gamepad();
        let down = held(slot::BUTTON_TAKEOFF);
        assert_eq!(pilot.fly(&down).manoeuvre, Some(Manoeuvre::Takeoff));
        assert_eq!(pilot.fly(&down).manoeuvre, None, "and not again while it is held");
    }

    #[test]
    fn the_sticks_reach_the_demand_shaped_by_their_bindings() {
        let mut pilot = Pilot::gamepad();
        pilot.bindings_mut().shape_mut(Intent::Roll).expect("roll").trim = 0.1;
        let reading = Reading::resting(slot::AXES).with_axis(slot::AXIS_PITCH, 1.0);
        let demand = pilot.fly(&reading);
        assert_eq!(demand.roll, 0.1, "a trimmed roll asks for something at rest");
        assert!(demand.pitch > 0.0, "and the pitch axis keeps the sense it was read in");
    }

    #[test]
    fn shifting_walks_the_box_and_the_gear_rides_along_in_the_demand() {
        let mut pilot = Pilot::wheel();
        pilot.set_top_gear(3);
        let up = held(slot::BUTTON_GEAR_UP);
        let rest = Reading::resting(slot::AXES);
        assert_eq!(pilot.drive(&up).gear, Gear::Forward(2));
        assert_eq!(pilot.drive(&up).gear, Gear::Forward(2), "a held paddle shifts once");
        pilot.drive(&rest);
        assert_eq!(pilot.drive(&up).gear, Gear::Forward(3));
        pilot.drive(&rest);
        assert_eq!(pilot.drive(&up).gear, Gear::Forward(3), "and stops at the top");
    }

    #[test]
    fn one_pilot_reads_either_kind_of_vehicle() {
        let mut pilot = Pilot::gamepad();
        let reading = Reading::resting(slot::AXES).with_axis(slot::AXIS_ROLL, 0.8);
        let rotor = pilot.read(VehicleKind::Rotor, &reading);
        let car = pilot.read(VehicleKind::Car, &reading);
        assert!(rotor.rotor().expect("a rotor demand").roll > 0.0);
        assert!(car.car().expect("a car demand").steer > 0.0);
        assert!(rotor.car().is_none());
    }
}
