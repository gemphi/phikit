// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of `integrator/trace/cpu.rs`: the CPU work over the CPU device.
use std::sync::Arc;

use crate::device::{CpuDevice, CpuKernel, DeviceError, KernelArgument};
use crate::integrator::*;
use crate::kernel::DeviceKernel;
use crate::session::{BufferParam, RenderBuffer};

/// A frame of `width` by `height` with four floats a pixel.
fn frame(width: i32, height: i32) -> BufferParam {
    let mut param = BufferParam::full(width, height);
    param.pass_stride = 4;
    param
}

fn work(device: CpuDevice, param: &BufferParam) -> PathTraceWorkCpu {
    let mut work = PathTraceWorkCpu::new(Arc::new(device));
    work.init_execution();
    work.reset_render_buffer(param);
    work.set_effective_buffer_param(param, param, param, param);
    work
}

#[test]
fn the_unported_megakernel_stops_the_work_with_a_typed_error() {
    let param = frame(16, 8);
    let mut work = work(CpuDevice::new(), &param);
    assert!(!work.has_error());
    work.render_samples(0, 4, 0);
    assert!(work.has_error());
    assert_eq!(work.error(), Some(&DeviceError::NotPorted(DeviceKernel::IntegratorMegakernel)));
    assert_eq!(work.enqueue_count(), 1);
    // A work with an error renders nothing more until it is started again.
    work.render_samples(4, 4, 0);
    assert_eq!(work.enqueue_count(), 0);
    work.init_execution();
    assert!(!work.has_error());
}

/// A megakernel stand-in that adds `sample_count` to the red of each pixel
/// of the tile: it shows the tile reached the kernel intact.
fn counting_device() -> CpuDevice {
    let mut kernels = CpuKernel::new();
    kernels.set(
        DeviceKernel::IntegratorMegakernel,
        Arc::new(|memory, work_size, args| {
            let bad = |index| DeviceError::BadArgument { kernel: DeviceKernel::IntegratorMegakernel, index };
            let buffer = args.first().and_then(KernelArgument::as_ptr).ok_or(bad(0))?;
            let tile: Vec<i32> = (1..10).map(|i| args.get(i).and_then(KernelArgument::as_i32).ok_or(bad(i))).collect::<Result<_, _>>()?;
            let (x, y, w, h, samples, offset, stride) = (tile[0], tile[1], tile[2], tile[3], tile[5], tile[7], tile[8]);
            assert_eq!(work_size, w * h, "one item for each pixel of the slice");
            let floats = memory.slice_mut::<f32>(buffer)?;
            for row in y..y + h {
                for column in x..x + w {
                    let pixel = (offset + row * stride + column) as usize * 4;
                    floats[pixel] += samples as f32;
                }
            }
            Ok(())
        }),
    );
    CpuDevice::with_kernels(kernels)
}

#[test]
fn a_ported_kernel_sees_every_pixel_of_every_tile_once() {
    let param = frame(37, 19);
    let mut work = work(counting_device(), &param);
    work.render_samples(0, 3, 0);
    assert!(!work.has_error(), "{:?}", work.error());
    assert_eq!(work.enqueue_count(), 1);
    let mut out = RenderBuffer::default();
    out.reset(&param);
    work.copy_to_render_buffer(&mut out);
    for y in 0..19 {
        for x in 0..37 {
            assert_eq!(out.pixel(x, y)[0], 3.0, "pixel {x},{y}");
            assert_eq!(out.pixel(x, y)[1], 0.0);
        }
    }
}

#[test]
fn zero_and_copy_from_move_the_buffer_to_the_device() {
    let param = frame(4, 4);
    let mut work = work(counting_device(), &param);
    let mut source = RenderBuffer::default();
    source.reset(&param);
    source.pixel_mut(1, 1)[2] = 7.0;
    work.copy_from_render_buffer(&source);
    let mut out = RenderBuffer::default();
    out.reset(&param);
    work.copy_to_render_buffer(&mut out);
    assert_eq!(out.pixel(1, 1)[2], 7.0);
    work.zero_render_buffer();
    work.copy_to_render_buffer(&mut out);
    assert_eq!(out.pixel(1, 1)[2], 0.0);
    assert!(!work.has_error());
}

#[test]
fn the_cpu_work_drives_a_path_trace() {
    // The work stands in for the host loop's: one work on one device.
    let path_trace = PathTrace::new(vec![PathTraceWorkCpu::new(Arc::new(CpuDevice::new()))]);
    assert_eq!(path_trace.work().len(), 1);
    assert!(!path_trace.need_schedule_rebalance());
}
