// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/device/cpu/kernel_arch_impl.h` (the
//! `kernel_cpu_*` functions) and `device/cpu/kernel.cpp` (`CPUKernels`
//! filling its table), with the loop over pixels and samples of
//! `integrator/path_trace_work_cpu.cpp` (`render_samples`, 69-130, and
//! `render_samples_full_pipeline`, 132-180).
//!
//! The functions registered in [`CpuKernel`] are:
//!
//! | Kernel | Arguments after the kernels' state |
//! |---|---|
//! | `IntegratorInitFromCamera` | the render buffer, `x`, `y`, `start_sample`, `sample_offset`, `offset`, `stride` |
//! | `IntegratorIntersectClosest` | none |
//! | `IntegratorShadeBackground`, `IntegratorShadeSurface` | the render buffer |
//! | `IntegratorMegakernel` | the render buffer and the work tile: `x y w h start_sample sample_count sample_offset offset stride` (no state: each pixel keeps its own) |
//!
//! The first four act on one path whose `IntegratorState` is the block
//! given first; its pixel is the path's `render_pixel_index` in the render
//! buffer. The megakernel runs every pixel of the tile for each of its
//! samples, over as many threads as the device has.
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::device::{CpuKernelFn, CpuKernel, CpuMemory, DeviceError, KernelArgument};
use crate::kernel::KernelGlobal;
use crate::kernel::integrator::*;
use crate::kernel::{DeviceKernel, KernelWorkTile};

fn int(args: &[KernelArgument], index: usize, kernel: DeviceKernel) -> Result<i32, DeviceError> {
    args.get(index).and_then(KernelArgument::as_i32).ok_or(DeviceError::BadArgument { kernel, index })
}

fn ptr(args: &[KernelArgument], index: usize, kernel: DeviceKernel) -> Result<crate::device::DevicePtr, DeviceError> {
    args.get(index).and_then(KernelArgument::as_ptr).ok_or(DeviceError::BadArgument { kernel, index })
}

/// The state block as an `IntegratorState` and the render buffer as floats.
fn state_and_buffer<R>(memory: &mut CpuMemory, state: crate::device::DevicePtr, buffer: crate::device::DevicePtr, body: impl FnOnce(&mut IntegratorState, &mut [f32]) -> R) -> Result<R, DeviceError> {
    let (state_bytes, buffer_bytes) = memory.bytes_pair_mut(state, buffer)?;
    let state: &mut IntegratorState = bytemuck::try_from_bytes_mut(&mut state_bytes[..size_of::<IntegratorState>()]).map_err(|_| DeviceError::BadPointer(buffer))?;
    let floats = buffer_bytes.len() / 4 * 4;
    let buffer: &mut [f32] = bytemuck::cast_slice_mut(&mut buffer_bytes[..floats]);
    Ok(body(state, buffer))
}

/// The floats of the pixel a path writes to.
fn pixel<'a>(kg: &KernelGlobal, state: &IntegratorState, buffer: &'a mut [f32]) -> &'a mut [f32] {
    let stride = kg.film.pass_stride as usize;
    let start = state.path.render_pixel_index as usize * stride;
    &mut buffer[start..start + stride]
}

/// The work tile of the megakernel's arguments.
fn tile_of(args: &[KernelArgument]) -> Result<KernelWorkTile, DeviceError> {
    let kernel = DeviceKernel::IntegratorMegakernel;
    let at = |index: usize| int(args, index, kernel);
    Ok(KernelWorkTile {
        x: at(1)? as u32,
        y: at(2)? as u32,
        w: at(3)? as u32,
        h: at(4)? as u32,
        start_sample: at(5)? as u32,
        sample_count: at(6)? as u32,
        sample_offset: at(7)? as u32,
        offset: at(8)?,
        stride: at(9)? as u32,
        ..KernelWorkTile::default()
    })
}

/// `render_samples_full_pipeline` for the pixel at `index` of the tile:
/// each sample is a path started from the camera and run until none is
/// queued. `buffer` is the pixel's floats.
fn render_pixel(kg: &KernelGlobal, tile: &KernelWorkTile, index: u32, buffer: &mut [f32]) {
    let (x, y) = (tile.x + index % tile.w, tile.y + index / tile.w);
    let mut state = IntegratorState::new();
    for sample in 0..tile.sample_count {
        let scheduled = (tile.start_sample + sample) as i32;
        if !integrator_init_from_camera(kg, &mut state, tile, buffer, x as i32, y as i32, scheduled) {
            break;
        }
        integrator_megakernel(kg, &mut state, buffer);
    }
}

