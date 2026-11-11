// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/device/cpu/device_impl.h` and
//! `device_impl.cpp` (`CPUDevice`): memory is host memory, a kernel is a
//! call. The queue runs a kernel when it is enqueued.
use std::sync::{Arc, Mutex, MutexGuard};

use super::{CpuKernels, CpuMemory};
use crate::device::{Device, DeviceError, DeviceInfo, DevicePtr, DeviceQueue, DeviceType, KernelArgument, MemoryType};
use crate::kernel::DeviceKernel;

/// `CPUDevice`.
#[derive(Debug)]
pub struct CpuDevice {
    info: DeviceInfo,
    memory: Arc<Mutex<CpuMemory>>,
    kernels: Arc<CpuKernels>,
}

impl CpuDevice {
    /// A CPU device whose kernels are all unported.
    pub fn new() -> Self {
        Self::with_kernels(CpuKernels::new())
    }
    pub fn with_kernels(kernels: CpuKernels) -> Self {
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get() as i32);
        let info = DeviceInfo { kind: DeviceType::Cpu, description: "CPU".into(), cpu_threads: threads, ..DeviceInfo::default() };
        Self { info, memory: Arc::default(), kernels: Arc::new(kernels) }
    }
    pub fn kernels(&self) -> &CpuKernels {
        &self.kernels
    }
    fn lock(&self) -> MutexGuard<'_, CpuMemory> {
        self.memory.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
    /// Run `body` over the blocks, for code that reads them as typed
    /// slices (a test, the path trace work).
    pub fn with_memory<R>(&self, body: impl FnOnce(&mut CpuMemory) -> R) -> R {
        body(&mut self.lock())
    }
}

impl Default for CpuDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for CpuDevice {
    fn info(&self) -> &DeviceInfo {
        &self.info
    }
    fn mem_alloc(&self, size: usize, kind: MemoryType) -> Result<DevicePtr, DeviceError> {
        self.lock().alloc(size, kind)
    }
    fn mem_zero(&self, ptr: DevicePtr) -> Result<(), DeviceError> {
        self.lock().zero(ptr)
    }
    fn mem_copy_to(&self, ptr: DevicePtr, offset: usize, data: &[u8]) -> Result<(), DeviceError> {
        self.lock().copy_to(ptr, offset, data)
    }
    fn mem_copy_from(&self, ptr: DevicePtr, offset: usize, out: &mut [u8]) -> Result<(), DeviceError> {
        self.lock().copy_from(ptr, offset, out)
    }
    fn mem_free(&self, ptr: DevicePtr) {
        self.lock().free(ptr);
    }
    fn mem_size(&self, ptr: DevicePtr) -> Option<usize> {
        self.lock().size(ptr)
    }
    fn mem_used(&self) -> usize {
        self.lock().used()
    }
    fn queue(&self) -> Box<dyn DeviceQueue> {
        Box::new(CpuQueue { memory: self.memory.clone(), kernels: self.kernels.clone(), error: None })
    }
}

/// The queue of a CPU device: a kernel runs inside `enqueue`. The first
/// error is kept and returned by `synchronize`, and later kernels are
/// skipped, as a GPU stream that failed would not run them.
struct CpuQueue {
    memory: Arc<Mutex<CpuMemory>>,
    kernels: Arc<CpuKernels>,
    error: Option<DeviceError>,
}

impl DeviceQueue for CpuQueue {
    fn enqueue(&mut self, kernel: DeviceKernel, work_size: i32, args: &[KernelArgument]) -> Result<(), DeviceError> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        let mut memory = self.memory.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let result = self.kernels.run(kernel, &mut memory, work_size, args);
        if let Err(error) = &result {
            self.error = Some(error.clone());
        }
        result
    }
    fn synchronize(&mut self) -> Result<(), DeviceError> {
        match self.error.take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}
