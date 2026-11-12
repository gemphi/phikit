// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Cycles' own tests, ported from
//! `intern/cycles/test/integrator_adaptive_sampling_test.cpp`.
use crate::integrator::*;

fn sampling(min_sample: i32) -> AdaptiveSampling {
    AdaptiveSampling { enabled: true, adaptive_step: 4, min_sample, threshold: 0.0 }
}

#[test]
fn schedule_samples() {
    let adaptive = sampling(0);
    for sample in 2..32 {
        for sample_count in 8..32 {
            let aligned = adaptive.align_sample(sample, sample_count);
            // `sample + aligned` is the count after rendering; the last sample is one before.
            assert!(adaptive.need_filter(sample + aligned - 1), "from {sample}, {sample_count} asked");
        }
    }
}

#[test]
fn align_samples() {
    // Rounded `sqrt(128)`. Filtering happens at 15, 19, 23, 27, 31, 35, 39, 43.
    let adaptive = sampling(11);
    let expect = [
        // Fewer samples than the minimum.
        ((0, 4), 4),
        ((0, 7), 7),
        // Past the minimum, before the first filtered sample.
        ((0, 15), 15),
        // Many samples from the start stop at the first filtered sample.
        ((0, 16), 16),
        ((0, 17), 16),
        ((0, 20), 16),
        ((0, 60), 16),
        // The same, not starting at 0.
        ((9, 8), 7),
        ((9, 20), 7),
        ((9, 60), 7),
        // Starting past the minimum, before the first filtered sample.
        ((12, 6), 4),
        ((12, 20), 4),
        ((12, 60), 4),
        // Starting on a filtered sample.
        ((15, 4), 1),
        ((15, 6), 1),
        ((15, 10), 1),
        ((58, 2), 2),
        // Starting past one.
        ((16, 3), 3),
        ((16, 4), 4),
        ((16, 5), 4),
        ((16, 10), 4),
        // Never more than asked.
        ((15, 2), 1),
        ((16, 2), 2),
        ((17, 2), 2),
        ((18, 2), 2),
    ];
    for ((start, count), aligned) in expect {
        assert_eq!(adaptive.align_sample(start, count), aligned, "align_sample({start}, {count})");
    }
}

#[test]
fn need_filter() {
    let adaptive = sampling(11);
    let filtered: Vec<i32> = (0..60).filter(|sample| adaptive.need_filter(*sample)).collect();
    assert_eq!(filtered, [15, 19, 23, 27, 31, 35, 39, 43, 47, 51, 55, 59]);
}

#[test]
fn without_adaptive_sampling_nothing_is_aligned_or_filtered() {
    let off = AdaptiveSampling { enabled: false, ..sampling(11) };
    assert_eq!(off.align_sample(0, 60), 60);
    assert!(!(0..60).any(|sample| off.need_filter(sample)));
}
