// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/adaptive_sampling.h` and
//! `adaptive_sampling.cpp` (`AdaptiveSampling`).

/// `AdaptiveSampling`: when the convergence filter runs. It runs on every
/// `adaptive_step`-th sample (a power of two) past the minimum.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AdaptiveSampling {
    /// `use`.
    pub enabled: bool,
    pub adaptive_step: i32,
    /// `min_samples`: no filtering at or before this sample.
    pub min_sample: i32,
    pub threshold: f32,
}

impl AdaptiveSampling {
    /// `align_samples`: how many of `sample_count` samples to render from
    /// `start_sample` so the batch ends on a sample that is filtered.
    pub fn align_sample(&self, start_sample: i32, sample_count: i32) -> i32 {
        // The 0-based sample at which filtering first happens.
        let first_filter = (self.min_sample + 1) | (self.adaptive_step - 1);
        let next_filter = first_filter.max(start_sample | (self.adaptive_step - 1));
        match (self.enabled, start_sample + sample_count <= first_filter) {
            (false, _) | (true, true) => sample_count,
            (true, false) => (next_filter - start_sample + 1).min(sample_count),
        }
    }

    /// `need_filter`: whether the filter runs after 0-based `sample`.
    pub fn need_filter(&self, sample: i32) -> bool {
        self.enabled && sample > self.min_sample && (sample & (self.adaptive_step - 1)) == (self.adaptive_step - 1)
    }
}

#[cfg(test)]
mod test;
