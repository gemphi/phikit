// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/session/buffers.h` and `buffers.cpp`
//! (`RenderBuffers`, `render_buffers_host_copy_denoised`). The copies to
//! and from a device come with the device layer.

use super::{BufferParam, PASS_UNUSED};
use crate::scene::{PassMode, PassType};

/// `RenderBuffers`: every pass of every pixel as flat floats, `pass_stride`
/// a pixel, rows top to bottom.
#[derive(Debug, Clone, Default)]
pub struct RenderBuffer {
    pub param: BufferParam,
    pub buffer: Vec<f32>,
}

impl RenderBuffer {
    /// `reset`: take `param` and allocate for it, zeroed. The passes must
    /// be laid out already.
    pub fn reset(&mut self, param: &BufferParam) {
        debug_assert!(param.pass_stride != -1, "lay the passes out before allocating");
        self.param = param.clone();
        let float_count = param.width.max(0) as usize * param.pass_stride.max(0) as usize * param.height.max(0) as usize;
        self.buffer.clear();
        self.buffer.resize(float_count, 0.0);
    }

    /// `zero`.
    pub fn zero(&mut self) {
        self.buffer.fill(0.0);
    }

    /// The floats of the pixel at `x`, `y` of the buffer.
    pub fn pixel(&self, x: i32, y: i32) -> &[f32] {
        let stride = self.param.pass_stride.max(0) as usize;
        let at = (y as usize * self.param.width as usize + x as usize) * stride;
        &self.buffer[at..at + stride]
    }

    /// The floats of the pixel at `x`, `y`, to write.
    pub fn pixel_mut(&mut self, x: i32, y: i32) -> &mut [f32] {
        let stride = self.param.pass_stride.max(0) as usize;
        let at = (y as usize * self.param.width as usize + x as usize) * stride;
        &mut self.buffer[at..at + stride]
    }

    /// `render_buffers_host_copy_denoised`: copy every denoised pass both
    /// buffers have from `source`, starting `source_offset` pixels in. The
    /// two may lay their passes out differently. Four floats a pass are
    /// copied, as Cycles does.
    pub fn copy_denoised(&mut self, source: &RenderBuffer, source_offset: usize) {
        debug_assert_eq!(self.param.width, source.param.width);
        let pair: Vec<(usize, usize)> = (0..PassType::count() as i32)
            .filter_map(|number| {
                let into = self.param.pass_offset(PassType(number), PassMode::Denoised);
                let from = source.param.pass_offset(PassType(number), PassMode::Denoised);
                (into != PASS_UNUSED && from != PASS_UNUSED).then_some((into as usize, from as usize))
            })
            .collect();
        let into_stride = self.param.pass_stride.max(1) as usize;
        let from_stride = source.param.pass_stride.max(1) as usize;
        let from_pixel = source.buffer[source_offset * from_stride..].chunks_exact(from_stride);
        for (into, from) in self.buffer.chunks_exact_mut(into_stride).zip(from_pixel) {
            for (into_at, from_at) in &pair {
                into[*into_at..*into_at + 4].copy_from_slice(&from[*from_at..*from_at + 4]);
            }
        }
    }
}
