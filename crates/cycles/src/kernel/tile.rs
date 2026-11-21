// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/types.h` (`KernelWorkTile`,
//! lines 1688-1701).

/// `KernelWorkTile`: the pixels and samples one batch of paths starts
/// from. A device's path states are filled from tiles like this one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KernelWorkTile {
    /// Left and top, in the full image.
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub start_sample: u32,
    /// `num_samples`.
    pub sample_count: u32,
    pub sample_offset: u32,
    /// The buffer's offset and stride (`BufferParam`), to find a pixel.
    pub offset: i32,
    pub stride: u32,
    /// Worked out for the camera kernel on a GPU.
    pub path_index_offset: i32,
    pub work_size: i32,
}
