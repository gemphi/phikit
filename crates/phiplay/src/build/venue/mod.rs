//! The venue round the field: its stepped bowl of terraces and the seats
//! on them (`bowl`), the shell the bowl is wrapped in and its roof
//! (`shell`), and its lights — the lamps over the field, the façade's light
//! groups that run the show, the LED boards (`light`).
//!
//! Everything is laid along rings round the floor (`ring::BRing`), so one
//! venue fits a basketball court or a soccer pitch: the bowl starts where
//! the floor ends, and the shell where the bowl does.

pub mod bowl;
pub mod light;
pub mod shell;

use super::mesh::BMesh;
use super::part::{BMat, BPart};
use crate::choice::BChoice;
use glam::Mat4;
use std::sync::Arc;

/// One mesh drawn many times: a seat colour's seats, a crowd variant's
/// fans. Phiber shares the mesh between them and leaves them out of the
/// Outliner, which lists the venue, not seven thousand chairs.
#[derive(Clone, Debug)]
pub struct BInstances {
    pub name: String,
    pub mesh: Arc<BMesh>,
    pub material: Arc<BMat>,
    pub at: Vec<Mat4>,
}

/// A place to sit: where the pan's top is and which way the seat faces
/// (the angle about z that turns −y toward the field).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPlace {
    pub at: glam::Vec3,
    pub heading: f32,
    pub tier: u8,
}

/// A group of the façade's lights: one material, lit together, at `s` of
/// the way round and `t` of the way up — where the show reads its level.
#[derive(Clone, Debug, PartialEq)]
pub struct BLightGroup {
    pub material: String,
    pub s: f32,
    pub t: f32,
    /// Its emission at full level.
    pub strength: f32,
}

/// A lamp lighting the field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BLamp {
    /// A rectangle `size` across, at `at` (pointing down its −z), of
    /// `power` watts.
    Area { at: Mat4, size: [f32; 2], colour: crate::colour::Rgb, power: f32 },
    /// The sun, from `at`'s −z, `strength` W/m².
    Sun { at: Mat4, colour: crate::colour::Rgb, strength: f32 },
}

/// The built venue.
#[derive(Clone, Debug, Default)]
pub struct BVenueBuild {
    pub parts: Vec<BPart>,
    pub seats: Vec<BInstances>,
    pub places: Vec<BPlace>,
    /// The fans in them (`crowd`).
    pub crowd: Vec<BInstances>,
    pub groups: Vec<BLightGroup>,
    pub lamps: Vec<BLamp>,
}

/// The venue the choice picks, round the sport's field.
pub fn venue(choice: &BChoice) -> BVenueBuild {
    let mut out = BVenueBuild::default();
    let bowl = bowl::bowl(choice, &mut out);
    shell::shell(choice, &bowl, &mut out);
    light::lights(choice, &bowl, &mut out);
    out.crowd = super::crowd::crowd(choice, &out.places);
    out
}

#[cfg(test)]
mod tests;
