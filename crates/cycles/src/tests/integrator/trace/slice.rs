// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::integrator::*;
use crate::session::BufferParam;

fn weighted(weight: &[f64]) -> Vec<WorkBalanceInfo> {
    weight.iter().map(|weight| WorkBalanceInfo { weight: *weight, ..WorkBalanceInfo::default() }).collect()
}

/// Top row in the frame, first row of its own window, rows of its window,
/// rows held.
fn row(slice: &[BufferParam]) -> Vec<(i32, i32, i32, i32)> {
    slice.iter().map(|slice| (slice.full_y, slice.window_y, slice.window_height, slice.height)).collect()
}

#[test]
fn a_buffer_at_a_divider_is_that_many_times_smaller() {
    let scaled = scale_buffer_param(&BufferParam::full(1024, 512), 8.0);
    assert_eq!((scaled.width, scaled.height, scaled.window_width, scaled.window_height), (128, 64, 128, 64));
    assert_eq!((scaled.full_width, scaled.full_height, scaled.stride), (128, 64, 128));
    let odd = scale_buffer_param(&BufferParam::full(100, 50), 3.0);
    assert_eq!((odd.width, odd.height), (33, 16));
}

#[test]
fn a_scaled_buffer_keeps_a_pixel_and_its_place_in_the_frame() {
    let tiny = scale_buffer_param(&BufferParam::full(4, 4), 8.0);
    assert_eq!((tiny.width, tiny.height, tiny.window_width, tiny.full_height), (1, 1, 1, 1));
    let mut tile = BufferParam::full(64, 64);
    (tile.full_x, tile.full_y, tile.full_width) = (64, 128, 256);
    let scaled = scale_buffer_param(&tile, 2.0);
    assert_eq!((scaled.full_x, scaled.full_y, scaled.full_width), (32, 64, 128));
    assert_eq!(scaled.offset, -(32 + 64 * 32));
}

#[test]
fn one_device_takes_the_whole_tile() {
    let tile = BufferParam::full(64, 64);
    assert_eq!(slice_buffer_param(&weighted(&[1.0]), &tile, 0), [tile]);
}

#[test]
fn two_equal_devices_take_half_the_rows_each() {
    let slice = slice_buffer_param(&weighted(&[0.5, 0.5]), &BufferParam::full(64, 64), 0);
    assert_eq!(row(&slice), [(0, 0, 32, 32), (32, 0, 32, 32)]);
    assert_eq!(slice[1].offset, -(32 * 64));
}

#[test]
fn a_faster_device_takes_more_rows_and_the_last_takes_the_rest() {
    let slice = slice_buffer_param(&weighted(&[9.0 / 14.0, 5.0 / 14.0]), &BufferParam::full(64, 64), 0);
    assert_eq!(row(&slice), [(0, 0, 41, 41), (41, 0, 23, 23)]);
}

#[test]
fn overscan_keeps_rows_of_the_neighbour_inside_the_tile() {
    let slice = slice_buffer_param(&weighted(&[0.5, 0.5]), &BufferParam::full(64, 64), 4);
    assert_eq!(row(&slice), [(0, 0, 32, 36), (28, 4, 32, 36)]);
}

#[test]
fn more_devices_than_rows_leave_the_last_devices_none() {
    let slice = slice_buffer_param(&weighted(&[0.25; 4]), &BufferParam::full(8, 2), 0);
    let height: Vec<i32> = slice.iter().map(|slice| slice.window_height).collect();
    assert_eq!(height, [1, 1, 0, 0]);
}

#[test]
fn the_slices_of_a_tile_inside_a_frame_start_at_the_tile() {
    let mut tile = BufferParam::full(64, 64);
    (tile.full_y, tile.full_height) = (128, 256);
    tile.update_offset_stride();
    let slice = slice_buffer_param(&weighted(&[0.5, 0.5]), &tile, 0);
    assert_eq!(row(&slice), [(128, 0, 32, 32), (160, 0, 32, 32)]);
}
