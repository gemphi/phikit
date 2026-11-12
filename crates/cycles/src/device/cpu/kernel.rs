// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/device/cpu/kernel.h` (`CPUKernels`): the
//! table of kernel functions a CPU device calls. Cycles fills it with the
//! function pointers of the best instruction set; here each entry is a
//! Rust function over the device's blocks.
//!
//! No kernel body is ported yet (gate 1 begins with them). A kernel with no
//! body returns [`DeviceError::NotPorted`], so a render that reaches one
//! fails with a typed error and not a panic.
use std::sync::Arc;

use super::CpuMemory;
use crate::device::{DeviceError, KernelArgument};
use crate::kernel::DeviceKernel;

/// A kernel on the CPU: the blocks, the work size, the arguments.
pub type CpuKernelFn = Arc<dyn Fn(&mut CpuMemory, i32, &[KernelArgument]) -> Result<(), DeviceError> + Send + Sync>;

/// `CPUKernels`.
#[derive(Clone)]
pub struct CpuKernel {
    table: Vec<Option<CpuKernelFn>>,
}

impl std::fmt::Debug for CpuKernel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CpuKernel({} ported)", self.table.iter().flatten().count())
    }
}

impl CpuKernel {
    /// Every kernel is named and none has a body: each call is
    /// [`DeviceError::NotPorted`].
    pub fn new() -> Self {
        Self { table: vec![None; DeviceKernel::IntegratorMegakernel as usize + 1] }
    }
    /// Give `kernel` a body. This is the one place a ported kernel is
    /// registered.
    pub fn set(&mut self, kernel: DeviceKernel, body: CpuKernelFn) {
        self.table[kernel as usize] = Some(body);
    }
    pub fn is_ported(&self, kernel: DeviceKernel) -> bool {
        self.table[kernel as usize].is_some()
    }
    /// Run `kernel`.
    pub fn run(&self, kernel: DeviceKernel, memory: &mut CpuMemory, work_size: i32, args: &[KernelArgument]) -> Result<(), DeviceError> {
        match &self.table[kernel as usize] {
            Some(body) => body(memory, work_size, args),
            None => Err(DeviceError::NotPorted(kernel)),
        }
    }
}

impl Default for CpuKernel {
    fn default() -> Self {
        Self::new()
    }
}
