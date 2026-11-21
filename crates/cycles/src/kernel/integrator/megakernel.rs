// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/integrator/megakernel.h`
//! (`integrator_megakernel`, lines 22-119): each kernel names the kernel the
//! path goes to next, so the loop reads what is queued and runs it until
//! nothing is. A shadow path or an ambient occlusion path is handled
//! before the main path creates more of them; there are none until lights.
use super::intersect::integrator_intersect_closest;
use super::shade::integrator_shade_background;
use super::shade::integrator_shade_surface;
use crate::kernel::integrator::state::IntegratorState;
use crate::kernel::DeviceKernel;
use crate::kernel::KernelGlobal;

/// `integrator_megakernel`. `buffer` is the floats of the path's pixel.
/// A queued kernel that is not ported ends the path.
pub fn integrator_megakernel(kg: &KernelGlobal, state: &mut IntegratorState, buffer: &mut [f32]) {
    loop {
        // A shadow path would be handled here first. None is made before
        // the lights, so a queued one ends the loop.
        if state.shadow.queued_kernel != 0 || state.ao.queued_kernel != 0 {
            break;
        }
        let queued = state.path.queued_kernel;
        if queued == 0 {
            break;
        }
        let kernel = DeviceKernel::INTEGRATOR.iter().copied().find(|kernel| *kernel as u32 == queued);
        match kernel {
            Some(DeviceKernel::IntegratorIntersectClosest) => integrator_intersect_closest(kg, state),
            Some(DeviceKernel::IntegratorShadeBackground) => integrator_shade_background(kg, state, buffer),
            Some(DeviceKernel::IntegratorShadeSurface) => integrator_shade_surface(kg, state, buffer),
            // The other kernels wait for volumes, lights, subsurface and MNEE.
            _ => break,
        }
    }
}
