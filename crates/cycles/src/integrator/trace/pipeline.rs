// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/path_trace.cpp`
//! (`PathTrace::render`, `render_pipeline`, `init_render_buffers`,
//! `path_trace`, `adaptive_sample`, `cryptomatte_postprocess`, `denoise`,
//! `denoise_volume_guiding_buffers`, `update_display`, `rebalance`,
//! `write_tile_buffer`, `finalize_full_buffer_on_disk`, lines 162-256,
//! 400-523, 606-692 and 739-908): one round of work, step by step.

use super::{PathTrace, PathTraceWork, TraceHost};
use crate::integrator::{RenderScheduler, RenderWork, WorkBalanceInfo};
use crate::session::{RenderBuffer, TileManager};
use crate::util::{Clock, next_power_of_two};

/// Fewer open pixels than this leave a GPU idle, so the threshold is
/// lowered early. Cycles calls the number empirical.
const IDLE_PIXEL_COUNT: u32 = 128 * 128;

impl<W: PathTraceWork> PathTrace<W> {
    /// `render`: do `work`. Nothing when a cancel was asked for.
    pub fn render<C: Clock>(&mut self, work: &RenderWork, scheduler: &mut RenderScheduler<C>, tile_manager: &TileManager, host: &mut impl TraceHost) {
        if self.cancel_requested() {
            return;
        }
        scheduler.set_need_schedule_cryptomatte(self.need_cryptomatte);
        self.work.iter_mut().for_each(W::init_execution);
        self.render_pipeline(*work, scheduler, tile_manager, host);
        self.work.iter_mut().for_each(W::deinit_execution);
    }

    /// `render_pipeline`. Only the instant cancel is checked between steps:
    /// the user's cancel is the session's, and its work is finished here.
    fn render_pipeline<C: Clock>(&mut self, mut work: RenderWork, scheduler: &mut RenderScheduler<C>, tile_manager: &TileManager, host: &mut impl TraceHost) {
        scheduler.report_work_begin(&work);
        self.init_render_buffers(&work, tile_manager, host);
        self.rebalance(&work, scheduler, tile_manager);
        // Volume guiding is denoised at powers of two, so a work may not
        // run past the next one.
        scheduler.set_sample_per_update_limit(0);
        if self.has_volume {
            let rendered = scheduler.rendered_sample_count() as u32;
            scheduler.set_sample_per_update_limit((next_power_of_two(rendered) - rendered) as i32);
        }
        self.path_trace(&work, scheduler, host);
        if self.cancel_requested() {
            return;
        }
        self.adaptive_sample(&mut work, scheduler, tile_manager, host);
        if self.cancel_requested() {
            return;
        }
        self.cryptomatte_postprocess(&work);
        if self.cancel_requested() {
            return;
        }
        self.denoise(&work, scheduler, host);
        if self.cancel_requested() {
            return;
        }
        self.denoise_volume_guiding_buffer(&work, scheduler);
        if self.cancel_requested() {
            return;
        }
        self.write_tile_buffer(&work, tile_manager, host);
        self.update_display(&work, scheduler, host);
        host.update_progress(&work);
        if work.full_write {
            host.finish_tile_file();
        }
    }

    /// `init_render_buffers`.
    fn init_render_buffers(&mut self, work: &RenderWork, tile_manager: &TileManager, host: &mut impl TraceHost) {
        self.update_work_buffer_param_if_needed(work, tile_manager.tile_overscan());
        if work.init_render_buffers {
            self.work.iter_mut().for_each(W::zero_render_buffer);
            host.read_tile_buffer();
        }
    }

    /// `rebalance`: with one device there is nothing to share out. When
    /// the shares change, the tile is copied out, the works' buffers are
    /// sliced by the new weights, and it is copied back.
    fn rebalance<C: Clock>(&mut self, work: &RenderWork, scheduler: &mut RenderScheduler<C>, tile_manager: &TileManager) {
        if !work.rebalance || self.work.len() == 1 {
            return;
        }
        let start_time = scheduler.clock().now();
        let changed = WorkBalanceInfo::rebalance(&mut self.balance);
        if changed {
            let mut big_tile = RenderBuffer::default();
            big_tile.reset(&self.state.effective_big_tile_param);
            self.copy_to_render_buffer(&mut big_tile);
            self.state.need_reset_param = true;
            self.update_work_buffer_param_if_needed(work, tile_manager.tile_overscan());
            self.copy_from_render_buffer(&big_tile);
        }
        scheduler.report_rebalance_time(work, scheduler.clock().now() - start_time, changed);
    }

