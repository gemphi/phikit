// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/work_tile_scheduler.h` and
//! `work_tile_scheduler.cpp` (`WorkTileScheduler`): cuts one device's slice
//! of the frame, and the samples asked for, into the work tiles its path
//! states are filled from.

use super::TileSize;
use crate::kernel::KernelWorkTile;
use crate::session::BufferParam;
use crate::util::divide_up;

/// `WorkTileScheduler`.
#[derive(Debug, Clone, Default)]
pub struct WorkTileScheduler {
    accelerated_rt: bool,
    /// `max_num_path_states_`: the unit work is scheduled in.
    max_path_state: i32,
    /// `image_full_offset_px_`: where the slice sits in the full image.
    image_full_offset: [i32; 2],
    /// `image_size_px_`.
    image_size: [i32; 2],
    scrambling_distance: f32,
    offset: i32,
    stride: i32,
    sample_start: i32,
    /// `samples_num_`.
    sample_count: i32,
    sample_offset: i32,
    tile_size: TileSize,
    tile_count_x: i32,
    tile_count_y: i32,
    /// `num_tiles_per_sample_range_`: tiles over one pixel area, each a
    /// part of the samples.
    tile_per_sample_range: i32,
    next_work_index: i32,
    total_work_size: i32,
}

impl WorkTileScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    /// The device traces rays in hardware.
    pub fn set_accelerated_rt(&mut self, accelerated_rt: bool) {
        self.accelerated_rt = accelerated_rt;
    }

    /// `set_max_num_path_states`.
    pub fn set_max_path_state(&mut self, max_path_state: i32) {
        self.max_path_state = max_path_state;
    }

    /// `reset`: schedule `sample_count` samples from `sample_start` over
    /// `buffer`.
    pub fn reset(&mut self, buffer: &BufferParam, sample_start: i32, sample_count: i32, sample_offset: i32, scrambling_distance: f32) {
        self.image_full_offset = [buffer.full_x, buffer.full_y];
        self.image_size = [buffer.width, buffer.height];
        self.scrambling_distance = scrambling_distance;
        self.offset = buffer.offset;
        self.stride = buffer.stride;
        self.sample_start = sample_start;
        self.sample_count = sample_count;
        self.sample_offset = sample_offset;
        self.reset_scheduler_state();
    }

    /// `reset_scheduler_state`.
    fn reset_scheduler_state(&mut self) {
        self.tile_size = TileSize::best(self.accelerated_rt, self.image_size, self.sample_count, self.max_path_state, self.scrambling_distance);
        let tile = self.tile_size;
        (self.tile_count_x, self.tile_count_y, self.tile_per_sample_range) = match tile.width * tile.height * tile.sample_count {
            // Nothing is left for the device.
            0 => (0, 0, 0),
            _ => (divide_up(self.image_size[0], tile.width), divide_up(self.image_size[1], tile.height), divide_up(self.sample_count, tile.sample_count)),
        };
        self.next_work_index = 0;
        self.total_work_size = self.tile_count_x * self.tile_count_y * self.tile_per_sample_range;
    }

    /// The size of the tiles handed out.
    pub fn tile_size(&self) -> TileSize {
        self.tile_size
    }

    /// `get_work`: the next work tile; `None` when all were handed out. A
    /// tile over `max_work_size` path states (zero is no limit) is not
    /// handed out now and stays for a later call. The limit may be past
    /// the path states set: a work can take several small tiles at once.
    pub fn work(&mut self, max_work_size: i32) -> Option<KernelWorkTile> {
        let work_index = self.next_work_index;
        if work_index >= self.total_work_size {
            return None;
        }
        let tile = self.tile_size;
        let sample_range_index = work_index % self.tile_per_sample_range;
        let start_sample = sample_range_index * tile.sample_count;
        let tile_index = work_index / self.tile_per_sample_range;
        let tile_y = tile_index / self.tile_count_x;
        let tile_x = tile_index - tile_y * self.tile_count_x;
        let (x, y) = (tile_x * tile.width, tile_y * tile.height);
        let w = tile.width.min(self.image_size[0] - x);
        let h = tile.height.min(self.image_size[1] - y);
        let sample_count = tile.sample_count.min(self.sample_count - start_sample);
        if max_work_size != 0 && w * h * sample_count > max_work_size {
            return None;
        }
        self.next_work_index += 1;
        Some(KernelWorkTile {
            x: (x + self.image_full_offset[0]) as u32,
            y: (y + self.image_full_offset[1]) as u32,
            w: w as u32,
            h: h as u32,
            start_sample: (self.sample_start + start_sample) as u32,
            sample_count: sample_count as u32,
            sample_offset: self.sample_offset as u32,
            offset: self.offset,
            stride: self.stride as u32,
            ..KernelWorkTile::default()
        })
    }
}

#[cfg(test)]
mod test;
