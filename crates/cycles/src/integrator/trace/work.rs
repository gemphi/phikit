// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/path_trace_work.h`
//! (`PathTraceWork`, `PathTraceWork::RenderStatistics`), and the calls
//! `PathTrace` makes beside its works (`integrator/path_trace.cpp`).
//!
//! `PathTraceWork` is the part of it the host loop calls; what implements
//! it, the CPU and GPU works, is step 5 and 6. The denoiser, the display,
//! the output driver and the tile file are not ported, so the loop reaches
//! them through [`TraceHost`]; each method names the call it stands for.

use crate::integrator::RenderWork;
use crate::session::{BufferParam, RenderBuffer};

/// `PathTraceWork::RenderStatistics`: what a work reports of its samples.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderStatistic {
    /// How full the path states were, 0 to 1.
    pub occupancy: f32,
}

impl Default for RenderStatistic {
    fn default() -> Self {
        Self { occupancy: 1.0 }
    }
}

/// `PathTraceWork`: path tracing on one device, into its own slice of the
/// frame.
pub trait PathTraceWork {
    /// `init_execution`: ready the kernels for this round.
    fn init_execution(&mut self);
    /// `deinit_execution`: release what `init_execution` took.
    fn deinit_execution(&mut self) {}
    /// `get_device()->have_error()`.
    fn has_error(&self) -> bool {
        false
    }
    /// `get_render_buffers()->reset(param)`: the work's buffer becomes
    /// `param`, its slice of the big tile at full size.
    fn reset_render_buffer(&mut self, param: &BufferParam);
    /// `set_effective_buffer_params`: what this round renders into: the
    /// big tile and this work's slice of it at the path tracer's
    /// resolution, then both at the denoiser's.
    fn set_effective_buffer_param(&mut self, big_tile: &BufferParam, slice: &BufferParam, denoised_big_tile: &BufferParam, denoised_slice: &BufferParam);
    /// `zero_render_buffers`.
    fn zero_render_buffer(&mut self);
    /// `copy_to_render_buffers`: this work's slice into `buffer`, which
    /// holds the whole big tile.
    fn copy_to_render_buffer(&mut self, buffer: &mut RenderBuffer);
    /// `copy_from_render_buffers`: this work's slice out of `buffer`.
    fn copy_from_render_buffer(&mut self, buffer: &RenderBuffer);
    /// `render_samples`: trace `sample_count` samples from `start_sample`,
    /// and return when they are done.
    fn render_samples(&mut self, start_sample: i32, sample_count: i32, sample_offset: i32) -> RenderStatistic;
    /// `adaptive_sampling_converge_filter_count_active`: check the pixels
    /// against `threshold`, filter the mask, and count those still open.
    /// `reset` checks the converged ones again.
    fn adaptive_sampling_converge_filter_count_active(&mut self, threshold: f32, reset: bool) -> u32;
    /// `cryptomatte_postproces`.
    fn cryptomatte_postprocess(&mut self);
    /// `denoise_volume_guiding_buffers`.
    fn denoise_volume_guiding_buffer(&mut self);
}

/// What the path tracer calls beside its works.
pub trait TraceHost {
    /// `progress_->get_cancel()`.
    fn cancelled(&self) -> bool;
    /// `display_->reset(big_tile_param, reset_rendering)`.
    fn reset_display(&mut self, big_tile_param: &BufferParam, reset_rendering: bool);
    /// `tile_buffer_read`: read pixels a host has ready, as when baking.
    fn read_tile_buffer(&mut self);
    /// `denoiser_->denoise_buffer` on a buffer of `sample_count` samples.
    /// `None` when no denoiser is configured; else whether it denoised.
    fn denoise(&mut self, sample_count: i32) -> Option<bool>;
    /// `tile_buffer_write`, or `tile_buffer_write_to_disk` when `to_disk`:
    /// a tile of many goes to the tile file, so its buffer is free again.
    fn write_tile(&mut self, to_disk: bool);
    /// The body of `update_display`: the output driver's `update_render_tile`
    /// and the copy to the display. `false` when there is neither, or the
    /// frame has no size.
    fn update_display(&mut self, show_denoised: bool, sample_count: i32) -> bool;
    /// `progress_update_if_needed`.
    fn update_progress(&mut self, work: &RenderWork);
    /// `tile_manager_.finish_write_tiles()`, when tiles were written.
    fn finish_tile_file(&mut self);
}
