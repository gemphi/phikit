// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/path_trace_work_cpu.h` and
//! `path_trace_work_cpu.cpp` (`PathTraceWorkCPU`, lines 31-180): path
//! tracing on the CPU device, into this work's slice of the frame.
//!
//! Cycles' `render_samples` hands each pixel of the slice to a thread as a
//! work tile of one pixel (`render_samples_full_pipeline`, lines 69-180).
//! Here the pixels are one `IntegratorMegakernel` enqueued on the device's
//! queue, over the slice's pixels; the kernel finds its slice and its
//! samples in the arguments (the order is [`TILE_ARGUMENT`]), and for each
//! sample runs `integrator_init_from_camera` and the megakernel. The kernel
//! is not ported (gate 1), so a render stops there with the device's
//! [`DeviceError::NotPorted`], which `has_error` then reports.
use std::sync::Arc;

use super::{PathTraceWork, RenderStatistic};
use crate::device::{CpuDevice, Device, DeviceError, DeviceVector, KernelArgument, MemoryType};
use crate::kernel::{DeviceKernel, KernelWorkTile};
use crate::session::{BufferParam, RenderBuffer};

/// The arguments the megakernel is enqueued with, after the pointer to the
/// render buffer: `x`, `y`, `w`, `h`, `start_sample`, `sample_count`,
/// `sample_offset`, `offset`, `stride` of the [`KernelWorkTile`].
pub const TILE_ARGUMENT: [&str; 9] = ["x", "y", "w", "h", "start_sample", "sample_count", "sample_offset", "offset", "stride"];

/// `PathTraceWorkCPU`.
#[derive(Debug)]
pub struct PathTraceWorkCpu {
    device: Arc<CpuDevice>,
    /// `buffers_`: the work's slice of the frame, on the device.
    buffer: DeviceVector<f32>,
    /// What the buffer holds: the slice at full size.
    buffer_param: BufferParam,
    /// `effective_buffer_params_`: what the next round renders into.
    effective_slice: BufferParam,
    error: Option<DeviceError>,
    /// Megakernel enqueues of the last round.
    enqueue_count: usize,
}

impl PathTraceWorkCpu {
    pub fn new(device: Arc<CpuDevice>) -> Self {
        Self {
            device,
            buffer: DeviceVector::new("render_buffer", MemoryType::ReadWrite),
            buffer_param: BufferParam::default(),
            effective_slice: BufferParam::default(),
            error: None,
            enqueue_count: 0,
        }
    }
    /// The device error that stopped the last round, if one did.
    pub fn error(&self) -> Option<&DeviceError> {
        self.error.as_ref()
    }
    /// Megakernel enqueues of the last round: one, unless it failed first.
    pub fn enqueue_count(&self) -> usize {
        self.enqueue_count
    }
    /// The floats of the work's buffer on the host: after
    /// `copy_from_device`.
    pub fn buffer(&self) -> &[f32] {
        self.buffer.host()
    }
    fn keep(&mut self, result: Result<(), DeviceError>) {
        if let Err(error) = result {
            self.error.get_or_insert(error);
        }
    }
    /// The arguments of the megakernel for `tile`.
    fn argument(&self, tile: &KernelWorkTile) -> [KernelArgument; 10] {
        let int = |value: u32| KernelArgument::I32(value as i32);
        [
            KernelArgument::Ptr(self.buffer.ptr()),
            int(tile.x),
            int(tile.y),
            int(tile.w),
            int(tile.h),
            int(tile.start_sample),
            int(tile.sample_count),
            int(tile.sample_offset),
            KernelArgument::I32(tile.offset),
            int(tile.stride),
        ]
    }
}

impl PathTraceWork for PathTraceWorkCpu {
    fn init_execution(&mut self) {
        self.error = None;
    }
    fn has_error(&self) -> bool {
        self.error.is_some()
    }
    fn reset_render_buffer(&mut self, param: &BufferParam) {
        let floats = param.width.max(0) as usize * param.height.max(0) as usize * param.pass_stride.max(0) as usize;
        self.buffer_param = param.clone();
        self.buffer.resize(floats);
        let result = self.buffer.zero_to_device(self.device.as_ref());
        self.keep(result);
    }
    fn set_effective_buffer_param(&mut self, _big_tile: &BufferParam, slice: &BufferParam, _denoised_big_tile: &BufferParam, _denoised_slice: &BufferParam) {
        self.effective_slice = slice.clone();
    }
    fn zero_render_buffer(&mut self) {
        let result = self.buffer.zero_to_device(self.device.as_ref());
        self.keep(result);
    }
    fn copy_to_render_buffer(&mut self, buffer: &mut RenderBuffer) {
        let result = self.buffer.copy_from_device(self.device.as_ref());
        self.keep(result);
        let at = slice_start(&self.buffer_param, &buffer.param);
        let count = self.buffer.len().min(buffer.buffer.len().saturating_sub(at));
        buffer.buffer[at..at + count].copy_from_slice(&self.buffer.host()[..count]);
    }
    fn copy_from_render_buffer(&mut self, buffer: &RenderBuffer) {
        let at = slice_start(&self.buffer_param, &buffer.param);
        let count = self.buffer.len().min(buffer.buffer.len().saturating_sub(at));
        self.buffer.host_mut()[..count].copy_from_slice(&buffer.buffer[at..at + count]);
        let result = self.buffer.copy_to_device(self.device.as_ref());
        self.keep(result);
    }
    fn render_samples(&mut self, start_sample: i32, sample_count: i32, sample_offset: i32) -> RenderStatistic {
        self.enqueue_count = 0;
        if self.error.is_some() {
            return RenderStatistic::default();
        }
        // `render_samples`, lines 65-110: every pixel of the effective
        // slice is a work tile of one pixel and one sample, repeated
        // `sample_count` times by `render_samples_full_pipeline`. The pixels
        // are one enqueue; the kernel walks them and, for each sample,
        // runs `integrator_init_from_camera` and the megakernel.
        let slice = &self.effective_slice;
        let tile = KernelWorkTile {
            x: slice.full_x as u32,
            y: slice.full_y as u32,
            w: slice.width.max(0) as u32,
            h: slice.height.max(0) as u32,
            start_sample: start_sample as u32,
            sample_count: sample_count as u32,
            sample_offset: sample_offset as u32,
            offset: slice.offset,
            stride: slice.stride as u32,
            ..KernelWorkTile::default()
        };
        let work_size = tile.w as i32 * tile.h as i32;
        let args = self.argument(&tile);
        let mut queue = self.device.queue();
        let result = queue.enqueue(DeviceKernel::IntegratorMegakernel, work_size, &args);
        self.enqueue_count = 1;
        self.keep(result);
        let result = queue.synchronize();
        self.keep(result);
        RenderStatistic::default()
    }    /// The adaptive sampling kernels are not ported: no pixel is open.
    fn adaptive_sampling_converge_filter_count_active(&mut self, _threshold: f32, _reset: bool) -> u32 {
        0
    }
    fn cryptomatte_postprocess(&mut self) {}
    fn denoise_volume_guiding_buffer(&mut self) {}
}

/// Where the work's slice starts in the big tile's floats: the rows above it.
fn slice_start(slice: &BufferParam, big_tile: &BufferParam) -> usize {
    let rows = (slice.full_y - big_tile.full_y).max(0) as usize;
    rows * big_tile.width.max(0) as usize * big_tile.pass_stride.max(0) as usize
}
