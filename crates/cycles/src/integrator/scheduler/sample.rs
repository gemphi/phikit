// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/render_scheduler.cpp`
//! (`RenderScheduler::calculate_num_samples_per_update`,
//! `get_start_sample_to_path_trace`, `round_num_samples_to_power_of_2`,
//! `get_num_samples_to_path_trace`, `get_num_samples_during_navigation`,
//! lines 818-1001): how many samples the next work traces.

use super::RenderScheduler;
use crate::util::Clock;

pub use crate::util::is_power_of_two;

/// `round_num_samples_to_power_of_2`: the nearest power of two, above or
/// below, so the sample count stays close to the planned one.
pub fn round_to_power_of_two(sample_count: u32) -> u32 {
    if sample_count < 2 || sample_count.is_power_of_two() {
        return sample_count;
    }
    let up = sample_count.next_power_of_two();
    let down = up >> 1;
    match up - sample_count <= sample_count - down {
        true => up,
        false => down,
    }
}

impl<C: Clock> RenderScheduler<C> {
    /// `calculate_num_samples_per_update`: the samples that fit in the
    /// display update interval. One until a sample was timed.
    pub(crate) fn sample_count_per_update(&self) -> i32 {
        let time_per_sample = self.task_time.path_trace.average();
        if time_per_sample == 0.0 {
            return 1;
        }
        let sample_per_second = f64::from(self.pixel_size * self.pixel_size) / time_per_sample;
        ((sample_per_second * self.display_update_interval()) as i32).max(1)
    }

    /// `get_start_sample_to_path_trace`.
    pub(crate) fn start_sample_to_path_trace(&self) -> i32 {
        self.sample_offset + self.state.rendered_sample_count
    }

    /// `get_num_samples_to_path_trace`.
    pub(crate) fn sample_count_to_path_trace(&self) -> i32 {
        if self.state.resolution_divider != self.pixel_size {
            return self.sample_count_during_navigation(self.state.resolution_divider);
        }
        // A full-resolution render starts with one sample: quick feedback,
        // and a timing for the work after it. Headless too.
        if self.state.rendered_sample_count == 0 {
            return 1;
        }
        let start_sample = self.start_sample_to_path_trace();
        // A power of two divides the path states into tiles evenly.
        let per_update = round_to_power_of_two(self.sample_count_per_update() as u32) as i32;
        let most = self.sample_offset + self.sample_count - start_sample;
        let planned = per_update.min(most);
        // Offline, once the occupancy was measured, keep the device busy.
        let occupied = match self.state.occupancy_sample_count != 0 && (self.background || self.headless) {
            true => planned.max(self.sample_count_to_occupy().min(most)),
            false => planned,
        };
        let limited = match self.sample_per_update_limit {
            0 => occupied,
            limit => limit.min(occupied),
        };
        // Without adaptive sampling: as many samples as possible, with
        // little spent on display updates.
        match self.adaptive_sampling.enabled {
            true => self.adaptive_sampling.align_sample(start_sample - self.sample_offset, limited),
            false => limited,
        }
    }

    /// The samples that bring the device's occupancy to about a half, cut
    /// to what the path tracing time limit allows.
    fn sample_count_to_occupy(&self) -> i32 {
        let measured = self.state.occupancy_sample_count;
        let occupancy = self.state.occupancy;
        let (ratio, to_occupy) = match occupancy > 0.0 && occupancy < 0.5 {
            true => (0.7 / occupancy, (measured as f32 * (0.7 / occupancy)).round() as i32),
            false => (1.0, measured),
        };
        let limit = self.path_trace_time_limit();
        // The last batch's time per sample makes the correction quick; the
        // occupancy gained is expected to speed it up.
        let predicted = f64::from(to_occupy) * self.task_time.path_trace.last_sample_time() / f64::from(ratio);
        match limit != 0.0 && predicted > limit {
            true => (f64::from(to_occupy) * (limit / predicted)).round() as i32,
            false => to_occupy,
        }
    }

    /// How long one path tracing work may take, in seconds, so slow scenes
    /// with low occupancy still show results and cancel quickly. Zero is no
    /// limit. With a render time limit, never past what is left of it.
    fn path_trace_time_limit(&self) -> f64 {
        let started = self.state.start_render_time;
        let now = self.clock.now();
        let limit = match (self.headless, self.background) {
            // From a command line over-scheduling gives the best time.
            (true, _) => 0.0,
            // Quick updates for the first seconds, so a wrong render can be
            // stopped early; then long ones for the best speed.
            (false, true) => match started == 0.0 || now - started < 10.0 {
                true => 2.0,
                false => 15.0,
            },
            (false, false) => self.display_update_interval(),
        };
        if self.time_limit == 0.0 || started == 0.0 {
            return limit;
        }
        let remaining = (self.time_limit - (now - started)).max(0.0);
        match limit == 0.0 {
            true => remaining,
            false => limit.min(remaining),
        }
    }

    /// `get_num_samples_during_navigation`: several samples while the view
    /// is small, up to 4, so there is enough on screen. One with a denoiser
    /// running, which does better on more pixels with fewer samples.
    pub(crate) fn sample_count_during_navigation(&self, resolution_divider: i32) -> i32 {
        if self.is_denoise_active_during_update() {
            return 1;
        }
        // `resolution_divider_for_time` relies on this formula.
        (resolution_divider / self.pixel_size).max(1).min(self.sample_count.min(4))
    }
}

#[cfg(test)]
mod test;
