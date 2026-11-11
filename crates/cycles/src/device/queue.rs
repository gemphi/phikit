// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/device/queue.h` (`DeviceQueue`): kernels
//! are enqueued over a work size with their arguments and run in order;
//! `synchronize` returns when they have all finished. Cycles' CPU has no
//! queue (its `PathTraceWorkCPU` calls the kernel functions directly);
//! this one exists so the same call runs on the CPU and on wgpu.
use super::{DeviceError, KernelArgument};
use crate::kernel::DeviceKernel;

/// `DeviceQueue`.
pub trait DeviceQueue: Send {
    /// `enqueue`: run `kernel` over `work_size` items with `args`. The
    /// kernel may not have run when this returns; an error that a later
    /// kernel runs into is returned by `synchronize` instead.
    fn enqueue(&mut self, kernel: DeviceKernel, work_size: i32, args: &[KernelArgument]) -> Result<(), DeviceError>;
    /// `synchronize`: wait for every kernel enqueued; the first error.
    fn synchronize(&mut self) -> Result<(), DeviceError>;
}
