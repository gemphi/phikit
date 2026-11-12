// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::session::*;

fn manager(width: i32, height: i32, tile: i32, adaptive: bool) -> TileManager {
    let buffer = BufferParam::full(width, height);
    let mut manager = TileManager::new();
    manager.reset_scheduling(&buffer, [tile, tile]);
    manager.update(&buffer, adaptive, false);
    manager
}

#[test]
fn a_tile_size_past_one_file_tile_is_a_multiple_of_it() {
    assert_eq!(TileManager::compute_render_tile_size(100), 100);
    assert_eq!(TileManager::compute_render_tile_size(128), 128);
    assert_eq!(TileManager::compute_render_tile_size(200), 256);
    assert_eq!(TileManager::compute_render_tile_size(2048), 2048);
    assert_eq!(TileManager::compute_render_tile_size(100_000), TileManager::MAX_TILE_SIZE);
}

#[test]
fn an_image_one_tile_covers_is_not_cut() {
    let param = SessionParam::default();
    assert_eq!(TileManager::effective_tile_size(&param, 1920, 1080), [1920, 1080]);
    assert_eq!(TileManager::effective_tile_size(&param, 5000, 3000), [2048, 2048]);
    let whole = SessionParam { use_auto_tile: false, ..SessionParam::default() };
    assert_eq!(TileManager::effective_tile_size(&whole, 5000, 3000), [5000, 3000]);
}

#[test]
fn a_frame_is_cut_in_rows_and_the_last_tile_of_each_is_what_is_left() {
    let mut manager = manager(5000, 3000, 2048, false);
    assert_eq!(manager.tile_count(), 6);
    let mut seen = Vec::new();
    while manager.next() {
        seen.push(manager.current_tile());
    }
    assert!(manager.done() && !manager.next());
    assert_eq!(seen.len(), 6);
    assert_eq!((seen[0].x, seen[0].y, seen[0].width, seen[0].height), (0, 0, 2048, 2048));
    assert_eq!((seen[2].x, seen[2].width), (4096, 5000 - 4096), "the third of a row takes the rest");
    assert_eq!((seen[3].x, seen[3].y, seen[3].height), (0, 2048, 3000 - 2048), "the second row starts at the left");
    let pixel: i32 = seen.iter().map(|tile| tile.window_width * tile.window_height).sum();
    assert_eq!(pixel, 5000 * 3000, "the windows cover the frame once");
}

#[test]
fn adaptive_sampling_gives_tiles_four_pixels_of_overscan_inside_the_frame() {
    let mut manager = manager(5000, 3000, 2048, true);
    assert_eq!(manager.tile_overscan(), 4);
    manager.next();
    let first = manager.current_tile();
    assert_eq!((first.x, first.window_x, first.width), (0, 0, 2048 + 4), "no overscan past the frame's edge");
    manager.next();
    let second = manager.current_tile();
    assert_eq!((second.x, second.window_x, second.window_width, second.width), (2048 - 4, 4, 2048, 2048 + 8));
}

#[test]
fn a_single_tile_has_no_overscan_even_with_adaptive_sampling() {
    let mut manager = manager(1920, 1080, 2048, true);
    assert_eq!((manager.tile_count(), manager.tile_overscan()), (1, 0));
    manager.next();
    let only = manager.current_tile();
    assert_eq!((only.width, only.height, only.window_width, only.window_height), (1920, 1080, 1920, 1080));
    assert_eq!(manager.size(), [1920, 1080]);
}
