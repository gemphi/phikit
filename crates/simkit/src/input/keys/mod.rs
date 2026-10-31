//! A keyboard as a stick.
//!
//! A key is on or off, and a drone commanded to full deflection the
//! instant a key goes down is unflyable. So each axis is *ramped* toward
//! where the held keys point and springs back toward rest when they are
//! let go, which is what the stick a key is standing in for would do.
//! The throttle is the exception: a collective lever stays where it is
//! put, so its keys move it and letting go holds it.
//!
//! The keyboard reports itself on the same [`slot`] numbers a gamepad
//! does, so one binding table reads either.

use super::device::{slot, Device, DeviceKind, Reading};

pub mod map;
#[cfg(test)]
mod tests;

pub use map::{Key, KeyMap, Slot};

/// How fast a keyed axis moves, in span units a second.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ramp {
    /// Toward where the keys point.
    pub press: f32,
    /// Back toward rest when nothing is held.
    pub release: f32,
    /// Whether letting go returns the axis to rest. A collective lever
    /// stays put; a stick springs back.
    pub springs_back: bool,
}

impl Ramp {
    /// A sprung stick: quick to deflect, quick to centre.
    pub fn sprung() -> Self {
        Self { press: 4.0, release: 6.0, springs_back: true }
    }

    /// A lever that holds where it is left.
    pub fn held() -> Self {
        Self { press: 1.0, release: 0.0, springs_back: false }
    }
}

/// A keyboard presented as a device.
///
/// A host calls [`press`](Self::press) and [`release`](Self::release) from
/// its key events and [`poll`](Device::poll) once a step.
pub struct Keys {
    id: String,
    map: KeyMap,
    held: Vec<Key>,
    reading: Reading,
    ramps: Vec<Ramp>,
}

impl Keys {
    /// A keyboard on `map`, with sprung sticks and a throttle that holds.
    pub fn new(map: KeyMap) -> Self {
        let mut ramps = vec![Ramp::sprung(); slot::AXES];
        ramps[usize::from(slot::AXIS_THROTTLE)] = Ramp::held();
        Self {
            id: String::from("keyboard"),
            map,
            held: Vec::new(),
            reading: Reading::resting(slot::AXES),
            ramps,
        }
    }

    /// The default flight layout.
    pub fn flight() -> Self {
        Self::new(KeyMap::wasd())
    }

    /// The default driving layout, where nothing springs back but the
    /// wheel.
    pub fn driving() -> Self {
        let mut keys = Self::new(KeyMap::driving());
        keys.ramps[usize::from(slot::AXIS_THROTTLE)] = Ramp::sprung();
        keys.ramps[usize::from(slot::AXIS_BRAKE)] = Ramp::sprung();
        keys
    }

    /// The layout, to remap.
    pub fn map_mut(&mut self) -> &mut KeyMap {
        &mut self.map
    }

    /// How an axis moves, to retune.
    pub fn ramp_mut(&mut self, axis: u8) -> Option<&mut Ramp> {
        self.ramps.get_mut(usize::from(axis))
    }

    /// Notes `key` as held. Pressing a key already held changes nothing,
    /// so a host that repeats key-down events is harmless.
    pub fn press(&mut self, key: Key) {
        if !self.held.contains(&key) {
            self.held.push(key);
        }
        if let Some(Slot::Button(index)) = self.map.slot_of(key) {
            self.reading.set_held(index, true);
        }
    }

    /// Notes `key` as let go.
    pub fn release(&mut self, key: Key) {
        self.held.retain(|k| *k != key);
        if let Some(Slot::Button(index)) = self.map.slot_of(key) {
            self.reading.set_held(index, false);
        }
    }

    /// Lets go of every key, for when a window loses focus — otherwise a
    /// drone keeps the climb it was given and flies away.
    pub fn release_all(&mut self) {
        self.held.clear();
        self.reading.buttons = 0;
    }

    /// Whether `key` is held.
    pub fn is_held(&self, key: Key) -> bool {
        self.held.contains(&key)
    }

    /// Where the held keys are pushing axis `index`, and `None` when
    /// nothing is pushing it.
    fn demand_on(&self, index: u8) -> Option<f32> {
        let pushes: Vec<f32> = self
            .held
            .iter()
            .filter_map(|k| match self.map.slot_of(*k) {
                Some(Slot::Axis { index: i, toward }) if i == index => Some(toward),
                _ => None,
            })
            .collect();
        match pushes.is_empty() {
            true => None,
            // Two keys pushing opposite ways cancel, as a stick held both
            // ways would.
            false => Some((pushes.iter().sum::<f32>() / pushes.len() as f32).clamp(-1.0, 1.0)),
        }
    }
}

impl Device for Keys {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> DeviceKind {
        DeviceKind::Keyboard
    }

    fn poll(&mut self, dt: f32) -> &Reading {
        for index in 0..slot::AXES {
            let axis = index as u8;
            let ramp = self.ramps[index];
            let now = self.reading.axis(axis);
            let (want, rate) = match (self.demand_on(axis), ramp.springs_back) {
                (Some(toward), _) => (toward, ramp.press),
                (None, true) => (0.0, ramp.release),
                (None, false) => (now, 0.0),
            };
            let step = rate * dt;
            let moved = match (want - now).abs() <= step {
                true => want,
                false => now + (want - now).signum() * step,
            };
            self.reading.set_axis(axis, moved.clamp(-1.0, 1.0));
        }
        &self.reading
    }
}
