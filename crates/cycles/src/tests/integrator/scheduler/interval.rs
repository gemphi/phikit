// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::device::DenoiseParam;

#[test]
fn a_headless_render_updates_every_half_minute() {
    let rig = rig(headless(), SMALL, 1000);
    assert_eq!(rig.scheduler.display_update_interval_no_limit(0), 30.0);
    assert_eq!(rig.scheduler.display_update_interval_no_limit(500), 30.0);
}

#[test]
fn a_background_render_updates_every_second_and_later_every_two() {
    let rig = rig(background(), SMALL, 1000);
    assert_eq!(rig.scheduler.display_update_interval_no_limit(31), 1.0);
    assert_eq!(rig.scheduler.display_update_interval_no_limit(32), 2.0);
}

#[test]
fn a_viewport_updates_often_at_first_and_less_as_time_passes() {
    let mut rig = rig(viewport(), SMALL, 1000);
    let work = rig.work();
    // A cancelled report adds to the wall time alone: 0.5, 1.5, 3, 5, 9.
    let interval: Vec<f64> = [0.5, 1.0, 1.5, 2.0, 4.0]
        .into_iter()
        .map(|second| {
            rig.scheduler.report_path_trace_time(&work, second, true);
            rig.scheduler.display_update_interval_no_limit(100)
        })
        .collect();
    assert_eq!(interval, [0.1, 0.25, 0.5, 1.0, 2.0]);
    // Past 8 seconds a render of few samples still updates every second.
    assert_eq!(rig.scheduler.display_update_interval_no_limit(31), 1.0);
}

#[test]
fn a_rebalance_about_to_happen_shortens_the_interval() {
    let mut rig = rig(headless(), SMALL, 1000);
    rig.scheduler.set_need_schedule_rebalance(true);
    let first = rig.work();
    assert_eq!(rig.scheduler.display_update_interval(), 0.1);
    let second = rig.work();
    assert!(!first.rebalance && second.rebalance);
    rig.scheduler.report_rebalance_time(&second, 0.1, true);
    assert_eq!(rig.scheduler.display_update_interval(), 0.2);
}

#[test]
fn the_interval_is_never_past_what_is_left_of_the_time_limit() {
    let mut rig = rig(background(), SMALL, 1000);
    rig.scheduler.set_time_limit(10.0);
    assert_eq!(rig.scheduler.display_update_interval_for(0), 1.0);
    let first = rig.work();
    rig.scheduler.report_work_begin(&first);
    rig.clock.advance(9.5);
    assert_eq!(rig.scheduler.display_update_interval_for(0), 0.5);
    rig.clock.advance(5.0);
    assert_eq!(rig.scheduler.display_update_interval_for(0), 0.0);
}

#[test]
fn navigation_updates_thirty_times_a_second_and_twelve_when_denoised() {
    let mut rig = rig(viewport(), SMALL, 1000);
    assert_eq!(rig.scheduler.viewport_navigation_update_interval(), 1.0 / 30.0);
    rig.scheduler.set_denoise_param(DenoiseParam { enabled: true, start_sample: 1, ..DenoiseParam::default() });
    assert!(rig.scheduler.is_denoise_active_during_update());
    assert_eq!(rig.scheduler.viewport_navigation_update_interval(), 1.0 / 12.0);
    rig.scheduler.set_denoise_param(DenoiseParam { enabled: true, start_sample: 2, ..DenoiseParam::default() });
    assert!(!rig.scheduler.is_denoise_active_during_update());
}
