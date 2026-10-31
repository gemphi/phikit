//! The remappable binding table: which bit of a device asks for which
//! part of the control state, and how that bit is shaped on the way.
//!
//! One table per device, because the whole difficulty is that devices
//! disagree. A gamepad's collective is a sprung stick that has to be read
//! as a lever; a flight stick's is a lever already, and often reversed; a
//! wheel reports steering on an axis of its own and the brake on a pedal.
//! AirSim settles this in `settings.json` under `RC`
//! (`AirSimSettings::RCSettings`, with its `allow_api_when_disconnected`
//! and per-channel remap); this is the same idea as a value, so a host can
//! hand a pilot a table to edit.

use super::device::{DeviceKind, Reading};
use super::shape::AxisShape;

/// Where on a device a demand comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// An axis, by number.
    Axis(u8),
    /// A button, by number.
    Button(u8),
    /// Two buttons read as one axis: a hat's two edges, a keyboard's left
    /// and right, a pair of shift paddles.
    Pair {
        /// The button that drives the axis negative.
        minus: u8,
        /// The button that drives it positive.
        plus: u8,
    },
}

impl Source {
    /// This source's raw value on `reading`, before any shaping. A
    /// button reads `1.0` held and `0.0` free.
    pub fn raw(self, reading: &Reading) -> f32 {
        match self {
            Source::Axis(i) => reading.axis(i),
            Source::Button(i) => f32::from(reading.held(i)),
            Source::Pair { minus, plus } => {
                f32::from(reading.held(plus)) - f32::from(reading.held(minus))
            }
        }
    }

    /// Whether this source is held, for the switch intents. An axis
    /// counts as held past half its travel, so a trigger can arm.
    pub fn is_held(self, reading: &Reading) -> bool {
        match self {
            Source::Axis(i) => reading.axis(i).abs() > 0.5,
            Source::Button(i) => reading.held(i),
            Source::Pair { plus, .. } => reading.held(plus),
        }
    }
}

/// What a bound source asks for.
///
/// The first group are axes and carry a value every frame. The rest are
/// switches: [`Arm`](Intent::Arm) and [`Handbrake`](Intent::Handbrake) are
/// read while held, the others fire once on the press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    /// Multirotor roll.
    Roll,
    /// Multirotor pitch.
    Pitch,
    /// Multirotor yaw rate.
    Yaw,
    /// Multirotor collective, or a car's accelerator.
    Throttle,
    /// Car steering.
    Steer,
    /// Car brake.
    Brake,
    /// Arm switch, read while held.
    Arm,
    /// Handbrake, read while held.
    Handbrake,
    /// Step the mode switch round, on the press.
    NextMode,
    /// Ask for a takeoff, on the press.
    Takeoff,
    /// Ask for a landing, on the press.
    Land,
    /// Shift up, on the press.
    GearUp,
    /// Shift down, on the press.
    GearDown,
}

/// One row of the table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bind {
    /// Where it comes from.
    pub source: Source,
    /// What it asks for.
    pub intent: Intent,
    /// How it is shaped on the way.
    pub shape: AxisShape,
}

impl Bind {
    /// A row.
    pub fn new(source: Source, intent: Intent, shape: AxisShape) -> Self {
        Self { source, intent, shape }
    }

    /// A switch row, which needs no shaping.
    pub fn switch(source: Source, intent: Intent) -> Self {
        Self { source, intent, shape: AxisShape::lever().with_dead_zone(0.0) }
    }
}

/// A device's whole table.
#[derive(Debug, Clone, PartialEq)]
pub struct Bindings {
    /// What kind of device this table is for.
    pub kind: DeviceKind,
    /// The rows, in no particular order; one per intent is the usual
    /// case, but nothing stops two sources asking for the same thing.
    pub binds: Vec<Bind>,
}

mod presets;
#[cfg(test)]
mod tests;

impl Bindings {
    /// An empty table for `kind`.
    pub fn empty(kind: DeviceKind) -> Self {
        Self { kind, binds: Vec::new() }
    }

    /// The row for `intent`, if there is one.
    pub fn bind(&self, intent: Intent) -> Option<&Bind> {
        self.binds.iter().find(|b| b.intent == intent)
    }

    /// Where `intent` is read from.
    pub fn source_of(&self, intent: Intent) -> Option<Source> {
        self.bind(intent).map(|b| b.source)
    }

    /// `intent`'s value this frame, shaped, or `0.0` if it is unbound.
    pub fn value(&self, intent: Intent, reading: &Reading) -> f32 {
        match self.bind(intent) {
            Some(b) => b.shape.apply(b.source.raw(reading)),
            None => 0.0,
        }
    }

    /// Whether `intent`'s switch is held this frame.
    pub fn is_held(&self, intent: Intent, reading: &Reading) -> bool {
        match self.bind(intent) {
            Some(b) => b.source.is_held(reading),
            None => false,
        }
    }

    /// Points `intent` at `source`, keeping its shaping. Adds the row if
    /// the intent was unbound. This is the remap a pilot does.
    pub fn remap(&mut self, intent: Intent, source: Source) {
        match self.binds.iter_mut().find(|b| b.intent == intent) {
            Some(b) => b.source = source,
            None => self.binds.push(Bind::switch(source, intent)),
        }
    }

    /// Reshapes `intent`, keeping its source.
    pub fn set_shape(&mut self, intent: Intent, shape: AxisShape) {
        match self.binds.iter_mut().find(|b| b.intent == intent) {
            Some(b) => b.shape = shape,
            None => self.binds.push(Bind::new(Source::Axis(0), intent, shape)),
        }
    }

    /// `intent`'s shaping, to tune a dead zone, an expo or a trim in
    /// place — the three knobs a pilot reaches for first.
    pub fn shape_mut(&mut self, intent: Intent) -> Option<&mut AxisShape> {
        self.binds.iter_mut().find(|b| b.intent == intent).map(|b| &mut b.shape)
    }

    /// Takes `intent`'s row out of the table, so nothing asks for it.
    pub fn unbind(&mut self, intent: Intent) -> Option<Bind> {
        let at = self.binds.iter().position(|b| b.intent == intent)?;
        Some(self.binds.remove(at))
    }
}

impl Default for Bindings {
    fn default() -> Self {
        Self::gamepad()
    }
}
