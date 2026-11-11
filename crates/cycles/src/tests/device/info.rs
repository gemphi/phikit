// SPDX-License-Identifier: Apache-2.0
//! Tests of `device/info.rs`.
use crate::device::{DeviceInfo, DeviceType, KernelOptimization};

#[test]
fn the_default_info_is_the_cpu() {
    let info = DeviceInfo::default();
    assert_eq!(info.kind, DeviceType::Cpu);
    assert_eq!(info.id, "CPU");
    assert!(info.has_mnee);
    assert_eq!(info.kernel_optimization, KernelOptimization::Full);
}

#[test]
fn the_type_mask_is_one_shifted_by_the_type() {
    // `device.h:40-52`: DEVICE_MASK_CPU = 1 << DEVICE_CPU, and so on.
    assert_eq!(DeviceType::Cpu.mask(), 1 << 1);
    assert_eq!(DeviceType::Cuda.mask(), 1 << 2);
    assert_eq!(DeviceType::Optix.mask(), 1 << 4);
    assert_eq!(DeviceType::Metal.mask(), 1 << 7);
}
