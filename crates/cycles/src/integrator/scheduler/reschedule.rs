// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/render_scheduler.cpp`
//! (`RenderScheduler::render_work_reschedule_on_converge`,
//! `render_work_reschedule_on_idle`, `render_work_reschedule_on_cancel`,
//! lines 185-279): changing work already handed out.

use super::RenderScheduler;
use crate::integrator::RenderWork;
use crate::session::TileManager;
use crate::util::Clock;

impl<C: Clock> RenderScheduler<C> {
    /// `render_work_reschedule_on_converge`: every pixel converged. `true`
    /// when `work` goes on with a lower threshold; `false` when the final
    /// threshold was reached, and then no more path tracing is scheduled.
    pub fn reschedule_on_converge(&mut self, work: &mut RenderWork, tile_manager: &TileManager) -> bool {
        // Not at full resolution yet: adaptive filtering waits.
        if self.state.resolution_divider != self.pixel_size {
            return false;
        }
        if self.reschedule_on_idle(work) {
            return true;
        }
        self.state.path_trace_finished = true;
        let need = self.work_need_denoise(tile_manager);
        work.tile.denoise = need.denoise;
        work.display.update = self.work_need_update_display(need.delayed);
        work.display.use_denoised_result = need.ready_to_display;
        false
    }

    /// `render_work_reschedule_on_idle`: the device is mostly idle though
    /// pixels remain. `true` when `work` goes on with half the threshold;
    /// `false` when the threshold is final and the open pixels are to be
    /// finished.
    pub fn reschedule_on_idle(&mut self, work: &mut RenderWork) -> bool {
        let at_full_resolution = self.state.resolution_divider == self.pixel_size;
        let target = self.adaptive_sampling.threshold;
        let lower = self.adaptive_sampling.enabled && self.state.adaptive_sampling_threshold > target;
        if !self.use_progressive_noise_floor || !at_full_resolution || !lower {
            return false;
        }
        self.state.adaptive_sampling_threshold = (self.state.adaptive_sampling_threshold / 2.0).max(target);
        work.adaptive_sampling.threshold = self.state.adaptive_sampling_threshold;
        work.adaptive_sampling.reset = true;
        true
    }

    /// `render_work_reschedule_on_cancel`: a cancel was asked for. Leave in
    /// `work` only what makes the passes rendered so far reach the host. To
    /// be called before the work is rendered.
    pub fn reschedule_on_cancel(&mut self, work: &mut RenderWork) {
        // Samples not rendered are not counted.
        self.state.rendered_sample_count -= work.path_trace.sample_count;
        let has_rendered_sample = self.rendered_sample_count() != 0;
        // Writes happen once and cannot be asked for again, so they stay. A
        // tile with no sample in it is not written, like any cancelled tile.
        let tile_write = work.tile.write || (!self.state.tile_result_was_written && has_rendered_sample);
        let full_write = work.full_write || !self.state.full_frame_was_written;
        *work = RenderWork::default();
        work.tile.write = tile_write;
        work.full_write = full_write;
        // Show the tile as it stands while the full buffer is processed.
        // After the full write the buffers may be gone from the device.
        work.display.update = has_rendered_sample && !self.state.full_frame_was_written;
    }
}

#[cfg(test)]
mod test;
