// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/path_trace.cpp`
//! (`foreach_sliced_buffer_params`, `scale_buffer_params`, lines 272-352):
//! the big tile cut into one horizontal slice for each device, and a
//! buffer made smaller by a resolution divider.

use crate::integrator::WorkBalanceInfo;
use crate::session::BufferParam;

/// `scale_buffer_params`: `param` at `resolution_divider`. Sizes stay at
/// least one pixel.
pub fn scale_buffer_param(param: &BufferParam, resolution_divider: f32) -> BufferParam {
    let scale = |value: i32| (value as f32 / resolution_divider) as i32;
    let mut scaled = BufferParam {
        width: scale(param.width).max(1),
        height: scale(param.height).max(1),
        window_x: scale(param.window_x),
        window_y: scale(param.window_y),
        window_width: scale(param.window_width).max(1),
        window_height: scale(param.window_height).max(1),
        full_x: scale(param.full_x),
        full_y: scale(param.full_y),
        full_width: scale(param.full_width).max(1),
        full_height: scale(param.full_height).max(1),
        ..param.clone()
    };
    scaled.update_offset_stride();
    scaled
}

/// `foreach_sliced_buffer_params`: `param` cut top to bottom, a slice for
/// each device as tall as its weight. A slice keeps `overscan` rows of its
/// neighbours above and below, inside the tile, and the last slice takes
/// the rows that are left.
pub fn slice_buffer_param(balance: &[WorkBalanceInfo], param: &BufferParam, overscan: i32) -> Vec<BufferParam> {
    let window_height = param.window_height;
    let mut current_y = 0;
    let cut = |(index, info): (usize, &WorkBalanceInfo)| {
        let slice_window_full_y = param.full_y + param.window_y + current_y;
        let slice_window_height = ((f64::from(window_height) * info.weight).round() as i32).max(1);
        // Never negative, for more devices than rows.
        let remaining_window_height = (window_height - current_y).max(0);
        let mut slice = param.clone();
        slice.full_y = (slice_window_full_y - overscan).max(param.full_y);
        slice.window_y = slice_window_full_y - slice.full_y;
        slice.window_height = match index + 1 < balance.len() {
            true => slice_window_height.min(remaining_window_height),
            false => remaining_window_height,
        };
        slice.height = (slice.window_y + slice.window_height + overscan).min(param.height + param.full_y - slice.full_y);
        slice.update_offset_stride();
        current_y += slice.window_height;
        slice
    };
    balance.iter().enumerate().map(cut).collect()
}

#[cfg(test)]
mod test;
