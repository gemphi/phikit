// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/session/tile.h` and `tile.cpp` (`Tile`,
//! `TileManager`: the scheduling half), and `Session::get_effective_tile_size`
//! from `session/session.cpp:499-523`. Writing tiles to disk and reading
//! them back is not ported yet.

use crate::util::{align_up, divide_up};
use super::{BufferParam, SessionParam};

/// `Tile`: one piece of a large render. `x`, `y`, `width`, `height` are the
/// pixels rendered, overscan included; the window is the part that is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub window_x: i32,
    pub window_y: i32,
    pub window_width: i32,
    pub window_height: i32,
}

/// `TileManager`: cuts a frame into tiles and hands them out in rows.
#[derive(Debug, Clone, Default)]
pub struct TileManager {
    tile_size: [i32; 2],
    /// Extra pixels rendered around a tile.
    overscan: i32,
    buffer: BufferParam,
    tile_count_x: i32,
    tile_count_y: i32,
    tile_count: i32,
    next_tile_index: i32,
    current_tile: Tile,
}

impl TileManager {
    /// Tile size in the image file.
    pub const IMAGE_TILE_SIZE: i32 = 128;
    /// The largest tile: a display must be able to hold a texture this wide.
    pub const MAX_TILE_SIZE: i32 = 8192;

    pub fn new() -> Self {
        Self::default()
    }

    /// `compute_render_tile_size`: a tile size the image file can take. Past
    /// one file tile it is a multiple of [`Self::IMAGE_TILE_SIZE`].
    pub fn compute_render_tile_size(suggested: i32) -> i32 {
        let computed = match suggested <= Self::IMAGE_TILE_SIZE {
            true => suggested,
            false => align_up(suggested, Self::IMAGE_TILE_SIZE),
        };
        computed.min(Self::MAX_TILE_SIZE)
    }

    /// `Session::get_effective_tile_size`: the whole image when tiling is
    /// off or one tile would cover it, else a square tile.
    pub fn effective_tile_size(param: &SessionParam, width: i32, height: i32) -> [i32; 2] {
        let tile = Self::compute_render_tile_size(param.tile_size);
        let covers = i64::from(tile) * i64::from(tile) >= i64::from(width) * i64::from(height);
        let fits = width <= Self::MAX_TILE_SIZE && height <= Self::MAX_TILE_SIZE;
        match !param.use_auto_tile || (covers && fits) {
            true => [width, height],
            false => [tile, tile],
        }
    }

    /// `reset_scheduling`: start over on a frame of `param`'s size.
    pub fn reset_scheduling(&mut self, param: &BufferParam, tile_size: [i32; 2]) {
        self.tile_size = tile_size;
        self.tile_count_x = match tile_size[0] {
            0 => 0,
            side => divide_up(param.width, side),
        };
        self.tile_count_y = match tile_size[1] {
            0 => 0,
            side => divide_up(param.height, side),
        };
        self.tile_count = self.tile_count_x * self.tile_count_y;
        self.next_tile_index = 0;
        self.current_tile = Tile::default();
    }

    /// `update`: take the buffer the tiles are cut from. Tiles of a
    /// many-tile render get 4 pixels of overscan when adaptive sampling is
    /// on and nothing is being baked, since its filter reads neighbours.
    pub fn update(&mut self, param: &BufferParam, adaptive_sampling: bool, baking: bool) {
        self.buffer = param.clone();
        self.overscan = match self.has_multiple_tile() && adaptive_sampling && !baking {
            true => 4,
            false => 0,
        };
    }

    /// `get_num_tiles`.
    pub fn tile_count(&self) -> i32 {
        self.tile_count
    }

    /// `has_multiple_tiles`.
    pub fn has_multiple_tile(&self) -> bool {
        self.tile_count > 1
    }

    /// `get_tile_overscan`.
    pub fn tile_overscan(&self) -> i32 {
        self.overscan
    }

    /// `done`: every tile has been handed out.
    pub fn done(&self) -> bool {
        self.next_tile_index == self.tile_count
    }

    /// `next`: move to the next tile. `false` when there is none.
    pub fn next(&mut self) -> bool {
        match self.done() {
            true => false,
            false => {
                self.current_tile = self.tile_for_index(self.next_tile_index);
                self.next_tile_index += 1;
                true
            }
        }
    }

    /// `get_current_tile`.
    pub fn current_tile(&self) -> Tile {
        self.current_tile
    }

    /// `get_size`: the frame's size.
    pub fn size(&self) -> [i32; 2] {
        [self.buffer.width, self.buffer.height]
    }

    /// `get_tile_for_index`: tiles run left to right, then down.
    fn tile_for_index(&self, index: i32) -> Tile {
        let index_y = index / self.tile_count_x;
        let index_x = index - index_y * self.tile_count_x;
        let window_left = index_x * self.tile_size[0];
        let window_top = index_y * self.tile_size[1];

        let x = (window_left - self.overscan).max(0);
        let y = (window_top - self.overscan).max(0);
        let window_x = window_left - x;
        let window_y = window_top - y;
        let window_width = self.tile_size[0].min(self.buffer.width - window_left);
        let window_height = self.tile_size[1].min(self.buffer.height - window_top);
        Tile {
            x,
            y,
            width: (self.buffer.width - x).min(window_x + window_width + self.overscan),
            height: (self.buffer.height - y).min(window_y + window_height + self.overscan),
            window_x,
            window_y,
            window_width,
            window_height,
        }
    }
}

#[cfg(test)]
mod test;
