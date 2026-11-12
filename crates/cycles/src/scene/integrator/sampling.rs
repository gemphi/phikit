// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/integrator.cpp`
//! (`Integrator::device_update`, lines 316-351 and 390-403): the sampling
//! pattern, the seed and the pixel jitter the kernel is given.

use super::Integrator;
use crate::kernel::flag;
use crate::util::{hash_uint, hash_uint2, hash_uint3, is_power_of_two, next_power_of_two};

impl Integrator {
    /// The sampling pattern the kernel runs and the length of its blue
    /// noise sequence. The rounding pattern is the pure one over a power
    /// of two; the first-sample pattern keeps one sample apart.
    pub(super) fn sampling_pattern(&self, sample_count: i32) -> (i32, i32) {
        let pattern = self.node.int("sampling_pattern");
        let pure = flag("SAMPLING_PATTERN_BLUE_NOISE_PURE") as i32;
        let round = pattern == flag("SAMPLING_PATTERN_BLUE_NOISE_ROUND") as i32;
        let first = pattern == flag("SAMPLING_PATTERN_BLUE_NOISE_FIRST") as i32;
        match (round, first, is_power_of_two(sample_count)) {
            (true, _, true) => (pure, sample_count),
            (true, _, false) => (pure, next_power_of_two(sample_count as u32) as i32),
            (false, true, _) => (pattern, sample_count - 1),
            (false, false, _) => (pattern, sample_count),
        }
    }

    /// The seed the kernel takes. With pixel jitter it changes every frame;
    /// blue noise needs it hashed, since a seed of 0 does not scramble.
    pub(super) fn seed(&self, pattern: i32) -> u32 {
        let seed = self.node.int("seed") as u32;
        let jitter = self.node.boolean("use_pixel_jitter");
        let custom = self.node.boolean("use_custom_pixel_jitter_sample");
        let blue_noise = pattern == flag("SAMPLING_PATTERN_BLUE_NOISE_FIRST") as i32 || pattern == flag("SAMPLING_PATTERN_BLUE_NOISE_PURE") as i32;
        match (jitter, custom, blue_noise) {
            (true, true, _) => hash_uint2(seed, self.pixel_jitter_frame as u32),
            (true, false, _) => hash_uint3(seed, self.pixel_jitter_state.a2 as u32, self.pixel_jitter_state.a3 as u32),
            (false, _, true) => hash_uint(seed),
            (false, _, false) => seed,
        }
    }

    /// This frame's pixel jitter, and the state moved on to the next
    /// frame's. Without jitter the kernel is told so by the largest float.
    pub(super) fn next_pixel_jitter(&mut self) -> [f32; 2] {
        let custom = self.node.float_array("custom_pixel_jitter_sample");
        match (self.node.boolean("use_pixel_jitter"), self.node.boolean("use_custom_pixel_jitter_sample")) {
            (true, true) => {
                let jitter = [custom.first().copied().unwrap_or(0.0), custom.get(1).copied().unwrap_or(0.0)];
                self.pixel_jitter_frame += 1;
                jitter
            }
            (true, false) => self.pixel_jitter_state.next(),
            (false, _) => {
                self.pixel_jitter_state.reset();
                [f32::MAX; 2]
            }
        }
    }
}

#[cfg(test)]
mod test;
