// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/types.h` (`RaySelfPrimitives`, `Ray`,
//! lines 588-610): what a kernel traces through the scene.

use std::sync::OnceLock;

use super::constant;

/// `PRIM_NONE`, `OBJECT_NONE`: no primitive, no object.
fn none(name: &'static str, kept: &'static OnceLock<i32>) -> i32 {
    *kept.get_or_init(|| constant(name).unwrap_or(-1) as i32)
}

/// `RaySelfPrimitives`: what a ray starts from, so that it does not hit
/// that again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RaySelfPrimitive {
    /// The primitive the ray starts from.
    pub prim: i32,
    /// The instance that primitive is part of.
    pub object: i32,
    pub light_prim: i32,
    pub light_object: i32,
}

impl Default for RaySelfPrimitive {
    /// A ray that starts from nothing, as a camera's does.
    fn default() -> Self {
        static PRIM: OnceLock<i32> = OnceLock::new();
        static OBJECT: OnceLock<i32> = OnceLock::new();
        let (prim, object) = (none("PRIM_NONE", &PRIM), none("OBJECT_NONE", &OBJECT));
        RaySelfPrimitive { prim, object, light_prim: prim, light_object: object }
    }
}

/// `Ray`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ray {
    /// `P`: where it starts.
    pub p: [f32; 3],
    /// `D`: where it goes, of length one.
    pub d: [f32; 3],
    /// How far along it the scene starts to count.
    pub tmin: f32,
    /// How far along it the scene stops counting.
    pub tmax: f32,
    /// The moment of the shutter it is sent at, from 0 to 1.
    pub time: f32,
    /// `dP`: how far apart neighbouring pixels' rays start, in one number.
    pub dp: f32,
    /// `dD`: how far apart their directions are.
    pub dd: f32,
    /// `self`.
    pub own: RaySelfPrimitive,
}

#[cfg(test)]
mod test;
