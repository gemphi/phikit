// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/render_scheduler.cpp`
//! (`RenderScheduler::work_need_adaptive_filter`, `work_adaptive_threshold`,
//! `work_need_denoise`, `work_need_update_display`, `work_need_rebalance`,
//! lines 1003-1150): what the work being planned needs besides samples.

use super::RenderScheduler;
use crate::session::TileManager;
use crate::util::Clock;

/// What `work_need_denoise` answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DenoiseNeed {
    pub denoise: bool,
    /// The denoiser is on but was put off to a later sample.
    pub delayed: bool,
    /// `false` when a denoised result may be older than the samples.
    pub ready_to_display: bool,
}

/// The least seconds between two rebalances.
const REBALANCE_INTERVAL: f64 = 1.0;

impl<C: Clock> RenderScheduler<C> {
    /// `work_need_adaptive_filter`.
    pub(crate) fn work_need_adaptive_filter(&self) -> bool {
        self.adaptive_sampling.need_filter(self.rendered_sample())
    }

    /// `work_adaptive_threshold`.
    pub(crate) fn work_adaptive_threshold(&self) -> f32 {
        match self.use_progressive_noise_floor {
            true => self.state.adaptive_sampling_threshold.max(self.adaptive_sampling.threshold),
            false => self.adaptive_sampling.threshold,
        }
    }

    /// `work_need_denoise`.
    pub(crate) fn work_need_denoise(&self, tile_manager: &TileManager) -> DenoiseNeed {
        let need = |denoise| DenoiseNeed { denoise, delayed: false, ready_to_display: true };
        // No denoiser; or many tiles, and then the full frame is denoised.
        if !self.denoise_param.enabled || tile_manager.has_multiple_tile() {
            return need(false);
        }
        // Always at the last sample. A background render only then.
        if self.done() || self.background {
            return need(self.done());
        }
        // Navigation renders several samples smaller; they count as one.
        let finished = match self.state.resolution_divider == self.pixel_size {
            true => self.state.rendered_sample_count,
            false => 1,
        };
        let start = self.denoise_param.start_sample;
        if finished == start || finished == self.sample_count {
            return need(true);
        }
        if finished < start {
            return DenoiseNeed { denoise: false, delayed: false, ready_to_display: false };
        }
        // Past some samples and seconds, a viewport is denoised less often.
        let since_display = self.clock.now() - self.state.last_display_update_time;
        let delayed = self.task_time.path_trace.wall() > 4.0 && finished >= 20 && since_display < 1.0;
        DenoiseNeed { denoise: !delayed, delayed, ready_to_display: true }
    }

    /// `work_need_update_display`. `denoiser_delayed` is what
    /// `work_need_denoise` answered.
    pub(crate) fn work_need_update_display(&self, denoiser_delayed: bool) -> bool {
        // Nothing to show on; or the display would be behind the render.
        if self.headless || denoiser_delayed {
            return false;
        }
        // Without adaptive sampling the work keeps the device busy long
        // enough for an update after each. With it, the first and the last
        // result are always shown.
        if !self.adaptive_sampling.enabled || self.done() || self.state.last_display_update_sample == -1 {
            return true;
        }
        // A fast device may get only a few samples of a simple scene, so no
        // filtering point is missed. Updates are spaced by the clock then.
        let interval = self.display_update_interval_for(self.state.last_display_update_sample);
        self.clock.now() - self.state.last_display_update_time > interval
    }

    /// `work_need_rebalance`: whether devices share the work out again.
    pub(crate) fn work_need_rebalance(&mut self) -> bool {
        // Not while navigating: it would stutter, and the timings of the
        // devices are not reliable yet.
        if !self.need_schedule_rebalance || self.state.resolution_divider != self.pixel_size {
            return false;
        }
        if self.state.rendered_sample_count == 0 {
            self.state.need_rebalance_at_next_work = true;
            return false;
        }
        if self.state.need_rebalance_at_next_work {
            self.state.need_rebalance_at_next_work = false;
            return true;
        }
        self.state.last_rebalance_changed || self.clock.now() - self.state.last_rebalance_time > REBALANCE_INTERVAL
    }
}

#[cfg(test)]
mod test;
