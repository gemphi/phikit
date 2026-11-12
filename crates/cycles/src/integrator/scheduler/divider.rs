// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/render_scheduler.cpp`
//! (`RenderScheduler::update_start_resolution_divider`,
//! `calculate_resolution_divider_for_time`, lines 1152-1198 and 1278-1299):
//! how much smaller a moving view is rendered, from how long the first
//! full-resolution work took.

use super::RenderScheduler;
use crate::util::Clock;

/// The long side of the view, in pixels, a divider never goes below.
const MIN_VIEW_SIDE: i32 = 128;

impl<C: Clock> RenderScheduler<C> {
    /// `update_start_resolution_divider`.
    pub(crate) fn update_start_resolution_divider(&mut self) {
        if self.default_start_resolution_divider == 0 {
            return;
        }
        let long_side = self.buffer_param.width.max(self.buffer_param.height);
        let most = long_side / MIN_VIEW_SIDE;
        // Never worked out before: start high, to get real timings.
        if self.start_resolution_divider == 0 {
            self.start_resolution_divider = self.default_start_resolution_divider.min(most);
            return;
        }
        // Nothing timed yet: keep the divider.
        let first = self.first_render_time;
        if first.path_trace_per_sample == 0.0 {
            return;
        }
        let actual = first.path_trace_per_sample + first.denoise_time + first.display_update_time;
        // Some tolerance, so a time near the higher resolution keeps it.
        let desired = self.viewport_navigation_update_interval() * 1.4;
        // Slow is better than an unreadable view.
        self.start_resolution_divider = self.resolution_divider_for_time(desired, actual).min(most);
    }

    /// `calculate_resolution_divider_for_time`: the smallest divider that
    /// brings `actual_time` under `desired_time`. Time is taken to follow
    /// the pixel count: a divider of 2 makes it a quarter.
    ///
    /// `actual / desired = divider² / sample_count`, and up to a divider of
    /// 4 the samples traced while navigating equal the divider, so the
    /// ratio stands in for the divider when asking for that sample count.
    pub(crate) fn resolution_divider_for_time(&self, desired_time: f64, actual_time: f64) -> i32 {
        let ratio = actual_time / desired_time;
        let navigation_sample = self.sample_count_during_navigation(ceil_to_int(ratio));
        ceil_to_int((f64::from(navigation_sample) * ratio).sqrt())
    }
}

/// `ceil_to_int` (`util/math_base.h`), which rounds a float up.
fn ceil_to_int(value: f64) -> i32 {
    (value as f32).ceil() as i32
}

#[cfg(test)]
mod test;
