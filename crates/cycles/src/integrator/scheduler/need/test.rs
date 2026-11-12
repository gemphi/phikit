// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::integrator::scheduler::test::*;
use crate::device::DenoiseParam;

fn adaptive() -> AdaptiveSampling {
    AdaptiveSampling { enabled: true, adaptive_step: 4, min_sample: 0, threshold: 0.01 }
}

fn denoise(start_sample: i32) -> DenoiseParam {
    DenoiseParam { enabled: true, start_sample, ..DenoiseParam::default() }
}

#[test]
fn the_filter_runs_on_every_fourth_sample() {
    let mut rig = rig(background(), SMALL, 100);
    rig.scheduler.set_adaptive_sampling(adaptive());
    let filter: Vec<bool> = (0..8).map(|_| rig.work().adaptive_sampling.filter).collect();
    assert_eq!(filter, [false, false, false, true, false, false, false, true]);
}

#[test]
fn a_viewport_starts_at_the_high_threshold_and_a_background_render_at_its_own() {
    let mut offline = rig(background(), SMALL, 100);
    offline.scheduler.set_adaptive_sampling(adaptive());
    assert_eq!(offline.work().adaptive_sampling.threshold, 0.01);
    let mut live = rig(viewport(), SMALL, 100);
    live.scheduler.set_adaptive_sampling(adaptive());
    assert_eq!(live.work().adaptive_sampling.threshold, 0.4);
}

#[test]
fn a_viewport_is_not_denoised_before_the_start_sample() {
    let mut rig = rig(viewport(), SMALL, 100);
    rig.scheduler.set_denoise_param(denoise(2));
    let early = rig.work();
    assert!(!early.tile.denoise && !early.display.use_denoised_result);
    assert!(early.display.update);
    let start = rig.work();
    assert!(start.tile.denoise && start.display.use_denoised_result);
    assert!(rig.work().tile.denoise);
}

#[test]
fn a_long_viewport_render_is_denoised_at_most_once_a_second() {
    let mut rig = rig(viewport(), SMALL, 1000);
    rig.scheduler.set_denoise_param(denoise(1));
    let first = rig.work();
    assert!(first.tile.denoise);
    // Fast samples, and 5 seconds on the clock of the path tracer.
    rig.scheduler.report_path_trace_time(&first, 0.01, false);
    rig.scheduler.report_path_trace_time(&first, 5.0, true);
    let soon = rig.work();
    assert_eq!(rig.scheduler.rendered_sample_count(), 129);
    assert!(!soon.tile.denoise && !soon.display.update);
    rig.clock.advance(2.0);
    let later = rig.work();
    assert!(later.tile.denoise && later.display.update);
}

#[test]
fn a_background_render_is_denoised_at_its_last_sample() {
    let mut rig = rig(background(), SMALL, 3);
    rig.scheduler.set_denoise_param(denoise(0));
    let denoised: Vec<bool> = (0..4).map(|_| rig.work().tile.denoise).collect();
    assert_eq!(denoised, [false, false, true, false]);
}

#[test]
fn a_tile_of_many_is_not_denoised() {
    let mut rig = tiled(background(), [128, 64], [64, 64], 1);
    rig.scheduler.set_denoise_param(denoise(0));
    let last = rig.work();
    assert!(last.tile.write && !last.tile.denoise);
}

#[test]
fn a_headless_render_updates_no_display_while_it_samples() {
    let mut rig = rig(headless(), SMALL, 3);
    let update: Vec<bool> = (0..2).map(|_| rig.work().display.update).collect();
    assert_eq!(update, [false, false]);
}

#[test]
fn with_adaptive_sampling_display_updates_are_spaced_by_the_clock() {
    let mut rig = rig(background(), SMALL, 100);
    rig.scheduler.set_adaptive_sampling(adaptive());
    // The first result is always shown; then a second has to pass.
    let update: Vec<bool> = (0..3).map(|_| rig.work().display.update).collect();
    assert_eq!(update, [true, false, false]);
    rig.clock.advance(1.5);
    assert!(rig.work().display.update);
}

#[test]
fn devices_rebalance_after_the_first_sample_and_then_by_the_clock() {
    let mut rig = rig(background(), SMALL, 100);
    rig.scheduler.set_need_schedule_rebalance(true);
    let rebalance: Vec<bool> = (0..3).map(|_| rig.work().rebalance).collect();
    assert_eq!(rebalance, [false, true, false]);
    rig.clock.advance(2.0);
    let work = rig.work();
    assert!(work.rebalance);
    assert_eq!(rig.scheduler.state().rebalance_request_count, 2);
    // A rebalance that moved work is followed by another at once.
    rig.scheduler.report_rebalance_time(&work, 0.1, true);
    assert!(rig.work().rebalance);
    assert_eq!(rig.scheduler.state().rebalance_change_count, 1);
}

#[test]
fn one_device_never_rebalances() {
    let mut rig = rig(background(), SMALL, 100);
    let rebalance: Vec<bool> = (0..3).map(|_| rig.work().rebalance).collect();
    assert_eq!(rebalance, [false, false, false]);
}
