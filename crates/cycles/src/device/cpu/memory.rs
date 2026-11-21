// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/device/cpu/device_impl.cpp` (`CPUDevice::
//! mem_alloc`, `mem_copy_to`, `mem_zero`, `mem_free`): the blocks of a
//! CPU device. In Cycles a block is host memory and the device pointer is
//! its address; here the handle indexes a table so a kernel borrows a
//! block and cannot reach past it.
use std::collections::HashMap;

use bytemuck::Pod;

use crate::device::{DeviceError, DevicePtr, MemoryType};

#[derive(Debug)]
struct Block {
    /// 8-byte words, so a block can be viewed as any `Pod` up to 8 bytes.
    word: Vec<u64>,
    size: usize,
}

/// The blocks of one CPU device.
#[derive(Debug, Default)]
pub struct CpuMemory {
    block: HashMap<u64, Block>,
    next: u64,
    used: usize,
}

impl CpuMemory {
    /// A zeroed block. The CPU treats every [`MemoryType`] alike.
    pub fn alloc(&mut self, size: usize, _kind: MemoryType) -> Result<DevicePtr, DeviceError> {
        let words = size.div_ceil(8);
        let mut word = Vec::new();
        word.try_reserve_exact(words).map_err(|_| DeviceError::OutOfMemory { requested: size })?;
        word.resize(words, 0u64);
        self.next += 1;
        self.used += size;
        self.block.insert(self.next, Block { word, size });
        Ok(DevicePtr(self.next))
    }
    pub fn free(&mut self, ptr: DevicePtr) {
        if let Some(block) = self.block.remove(&ptr.0) {
            self.used -= block.size;
        }
    }
    pub fn size(&self, ptr: DevicePtr) -> Option<usize> {
        self.block.get(&ptr.0).map(|block| block.size)
    }
    pub fn used(&self) -> usize {
        self.used
    }
    /// The bytes of a block.
    pub fn bytes(&self, ptr: DevicePtr) -> Result<&[u8], DeviceError> {
        let block = self.block.get(&ptr.0).ok_or(DeviceError::BadPointer(ptr))?;
        Ok(&bytemuck::cast_slice::<u64, u8>(&block.word)[..block.size])
    }
    pub fn bytes_mut(&mut self, ptr: DevicePtr) -> Result<&mut [u8], DeviceError> {
        let block = self.block.get_mut(&ptr.0).ok_or(DeviceError::BadPointer(ptr))?;
        Ok(&mut bytemuck::cast_slice_mut::<u64, u8>(&mut block.word)[..block.size])
    }
    /// Two blocks at once, both writable: a kernel that reads one and
    /// writes the other, such as a path state and a render buffer.
    pub fn bytes_pair_mut(&mut self, a: DevicePtr, b: DevicePtr) -> Result<(&mut [u8], &mut [u8]), DeviceError> {
        if a == b {
            return Err(DeviceError::BadPointer(a));
        }
        let [first, second] = self.block.get_disjoint_mut([&a.0, &b.0]);
        let first = first.ok_or(DeviceError::BadPointer(a))?;
        let second = second.ok_or(DeviceError::BadPointer(b))?;
        let (size_a, size_b) = (first.size, second.size);
        let first = &mut bytemuck::cast_slice_mut::<u64, u8>(&mut first.word)[..size_a];
        let second = &mut bytemuck::cast_slice_mut::<u64, u8>(&mut second.word)[..size_b];
        Ok((first, second))
    }    /// A block as elements of `T` (what the host's `device_vector<T>` is);
    /// a trailing part smaller than a `T` is left out.
    pub fn slice<T: Pod>(&self, ptr: DevicePtr) -> Result<&[T], DeviceError> {
        let bytes = self.bytes(ptr)?;
        Ok(bytemuck::cast_slice(&bytes[..bytes.len() / size_of::<T>() * size_of::<T>()]))
    }
    pub fn slice_mut<T: Pod>(&mut self, ptr: DevicePtr) -> Result<&mut [T], DeviceError> {
        let bytes = self.bytes_mut(ptr)?;
        let len = bytes.len() / size_of::<T>() * size_of::<T>();
        Ok(bytemuck::cast_slice_mut(&mut bytes[..len]))
    }
    /// The range `offset..offset + len` of a block, checked.
    fn check(&self, ptr: DevicePtr, offset: usize, len: usize) -> Result<(), DeviceError> {
        let size = self.size(ptr).ok_or(DeviceError::BadPointer(ptr))?;
        match offset.checked_add(len) {
            Some(end) if end <= size => Ok(()),
            _ => Err(DeviceError::OutOfRange { ptr, offset, len, size }),
        }
    }
    pub fn zero(&mut self, ptr: DevicePtr) -> Result<(), DeviceError> {
        self.bytes_mut(ptr)?.fill(0);
        Ok(())
    }
    pub fn copy_to(&mut self, ptr: DevicePtr, offset: usize, data: &[u8]) -> Result<(), DeviceError> {
        self.check(ptr, offset, data.len())?;
        self.bytes_mut(ptr)?[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }
    pub fn copy_from(&self, ptr: DevicePtr, offset: usize, out: &mut [u8]) -> Result<(), DeviceError> {
        self.check(ptr, offset, out.len())?;
        out.copy_from_slice(&self.bytes(ptr)?[offset..offset + out.len()]);
        Ok(())
    }
}