/// The megakernel over the whole tile, a row of pixels at a time on each
/// thread.
fn megakernel_tile(kg: &KernelGlobal, memory: &mut CpuMemory, work_size: i32, args: &[KernelArgument]) -> Result<(), DeviceError> {
    let kernel = DeviceKernel::IntegratorMegakernel;
    let buffer_ptr = ptr(args, 0, kernel)?;
    let tile = tile_of(args)?;
    let stride = kg.film.pass_stride as usize;
    let pixels = (work_size.max(0) as usize).min(tile.w as usize * tile.h as usize);
    let floats = memory.slice_mut::<f32>(buffer_ptr)?;
    let rows = std::cmp::min(pixels, floats.len() / stride.max(1));
    // Rows of the tile, handed out one by one.
    let width = tile.w.max(1) as usize;
    let row_queue: Mutex<Vec<(usize, &mut [f32])>> = Mutex::new(floats[..rows * stride].chunks_mut(width * stride).enumerate().collect());
    let next = AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                loop {
                    let claim = next.fetch_add(1, Ordering::Relaxed);
                    let item = row_queue.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).get_mut(claim).map(|(row, chunk)| (*row, std::mem::take(chunk)));
                    let Some((row, chunk)) = item else { break };
                    for (column, pixel_buffer) in chunk.chunks_mut(stride).enumerate() {
                        render_pixel(kg, &tile, (row * width + column) as u32, pixel_buffer);
                    }
                }
            });
        }
    });
    Ok(())
}

/// Fill `kernels` with the kernels of gate 1, over the scene `kg`.
pub fn register_cpu_kernels(kernels: &mut CpuKernel, kg: Arc<KernelGlobal>) {
    let globals = kg.clone();
    let init: CpuKernelFn = Arc::new(move |memory, _work_size, args| {
        let kernel = DeviceKernel::IntegratorInitFromCamera;
        let (state, buffer) = (ptr(args, 0, kernel)?, ptr(args, 1, kernel)?);
        let tile = KernelWorkTile { start_sample: int(args, 4, kernel)? as u32, sample_offset: int(args, 5, kernel)? as u32, offset: int(args, 6, kernel)?, stride: int(args, 7, kernel)? as u32, ..KernelWorkTile::default() };
        let (x, y, sample) = (int(args, 2, kernel)?, int(args, 3, kernel)?, int(args, 4, kernel)?);
        state_and_buffer(memory, state, buffer, |state, buffer| {
            let pixel_buffer = pixel(&globals, state, buffer);
            integrator_init_from_camera(&globals, state, &tile, pixel_buffer, x, y, sample);
        })
    });
    kernels.set(DeviceKernel::IntegratorInitFromCamera, init);
    let globals = kg.clone();
    kernels.set(
        DeviceKernel::IntegratorIntersectClosest,
        Arc::new(move |memory, _work_size, args| {
            let state = ptr(args, 0, DeviceKernel::IntegratorIntersectClosest)?;
            let bytes = memory.bytes_mut(state)?;
            let state: &mut IntegratorState = bytemuck::try_from_bytes_mut(&mut bytes[..size_of::<IntegratorState>()]).map_err(|_| DeviceError::BadPointer(state))?;
            integrator_intersect_closest(&globals, state);
            Ok(())
        }),
    );
    for (kernel, body) in [(DeviceKernel::IntegratorShadeBackground, integrator_shade_background as fn(&KernelGlobal, &mut IntegratorState, &mut [f32])), (DeviceKernel::IntegratorShadeSurface, integrator_shade_surface)] {
        let globals = kg.clone();
        kernels.set(
            kernel,
            Arc::new(move |memory, _work_size, args| {
                let (state, buffer) = (ptr(args, 0, kernel)?, ptr(args, 1, kernel)?);
                state_and_buffer(memory, state, buffer, |state, buffer| {
                    let pixel_buffer = pixel(&globals, state, buffer);
                    body(&globals, state, pixel_buffer);
                })
            }),
        );
    }
    let globals = kg;
    kernels.set(DeviceKernel::IntegratorMegakernel, Arc::new(move |memory, work_size, args| megakernel_tile(&globals, memory, work_size, args)));
}

/// A table of the CPU kernels, those of gate 1 ported.
pub fn cpu_kernels(kg: Arc<KernelGlobal>) -> CpuKernel {
    let mut kernels = CpuKernel::new();
    register_cpu_kernels(&mut kernels, kg);
    kernels
}
