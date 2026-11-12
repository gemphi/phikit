// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/render_scheduler.h` and
//! `render_scheduler.cpp` (`RenderScheduler`): what to render next, how many
//! samples, at what resolution, and when to filter, denoise and show.
//!
//! Two things differ from Cycles. It keeps a reference to the tile manager;
//! here the calls that read it are handed it. It reads a global clock; here
//! the scheduler holds a [`Clock`].

mod divider;
mod interval;
mod need;
mod report;
mod reschedule;
mod resolution;
mod sample;
mod schedule;
mod state;
mod summary;
mod time;

pub use resolution::{divider_for_resolution, resolution_for_divider};
pub use state::{FirstRenderTime, SchedulerState};
pub use time::{TaskTime, TimeAverage};

pub use report::approximate_final_time;
pub use sample::{is_power_of_two, round_to_power_of_two};

use super::AdaptiveSampling;
use crate::device::DenoiseParam;
use crate::scene::MAX_SAMPLE;
use crate::session::{BufferParam, SessionParam};
use crate::util::Clock;

/// `RenderScheduler`.
#[derive(Debug, Clone)]
pub struct RenderScheduler<C: Clock> {
    clock: C,
    state: SchedulerState,
    first_render_time: FirstRenderTime,
    task_time: TaskTime,
    need_schedule_cryptomatte: bool,
    /// `need_schedule_rebalance_works_`.
    need_schedule_rebalance: bool,
    /// Samples `sample_offset..sample_offset + sample_count` are traced.
    sample_offset: i32,
    sample_count: i32,
    /// Seconds path tracing may take. Zero is no limit.
    time_limit: f64,
    headless: bool,
    background: bool,
    /// Past 1 the final render is smaller, for a high-density display.
    pixel_size: i32,
    buffer_param: BufferParam,
    denoise_param: DenoiseParam,
    adaptive_sampling: AdaptiveSampling,
    /// Lower the adaptive threshold step by step, keeping the noise even.
    use_progressive_noise_floor: bool,
    /// The divider used before anything was timed, and the most the timed
    /// divider may be. Zero turns the divider off.
    default_start_resolution_divider: i32,
    start_resolution_divider: i32,
    /// `limit_samples_per_update_`: zero is no limit.
    sample_per_update_limit: i32,
}

impl<C: Clock> RenderScheduler<C> {
    pub fn new(clock: C, param: &SessionParam) -> Self {
        Self {
            clock,
            state: SchedulerState::default(),
            first_render_time: FirstRenderTime::default(),
            task_time: TaskTime::default(),
            need_schedule_cryptomatte: false,
            need_schedule_rebalance: false,
            sample_offset: 0,
            sample_count: 0,
            time_limit: 0.0,
            headless: param.headless,
            background: param.background,
            pixel_size: param.pixel_size,
            buffer_param: BufferParam::default(),
            denoise_param: DenoiseParam::default(),
            adaptive_sampling: AdaptiveSampling::default(),
            use_progressive_noise_floor: !param.background,
            default_start_resolution_divider: match param.use_resolution_divider {
                true => param.pixel_size * 8,
                false => 0,
            },
            start_resolution_divider: 0,
            sample_per_update_limit: 0,
        }
    }

    pub fn set_need_schedule_cryptomatte(&mut self, need: bool) {
        self.need_schedule_cryptomatte = need;
    }

    /// Off, as much as possible goes to one device.
    pub fn set_need_schedule_rebalance(&mut self, need: bool) {
        self.need_schedule_rebalance = need;
    }

    pub fn is_background(&self) -> bool {
        self.background
    }

    /// `set_denoiser_params`.
    pub fn set_denoise_param(&mut self, param: DenoiseParam) {
        self.denoise_param = param;
    }

    pub fn is_denoiser_gpu_used(&self) -> bool {
        self.denoise_param.use_gpu
    }

    /// `set_limit_samples_per_update`: a limit already set is only lowered.
    pub fn set_sample_per_update_limit(&mut self, limit: i32) {
        self.sample_per_update_limit = match self.sample_per_update_limit {
            0 => limit,
            current => current.min(limit),
        };
    }

    pub fn set_adaptive_sampling(&mut self, adaptive_sampling: AdaptiveSampling) {
        self.adaptive_sampling = adaptive_sampling;
    }

    pub fn is_adaptive_sampling_used(&self) -> bool {
        self.adaptive_sampling.enabled
    }

    /// `set_sample_params`: the range of samples to render. A subset, used
    /// to share one frame between computers, starts at `subset_offset` and
    /// is cut to the frame's `sample_count`.
    pub fn set_sample_param(&mut self, sample_count: i32, use_subset: bool, subset_offset: i32, subset_length: i32) {
        let count = sample_count.min(MAX_SAMPLE);
        (self.sample_offset, self.sample_count) = match use_subset {
            true => (subset_offset, ((subset_offset + subset_length).min(count) - subset_offset).max(0)),
            false => (0, count),
        };
    }

    /// `get_num_samples`.
    pub fn sample_count(&self) -> i32 {
        self.sample_count
    }

    /// `get_sample_offset`: what a subset adds to the sample index, so the
    /// sampling pattern is shifted.
    pub fn sample_offset(&self) -> i32 {
        self.sample_offset
    }

    pub fn set_time_limit(&mut self, time_limit: f64) {
        self.time_limit = time_limit;
    }

    /// `get_time_limit`.
    pub fn time_limit(&self) -> f64 {
        self.time_limit
    }

    /// `get_rendered_sample`: the 0-based sample rendering has reached.
    /// Meaningless before a sample was rendered.
    pub fn rendered_sample(&self) -> i32 {
        self.rendered_sample_count() - 1
    }

    /// `get_num_rendered_samples`: samples scheduled since the last reset.
    /// Work asked for counts as rendered.
    pub fn rendered_sample_count(&self) -> i32 {
        self.state.rendered_sample_count
    }

    /// The clock the scheduler reads. The path tracer times its steps by it.
    pub fn clock(&self) -> &C {
        &self.clock
    }

    /// The scheduler's state, to read.
    pub fn state(&self) -> &SchedulerState {
        &self.state
    }

    /// The scheduler's timers, to read.
    pub fn task_time(&self) -> &TaskTime {
        &self.task_time
    }

    /// `reset`: rendering starts from scratch on `buffer`. An interactive
    /// render starts small, at the start resolution divider.
    pub fn reset(&mut self, buffer: &BufferParam) {
        self.buffer_param = buffer.clone();
        self.update_start_resolution_divider();
        // A background render is never shown smaller.
        let navigate = !self.background && self.start_resolution_divider != 0;
        self.state = SchedulerState {
            user_is_navigating: self.state.user_is_navigating || navigate,
            resolution_divider: match navigate {
                true => self.start_resolution_divider,
                false => 1,
            },
            // The adaptive sampling settings may not be known here yet.
            adaptive_sampling_threshold: 0.4,
            ..SchedulerState::default()
        };
        self.first_render_time = FirstRenderTime::default();
        self.task_time = TaskTime::default();
    }

    /// `reset_for_next_tile`: the same samples and frame, from the start of
    /// a new tile.
    pub fn reset_for_next_tile(&mut self) {
        let buffer = self.buffer_param.clone();
        self.reset(&buffer);
    }
}

#[cfg(test)]
pub(crate) mod test;
