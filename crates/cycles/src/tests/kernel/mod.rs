// SPDX-License-Identifier: Apache-2.0
mod constant;
mod data;
mod record;
mod tile;

use crate::kernel::*;

#[test]
fn the_kernels_keep_the_numbers_cycles_gives_them() {
    assert_eq!(DeviceKernel::IntegratorInitFromCamera as i32, 0);
    assert_eq!(DeviceKernel::IntegratorShadeSurface as i32, 11);
    assert_eq!(DeviceKernel::IntegratorMegakernel as i32, 18);
    for (index, kernel) in DeviceKernel::INTEGRATOR.iter().enumerate() {
        assert_eq!(*kernel as i32, index as i32);
    }
}
