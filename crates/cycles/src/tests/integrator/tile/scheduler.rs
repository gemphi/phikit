// SPDX-License-Identifier: Apache-2.0
use std::collections::HashSet;

use crate::integrator::*;
use crate::kernel::KernelWorkTile;
use crate::session::BufferParam;

/// A scheduler over `buffer` for `sample_count` samples from sample 10.
fn scheduler(buffer: &BufferParam, sample_count: i32, max_path_state: i32) -> WorkTileScheduler {
    let mut scheduler = WorkTileScheduler::new();
    scheduler.set_max_path_state(max_path_state);
    scheduler.reset(buffer, 10, sample_count, 3, 1.0);
    scheduler
}

fn all(scheduler: &mut WorkTileScheduler) -> Vec<KernelWorkTile> {
    std::iter::from_fn(|| scheduler.work(0)).collect()
}

#[test]
fn with_one_path_state_every_pixel_and_sample_is_a_tile_of_its_own() {
    let mut scheduler = scheduler(&BufferParam::full(4, 2), 3, 1);
    let tile = all(&mut scheduler);
    assert_eq!(tile.len(), 4 * 2 * 3);
    // The samples of a pixel come first, then the next pixel.
    let first: Vec<(u32, u32, u32)> = tile[..4].iter().map(|tile| (tile.x, tile.y, tile.start_sample)).collect();
    assert_eq!(first, [(0, 0, 10), (0, 0, 11), (0, 0, 12), (1, 0, 10)]);
    let once: HashSet<(u32, u32, u32)> = tile.iter().map(|tile| (tile.x, tile.y, tile.start_sample)).collect();
    assert_eq!(once.len(), tile.len());
    assert!(tile.iter().all(|tile| (tile.w, tile.h, tile.sample_count, tile.sample_offset) == (1, 1, 1, 3)));
}

#[test]
fn an_image_that_fits_the_path_states_is_one_tile() {
    let buffer = BufferParam::full(8, 4);
    let mut scheduler = scheduler(&buffer, 2, 64);
    let tile = all(&mut scheduler);
    assert_eq!(tile.len(), 1);
    assert_eq!((tile[0].w, tile[0].h, tile[0].start_sample, tile[0].sample_count), (8, 4, 10, 2));
    assert_eq!((tile[0].offset, tile[0].stride), (buffer.offset, 8));
    assert_eq!(scheduler.work(0), None);
}

#[test]
fn tiles_at_the_right_and_bottom_edge_are_cut_to_the_image() {
    let mut scheduler = scheduler(&BufferParam::full(5, 3), 1, 4);
    assert_eq!(scheduler.tile_size(), TileSize::new(2, 2, 1));
    let size: Vec<(u32, u32, u32, u32)> = all(&mut scheduler).iter().map(|tile| (tile.x, tile.y, tile.w, tile.h)).collect();
    assert_eq!(size, [(0, 0, 2, 2), (2, 0, 2, 2), (4, 0, 1, 2), (0, 2, 2, 1), (2, 2, 2, 1), (4, 2, 1, 1)]);
}

#[test]
fn a_slice_of_the_frame_hands_out_tiles_in_the_frame_s_pixels() {
    let mut buffer = BufferParam::full(4, 2);
    (buffer.full_x, buffer.full_y) = (5, 7);
    buffer.update_offset_stride();
    let mut scheduler = scheduler(&buffer, 1, 8);
    let tile = scheduler.work(0).expect("one tile");
    assert_eq!((tile.x, tile.y, tile.w, tile.h), (5, 7, 4, 2));
    assert_eq!(tile.offset, -(5 + 7 * 4));
}

#[test]
fn the_last_tile_of_a_pixel_takes_the_samples_that_are_left() {
    let mut scheduler = scheduler(&BufferParam::full(1, 1), 7, 4);
    let sample: Vec<(u32, u32)> = all(&mut scheduler).iter().map(|tile| (tile.start_sample, tile.sample_count)).collect();
    assert_eq!(sample, [(10, 2), (12, 2), (14, 2), (16, 1)]);
}

#[test]
fn a_tile_over_the_work_size_asked_for_waits_for_a_later_call() {
    let mut scheduler = scheduler(&BufferParam::full(8, 4), 2, 64);
    assert_eq!(scheduler.work(32), None);
    assert!(scheduler.work(64).is_some());
    assert_eq!(scheduler.work(64), None);
}

#[test]
fn no_sample_is_no_tile() {
    let mut scheduler = scheduler(&BufferParam::full(8, 4), 0, 64);
    assert_eq!(scheduler.work(0), None);
}
