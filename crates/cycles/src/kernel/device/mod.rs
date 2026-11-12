// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/device/`: the kernels as a device
//! calls them. `cpu.rs` is `device/cpu/kernel_arch_impl.h` and
//! `device/cpu/kernel.cpp`: the functions of the CPU kernel table.
mod cpu;
pub use cpu::{cpu_kernels, register_cpu_kernels};

#[cfg(test)]
mod test;
