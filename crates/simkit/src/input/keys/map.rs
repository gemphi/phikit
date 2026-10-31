//! Which key sits in which [`slot`]: the keyboard's own remapping
//! layer, below the binding table and independent of it.

use crate::input::device::slot;

/// A key, as a host reports it. Letters and digits come through as
/// [`Char`](Key::Char) in lower case; the rest are named.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A printable key, lower case.
    Char(char),
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Space.
    Space,
    /// Return.
    Enter,
    /// Tab.
    Tab,
    /// Either shift.
    Shift,
    /// Either control.
    Ctrl,
    /// Either alt.
    Alt,
    /// Escape.
    Escape,
}

/// What a key does to the reading: push an axis one way, or hold a button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Slot {
    /// Push axis `index` toward `toward` while the key is held.
    Axis {
        /// The axis number, from [`slot`].
        index: u8,
        /// Where the axis is being pushed, usually `-1.0` or `1.0`.
        toward: f32,
    },
    /// Hold button `index` while the key is held.
    Button(u8),
}

/// Which key sits in which slot. This is the keyboard's own remapping
/// layer; the binding table above it is unchanged.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyMap {
    /// Key to slot, in the order a host's key events are matched.
    pub entries: Vec<(Key, Slot)>,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self::wasd()
    }
}

impl KeyMap {
    /// The usual flight layout: the arrows are the right stick (roll and
    /// pitch), `A`/`D` yaw, `W`/`S` the collective, space arms, `M` steps
    /// the mode switch, `T` takes off and `L` lands.
    pub fn wasd() -> Self {
        Self {
            entries: vec![
                (Key::Left, Slot::Axis { index: slot::AXIS_ROLL, toward: -1.0 }),
                (Key::Right, Slot::Axis { index: slot::AXIS_ROLL, toward: 1.0 }),
                (Key::Up, Slot::Axis { index: slot::AXIS_PITCH, toward: 1.0 }),
                (Key::Down, Slot::Axis { index: slot::AXIS_PITCH, toward: -1.0 }),
                (Key::Char('a'), Slot::Axis { index: slot::AXIS_YAW, toward: -1.0 }),
                (Key::Char('d'), Slot::Axis { index: slot::AXIS_YAW, toward: 1.0 }),
                (Key::Char('w'), Slot::Axis { index: slot::AXIS_THROTTLE, toward: 1.0 }),
                (Key::Char('s'), Slot::Axis { index: slot::AXIS_THROTTLE, toward: -1.0 }),
                (Key::Space, Slot::Button(slot::BUTTON_ARM)),
                (Key::Char('m'), Slot::Button(slot::BUTTON_MODE)),
                (Key::Char('t'), Slot::Button(slot::BUTTON_TAKEOFF)),
                (Key::Char('l'), Slot::Button(slot::BUTTON_LAND)),
            ],
        }
    }

    /// The driving layout: the arrows steer and work the pedals, space is
    /// the handbrake, `E` and `Q` shift.
    pub fn driving() -> Self {
        Self {
            entries: vec![
                (Key::Left, Slot::Axis { index: slot::AXIS_STEER, toward: -1.0 }),
                (Key::Right, Slot::Axis { index: slot::AXIS_STEER, toward: 1.0 }),
                (Key::Up, Slot::Axis { index: slot::AXIS_THROTTLE, toward: 1.0 }),
                (Key::Down, Slot::Axis { index: slot::AXIS_BRAKE, toward: 1.0 }),
                (Key::Space, Slot::Button(slot::BUTTON_HANDBRAKE)),
                (Key::Char('e'), Slot::Button(slot::BUTTON_GEAR_UP)),
                (Key::Char('q'), Slot::Button(slot::BUTTON_GEAR_DOWN)),
            ],
        }
    }

    /// Where `key` points, if anywhere.
    pub fn slot_of(&self, key: Key) -> Option<Slot> {
        self.entries.iter().find(|(k, _)| *k == key).map(|(_, s)| *s)
    }
}
