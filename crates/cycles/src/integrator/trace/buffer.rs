// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/path_trace.cpp`
//! (`PathTrace::reset`, `update_allocated_work_buffer_params`,
//! `update_effective_work_buffer_params`,
//! `update_work_buffer_params_if_needed`, `copy_to_render_buffers`,
//! `copy_from_render_buffers`, lines 121-143, 319-398 and 1034-1050): the
//! buffers of the works, sized for the tile and the resolution divider.

use super::slice::{scale_buffer_param, slice_buffer_param};
use super::{PathTrace, PathTraceWork, TraceHost};
use crate::integrator::RenderWork;
use crate::session::{BufferParam, RenderBuffer};

impl<W: PathTraceWork> PathTrace<W> {
    /// `reset`: render `big_tile_param`, a tile of the `full_param` frame.
    /// `reset_rendering` when the render starts over, not only its tile.
    pub fn reset(&mut self, full_param: &BufferParam, big_tile_param: &BufferParam, host: &mut impl TraceHost, reset_rendering: bool) {
        if self.big_tile_param.modified(big_tile_param) {
            self.big_tile_param = big_tile_param.clone();
            self.state.need_reset_param = true;
        }
        self.full_param = full_param.clone();
        // The display is told of every reset, to keep track of redraws.
        host.reset_display(big_tile_param, reset_rendering);
        self.state.has_denoised_result = false;
        self.state.tile_written = false;
    }

    /// `update_allocated_work_buffer_params`: each work's buffer becomes
    /// its slice of the big tile.
    fn update_allocated_work_buffer_param(&mut self, overscan: i32) {
        let slice = slice_buffer_param(&self.balance, &self.big_tile_param, overscan);
        self.work.iter_mut().zip(&slice).for_each(|(device, param)| device.reset_render_buffer(param));
    }

    /// `update_effective_work_buffer_params`: what the works render into
    /// for `work`. The denoiser's divider gives the size shown; the path
    /// tracer's, when the denoiser upscales, a smaller one.
    fn update_effective_work_buffer_param(&mut self, work: &RenderWork, overscan: i32) {
        let denoised_divider = work.denoised_resolution_divider;
        let divider = work.resolution_divider / denoised_divider;
        let denoised_big_tile = scale_buffer_param(&self.big_tile_param, denoised_divider);
        let scaled_big_tile = scale_buffer_param(&denoised_big_tile, divider);
        let slice = slice_buffer_param(&self.balance, &denoised_big_tile, overscan);
        for (device, denoised) in self.work.iter_mut().zip(&slice) {
            // Scaled again, to match slicing the scaled big tile directly.
            device.set_effective_buffer_param(&scaled_big_tile, &scale_buffer_param(denoised, divider), &denoised_big_tile, denoised);
        }
        self.state.effective_big_tile_param = scaled_big_tile;
        self.state.effective_denoised_big_tile_param = denoised_big_tile;
    }

    /// `update_work_buffer_params_if_needed`: after a reset, a rebalance or
    /// a new resolution divider.
    pub(super) fn update_work_buffer_param_if_needed(&mut self, work: &RenderWork, overscan: i32) {
        if self.state.need_reset_param {
            self.update_allocated_work_buffer_param(overscan);
        }
        if self.state.need_reset_param || self.state.resolution_divider != work.resolution_divider {
            self.update_effective_work_buffer_param(work, overscan);
        }
        self.state.resolution_divider = work.resolution_divider;
        self.state.need_reset_param = false;
    }

    /// `copy_to_render_buffers`: every work's slice into `buffer`.
    pub fn copy_to_render_buffer(&mut self, buffer: &mut RenderBuffer) {
        self.work.iter_mut().for_each(|device| device.copy_to_render_buffer(buffer));
    }

    /// `copy_from_render_buffers`: every work's slice out of `buffer`.
    pub fn copy_from_render_buffer(&mut self, buffer: &RenderBuffer) {
        self.work.iter_mut().for_each(|device| device.copy_from_render_buffer(buffer));
    }
}

#[cfg(test)]
mod test;
