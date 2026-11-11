// SPDX-License-Identifier: Apache-2.0
//! The render buffer: `intern/cycles/session/buffers.h` and `buffers.cpp`.
//! `param` is its size and place, `pass` what a pixel holds, `render` the
//! floats themselves.

mod param;
mod pass;
mod render;

pub use param::BufferParam;
pub use pass::{BufferPass, PASS_UNUSED};
pub use render::RenderBuffer;
