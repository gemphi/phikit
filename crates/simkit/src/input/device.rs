//! The seam a device backend plugs in at.
//!
//! Nothing in simkit reads a USB descriptor. A backend — `gilrs`, SDL,
//! a window library's gamepad support, a real radio over a serial link —
//! implements [`Device`] and hands over a [`Reading`]: axes as floats,
//! buttons as bits, and a timestamp. Everything above this line works on
//! `Reading`s, so the mapping, the binding table and the whole phiecs
//! bridge are testable without a device in the room.
//!
//! Axis and button *numbers* are a device's own business, which is why
//! there is a binding table. But the presets and the keyboard agree on the
//! layout in [`slot`], so a keyboard can stand in for a gamepad without a
//! second table.

use crate::common::TimePoint;

/// The axis and button numbers simkit's own binding presets and its
/// keyboard speak.
///
/// The sense is the pilot's: **positive is up, forward or right**. So
/// pushing the pitch axis positive dips the nose and flies forward, and
/// opening the throttle axis raises the collective.
///
/// Real hardware disagrees among itself about this — DirectInput and SDL
/// report a stick pushed away as *negative*, `gilrs` as positive — which
/// is what [`AxisShape::invert`](super::AxisShape::invert) is for. A
/// backend either fills a [`Reading`] in this sense or binds the axis
/// inverted; either way the reconciling happens once, in
/// [`Bindings`](super::Bindings), and not again.
pub mod slot {
    /// Roll: a gamepad's right stick, across.
    pub const AXIS_ROLL: u8 = 0;
    /// Pitch: a gamepad's right stick, fore and aft.
    pub const AXIS_PITCH: u8 = 1;
    /// Yaw: a gamepad's left stick across, or a stick's twist.
    pub const AXIS_YAW: u8 = 2;
    /// Throttle: a gamepad's left stick fore and aft, or a lever.
    pub const AXIS_THROTTLE: u8 = 3;
    /// Brake: a gamepad's left trigger, or a pedal.
    pub const AXIS_BRAKE: u8 = 4;
    /// Steering, for a wheel that reports it on its own axis.
    pub const AXIS_STEER: u8 = 5;

    /// How many axes a preset reading carries.
    pub const AXES: usize = 6;

    /// Arm / disarm.
    pub const BUTTON_ARM: u8 = 0;
    /// Step the mode switch round.
    pub const BUTTON_MODE: u8 = 1;
    /// Take off.
    pub const BUTTON_TAKEOFF: u8 = 2;
    /// Land.
    pub const BUTTON_LAND: u8 = 3;
    /// Handbrake.
    pub const BUTTON_HANDBRAKE: u8 = 4;
    /// Shift up.
    pub const BUTTON_GEAR_UP: u8 = 5;
    /// Shift down.
    pub const BUTTON_GEAR_DOWN: u8 = 6;
}

/// What kind of thing is being held, so a host can pick a preset and a
/// log can say what flew it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeviceKind {
    /// A keyboard standing in for a stick.
    #[default]
    Keyboard,
    /// A two-stick gamepad.
    Gamepad,
    /// A flight stick, with the throttle on its own lever.
    FlightStick,
    /// A wheel and pedals.
    Wheel,
}

/// One frame off a device, with nothing interpreted.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Reading {
    /// Axis positions, in whatever span the device reports on. The
    /// binding's [`AxisShape`](super::AxisShape) says which.
    pub axes: Vec<f32>,
    /// One bit per button, set while held.
    pub buttons: u64,
    /// When the frame was taken.
    pub at: TimePoint,
}

impl Reading {
    /// A reading with `axes` axes, all at rest and no button held.
    pub fn resting(axes: usize) -> Self {
        Self { axes: vec![0.0; axes], buttons: 0, at: 0 }
    }

    /// The position of axis `index`, or `0.0` if the device has no such
    /// axis — a binding for an axis a device does not have asks for
    /// nothing rather than failing.
    pub fn axis(&self, index: u8) -> f32 {
        self.axes.get(usize::from(index)).copied().unwrap_or(0.0)
    }

