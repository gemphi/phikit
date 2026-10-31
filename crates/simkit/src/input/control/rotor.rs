//! What a multirotor is asked for: four channels, a mode switch, an
//! arm switch, and the momentary switches that ask for a manoeuvre.
//!
//! After AirSim's `RCData` and `simple_flight`'s `GoalMode` channels.

use crate::common::{RcData, TimePoint};
use crate::controllers::FlightMode;

/// Which of the sticks' meanings is in force.
///
/// A radio's mode switch, and the same ladder AirSim's `simple_flight`
/// climbs through its `GoalMode` channels: the further down this list,
/// the more of the work the controller does and the less the stick means
/// directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RotorMode {
    /// Acro. The sticks are body rates and the throttle is collective
    /// thrust: nothing is levelled for you.
    Rate,
    /// The sticks are roll and pitch *angles*, levelled when centred.
    /// The throttle is still collective thrust.
    #[default]
    Angle,
    /// As [`Angle`](Self::Angle), but the throttle is a climb rate about
    /// its centre, so a centred throttle holds height.
    AltitudeHold,
    /// The sticks are horizontal velocities and the throttle is a climb
    /// rate, so centred sticks hold position.
    PositionHold,
}

impl RotorMode {
    /// The modes in the order the mode switch walks them.
    pub const LADDER: &'static [RotorMode] =
        &[RotorMode::Rate, RotorMode::Angle, RotorMode::AltitudeHold, RotorMode::PositionHold];

    /// The next mode round the switch.
    pub fn next(self) -> Self {
        match self {
            RotorMode::Rate => RotorMode::Angle,
            RotorMode::Angle => RotorMode::AltitudeHold,
            RotorMode::AltitudeHold => RotorMode::PositionHold,
            RotorMode::PositionHold => RotorMode::Rate,
        }
    }

    /// How a radio reports it in `RCData::switches` (bits 1 and 2), which
    /// is where `simple_flight` reads its mode from.
    pub fn switch_bits(self) -> u16 {
        match self {
            RotorMode::Rate => 0,
            RotorMode::Angle => 1,
            RotorMode::AltitudeHold => 2,
            RotorMode::PositionHold => 3,
        }
    }

    /// The mode those two switch bits name.
    pub fn from_switch_bits(bits: u16) -> Self {
        match bits & 0b11 {
            0 => RotorMode::Rate,
            1 => RotorMode::Angle,
            2 => RotorMode::AltitudeHold,
            _ => RotorMode::PositionHold,
        }
    }
}

impl From<RotorMode> for FlightMode {
    fn from(mode: RotorMode) -> Self {
        match mode {
            RotorMode::Rate => FlightMode::Manual,
            RotorMode::Angle => FlightMode::Stabilized,
            RotorMode::AltitudeHold => FlightMode::AltitudeHold,
            RotorMode::PositionHold => FlightMode::PositionHold,
        }
    }
}

/// A manoeuvre asked for by a momentary switch rather than held on a
/// stick: the controller flies it and hands the sticks back afterwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Manoeuvre {
    /// Climb to the takeoff altitude and hold.
    Takeoff,
    /// Descend until the ground, then disarm.
    Land,
    /// Fly back to where it was armed.
    ReturnHome,
}

/// What a multirotor is being asked for.
///
/// The four channels are a radio's, in a radio's senses: roll positive to
/// the right, pitch positive *forward* (the stick pushed away, the nose
/// dipped), yaw positive to the right, throttle from closed to open —
/// the same sense [`slot`](crate::input::slot) asks a device to report
/// in, so the presets invert nothing. What `throttle` *means*
/// depends on
/// [`mode`](Self::mode) — collective thrust in [`RotorMode::Rate`] and
/// [`RotorMode::Angle`], a climb rate about half travel in the two hold
/// modes.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RotorDemand {
    /// Roll, `-1.0 ..= 1.0`, positive to the right.
    pub roll: f32,
    /// Pitch, `-1.0 ..= 1.0`, positive nose-up.
    pub pitch: f32,
    /// Yaw rate, `-1.0 ..= 1.0`, positive to the right.
    pub yaw: f32,
    /// Throttle, `0.0 ..= 1.0`.
    pub throttle: f32,
    /// What the sticks mean.
    pub mode: RotorMode,
    /// Whether the arm switch is on. A disarmed craft ignores its sticks.
    pub armed: bool,
    /// A manoeuvre asked for this frame.
    pub manoeuvre: Option<Manoeuvre>,
}

impl RotorDemand {
    /// Centred sticks, throttle closed, disarmed: what a craft is asked
    /// for when nothing is holding the stick.
    pub fn idle() -> Self {
        Self::default()
    }

    /// The demand an AirSim client's `moveByRC` carries.
    ///
    /// AirSim's `RCData` holds the four channels plus a `switches`
    /// bitfield; `simple_flight` reads bit 0 as arm and bits 1 and 2 as
    /// the mode, and that is the reading taken here.
    pub fn from_rc(rc: &RcData) -> Self {
        Self {
            roll: rc.roll.clamp(-1.0, 1.0),
            pitch: rc.pitch.clamp(-1.0, 1.0),
            yaw: rc.yaw.clamp(-1.0, 1.0),
            // A radio's throttle channel is centred on the wire; a
            // collective is not.
            throttle: ((rc.throttle + 1.0) * 0.5).clamp(0.0, 1.0),
            mode: RotorMode::from_switch_bits(rc.switches >> 1),
            armed: rc.get_switch(0) == 1,
            manoeuvre: None,
        }
    }

    /// This demand as an AirSim client would read it back.
    pub fn to_rc(self, at: TimePoint) -> RcData {
        let switches = u16::from(self.armed) | (self.mode.switch_bits() << 1);
        RcData {
            timestamp: at,
            pitch: self.pitch,
            roll: self.roll,
            throttle: self.throttle * 2.0 - 1.0,
            yaw: self.yaw,
            left_z: 0.0,
            right_z: 0.0,
            switches,
            vendor_id: String::from("simkit"),
            is_initialized: true,
            is_valid: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_demand_survives_the_round_trip_through_airsims_rc_data() {
        let want = RotorDemand {
            roll: 0.25,
            pitch: -0.5,
            yaw: 0.75,
            throttle: 0.8,
            mode: RotorMode::AltitudeHold,
            armed: true,
            manoeuvre: None,
        };
        let back = RotorDemand::from_rc(&want.to_rc(7));
        assert_eq!(back, want, "the wire keeps every channel and both switches");
    }

    #[test]
    fn a_disarmed_demand_reports_its_switch_down() {
        let rc = RotorDemand { armed: false, mode: RotorMode::Rate, ..RotorDemand::idle() }.to_rc(0);
        assert_eq!(rc.get_switch(0), 0);
        assert!(!RotorDemand::from_rc(&rc).armed);
    }

    #[test]
    fn the_mode_switch_walks_the_whole_ladder_and_comes_back() {
        let mut mode = RotorMode::Rate;
        for want in RotorMode::LADDER.iter().skip(1).chain(std::iter::once(&RotorMode::Rate)) {
            mode = mode.next();
            assert_eq!(&mode, want);
        }
    }
}
