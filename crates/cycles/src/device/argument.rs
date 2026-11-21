// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/device/kernel.h` (`DeviceKernelArguments`):
//! what a queue is handed beside the kernel and its work size.
use super::DevicePtr;

/// One argument: Cycles' `POINTER`, `INT32`, `FLOAT32` and `BOOLEAN`
/// (`KERNEL_FILM_CONVERT`, a pointer to a struct, is a `Ptr` to bytes).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KernelArgument {
    Ptr(DevicePtr),
    I32(i32),
    F32(f32),
    Bool(bool),
}

impl KernelArgument {
    pub fn as_ptr(&self) -> Option<DevicePtr> {
        if let Self::Ptr(ptr) = self { Some(*ptr) } else { None }
    }
    pub fn as_i32(&self) -> Option<i32> {
        if let Self::I32(value) = self { Some(*value) } else { None }
    }
    pub fn as_f32(&self) -> Option<f32> {
        if let Self::F32(value) = self { Some(*value) } else { None }
    }
}
