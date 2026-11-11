// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/tile.h` and `tile.cpp`
//! (`TileSize`, `tile_calculate_best_size`). This is the size of a work
//! tile on a device, not a tile of the frame (`session::Tile`).

/// `TileSize`: the pixels and samples of one unit of device work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TileSize {
    pub width: i32,
    pub height: i32,
    /// `num_samples`.
    pub sample_count: i32,
}

impl TileSize {
    pub fn new(width: i32, height: i32, sample_count: i32) -> TileSize {
        TileSize { width, height, sample_count }
    }

    /// `tile_calculate_best_size`: the work tile that fits `max_path_state`
    /// path states. Small tiles with many samples keep paths coherent, and
    /// powers of two divide the states evenly.
    pub fn best(accel_rt: bool, image_size: [i32; 2], sample_count: i32, max_path_state: i32, scrambling_distance: f32) -> TileSize {
        let pixel_sample = i64::from(image_size[0]) * i64::from(image_size[1]) * i64::from(sample_count);
        // One state is the CPU's case: no arithmetic, so no rounding.
        if max_path_state == 1 {
            return TileSize::new(1, 1, 1);
        }
        // The whole image and every sample fit at once.
        if i64::from(max_path_state) >= pixel_sample {
            return TileSize::new(image_size[0], image_size[1], sample_count);
        }
        let state_per_sample = max_path_state / sample_count;
        let (width, height) = match (scrambling_distance < 0.9 && accel_rt, state_per_sample) {
            // Large tiles suit a small scrambling distance.
            (true, _) => {
                let width = image_size[0].min(max_path_state);
                (width, image_size[1].min((max_path_state / width).max(1)))
            }
            (false, 0) => (1, 1),
            (false, state) => {
                let side = round_down_to_power_of_two(root(state)) as i32;
                (side, side)
            }
        };
        let sample = match sample_count {
            1 => 1,
            // Prefer an even division of the samples: 32 thirty-eight times
            // and then 8, over 1024 and then 200.
            _ => {
                let even = round_up_to_power_of_two(root(sample_count / 2)).min(sample_count as u32) as i32;
                even.min(max_path_state / (width * height))
            }
        };
        TileSize::new(width, height, sample)
    }
}

/// `lround(sqrt(x))`.
fn root(value: i32) -> u32 {
    f64::from(value).sqrt().round() as u32
}

/// `round_down_to_power_of_two`.
fn round_down_to_power_of_two(value: u32) -> u32 {
    match value {
        0 => 0,
        _ => 1 << (31 - value.leading_zeros()),
    }
}

/// `round_up_to_power_of_two`.
fn round_up_to_power_of_two(value: u32) -> u32 {
    match value {
        0 => 0,
        _ => value.next_power_of_two(),
    }
}
