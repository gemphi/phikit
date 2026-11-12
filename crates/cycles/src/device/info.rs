// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/device/device.h` (`DeviceType`,
//! `DeviceTypeMask`, `DeviceInfo`, lines 40-130). Cycles' `DeviceInfo` also
//! holds the denoisers a device supports and the list of a multi device's
//! devices; neither is needed until a second device exists.

/// `DeviceType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DeviceType {
    None = 0,
    Cpu,
    Cuda,
    Multi,
    Optix,
    Hip,
    Hiprt,
    Metal,
    Oneapi,
    Dummy,
    /// Not in Cycles: the wgpu device phiray opens, in the place its GPU
    /// back ends have there.
    Wgpu,
}

impl DeviceType {
    /// `DEVICE_MASK(type)`: `1 << type`.
    pub const fn mask(self) -> u32 {
        1 << (self as u32)
    }
}

/// `KernelOptimizationLevel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelOptimization {
    Off = 0,
    Intersect = 1,
    Full = 2,
}

/// `DeviceInfo`.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceInfo {
    pub kind: DeviceType,
    pub description: String,
    /// Fixed across hardware changes: what a user's choice is kept by.
    pub id: String,
    pub num: i32,
    pub display_device: bool,
    pub has_nanovdb: bool,
    /// `has_mnee_`.
    pub has_mnee: bool,
    pub has_osl: bool,
    pub has_guiding: bool,
    pub has_profiling: bool,
    pub has_peer_memory: bool,
    /// The device runs a wavefront queue (GPU); the CPU runs a megakernel.
    pub has_gpu_queue: bool,
    pub use_hardware_raytracing: bool,
    pub kernel_optimization: KernelOptimization,
    pub cpu_threads: i32,
    pub error_msg: String,
}

impl Default for DeviceInfo {
    /// `DeviceInfo()`: the CPU.
    fn default() -> Self {
        Self {
            kind: DeviceType::Cpu,
            description: String::new(),
            id: "CPU".into(),
            num: 0,
            display_device: false,
            has_nanovdb: false,
            has_mnee: true,
            has_osl: false,
            has_guiding: false,
            has_profiling: false,
            has_peer_memory: false,
            has_gpu_queue: false,
            use_hardware_raytracing: false,
            kernel_optimization: KernelOptimization::Full,
            cpu_threads: 0,
            error_msg: String::new(),
        }
    }
}

#[cfg(test)]
mod test;
