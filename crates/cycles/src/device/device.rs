// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/device/device.h` (`Device`): the contract a
//! device keeps, here as a trait the CPU device and, later, phiray's wgpu
//! device both implement. See `device/README.md` for how one kernel is run
//! on both and the results compared.
//!
//! Cycles' `Device` is a class with `mem_alloc`, `mem_copy_to`, `mem_zero`,
//! `mem_free` and `gpu_queue_create`; the methods here are those, by block
//! handle ([`DevicePtr`]) and bytes, so no type of the host leaks into a
//! device. Every method takes `&self`: a device is shared by its queues
//! and its buffers, and keeps its own lock.
use super::{DeviceInfo, DevicePtr, DeviceQueue, MemoryType};
use crate::kernel::DeviceKernel;

/// What a device can fail at. Cycles sets `have_error` and a message; here
/// each call returns it.
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceError {
    OutOfMemory { requested: usize },
    /// A handle the device does not know (never allocated, or freed).
    BadPointer(DevicePtr),
    /// A copy past the end of its block.
    OutOfRange { ptr: DevicePtr, offset: usize, len: usize, size: usize },
    /// The kernel is named, and its body is not ported yet.
    NotPorted(DeviceKernel),
    /// The kernel was handed arguments of the wrong number or type.
    BadArgument { kernel: DeviceKernel, index: usize },
}

impl std::fmt::Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfMemory { requested } => write!(f, "device out of memory asking for {requested} bytes"),
            Self::BadPointer(ptr) => write!(f, "device block {} does not exist", ptr.0),
            Self::OutOfRange { ptr, offset, len, size } => write!(f, "copy of {len} bytes at {offset} is past the {size} bytes of block {}", ptr.0),
            Self::NotPorted(kernel) => write!(f, "kernel {kernel:?} is not ported"),
            Self::BadArgument { kernel, index } => write!(f, "kernel {kernel:?} got a wrong argument at {index}"),
        }
    }
}

impl std::error::Error for DeviceError {}

/// `Device`.
pub trait Device: Send + Sync {
    /// `info`.
    fn info(&self) -> &DeviceInfo;
    /// `mem_alloc`: a zeroed block of `size` bytes.
    fn mem_alloc(&self, size: usize, kind: MemoryType) -> Result<DevicePtr, DeviceError>;
    /// `mem_zero`: the whole block to zero.
    fn mem_zero(&self, ptr: DevicePtr) -> Result<(), DeviceError>;
    /// `mem_copy_to`: `data` into the block, `offset` bytes in.
    fn mem_copy_to(&self, ptr: DevicePtr, offset: usize, data: &[u8]) -> Result<(), DeviceError>;
    /// `mem_copy_from`: `out.len()` bytes of the block, from `offset`.
    fn mem_copy_from(&self, ptr: DevicePtr, offset: usize, out: &mut [u8]) -> Result<(), DeviceError>;
    /// `mem_free`. A handle the device does not know is ignored.
    fn mem_free(&self, ptr: DevicePtr);
    /// Bytes of the block, `None` for a handle the device does not know.
    fn mem_size(&self, ptr: DevicePtr) -> Option<usize>;
    /// Bytes held by all blocks (`stats.mem_alloc`).
    fn mem_used(&self) -> usize;
    /// `gpu_queue_create`, for the CPU too: a queue over this device.
    fn queue(&self) -> Box<dyn DeviceQueue>;
}
