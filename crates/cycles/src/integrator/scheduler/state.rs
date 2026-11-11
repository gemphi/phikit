// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/render_scheduler.h`
//! (`RenderScheduler::state_`, `RenderScheduler::first_render_time_`).

/// `state_`: where the scheduler is in the render.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SchedulerState {
    pub user_is_navigating: bool,
    pub resolution_divider: i32,
    /// `num_rendered_samples`: samples on top of the start sample.
    pub rendered_sample_count: i32,
    /// When display work was last scheduled.
    pub last_display_update_time: f64,
    /// -1 when the display was never updated.
    pub last_display_update_sample: i32,
    pub last_rebalance_time: f64,
    /// `num_rebalance_requested`. The path tracer ignores a request when one
    /// device renders.
    pub rebalance_request_count: i32,
    /// `num_rebalance_changes`: requests that moved work between devices.
    pub rebalance_change_count: i32,
    pub need_rebalance_at_next_work: bool,
    pub last_rebalance_changed: bool,
    /// The threshold scheduled while the noise floor is still being lowered.
    pub adaptive_sampling_threshold: f32,
    pub last_work_tile_was_denoised: bool,
    pub tile_result_was_written: bool,
    pub postprocess_work_scheduled: bool,
    pub full_frame_work_scheduled: bool,
    pub full_frame_was_written: bool,
    pub path_trace_finished: bool,
    pub time_limit_reached: bool,
    pub start_render_time: f64,
    pub end_render_time: f64,
    /// `occupancy_num_samples`: the samples of the work the occupancy was
    /// measured on. It trails: new work is planned from the last work's.
    pub occupancy_sample_count: i32,
    pub occupancy: f32,
}

impl Default for SchedulerState {
    fn default() -> Self {
        Self {
            user_is_navigating: false,
            resolution_divider: 1,
            rendered_sample_count: 0,
            last_display_update_time: 0.0,
            last_display_update_sample: -1,
            last_rebalance_time: 0.0,
            rebalance_request_count: 0,
            rebalance_change_count: 0,
            need_rebalance_at_next_work: false,
            last_rebalance_changed: false,
            adaptive_sampling_threshold: 0.0,
            last_work_tile_was_denoised: false,
            tile_result_was_written: false,
            postprocess_work_scheduled: false,
            full_frame_work_scheduled: false,
            full_frame_was_written: false,
            path_trace_finished: false,
            time_limit_reached: false,
            start_render_time: 0.0,
            end_render_time: 0.0,
            occupancy_sample_count: 0,
            occupancy: 1.0,
        }
    }
}

/// `first_render_time_`: how long the first full-resolution work took. The
/// resolution divider for navigation is worked out from it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FirstRenderTime {
    pub path_trace_per_sample: f64,
    pub denoise_time: f64,
    pub display_update_time: f64,
}
