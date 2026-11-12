// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::integrator::scheduler::test::*;
use crate::device::DenoiseParam;

#[test]
fn a_sample_count_is_rounded_to_the_nearer_power_of_two() {
    let rounded: Vec<u32> = [0, 1, 2, 3, 5, 6, 7, 9, 11, 12, 95, 100, 1000].into_iter().map(round_to_power_of_two).collect();
    assert_eq!(rounded, [0, 1, 2, 4, 4, 8, 8, 8, 8, 16, 64, 128, 1024]);
}

#[test]
fn zero_counts_as_a_power_of_two_as_in_cycles() {
    let power: Vec<i32> = (-1..=9).filter(|value| is_power_of_two(*value)).collect();
    assert_eq!(power, [0, 1, 2, 4, 8]);
}

#[test]
fn full_resolution_starts_with_one_sample() {
    let mut rig = rig(headless(), SMALL, 1000);
    assert_eq!(rig.work().path_trace.sample_count, 1);
}

#[test]
fn the_samples_of_a_work_fill_the_display_update_interval() {
    let mut rig = rig(background(), SMALL, 1000);
    assert_eq!(rig.scheduler.sample_count_per_update(), 1);
    let first = rig.work();
    // Half a second a sample, an update every second.
    rig.scheduler.report_path_trace_time(&first, 0.5, false);
    assert_eq!(rig.scheduler.sample_count_per_update(), 2);
    assert_eq!(rig.work().path_trace.sample_count, 2);
}

#[test]
fn a_work_never_goes_past_the_last_sample() {
    let mut rig = rig(background(), SMALL, 1000);
    let first = rig.work();
    rig.scheduler.report_path_trace_time(&first, 0.001, false);
    let rest = rig.work();
    assert_eq!((rest.path_trace.start_sample, rest.path_trace.sample_count), (1, 999));
}

#[test]
fn the_limit_caps_the_samples_of_a_work() {
    let mut rig = rig(background(), SMALL, 1000);
    let first = rig.work();
    rig.scheduler.report_path_trace_time(&first, 0.001, false);
    rig.scheduler.set_sample_per_update_limit(16);
    // A limit already set is only lowered.
    rig.scheduler.set_sample_per_update_limit(64);
    assert_eq!(rig.work().path_trace.sample_count, 16);
}

#[test]
fn a_subset_starts_at_its_offset() {
    let mut rig = rig(background(), SMALL, 100);
    rig.scheduler.set_sample_param(100, true, 90, 20);
    let work = rig.work();
    assert_eq!((work.path_trace.start_sample, work.path_trace.sample_offset), (90, 90));
    assert!(work.init_render_buffers);
}

#[test]
fn a_small_view_gets_a_sample_for_each_step_of_its_divider_up_to_four() {
    let rig = rig(viewport(), SMALL, 100);
    let count: Vec<i32> = [1, 2, 3, 4, 8, 30].into_iter().map(|divider| rig.scheduler.sample_count_during_navigation(divider)).collect();
    assert_eq!(count, [1, 2, 3, 4, 4, 4]);
    let few = crate::integrator::scheduler::test::rig(viewport(), SMALL, 2);
    assert_eq!(few.scheduler.sample_count_during_navigation(8), 2);
}

#[test]
fn a_small_view_gets_one_sample_when_it_is_denoised() {
    let mut rig = rig(viewport(), SMALL, 100);
    rig.scheduler.set_denoise_param(DenoiseParam { enabled: true, start_sample: 1, ..DenoiseParam::default() });
    assert_eq!(rig.scheduler.sample_count_during_navigation(8), 1);
}

/// One sample traced in 100 seconds at the given occupancy; the samples of
/// the work after it.
fn after_slow_sample(param: SessionParam, occupancy: f32) -> i32 {
    let mut rig = rig(param, SMALL, 1000);
    let first = rig.work();
    rig.scheduler.report_path_trace_time(&first, 100.0, false);
    rig.scheduler.report_path_trace_occupancy(&first, occupancy);
    rig.work().path_trace.sample_count
}

#[test]
fn a_headless_render_schedules_more_to_fill_an_idle_device() {
    // 0.7 / 0.25 times the one sample measured, rounded.
    assert_eq!(after_slow_sample(headless(), 0.25), 3);
    assert_eq!(after_slow_sample(headless(), 0.9), 1);
}

#[test]
fn a_background_render_keeps_a_work_short_in_its_first_seconds() {
    // Three samples would take far past the 2 seconds a work may.
    assert_eq!(after_slow_sample(background(), 0.25), 1);
}

#[test]
fn adaptive_sampling_ends_a_work_on_a_filtered_sample() {
    let mut rig = rig(background(), SMALL, 1000);
    rig.scheduler.set_adaptive_sampling(AdaptiveSampling { enabled: true, adaptive_step: 16, min_sample: 0, threshold: 0.01 });
    let first = rig.work();
    rig.scheduler.report_path_trace_time(&first, 0.001, false);
    // 999 samples fit the interval; the first filtered sample is 15.
    let work = rig.work();
    assert_eq!((work.path_trace.start_sample, work.path_trace.sample_count), (1, 15));
    assert!(work.adaptive_sampling.filter);
}
