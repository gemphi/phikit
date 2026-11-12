// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::super::scheduler::viewport;
use super::*;

fn size(param: &[BufferParam]) -> Vec<(i32, i32)> {
    param.iter().map(|param| (param.width, param.height)).collect()
}

#[test]
fn a_reset_keeps_the_frame_and_the_tile_and_asks_for_new_buffers() {
    let bench = single(rig(background(), SMALL, 2), 0.5);
    assert_eq!((bench.trace.full_param().width, bench.trace.big_tile_param().height), (64, 64));
    assert!(bench.trace.state().need_reset_param);
}

#[test]
fn the_first_round_sizes_the_work_s_buffer_and_later_rounds_leave_it() {
    let mut bench = single(rig(background(), SMALL, 2), 0.5);
    bench.round();
    bench.round();
    let device = &bench.trace.work()[0];
    assert_eq!(size(&device.allocated), [(64, 64)]);
    assert_eq!(size(&device.effective), [(64, 64)]);
    assert!(!bench.trace.state().need_reset_param);
}

#[test]
fn a_small_view_renders_into_a_small_part_of_the_same_buffer() {
    let mut bench = single(rig(viewport(), [1024, 512], 100), 0.5);
    bench.round();
    bench.round();
    let device = &bench.trace.work()[0];
    assert_eq!(size(&device.allocated), [(1024, 512)]);
    assert_eq!(size(&device.effective), [(128, 64), (1024, 512)]);
    assert_eq!(bench.trace.state().resolution_divider, 1.0);
    assert_eq!(bench.trace.state().effective_big_tile_param.width, 1024);
}

#[test]
fn two_devices_get_a_slice_each() {
    let mut bench = pair(rig(background(), SMALL, 2), [0.5, 0.5]);
    bench.round();
    let slice: Vec<(i32, i32)> = bench.trace.work().iter().map(|device| (device.allocated[0].full_y, device.allocated[0].height)).collect();
    assert_eq!(slice, [(0, 32), (32, 32)]);
}

#[test]
fn a_rebalance_slices_the_tile_again_by_the_new_weights() {
    let mut rig = rig(background(), SMALL, 100);
    rig.scheduler.set_need_schedule_rebalance(true);
    let mut bench = pair(rig, [1.0, 3.0]);
    bench.round();
    bench.round();
    let height: Vec<Vec<i32>> = bench.trace.work().iter().map(|device| device.allocated.iter().map(|param| param.height).collect()).collect();
    assert_eq!(height, [[32, 41], [32, 23]]);
}

#[test]
fn the_same_tile_again_needs_no_new_buffers_and_a_new_tile_does() {
    let mut bench = single(rig(background(), SMALL, 1), 0.5);
    bench.round();
    assert!(bench.trace.state().tile_written);
    let tile = bench.trace.big_tile_param().clone();
    bench.trace.reset(&bench.rig.buffer, &tile, &mut bench.host, false);
    assert!(!bench.trace.state().need_reset_param && !bench.trace.state().tile_written);
    assert_eq!(bench.take().last().map(String::as_str), Some("reset false"));
    let mut other = tile.clone();
    other.full_x = 64;
    bench.trace.reset(&bench.rig.buffer, &other, &mut bench.host, false);
    assert!(bench.trace.state().need_reset_param);
    assert_eq!(bench.trace.big_tile_param().full_x, 64);
}
