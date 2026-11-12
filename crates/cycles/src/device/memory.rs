// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/device/memory.h` (`MemoryType`,
//! `device_memory`, `device_vector<T>`, `device_only_memory<T>`). A
//! `DeviceVector<T>` keeps a host copy and the handle of its device block,
//! as Cycles' does with `host_pointer` and `device_pointer`; the block is
//! allocated and moved by a [`Device`](super::Device).
use bytemuck::Pod;

use super::{Device, DeviceError};

/// `MemoryType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    ReadOnly,
    ReadWrite,
    /// Only the device holds it: no host copy.
    DeviceOnly,
    /// Constant data every kernel reads by name.
    Global,
    ImageTexture,
}

/// `device_ptr`: a device's name for one of its blocks. Zero is none. On
/// the CPU it indexes the device's block table; on wgpu it indexes buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DevicePtr(pub u64);

impl DevicePtr {
    pub const NULL: DevicePtr = DevicePtr(0);
    pub fn is_null(self) -> bool {
        self.0 == 0
    }
}

/// `device_vector<T>`.
#[derive(Debug)]
pub struct DeviceVector<T: Pod> {
    pub name: String,
    pub kind: MemoryType,
    /// `data()`: the host copy. Empty for [`MemoryType::DeviceOnly`].
    host: Vec<T>,
    /// `device_size`: elements the block holds.
    device_len: usize,
    ptr: DevicePtr,
}

impl<T: Pod> DeviceVector<T> {
    pub fn new(name: &str, kind: MemoryType) -> Self {
        Self { name: name.into(), kind, host: Vec::new(), device_len: 0, ptr: DevicePtr::NULL }
    }
    /// `alloc(width)` on the host: `len` zeroed elements.
    pub fn resize(&mut self, len: usize) {
        self.host.clear();
        self.host.resize(len, T::zeroed());
    }
    /// The host copy, to read.
    pub fn host(&self) -> &[T] {
        &self.host
    }
    /// The host copy, to write; then call [`Self::copy_to_device`].
    pub fn host_mut(&mut self) -> &mut [T] {
        &mut self.host
    }
    /// `size()` in elements on the host.
    pub fn len(&self) -> usize {
        self.host.len()
    }
    pub fn is_empty(&self) -> bool {
        self.host.is_empty()
    }
    /// `memory_size()`: bytes of the host copy.
    pub fn memory_size(&self) -> usize {
        self.host.len() * size_of::<T>()
    }
    /// `device_pointer`.
    pub fn ptr(&self) -> DevicePtr {
        self.ptr
    }
    /// Elements the device block holds.
    pub fn device_len(&self) -> usize {
        self.device_len
    }
    /// `alloc_to_device(len)`: a device block of `len` elements, zeroed.
    pub fn alloc_to_device(&mut self, device: &dyn Device, len: usize) -> Result<(), DeviceError> {
        self.free(device);
        self.ptr = device.mem_alloc(len * size_of::<T>(), self.kind)?;
        self.device_len = len;
        Ok(())
    }
    /// `copy_to_device`: the host copy into the block, made when it is not
    /// there or too small.
    pub fn copy_to_device(&mut self, device: &dyn Device) -> Result<(), DeviceError> {
        if self.ptr.is_null() || self.device_len != self.host.len() {
            self.alloc_to_device(device, self.host.len())?;
        }
        device.mem_copy_to(self.ptr, 0, bytemuck::cast_slice(&self.host))
    }
    /// `copy_from_device`: the block into the host copy.
    pub fn copy_from_device(&mut self, device: &dyn Device) -> Result<(), DeviceError> {
        self.host.resize(self.device_len, T::zeroed());
        device.mem_copy_from(self.ptr, 0, bytemuck::cast_slice_mut(&mut self.host))
    }
    /// `zero_to_device`: the block zeroed, made when it is not there.
    pub fn zero_to_device(&mut self, device: &dyn Device) -> Result<(), DeviceError> {
        if self.ptr.is_null() {
            self.alloc_to_device(device, self.host.len())?;
        }
        self.host.fill(T::zeroed());
        device.mem_zero(self.ptr)
    }
    /// `free`: the block goes back to the device; the host copy stays.
    pub fn free(&mut self, device: &dyn Device) {
        if !self.ptr.is_null() {
            device.mem_free(self.ptr);
            self.ptr = DevicePtr::NULL;
            self.device_len = 0;
        }
    }
}

#[cfg(test)]
mod test;
