// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/session/buffers.h` and `buffers.cpp`
//! (`BufferParams`). Its passes and their offsets are in `pass.rs`.

use super::pass::{BufferPass, PASS_UNUSED};
use crate::scene::PassType;

/// `BufferParams`: the size of a render buffer and how it sits in the full
/// image. The part outside the window is overscan.
#[derive(Debug, Clone, PartialEq)]
pub struct BufferParam {
    /// The physical buffer.
    pub width: i32,
    pub height: i32,
    /// The visible part, relative to the buffer's place in the full buffer.
    pub window_x: i32,
    pub window_y: i32,
    pub window_width: i32,
    pub window_height: i32,
    /// Offset into, and size of, the full buffer.
    pub full_x: i32,
    pub full_y: i32,
    pub full_width: i32,
    pub full_height: i32,
    /// Set by [`Self::update_offset_stride`]; -1 before.
    pub offset: i32,
    pub stride: i32,
    /// Floats a pixel. Set when the passes are known; -1 before.
    pub pass_stride: i32,
    /// `passes`: what a pixel holds, in buffer order.
    pub pass: Vec<BufferPass>,
    pub layer: String,
    pub view: String,
    /// `pass_offset_`: for each pass type and mode, the lowest offset of
    /// such a pass in a pixel, or [`PASS_UNUSED`].
    pub(crate) pass_offset: Vec<i32>,
    /// `samples`.
    pub sample_count: i32,
    pub exposure: f32,
    pub use_approximate_shadow_catcher: bool,
    pub use_transparent_background: bool,
}

impl Default for BufferParam {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            window_x: 0,
            window_y: 0,
            window_width: 0,
            window_height: 0,
            full_x: 0,
            full_y: 0,
            full_width: 0,
            full_height: 0,
            offset: -1,
            stride: -1,
            pass_stride: -1,
            pass: Vec::new(),
            layer: String::new(),
            view: String::new(),
            pass_offset: vec![PASS_UNUSED; PassType::count() * 2],
            sample_count: 0,
            exposure: 1.0,
            use_approximate_shadow_catcher: false,
            use_transparent_background: false,
        }
    }
}

impl BufferParam {
    /// A buffer that is the whole of a `width` by `height` image.
    pub fn full(width: i32, height: i32) -> Self {
        let mut param = Self {
            width,
            height,
            window_width: width,
            window_height: height,
            full_width: width,
            full_height: height,
            ..Self::default()
        };
        param.update_offset_stride();
        param
    }

    /// `update_offset_stride`: where pixel (0, 0) of the full image falls in
    /// this buffer, and how many pixels a row is.
    pub fn update_offset_stride(&mut self) {
        self.offset = -(self.full_x + self.full_y * self.width);
        self.stride = self.width;
    }
}
