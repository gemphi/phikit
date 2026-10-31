//! The tables simkit ships: one per shape of device, because the
//! whole difficulty of input is that devices disagree.
//!
//! A host hands a pilot one of these to start from and lets them
//! remap it; AirSim settles the same thing once, in `settings.json`
//! under `RC` (`AirSimSettings::RCSettings`).

use super::{Bind, Bindings, Intent, Source};
use crate::input::device::{slot, DeviceKind};
use crate::input::shape::AxisShape;

impl Bindings {
    /// The table for a two-stick gamepad, and the one a [`Keys`]
    /// keyboard is read through too, since the keyboard reports on the
    /// same [`slot`] numbers.
    ///
    /// Nothing is inverted, because a [`Reading`](crate::input::Reading) already comes in
    /// [`slot`]'s sense — positive up, forward and right. The collective
    /// is the one shape that changes kind: a sprung stick read as a
    /// lever, so a centred stick is half throttle, which in the hold
    /// modes is "stay where you are".
    ///
    /// [`Keys`]: crate::input::Keys
    pub fn gamepad() -> Self {
        Self {
            kind: DeviceKind::Gamepad,
            binds: vec![
                Bind::new(Source::Axis(slot::AXIS_ROLL), Intent::Roll, AxisShape::stick()),
                Bind::new(Source::Axis(slot::AXIS_PITCH), Intent::Pitch, AxisShape::stick()),
                Bind::new(
                    Source::Axis(slot::AXIS_YAW),
                    Intent::Yaw,
                    AxisShape::stick().with_expo(0.2),
                ),
                Bind::new(
                    Source::Axis(slot::AXIS_THROTTLE),
                    Intent::Throttle,
                    AxisShape::stick_as_lever(),
                ),
                Bind::new(
                    Source::Axis(slot::AXIS_BRAKE),
                    Intent::Brake,
                    AxisShape::lever().with_dead_zone(0.05),
                ),
                Bind::new(Source::Axis(slot::AXIS_ROLL), Intent::Steer, AxisShape::stick()),
                Bind::switch(Source::Button(slot::BUTTON_ARM), Intent::Arm),
                Bind::switch(Source::Button(slot::BUTTON_MODE), Intent::NextMode),
                Bind::switch(Source::Button(slot::BUTTON_TAKEOFF), Intent::Takeoff),
                Bind::switch(Source::Button(slot::BUTTON_LAND), Intent::Land),
                Bind::switch(Source::Button(slot::BUTTON_HANDBRAKE), Intent::Handbrake),
                Bind::switch(Source::Button(slot::BUTTON_GEAR_UP), Intent::GearUp),
                Bind::switch(Source::Button(slot::BUTTON_GEAR_DOWN), Intent::GearDown),
            ],
        }
    }

    /// The table for a flight stick: roll and pitch on the stick, yaw on
    /// its twist, and the collective on a lever of its own — reversed,
    /// as most of them report, and with a wider dead zone, because a
    /// twist axis is the first thing on a stick to wear out.
    pub fn flight_stick() -> Self {
        let mut table = Self::gamepad();
        table.kind = DeviceKind::FlightStick;
        table.set_shape(Intent::Yaw, AxisShape::stick().with_dead_zone(0.12).with_expo(0.4));
        table.set_shape(Intent::Throttle, AxisShape::lever().inverted());
        table
    }

    /// The table for a wheel and pedals: steering on its own axis, the
    /// accelerator and brake on pedals, and no expo on the wheel,
    /// because a wheel's travel is already geared.
    pub fn wheel() -> Self {
        Self {
            kind: DeviceKind::Wheel,
            binds: vec![
                Bind::new(
                    Source::Axis(slot::AXIS_STEER),
                    Intent::Steer,
                    AxisShape::stick().with_expo(0.0).with_dead_zone(0.02),
                ),
                Bind::new(Source::Axis(slot::AXIS_THROTTLE), Intent::Throttle, AxisShape::lever()),
                Bind::new(Source::Axis(slot::AXIS_BRAKE), Intent::Brake, AxisShape::lever()),
                Bind::switch(Source::Button(slot::BUTTON_HANDBRAKE), Intent::Handbrake),
                Bind::switch(Source::Button(slot::BUTTON_GEAR_UP), Intent::GearUp),
                Bind::switch(Source::Button(slot::BUTTON_GEAR_DOWN), Intent::GearDown),
            ],
        }
    }
}
