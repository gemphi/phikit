// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/render_scheduler.cpp`
//! (`RenderScheduler::done`, `get_render_work`,
//! `update_state_for_render_work`, `set_postprocess_render_work`,
//! `set_full_frame_render_work`, lines 281-462).

use super::{RenderScheduler, is_power_of_two};
use crate::integrator::{RenderWork, WorkAdaptive, WorkDisplay, WorkPathTrace, WorkTile};
use crate::session::TileManager;
use crate::util::Clock;

impl<C: Clock> RenderScheduler<C> {
    /// `done`: all samples are scheduled, or the time is up. When the time
    /// is up there is still work left: denoising and writing the result.
    pub(crate) fn done(&self) -> bool {
        if self.state.resolution_divider != self.pixel_size {
            return false;
        }
        if self.state.path_trace_finished || self.state.time_limit_reached {
            return true;
        }
        self.rendered_sample_count() >= self.sample_count
    }

    /// `get_render_work`: the next unit of work. Nothing in it means the
    /// render is over.
    pub fn render_work(&mut self, tile_manager: &TileManager) -> RenderWork {
        self.check_time_limit_reached();
        let time_now = self.clock.now();
        if self.done() {
            let mut work = self.work_at_divider();
            if !self.set_postprocess_render_work(&mut work, tile_manager) {
                self.set_full_frame_render_work(&mut work, tile_manager);
            }
            if !work.any() {
                self.state.end_render_time = time_now;
            }
            self.update_state_for_render_work(&work);
            return work;
        }
        if self.state.resolution_divider != self.pixel_size {
            self.next_resolution_divider();
        }
        let mut work = self.work_at_divider();
        work.path_trace = WorkPathTrace {
            start_sample: self.start_sample_to_path_trace(),
            sample_count: self.sample_count_to_path_trace(),
            sample_offset: self.sample_offset,
        };
        work.init_render_buffers = work.path_trace.start_sample == self.sample_offset;
        // The rebalance check needs the sample count not yet moved on.
        work.rebalance = self.work_need_rebalance();
        // Moved on now, so the checks below see every sample as rendered.
        self.state.rendered_sample_count += work.path_trace.sample_count;
        work.adaptive_sampling = WorkAdaptive { filter: self.work_need_adaptive_filter(), threshold: self.work_adaptive_threshold(), reset: false };
        // Volume guiding is denoised at powers of two, not at the end.
        work.volume_guiding_denoise = is_power_of_two(self.state.rendered_sample_count) && !self.done();
        let need = self.work_need_denoise(tile_manager);
        work.tile = WorkTile { write: self.done(), denoise: need.denoise };
        work.display = WorkDisplay { update: self.work_need_update_display(need.delayed), use_denoised_result: need.ready_to_display };
        if self.done() {
            self.set_postprocess_render_work(&mut work, tile_manager);
        }
        self.update_state_for_render_work(&work);
        work
    }

    /// Work with nothing asked for, at the current resolution divider. The
    /// denoiser's upscaling lets the path tracer render smaller still.
    fn work_at_divider(&self) -> RenderWork {
        let divider = self.state.resolution_divider as f32;
        let traced = match self.denoise_param.enabled {
            true => divider * self.denoise_param.upscale_factor,
            false => divider,
        };
        RenderWork { resolution_divider: traced, denoised_resolution_divider: divider, ..RenderWork::default() }
    }

    /// The divider's next step toward full resolution. While the user is
    /// navigating it stays. A divider past the default drops to 4 first, so
    /// slow hardware shows something before the full render.
    fn next_resolution_divider(&mut self) {
        let state = &mut self.state;
        match (state.user_is_navigating, self.default_start_resolution_divider) {
            (true, _) => state.user_is_navigating = false,
            (false, 0) => {}
            (false, default) => {
                state.resolution_divider = match state.resolution_divider > default {
                    true => 4 * self.pixel_size,
                    false => 1,
                }
            }
        }
        state.resolution_divider = state.resolution_divider.max(self.pixel_size);
        state.rendered_sample_count = 0;
        state.last_display_update_sample = -1;
    }

    /// `update_state_for_render_work`: remember what `work` asks for.
    fn update_state_for_render_work(&mut self, work: &RenderWork) {
        let time_now = self.clock.now();
        let state = &mut self.state;
        if work.rebalance {
            state.last_rebalance_time = time_now;
            state.rebalance_request_count += 1;
        }
        // A fallback time, for when the display fails or there is none.
        if work.display.update {
            state.last_display_update_time = time_now;
            state.last_display_update_sample = state.rendered_sample_count;
        }
        state.last_work_tile_was_denoised = work.tile.denoise;
        state.tile_result_was_written |= work.tile.write;
        state.full_frame_was_written |= work.full_write;
    }

    /// `set_postprocess_render_work`: what follows the last sample of a
    /// tile, once. `true` when anything was scheduled.
    fn set_postprocess_render_work(&mut self, work: &mut RenderWork, tile_manager: &TileManager) -> bool {
        if self.state.postprocess_work_scheduled {
            return false;
        }
        self.state.postprocess_work_scheduled = true;
        let denoise = self.denoise_param.enabled && !self.state.last_work_tile_was_denoised;
        let write = !self.state.tile_result_was_written;
        if self.need_schedule_cryptomatte {
            work.cryptomatte_postprocess = true;
        }
        if denoise {
            work.tile.denoise = !tile_manager.has_multiple_tile();
        }
        if write {
            work.tile.write = true;
        }
        let any_scheduled = self.need_schedule_cryptomatte || denoise || write;
        if any_scheduled {
            work.display.update = true;
        }
        any_scheduled
    }

    /// `set_full_frame_render_work`: what follows the last tile of a
    /// many-tile render, once: writing the full frame.
    fn set_full_frame_render_work(&mut self, work: &mut RenderWork, tile_manager: &TileManager) {
        let state = &mut self.state;
        // With one tile everything was done already.
        let ready = tile_manager.has_multiple_tile() && tile_manager.done();
        if state.full_frame_work_scheduled || !ready || state.full_frame_was_written {
            return;
        }
        state.full_frame_work_scheduled = true;
        work.full_write = true;
    }
}
