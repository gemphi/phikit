// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::integrator::scheduler::test::*;

#[test]
fn the_divider_for_a_time_is_the_ratio_up_to_four_and_its_root_past_it() {
    let rig = rig(viewport(), SMALL, 100);
    let divider: Vec<i32> = [1.0, 2.5, 4.0, 16.0].into_iter().map(|actual| rig.scheduler.resolution_divider_for_time(1.0, actual)).collect();
    // 2.5 times too slow: 3 samples, the root of 7.5, rounded up.
    assert_eq!(divider, [1, 3, 4, 8]);
}

#[test]
fn the_first_divider_keeps_the_long_side_of_the_view_at_128_pixels() {
    let start = |size| rig(viewport(), size, 100).scheduler.state().resolution_divider;
    assert_eq!(start([4096, 2048]), 8);
    assert_eq!(start([512, 256]), 4);
    assert_eq!(start([256, 512]), 4);
    assert_eq!(start([200, 100]), 1);
}

#[test]
fn a_slow_first_render_raises_the_divider_and_it_then_steps_down_through_four() {
    let mut rig = rig(viewport(), [4096, 2048], 100);
    assert_eq!(rig.work().resolution_divider, 8.0);
    let full = rig.work();
    assert_eq!(full.resolution_divider, 1.0);
    // 10 seconds a sample against the 30 updates a second asked for.
    rig.scheduler.report_path_trace_time(&full, 10.0, false);
    rig.scheduler.reset_for_next_tile();
    let divider: Vec<f32> = (0..3).map(|_| rig.work().resolution_divider).collect();
    assert_eq!(divider, [30.0, 4.0, 1.0]);
}

#[test]
fn a_fast_first_render_needs_no_divider() {
    let mut rig = rig(viewport(), [4096, 2048], 100);
    rig.work();
    let full = rig.work();
    rig.scheduler.report_path_trace_time(&full, 0.001, false);
    rig.scheduler.reset_for_next_tile();
    assert_eq!(rig.work().resolution_divider, 1.0);
}

#[test]
fn denoising_and_display_time_count_toward_the_divider() {
    let mut rig = rig(viewport(), [4096, 2048], 100);
    rig.work();
    let full = rig.work();
    rig.scheduler.report_path_trace_time(&full, 0.001, false);
    rig.scheduler.report_denoise_time(&full, 5.0);
    rig.scheduler.report_display_update_time(&full, 5.0);
    rig.scheduler.reset_for_next_tile();
    assert_eq!(rig.work().resolution_divider, 30.0);
}
