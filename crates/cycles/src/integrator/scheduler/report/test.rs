// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::integrator::scheduler::test::*;

#[test]
fn a_time_at_a_divider_is_scaled_by_its_square() {
    let at = |resolution_divider| RenderWork { resolution_divider, ..RenderWork::default() };
    assert_eq!(approximate_final_time(&at(1.0), 3.0), 3.0);
    assert_eq!(approximate_final_time(&at(4.0), 3.0), 48.0);
}

#[test]
fn the_render_time_counts_from_the_first_full_resolution_samples() {
    let mut rig = rig(viewport(), [1024, 512], 100);
    let small = rig.work();
    rig.scheduler.report_work_begin(&small);
    assert_eq!(rig.scheduler.state().start_render_time, 0.0);
    rig.clock.advance(1.0);
    let full = rig.work();
    rig.scheduler.report_work_begin(&full);
    assert_eq!(rig.scheduler.state().start_render_time, 101.0);
    // Later work does not move it.
    rig.clock.advance(1.0);
    let next = rig.work();
    rig.scheduler.report_work_begin(&next);
    assert_eq!(rig.scheduler.state().start_render_time, 101.0);
}

#[test]
fn path_tracing_is_averaged_for_each_sample_at_the_final_resolution() {
    let mut rig = rig(viewport(), [1024, 512], 100);
    let small = rig.work();
    // 4 samples at a divider of 8 in one second: 64 seconds at full size.
    rig.scheduler.report_path_trace_time(&small, 1.0, false);
    assert_eq!(rig.scheduler.task_time().path_trace.wall(), 1.0);
    assert_eq!(rig.scheduler.task_time().path_trace.average(), 16.0);
    let full = rig.work();
    rig.scheduler.report_path_trace_time(&full, 2.0, false);
    assert_eq!(rig.scheduler.task_time().path_trace.wall(), 3.0);
    assert_eq!(rig.scheduler.task_time().path_trace.average(), 66.0 / 5.0);
    assert_eq!(rig.scheduler.task_time().path_trace.last_sample_time(), 2.0);
}

#[test]
fn a_cancelled_work_adds_to_the_wall_time_alone() {
    let mut rig = rig(background(), SMALL, 100);
    let work = rig.work();
    rig.scheduler.report_path_trace_time(&work, 2.0, true);
    rig.scheduler.report_adaptive_filter_time(&work, 1.0, true);
    let time = rig.scheduler.task_time();
    assert_eq!((time.path_trace.wall(), time.path_trace.average()), (2.0, 0.0));
    assert_eq!((time.adaptive_filter.wall(), time.adaptive_filter.average()), (1.0, 0.0));
}

#[test]
fn every_kind_of_task_is_timed() {
    let mut rig = rig(background(), SMALL, 100);
    let work = rig.work();
    rig.scheduler.report_adaptive_filter_time(&work, 1.0, false);
    rig.scheduler.report_denoise_time(&work, 2.0);
    rig.scheduler.report_display_update_time(&work, 3.0);
    rig.scheduler.report_rebalance_time(&work, 4.0, false);
    rig.scheduler.report_volume_guiding_denoise_time(&work, 5.0);
    let time = rig.scheduler.task_time();
    let average = [time.adaptive_filter, time.denoise, time.display_update, time.rebalance, time.volume_guiding_denoise].map(|task| task.average());
    assert_eq!(average, [1.0, 2.0, 3.0, 4.0, 5.0]);
}

#[test]
fn the_occupancy_is_kept_with_the_samples_it_was_measured_on() {
    let mut rig = rig(background(), SMALL, 100);
    let work = rig.work();
    rig.scheduler.report_path_trace_occupancy(&work, 0.25);
    assert_eq!((rig.scheduler.state().occupancy_sample_count, rig.scheduler.state().occupancy), (1, 0.25));
}

#[test]
fn a_display_update_report_moves_the_last_update_to_now() {
    let mut rig = rig(background(), SMALL, 100);
    let work = rig.work();
    rig.clock.advance(3.0);
    rig.scheduler.report_display_update_time(&work, 0.1);
    assert_eq!(rig.scheduler.state().last_display_update_time, 103.0);
}

#[test]
fn no_time_limit_is_reached_before_the_render_starts() {
    let mut rig = rig(background(), SMALL, 100);
    rig.scheduler.set_time_limit(5.0);
    rig.clock.advance(60.0);
    rig.scheduler.check_time_limit_reached();
    assert!(!rig.scheduler.state().time_limit_reached);
}
