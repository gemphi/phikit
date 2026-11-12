// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/render_scheduler.cpp`
//! (`approximate_final_time`, `RenderScheduler::report_work_begin` and the
//! seven `report_*` calls, `work_is_usable_for_first_render_estimation`,
//! `work_report_reset_average`, `check_time_limit_reached`, lines 464-628
//! and 1235-1272): the path tracer telling the scheduler how long work took.

use super::RenderScheduler;
use crate::integrator::RenderWork;
use crate::util::Clock;

/// `approximate_final_time`: what `time` at the work's resolution divider
/// would have been at the final resolution.
pub fn approximate_final_time(work: &RenderWork, time: f64) -> f64 {
    match work.resolution_divider == 1.0 {
        true => time,
        false => time * f64::from(work.resolution_divider * work.resolution_divider),
    }
}

impl<C: Clock> RenderScheduler<C> {
    /// `report_work_begin`: the path tracer starts on `work`, the scene
    /// updated and the kernels loaded. The render time counts from the
    /// first samples at the final resolution. Post-processing work has no
    /// samples, so a start sample alone would not tell.
    pub fn report_work_begin(&mut self, work: &RenderWork) {
        let first = work.path_trace.sample_count != 0 && work.path_trace.start_sample == self.sample_offset;
        if work.resolution_divider == self.pixel_size as f32 && first {
            self.state.start_render_time = self.clock.now();
        }
    }

    /// `report_path_trace_time`, in seconds.
    pub fn report_path_trace_time(&mut self, work: &RenderWork, time: f64, is_cancelled: bool) {
        self.task_time.path_trace.add_wall(time);
        if is_cancelled {
            return;
        }
        let final_time = approximate_final_time(work, time);
        if self.work_is_usable_for_first_render_estimation(work) {
            self.first_render_time.path_trace_per_sample = final_time / f64::from(work.path_trace.sample_count);
        }
        if self.work_report_reset_average(work) {
            self.task_time.path_trace.reset_average();
        }
        self.task_time.path_trace.add_average(final_time, work.path_trace.sample_count);
    }

    /// `report_path_trace_occupancy`: how full the device's path states
    /// were while `work` rendered.
    pub fn report_path_trace_occupancy(&mut self, work: &RenderWork, occupancy: f32) {
        self.state.occupancy_sample_count = work.path_trace.sample_count;
        self.state.occupancy = occupancy;
    }

    /// `report_adaptive_filter_time`.
    pub fn report_adaptive_filter_time(&mut self, work: &RenderWork, time: f64, is_cancelled: bool) {
        self.task_time.adaptive_filter.add_wall(time);
        if is_cancelled {
            return;
        }
        if self.work_report_reset_average(work) {
            self.task_time.adaptive_filter.reset_average();
        }
        self.task_time.adaptive_filter.add_average(approximate_final_time(work, time), work.path_trace.sample_count);
    }

    /// `report_denoise_time`.
    pub fn report_denoise_time(&mut self, work: &RenderWork, time: f64) {
        self.task_time.denoise.add_wall(time);
        let final_time = approximate_final_time(work, time);
        if self.work_is_usable_for_first_render_estimation(work) {
            self.first_render_time.denoise_time = final_time;
        }
        if self.work_report_reset_average(work) {
            self.task_time.denoise.reset_average();
        }
        self.task_time.denoise.add_average(final_time, 1);
    }

    /// `report_volume_guiding_denoise_time`.
    pub fn report_volume_guiding_denoise_time(&mut self, work: &RenderWork, time: f64) {
        self.task_time.volume_guiding_denoise.add_wall(time);
        if self.work_report_reset_average(work) {
            self.task_time.volume_guiding_denoise.reset_average();
        }
        self.task_time.volume_guiding_denoise.add_average(approximate_final_time(work, time), work.path_trace.sample_count);
    }

    /// `report_display_update_time`. It also moves the last display update
    /// to now, so what is timed from it leaves out the tracing and
    /// denoising of the work.
    pub fn report_display_update_time(&mut self, work: &RenderWork, time: f64) {
        self.task_time.display_update.add_wall(time);
        let final_time = approximate_final_time(work, time);
        if self.work_is_usable_for_first_render_estimation(work) {
            self.first_render_time.display_update_time = final_time;
        }
        if self.work_report_reset_average(work) {
            self.task_time.display_update.reset_average();
        }
        self.task_time.display_update.add_average(final_time, 1);
        self.state.last_display_update_time = self.clock.now();
    }

    /// `report_rebalance_time`. `balance_changed` when work moved between
    /// devices.
    pub fn report_rebalance_time(&mut self, work: &RenderWork, time: f64, balance_changed: bool) {
        self.task_time.rebalance.add_wall(time);
        if self.work_report_reset_average(work) {
            self.task_time.rebalance.reset_average();
        }
        self.task_time.rebalance.add_average(time, 1);
        self.state.rebalance_change_count += i32::from(balance_changed);
        self.state.last_rebalance_changed = balance_changed;
    }

    /// `work_is_usable_for_first_render_estimation`: the first work at the
    /// final resolution.
    fn work_is_usable_for_first_render_estimation(&self, work: &RenderWork) -> bool {
        work.resolution_divider == self.pixel_size as f32 && work.path_trace.start_sample == self.sample_offset
    }

    /// `work_report_reset_average`: a smaller render would pull the average
    /// down, so only the latest resolution divider is averaged.
    fn work_report_reset_average(&self, work: &RenderWork) -> bool {
        work.resolution_divider != self.pixel_size as f32
    }

    /// `check_time_limit_reached`: past the limit the render counts as
    /// finished, and its end time is kept for the report.
    pub(crate) fn check_time_limit_reached(&mut self) {
        let started = self.state.start_render_time;
        let now = self.clock.now();
        if self.time_limit == 0.0 || started == 0.0 || now - started < self.time_limit {
            return;
        }
        self.state.time_limit_reached = true;
        self.state.end_render_time = now;
    }
}

#[cfg(test)]
mod test;
