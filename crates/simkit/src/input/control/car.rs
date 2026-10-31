//! What a car is asked for: AirSim's `CarApiBase::CarControls`, with
//! the gear spelled out rather than carried as a flag and an integer.

/// Which way the gearbox is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Gear {
    /// Reverse.
    Reverse,
    /// Out of gear: the throttle drives nothing.
    #[default]
    Neutral,
    /// A forward gear, from `1`.
    Forward(u8),
}

impl Gear {
    /// The way the wheels turn in this gear: `1.0` forward, `-1.0` back,
    /// `0.0` in neutral.
    pub fn sense(self) -> f32 {
        match self {
            Gear::Reverse => -1.0,
            Gear::Neutral => 0.0,
            Gear::Forward(_) => 1.0,
        }
    }

    /// How AirSim's `CarControls` numbers it: negative for reverse, `0`
    /// for neutral, positive for a forward gear.
    pub fn as_manual_gear(self) -> i32 {
        match self {
            Gear::Reverse => -1,
            Gear::Neutral => 0,
            Gear::Forward(n) => i32::from(n),
        }
    }

    /// The gear that number names.
    pub fn from_manual_gear(gear: i32) -> Self {
        match gear {
            0 => Gear::Neutral,
            g if g < 0 => Gear::Reverse,
            g => Gear::Forward(g.min(255) as u8),
        }
    }

    /// The next gear up, stopping at `top`.
    pub fn up(self, top: u8) -> Self {
        match self {
            Gear::Reverse => Gear::Neutral,
            Gear::Neutral => Gear::Forward(1),
            Gear::Forward(n) => Gear::Forward(n.saturating_add(1).min(top.max(1))),
        }
    }

    /// The next gear down.
    pub fn down(self) -> Self {
        match self {
            Gear::Reverse => Gear::Reverse,
            Gear::Neutral => Gear::Reverse,
            Gear::Forward(1) => Gear::Neutral,
            Gear::Forward(n) => Gear::Forward(n - 1),
        }
    }
}

/// What a car is being asked for: AirSim's `CarApiBase::CarControls`,
/// with the gear spelled out rather than carried as a flag and an integer.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CarDemand {
    /// Steering, `-1.0 ..= 1.0`, positive to the right.
    pub steer: f32,
    /// Throttle, `0.0 ..= 1.0`.
    pub throttle: f32,
    /// Brake, `0.0 ..= 1.0`.
    pub brake: f32,
    /// Whether the handbrake is pulled.
    pub handbrake: bool,
    /// The gear.
    pub gear: Gear,
}

impl CarDemand {
    /// Hands off the wheel, off the throttle, off the brake.
    pub fn idle() -> Self {
        Self { gear: Gear::Forward(1), ..Self::default() }
    }

    /// The demand an AirSim client's `setCarControls` carries.
    pub fn from_car_controls(
        throttle: f32,
        steering: f32,
        brake: f32,
        handbrake: bool,
        is_manual_gear: bool,
        manual_gear: i32,
    ) -> Self {
        Self {
            steer: steering.clamp(-1.0, 1.0),
            throttle: throttle.clamp(0.0, 1.0),
            brake: brake.clamp(0.0, 1.0),
            handbrake,
            gear: match is_manual_gear {
                true => Gear::from_manual_gear(manual_gear),
                false => Gear::Forward(1),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gears_shift_and_number_as_airsim_numbers_them() {
        assert_eq!(Gear::Neutral.up(5), Gear::Forward(1));
        assert_eq!(Gear::Forward(5).up(5), Gear::Forward(5), "the top gear is the top");
        assert_eq!(Gear::Forward(1).down(), Gear::Neutral);
        assert_eq!(Gear::Neutral.down(), Gear::Reverse);
        assert_eq!(Gear::Reverse.down(), Gear::Reverse, "there is one reverse");
        assert_eq!(Gear::Reverse.sense(), -1.0);
        assert_eq!(Gear::Neutral.sense(), 0.0);
        assert_eq!(Gear::from_manual_gear(Gear::Reverse.as_manual_gear()), Gear::Reverse);
        assert_eq!(Gear::from_manual_gear(Gear::Forward(3).as_manual_gear()), Gear::Forward(3));
    }

    #[test]
    fn car_controls_off_the_wire_clamp_and_pick_up_a_gear() {
        let d = CarDemand::from_car_controls(1.5, -2.0, 0.5, true, false, 0);
        assert_eq!(d.throttle, 1.0);
        assert_eq!(d.steer, -1.0);
        assert!(d.handbrake);
        assert_eq!(d.gear, Gear::Forward(1), "automatic means a forward gear");
        let m = CarDemand::from_car_controls(0.0, 0.0, 0.0, false, true, -1);
        assert_eq!(m.gear, Gear::Reverse);
    }
}
