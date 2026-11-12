// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of `src/kernel`, one file or folder for each there; `rig.rs` is
//! the joined scene the CPU kernels are run on.
mod bit;
mod bvh;
mod camera;
mod closure;
mod constant;
mod data;
mod device;
mod film;
mod geom;
mod global;
mod integrator;
mod intersection;
mod ray;
mod record;
pub(crate) mod rig;
mod sample;
mod svm;
mod tile;
mod util;
pub(crate) use rig::rig_scene;
use crate::kernel::*;
/// Two lists of floats that agree to four places.
pub(crate) fn near(left: &[f32], right: &[f32]) -> bool {
    left.len() == right.len() && left.iter().zip(right).all(|(left, right)| (left - right).abs() < 1e-4)
}
#[test]
fn the_kernels_keep_the_numbers_cycles_gives_them() {
    assert_eq!(DeviceKernel::IntegratorInitFromCamera as i32, 0);
    assert_eq!(DeviceKernel::IntegratorShadeSurface as i32, 11);
    assert_eq!(DeviceKernel::IntegratorMegakernel as i32, 18);
    for (index, kernel) in DeviceKernel::INTEGRATOR.iter().enumerate() {
        assert_eq!(*kernel as i32, index as i32);
    }
}
