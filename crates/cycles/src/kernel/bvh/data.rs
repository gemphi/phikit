// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/types.h` (`KernelBVH`) and
//! `kernel/bvh/types.h`: what a walk of the tree reads for every ray, the
//! root out of the constant data and Cycles' numbers out of
//! `data/constant.tsv`, taken once.

use crate::kernel::{KernelData, flag};

/// `kernel_data.bvh`, and the numbers a walk compares with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelBvh {
    /// `root`: the node the walk starts at.
    pub root: i32,
    /// `ENTRYPOINT_SENTINEL`: the mark at the bottom of the stack.
    pub sentinel: i32,
    /// `PRIM_NONE`.
    pub prim_none: i32,
    /// `OBJECT_NONE`.
    pub object_none: i32,
    /// `PRIMITIVE_ALL`: the bits of a leaf's type that say what is in it.
    pub primitive_all: u32,
    /// `PRIMITIVE_TRIANGLE`.
    pub primitive_triangle: u32,
    /// `PATH_RAY_VISIBILITY_SHADOW_OPAQUE`: a ray that asks only whether
    /// anything is in the way, and stops at the first hit.
    pub shadow_opaque: u32,
}

impl KernelBvh {
    pub fn of(data: &KernelData) -> KernelBvh {
        KernelBvh {
            root: data.int("bvh", "root"),
            sentinel: flag("ENTRYPOINT_SENTINEL") as i32,
            prim_none: flag("PRIM_NONE") as i32,
            object_none: flag("OBJECT_NONE") as i32,
            primitive_all: flag("PRIMITIVE_ALL"),
            primitive_triangle: flag("PRIMITIVE_TRIANGLE"),
            shadow_opaque: flag("PATH_RAY_VISIBILITY_SHADOW_OPAQUE"),
        }
    }
}

#[cfg(test)]
mod test;