    /// `path_trace`: every work traces the samples into its slice. The
    /// scheduler is told how long it took and how busy the devices were.
    fn path_trace<C: Clock>(&mut self, work: &RenderWork, scheduler: &mut RenderScheduler<C>, host: &mut impl TraceHost) {
        let sample = work.path_trace;
        if sample.sample_count == 0 {
            return;
        }
        let start_time = scheduler.clock().now();
        for (device, info) in self.work.iter_mut().zip(&mut self.balance).filter(|(device, _)| !device.has_error()) {
            let work_start_time = scheduler.clock().now();
            let statistic = device.render_samples(sample.start_sample, sample.sample_count, sample.sample_offset);
            info.time_spent += scheduler.clock().now() - work_start_time;
            info.occupancy = statistic.occupancy;
        }
        let occupancy = self.balance.iter().map(|info| info.occupancy).sum::<f32>() / self.work.len() as f32;
        scheduler.report_path_trace_occupancy(work, occupancy);
        scheduler.report_path_trace_time(work, scheduler.clock().now() - start_time, self.is_cancel_requested(host));
    }

    /// `adaptive_sample`: filter, count the pixels still open, and go on
    /// with a lower threshold while the scheduler allows it: when every
    /// pixel converged, or once when so few are open that the device idles.
    fn adaptive_sample<C: Clock>(&mut self, work: &mut RenderWork, scheduler: &mut RenderScheduler<C>, tile_manager: &TileManager, host: &mut impl TraceHost) {
        if !work.adaptive_sampling.filter {
            return;
        }
        let mut did_reschedule_on_idle = false;
        loop {
            let adaptive = work.adaptive_sampling;
            let start_time = scheduler.clock().now();
            let active: u32 = self.work.iter_mut().map(|device| device.adaptive_sampling_converge_filter_count_active(adaptive.threshold, adaptive.reset)).sum();
            scheduler.report_adaptive_filter_time(work, scheduler.clock().now() - start_time, self.is_cancel_requested(host));
            let go_on = match (active, did_reschedule_on_idle) {
                (0, _) => scheduler.reschedule_on_converge(work, tile_manager),
                (_, true) => false,
                (count, false) if count < IDLE_PIXEL_COUNT => {
                    did_reschedule_on_idle = scheduler.reschedule_on_idle(work);
                    did_reschedule_on_idle
                }
                (_, false) => false,
            };
            if !go_on {
                break;
            }
        }
    }

    /// `cryptomatte_postprocess`.
    fn cryptomatte_postprocess(&mut self, work: &RenderWork) {
        if work.cryptomatte_postprocess {
            self.work.iter_mut().for_each(W::cryptomatte_postprocess);
        }
    }

    /// `denoise`. Without a denoiser nothing is timed.
    fn denoise<C: Clock>(&mut self, work: &RenderWork, scheduler: &mut RenderScheduler<C>, host: &mut impl TraceHost) {
        if !work.tile.denoise {
            return;
        }
        let start_time = scheduler.clock().now();
        let Some(denoised) = host.denoise(scheduler.rendered_sample_count()) else {
            return;
        };
        self.state.has_denoised_result |= denoised;
        scheduler.report_denoise_time(work, scheduler.clock().now() - start_time);
    }

    /// `denoise_volume_guiding_buffers`.
    fn denoise_volume_guiding_buffer<C: Clock>(&mut self, work: &RenderWork, scheduler: &mut RenderScheduler<C>) {
        if !self.has_volume || !work.volume_guiding_denoise {
            return;
        }
        let start_time = scheduler.clock().now();
        self.work.iter_mut().for_each(W::denoise_volume_guiding_buffer);
        scheduler.report_volume_guiding_denoise_time(work, scheduler.clock().now() - start_time);
    }

    /// `write_tile_buffer`: the one tile of a frame goes to the host, once;
    /// a tile of many to the tile file.
    fn write_tile_buffer(&mut self, work: &RenderWork, tile_manager: &TileManager, host: &mut impl TraceHost) {
        if !work.tile.write {
            return;
        }
        self.state.tile_written = true;
        host.write_tile(tile_manager.has_multiple_tile());
    }

    /// `update_display`. The denoised result is shown when the work allows
    /// it and there is one.
    fn update_display<C: Clock>(&mut self, work: &RenderWork, scheduler: &mut RenderScheduler<C>, host: &mut impl TraceHost) {
        if !work.display.update {
            return;
        }
        let start_time = scheduler.clock().now();
        let show_denoised = work.display.use_denoised_result && self.state.has_denoised_result;
        if host.update_display(show_denoised, scheduler.rendered_sample_count()) {
            scheduler.report_display_update_time(work, scheduler.clock().now() - start_time);
        }
    }
}
