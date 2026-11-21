// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! What a device is asked for: `intern/cycles/device`. The contract
//! (`device.rs`, `memory.rs`, `queue.rs`, `argument.rs`, `info.rs`), the
//! CPU device that runs kernels in Rust (`cpu/`), and the denoiser's
//! settings the scheduler plans around. The wgpu device is phiray's gpu
//! layer, which implements the same [`Device`]; see `README.md`.
mod argument;
pub mod cpu;
mod denoise;
mod device;
mod info;
mod memory;
mod queue;
pub use argument::KernelArgument;
pub use cpu::{CpuDevice, CpuKernelFn, CpuKernel, CpuMemory};
pub use denoise::{DenoiseParam, DenoiserPass, DenoiserPrefilter, DenoiserQuality, DenoiserType};
pub use device::{Device, DeviceError};
pub use info::{DeviceInfo, DeviceType, KernelOptimization};
pub use memory::{DevicePtr, DeviceVector, MemoryType};
pub use queue::DeviceQueue;
