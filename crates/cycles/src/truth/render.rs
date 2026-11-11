// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/session/session.cpp` (`Session::
//! run_main_render_loop`, the part a background render takes): the harness's
//! renderer. It runs the session's rounds (`Session::next_work`, then
//! `PathTrace::render`) over a `PathTraceWorkCpu` on a `CpuDevice`, and turns
//! the combined pass into a picture. Until the kernels are ported the first
//! round ends with the device's `NotPorted`, which is returned as
//! [`TruthError::Device`] and not as a black picture.
use std::sync::Arc;

use super::{Image, Renderer, TestScene, TruthError};
use crate::device::CpuDevice;
use crate::integrator::{PathTrace, PathTraceWorkCpu, RenderScheduler, RenderWork, TraceHost};
use crate::session::{BufferParam, RenderBuffer, Session, SessionParam};
use crate::util::SystemClock;

/// The host of a background render: no display, no denoiser, no tile file.
struct Offline;

impl TraceHost for Offline {
    fn cancelled(&self) -> bool {
        false
    }
    fn reset_display(&mut self, _big_tile_param: &BufferParam, _reset_rendering: bool) {}
    fn read_tile_buffer(&mut self) {}
    fn denoise(&mut self, _sample_count: i32) -> Option<bool> {
        None
    }
    fn write_tile(&mut self, _to_disk: bool) {}
    fn update_display(&mut self, _show_denoised: bool, _sample_count: i32) -> bool {
        false
    }
    fn update_progress(&mut self, _work: &RenderWork) {}
    fn finish_tile_file(&mut self) {}
}

/// Renders a scene through the session at the size of its reference.
#[derive(Debug)]
pub struct SessionRenderer {
    pub device: Arc<CpuDevice>,
    pub sample_count: i32,
    /// The size to render at: the reference's, read by the caller.
    pub size: (usize, usize),
    /// Film > Transparent: the picture has an alpha. Blender's opaque
    /// renders are written as RGB, and so are the references.
    pub transparent: bool,
}

/// The floats a pixel has in the buffer so far: the combined pass, RGBA.
/// The pass layout comes from the film when the scene is joined.
const PASS_STRIDE: i32 = 4;

impl SessionRenderer {
    pub fn new(device: Arc<CpuDevice>, size: (usize, usize), sample_count: i32) -> Self {
        Self { device, sample_count, size, transparent: false }
    }
    /// Run the rounds; the buffer of the whole frame.
    pub fn render_buffer(&mut self) -> Result<RenderBuffer, TruthError> {
        let (width, height) = (self.size.0 as i32, self.size.1 as i32);
        let mut frame = BufferParam::full(width, height);
        frame.pass_stride = PASS_STRIDE;
        let param = SessionParam { background: true, sample_count: self.sample_count, tile_size: width.max(height).max(2048), ..SessionParam::default() };
        let mut session = Session::new(param);
        session.reset(&frame);
        if session.tile_manager.has_multiple_tile() {
            return Err(TruthError::Unsupported("a frame of more than one tile".into()));
        }
        let mut scheduler = RenderScheduler::new(SystemClock, &session.param);
        let mut trace = PathTrace::new(vec![PathTraceWorkCpu::new(self.device.clone())]);
        let mut host = Offline;
        loop {
            let work = session.next_work(&mut scheduler, &mut trace, &mut host);
            trace.render(&work, &mut scheduler, &session.tile_manager, &mut host);
            if let Some(error) = trace.work().iter().find_map(PathTraceWorkCpu::error) {
                return Err(TruthError::Device(error.clone()));
            }
            if !work.any() {
                break;
            }
        }
        let mut buffer = RenderBuffer::default();
        buffer.reset(&frame);
        trace.copy_to_render_buffer(&mut buffer);
        Ok(buffer)
    }
}

impl Renderer for SessionRenderer {
    fn render(&mut self, _scene: &TestScene) -> Result<Image, TruthError> {
        let buffer = self.render_buffer()?;
        Ok(display(&buffer, self.sample_count, self.transparent))
    }
}

/// The combined pass as the picture a "Standard" view transform gives:
/// the sum over `sample_count`, then the sRGB curve on the colour. (The
/// Filmic and AgX views, and the look, come with the colour management of
/// gate 2.) Alpha is the sum over `sample_count`, clamped.
pub fn display(buffer: &RenderBuffer, sample_count: i32, transparent: bool) -> Image {
    let (width, height) = (buffer.param.width as usize, buffer.param.height as usize);
    let scale = 1.0 / sample_count.max(1) as f32;
    let channels = if transparent { 4 } else { 3 };
    let mut image = Image::new(width, height, channels);
    for (out, source) in image.pixel.chunks_exact_mut(channels).zip(buffer.buffer.chunks_exact(buffer.param.pass_stride.max(1) as usize)) {
        for c in 0..3 {
            out[c] = srgb_encode(source[c] * scale);
        }
        if transparent {
            out[3] = (source[3] * scale).clamp(0.0, 1.0);
        }
    }
    image
}

/// The sRGB transfer function of a linear value, clamped to 0..1.
pub fn srgb_encode(linear: f32) -> f32 {
    let value = linear.clamp(0.0, 1.0);
    if value >= 1.0 {
        return 1.0;
    }
    if value <= 0.003_130_8 { value * 12.92 } else { 1.055 * value.powf(1.0 / 2.4) - 0.055 }
}