    /// Whether button `index` is held.
    pub fn held(&self, index: u8) -> bool {
        match index < 64 {
            true => self.buttons & (1 << index) != 0,
            false => false,
        }
    }

    /// Sets axis `index`, growing the reading if it is short.
    pub fn set_axis(&mut self, index: u8, value: f32) {
        let i = usize::from(index);
        match i < self.axes.len() {
            true => self.axes[i] = value,
            false => {
                self.axes.resize(i + 1, 0.0);
                self.axes[i] = value;
            }
        }
    }

    /// Holds or releases button `index`.
    pub fn set_held(&mut self, index: u8, held: bool) {
        match (index < 64, held) {
            (true, true) => self.buttons |= 1 << index,
            (true, false) => self.buttons &= !(1 << index),
            (false, _) => {}
        }
    }

    /// The same reading with an axis set, for building one inline.
    pub fn with_axis(mut self, index: u8, value: f32) -> Self {
        self.set_axis(index, value);
        self
    }

    /// The same reading with a button held, for building one inline.
    pub fn with_held(mut self, index: u8) -> Self {
        self.set_held(index, true);
        self
    }
}

/// Something that can be read for a control input.
///
/// A backend owns the device and the platform call that reads it;
/// `poll` is given the step so a backend that has to synthesise smooth
/// motion from on-or-off keys can do it.
pub trait Device: Send {
    /// What to call this device in a log or a binding file.
    fn id(&self) -> &str;

    /// What kind of thing it is.
    fn kind(&self) -> DeviceKind;

    /// This step's reading.
    fn poll(&mut self, dt: f32) -> &Reading;

    /// Whether the device is still there. A backend that can tell says
    /// so; one that cannot says it is.
    fn connected(&self) -> bool {
        true
    }
}

/// A device that plays back readings frame by frame.
///
/// This is how a flight is scripted in a test or replayed from a
/// recording: the last frame is held once the script runs out, so a
/// script does not have to cover every step of a simulation.
pub struct Scripted {
    id: String,
    kind: DeviceKind,
    frames: Vec<Reading>,
    next: usize,
    held: Reading,
}

impl Scripted {
    /// A device that plays `frames` and then holds the last of them.
    pub fn new(id: impl Into<String>, frames: Vec<Reading>) -> Self {
        Self {
            id: id.into(),
            kind: DeviceKind::Gamepad,
            held: frames.first().cloned().unwrap_or_else(|| Reading::resting(slot::AXES)),
            frames,
            next: 0,
        }
    }

    /// The same device reporting as `kind`.
    pub fn as_kind(mut self, kind: DeviceKind) -> Self {
        self.kind = kind;
        self
    }

    /// Whether every scripted frame has been handed out.
    pub fn spent(&self) -> bool {
        self.next >= self.frames.len()
    }
}

impl Device for Scripted {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> DeviceKind {
        self.kind
    }

    fn poll(&mut self, _dt: f32) -> &Reading {
        if let Some(frame) = self.frames.get(self.next) {
            self.held = frame.clone();
            self.next += 1;
        }
        &self.held
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reading_answers_for_axes_and_buttons_it_does_not_have() {
        let r = Reading::resting(2).with_axis(1, 0.5).with_held(3);
        assert_eq!(r.axis(1), 0.5);
        assert_eq!(r.axis(9), 0.0, "an unbound axis asks for nothing");
        assert!(r.held(3));
        assert!(!r.held(4));
        assert!(!r.held(200), "and a button past the bitset is not held");
    }

    #[test]
    fn a_script_runs_out_by_holding_its_last_frame() {
        let mut d = Scripted::new(
            "stick",
            vec![Reading::resting(1).with_axis(0, 0.1), Reading::resting(1).with_axis(0, 0.9)],
        );
        assert_eq!(d.poll(0.1).axis(0), 0.1);
        assert_eq!(d.poll(0.1).axis(0), 0.9);
        assert!(d.spent());
        assert_eq!(d.poll(0.1).axis(0), 0.9, "the last frame is held, not dropped to rest");
    }
}
