// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::session::*;

#[test]
fn a_buffer_placed_in_a_larger_image_knows_where_the_image_starts() {
    let mut param = BufferParam { width: 100, height: 50, full_x: 20, full_y: 3, ..BufferParam::default() };
    param.update_offset_stride();
    assert_eq!((param.offset, param.stride), (-(20 + 3 * 100), 100));
}

#[test]
fn a_full_buffer_is_its_own_window() {
    let param = BufferParam::full(640, 480);
    assert_eq!((param.window_width, param.window_height, param.offset, param.stride), (640, 480, 0, 640));
}
