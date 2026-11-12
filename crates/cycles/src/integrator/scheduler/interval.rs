// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/render_scheduler.cpp`
//! (`RenderScheduler::guess_display_update_interval_in_seconds` and its two
//! variants, `guess_viewport_navigation_update_interval_in_seconds`,
//! `is_denoise_active_during_update`, lines 747-816 and 1200-1233): how
//! often the picture is shown. Early and at few samples often; later
//! seldom, so the device stays busy.

use super::RenderScheduler;
use crate::util::Clock;

impl<C: Clock> RenderScheduler<C> {
    /// `guess_display_update_interval_in_seconds`.
    pub(crate) fn display_update_interval(&self) -> f64 {
        self.display_update_interval_for(self.state.rendered_sample_count)
    }

    /// `guess_display_update_interval_in_seconds_for_num_samples`: never
    /// past what is left of the render time limit.
    pub(crate) fn display_update_interval_for(&self, rendered_sample_count: i32) -> f64 {
        let interval = self.display_update_interval_no_limit(rendered_sample_count);
        let started = self.state.start_render_time;
        match self.time_limit != 0.0 && started != 0.0 {
            true => interval.min((self.time_limit - (self.clock.now() - started)).max(0.0)),
            false => interval,
        }
    }

    /// `guess_display_update_interval_in_seconds_for_num_samples_no_limit`.
    pub(crate) fn display_update_interval_no_limit(&self, rendered_sample_count: i32) -> f64 {
        if self.state.need_rebalance_at_next_work {
            return 0.1;
        }
        if self.state.last_rebalance_changed {
            return 0.2;
        }
        // Headless, updates are rare: the device stays busy and the log
        // still shows progress.
        if self.headless {
            return 30.0;
        }
        if self.background {
            return match rendered_sample_count < 32 {
                true => 1.0,
                false => 2.0,
            };
        }
        // The render time gives fast updates in the first seconds on a
        // fast device; the sample count gives them on a slow one.
        match self.task_time.path_trace.wall() {
            time if time < 1.0 => 0.1,
            time if time < 2.0 => 0.25,
            time if time < 4.0 => 0.5,
            time if time < 8.0 || rendered_sample_count < 32 => 1.0,
            _ => 2.0,
        }
    }

    /// `guess_viewport_navigation_update_interval_in_seconds`: 30 updates a
    /// second while navigating. 60 would match Blender's viewport, but only
    /// at dividers too high to look at. 12 with a denoiser, which needs
    /// more pixels to rebuild the image from.
    pub(crate) fn viewport_navigation_update_interval(&self) -> f64 {
        match self.is_denoise_active_during_update() {
            true => 1.0 / 12.0,
            false => 1.0 / 30.0,
        }
    }

    /// `is_denoise_active_during_update`: denoising runs while the view is
    /// still small.
    pub(crate) fn is_denoise_active_during_update(&self) -> bool {
        self.denoise_param.enabled && self.denoise_param.start_sample <= 1
    }
}

#[cfg(test)]
mod test;
