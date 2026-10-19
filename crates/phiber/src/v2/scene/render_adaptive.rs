//! Adaptive Sampling as Cycles resolves it: the Render panel's three rows
//! turned into the numbers the kernel actually uses.
//!
//! Split out of `render_set.rs`, which holds the panel's other rows. The
//! panel values live on [`BRenderSet`]; resolving them is this file's one
//! job, so it sits beside that type rather than inside it.

use super::render_set::BRenderSet;

/// Adaptive Sampling as Cycles resolves it, ported from
/// `intern/cycles/scene/integrator.cpp`'s `Integrator::get_adaptive_sampling`
/// and `integrator/adaptive_sampling.cpp`.
///
/// The three panel rows are not the numbers the kernel uses. Blender's
/// manual says so without saying what the numbers are:
///
/// > **Noise Threshold** — "Setting this to 0 makes Cycles guess an
/// > automatic value based on the total sample count."
/// > **Min Samples** — "When set to 0 (default), Cycles automatically picks
/// > a value determined by the Noise Threshold."
///
/// Those guesses are below, exactly as Cycles makes them. Until now phiber
/// showed all three rows and honoured none of them: `BRenderSet` did not
/// read them and phiray had no notion of adaptive sampling, so every pixel
/// of every render got the full sample count — 2048 on pose 26, where the
/// flat floor settles in a handful.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BAdaptive {
    /// Off: every pixel takes every sample.
    pub use_it: bool,
    /// The threshold the kernel compares against (already × 5).
    pub threshold: f32,
    /// No pixel stops before this many samples.
    pub min_samples: u32,
    /// Convergence is only tested on samples where `needs_check` holds.
    pub step: u32,
}

impl BAdaptive {
    /// Cycles tests convergence every sixteenth sample, and the step must
    /// be a power of two for its bitwise test to work
    /// (`adaptive_sampling.cpp` asserts it).
    pub const STEP: u32 = 16;

    /// Resolve the panel's three rows against `samples`, the render's total.
    pub fn of(set: &BRenderSet, samples: u32) -> Self {
        if !set.adaptive {
            return Self { use_it: false, threshold: 0.0, min_samples: 0, step: Self::STEP };
        }
        // "if (clamped_aa_samples > 0 && adaptive_threshold == 0.0f)
        //      threshold = max(0.001f, 1.0f / (float)aa_samples);"
        let mut threshold = if samples > 0 && set.adaptive_threshold == 0.0 {
            0.001_f32.max(1.0 / samples as f32)
        } else {
            set.adaptive_threshold
        };
        // "Threshold 0.1 -> 32, 0.01 -> 64, 0.001 -> 128." Taken from the
        // threshold BEFORE the ×5 below, as Cycles takes it.
        let min_samples = if threshold > 0.0 && set.adaptive_min_samples == 0 {
            (16.0 / threshold.powf(0.3)).ceil() as u32
        } else {
            set.adaptive_min_samples
        }
        .max(4);
        // "Arbitrary factor that makes the threshold more similar to what
        // it was before, and gives arguably more intuitive values."
        threshold *= 5.0;
        Self { use_it: true, threshold, min_samples, step: Self::STEP }
    }

    /// `AdaptiveSampling::need_filter`: convergence is tested after the
    /// minimum is past, on the last sample of each step.
    pub fn needs_check(&self, sample: u32) -> bool {
        self.use_it && sample > self.min_samples && (sample & (self.step - 1)) == self.step - 1
    }
}

#[cfg(test)]
mod adaptive_tests {
    use super::*;

    fn set(adaptive: bool, threshold: f32, min: u32) -> BRenderSet {
        BRenderSet { adaptive, adaptive_threshold: threshold, adaptive_min_samples: min, ..Default::default() }
    }

    /// The three rows resolve exactly as `Integrator::get_adaptive_sampling`
    /// resolves them, including both "automatic" cases.
    #[test]
    fn cycles_rules_for_the_automatic_values() {
        // Blender's own default row: 0.01, min 0.
        let a = BAdaptive::of(&set(true, 0.01, 0), 2048);
        assert!((a.threshold - 0.05).abs() < 1e-6, "0.01 × 5 = 0.05, got {}", a.threshold);
        // The comment in Cycles promises 0.01 -> 64.
        assert_eq!(a.min_samples, 64, "16 / 0.01^0.3");
        assert_eq!(BAdaptive::of(&set(true, 0.1, 0), 2048).min_samples, 32, "Cycles: 0.1 -> 32");
        assert_eq!(BAdaptive::of(&set(true, 0.001, 0), 2048).min_samples, 128, "Cycles: 0.001 -> 128");

        // Threshold 0 is "automatic": max(0.001, 1 / samples), then × 5.
        // Below 1000 samples 1/samples clears the 0.001 floor, so this is
        // the case that exercises the division; at 2048 it would not, and
        // the floor below would be tested twice.
        let auto = BAdaptive::of(&set(true, 0.0, 512), 512);
        assert!((auto.threshold - 5.0 / 512.0).abs() < 1e-6, "1/512 clears the floor, then ×5: got {}", auto.threshold);
        let floored = BAdaptive::of(&set(true, 0.0, 0), 100_000);
        assert!((floored.threshold - 0.005).abs() < 1e-6, "the 0.001 floor, then ×5");

        // An explicit minimum is honoured, but never below four.
        assert_eq!(BAdaptive::of(&set(true, 0.01, 7), 2048).min_samples, 7);
        assert_eq!(BAdaptive::of(&set(true, 0.01, 1), 2048).min_samples, 4, "max(4, …)");

        assert!(!BAdaptive::of(&set(false, 0.01, 0), 2048).use_it, "the switch is honoured");
    }

    /// `need_filter`: nothing before the minimum, then every sixteenth.
    #[test]
    fn convergence_is_tested_every_sixteenth_sample_after_the_minimum() {
        let a = BAdaptive::of(&set(true, 0.01, 0), 2048); // min 64
        assert!(!a.needs_check(15), "inside the minimum");
        assert!(!a.needs_check(64), "the minimum itself is not past it");
        assert!(a.needs_check(79), "79 & 15 == 15, and 79 > 64");
        assert!(!a.needs_check(80));
        assert!(a.needs_check(95));
        assert!(!BAdaptive::of(&set(false, 0.01, 0), 2048).needs_check(95), "off means never");
    }
}
