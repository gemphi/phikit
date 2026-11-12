// SPDX-License-Identifier: Apache-2.0
//! Tests of `scene/integrator/tabulated.rs`: the points of the tabulated
//! Sobol sampler and the table the scene's update makes of them.
use crate::scene::{DeviceScene, fill_sample_pattern, tabulated_sobol_generate_4d};

// The expected points are `tabulated_sobol_generate_4D` written out apart
// from this port, in Python (`_temp/tab_ref.py` at the time of writing).
#[test]
fn the_points_are_those_of_the_generator() {
    let points = tabulated_sobol_generate_4d(8, 3);
    let expect = [[0.365_471_72, 0.729_411_84, 0.245_753_91, 0.669_897_14], [0.717_199_5, 0.230_378_08, 0.843_455_08, 0.483_577_43], [0.011_972_894, 0.488_698_43, 0.730_844_86, 0.150_212_82]];
    for (point, expect) in points.iter().zip(expect) {
        for axis in 0..4 {
            assert!((point[axis] - expect[axis]).abs() < 1e-6, "{point:?} against {expect:?}");
        }
    }
    let last = points[7];
    assert!((last[0] - 0.839_311_7).abs() < 1e-6 && (last[3] - 0.775_633_3).abs() < 1e-6);
}

#[test]
fn the_first_points_are_stratified_in_every_dimension() {
    // Every power-of-two count of points puts one in each interval of 1 / count.
    for count in [2usize, 4, 16, 64, 256] {
        let points = tabulated_sobol_generate_4d(256, 5);
        for axis in 0..4 {
            let mut cell = vec![0u32; count];
            points[..count].iter().for_each(|point| cell[(point[axis] * count as f32) as usize] += 1);
            assert!(cell.iter().all(|taken| *taken == 1), "axis {axis}, {count} points: {cell:?}");
        }
    }
}

#[test]
fn a_seed_is_a_pattern_and_a_size_that_is_not_a_power_of_two_is_a_prefix() {
    let a = tabulated_sobol_generate_4d(16, 1);
    assert_ne!(a, tabulated_sobol_generate_4d(16, 2));
    assert_eq!(a, tabulated_sobol_generate_4d(16, 1));
    assert_eq!(tabulated_sobol_generate_4d(10, 1), a[..10]);
    assert!(tabulated_sobol_generate_4d(0, 1).is_empty());
    assert!(a.iter().flatten().all(|value| (0.0..1.0).contains(value)));
}

#[test]
fn the_table_is_made_when_the_pattern_is_the_tabulated_one() {
    let mut dscene = DeviceScene::new();
    dscene.data.set_int("integrator", "sampling_pattern", 1);
    dscene.data.set_int("integrator", "tabulated_sobol_sequence_size", 256);
    fill_sample_pattern(&mut dscene);
    assert_eq!(dscene.sample_pattern_lut.len(), 256 * 256 * 4);
    // Pattern 3 starts at its place, with its own seed.
    let third = tabulated_sobol_generate_4d(256, 3);
    assert_eq!(&dscene.sample_pattern_lut[3 * 256 * 4..3 * 256 * 4 + 4], &third[0]);
    // Another pattern leaves the table empty.
    let mut other = DeviceScene::new();
    other.data.set_int("integrator", "sampling_pattern", 2);
    other.data.set_int("integrator", "tabulated_sobol_sequence_size", 256);
    fill_sample_pattern(&mut other);
    assert!(other.sample_pattern_lut.is_empty());
}
