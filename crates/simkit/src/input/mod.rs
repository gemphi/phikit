//! The game-control layer: a gamepad, a flight stick or a keyboard turned
//! into the demand a vehicle is actually commanded with.
//!
//! # The shape of it
//!
//! ```text
//!   a device            a table              a demand
//!   ─────────           ───────              ────────
//!   Device::poll  ──▶   Bindings      ──▶    ControlState
//!   -> Reading          + AxisShape          (RotorDemand | CarDemand)
//!   (axes, buttons)     via Pilot
//! ```
//!
//! Four pieces, each testable on its own:
//!
//! - [`Device`] is the seam a backend plugs in at. It reports a
//!   [`Reading`]: axes as floats and buttons as bits, nothing
//!   interpreted. [`Keys`] is the keyboard implementation and [`Scripted`]
//!   plays a flight back frame by frame.
//! - [`Bindings`] is the remappable table saying which axis or button
//!   asks for which [`Intent`].
//! - [`AxisShape`] is the shaping each row gets on the way: a dead zone,
//!   an expo curve, a scale, a trim, an inversion, and a change of
//!   [`Span`] when a sprung stick has to read as a lever.
//! - [`Pilot`] puts those together and keeps what a switch has to
//!   remember between frames, producing a [`ControlState`].
//!
//! # Why a control state and not a button code
//!
//! A [`ControlState`] is what a *vehicle* is commanded with, not what a
//! device reports: roll, pitch, a yaw rate, a throttle and a mode switch
//! for a multirotor; a steer, a throttle, a brake, a handbrake and a gear
//! for a car. That is deliberate, and it is how AirSim draws the same
//! line — `RCData` and `MultirotorApiBase::moveByRC` on one side,
//! `CarApiBase::CarControls` on the other, with the device's own numbering
//! settled in `settings.json` before any of it. Because the demand is
//! device-independent, the same struct arrives whether a stick produced it
//! locally or an AirSim client sent it over msgpack-rpc, and
//! [`ecs`](crate::ecs) can take the one and only one path to the vehicle.
//!
//! # The device backend
//!
//! simkit reads no USB. Nothing in phikit's lockfile does gamepad input —
//! no `gilrs`, no `sdl2`, no `winit` — so rather than pull a platform
//! dependency in unasked, this module defines [`Device`] and ships the
//! keyboard. A backend is about thirty lines: poll the library, fill a
//! [`Reading`]'s axes and buttons, hand it back. The [`slot`] numbers say
//! which axis is which if you want the presets to read it unchanged.

pub mod binding;
pub mod control;
pub mod device;
pub mod keys;
pub mod pilot;
pub mod shape;

pub use binding::{Bind, Bindings, Intent, Source};
pub use control::{
    CarDemand, ControlState, Gear, Manoeuvre, RotorDemand, RotorMode, VehicleKind,
};
pub use device::{slot, Device, DeviceKind, Reading, Scripted};
pub use keys::{Key, KeyMap, Keys, Ramp, Slot};
pub use pilot::Pilot;
pub use shape::{AxisShape, Span};
