// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of `kernel/device/cpu.rs`: the kernels in the CPU table, run one
//! after another on the queue of a device as a wavefront would run them,
//! give what the megakernel gives.
use std::mem::size_of;

use crate::kernel::test::rig::{Rig, cube, translation};
use crate::device::{CpuDevice, Device, DeviceError, DevicePtr, KernelArgument, MemoryType};
use crate::kernel::device::cpu_kernels;
use crate::kernel::integrator::IntegratorState;
use crate::kernel::{DeviceKernel, KernelGlobal};
use crate::scene::Surface;
use std::sync::Arc;

fn device(kg: &Arc<KernelGlobal>) -> CpuDevice {
    CpuDevice::with_kernels(cpu_kernels(kg.clone()))
}

fn state_of(device: &CpuDevice, ptr: DevicePtr) -> IntegratorState {
    let mut bytes = vec![0u8; size_of::<IntegratorState>()];
    device.mem_copy_from(ptr, 0, &mut bytes).unwrap();
    bytemuck::pod_read_unaligned(&bytes)
}

/// The pixel at `(x, y)` of an 8 by 8 image, `samples` samples, by the
/// kernels one at a time: each runs where `queued_kernel` says.
fn by_kernel(rig: &mut Rig, x: i32, y: i32, samples: i32) -> [f32; 4] {
    let kg = rig.kernel();
    let device = device(&kg);
    let state = device.mem_alloc(size_of::<IntegratorState>(), MemoryType::ReadWrite).unwrap();
    let buffer = device.mem_alloc(4 * 4 * 64, MemoryType::ReadWrite).unwrap();
    let mut queue = device.queue();
    let pixel = (x + y * 8) as usize;
    for sample in 0..samples {
        let init = [KernelArgument::Ptr(state), KernelArgument::Ptr(buffer), KernelArgument::I32(x), KernelArgument::I32(y), KernelArgument::I32(sample), KernelArgument::I32(0), KernelArgument::I32(0), KernelArgument::I32(8)];
        queue.enqueue(DeviceKernel::IntegratorInitFromCamera, 1, &init).unwrap();
        // The pixel of the state: `offset + x + y * stride`, with offset 0.
        loop {
            let queued = state_of(&device, state).path.queued_kernel;
            let Some(kernel) = DeviceKernel::INTEGRATOR.iter().copied().find(|kernel| *kernel as u32 == queued && queued != 0) else { break };
            let args = match kernel {
                DeviceKernel::IntegratorIntersectClosest => vec![KernelArgument::Ptr(state)],
                _ => vec![KernelArgument::Ptr(state), KernelArgument::Ptr(buffer)],
            };
            queue.enqueue(kernel, 1, &args).unwrap();
        }
    }
    queue.synchronize().unwrap();
    let mut bytes = [0u8; 16];
    device.mem_copy_from(buffer, pixel * 16, &mut bytes).unwrap();
    bytemuck::pod_read_unaligned(&bytes)
}

/// The same pixel by the megakernel over the whole image.
fn by_megakernel(rig: &mut Rig, x: usize, y: usize, samples: i32) -> [f32; 4] {
    let image = rig.render(samples);
    let pixel = image[x + y * 8];
    [pixel[0] * samples as f32, pixel[1] * samples as f32, pixel[2] * samples as f32, 0.0]
}

#[test]
fn the_kernels_one_at_a_time_give_the_megakernel_s_pixel() {
    let build = || {
        let mut rig = Rig::new(8, 8, [0.5, 0.25, 0.75]);
        rig.add(&cube(0.8), Surface::Diffuse { color: [0.9, 0.6, 0.3] }, translation(0.0, 0.0, 0.0));
        rig
    };
    for (x, y) in [(4, 4), (3, 5), (0, 0), (7, 2)] {
        let one = by_kernel(&mut build(), x, y, 3);
        let all = by_megakernel(&mut build(), x as usize, y as usize, 3);
        for c in 0..3 {
            assert!((one[c] - all[c]).abs() < 1e-4, "pixel {x},{y} channel {c}: {one:?} against {all:?}");
        }
    }
}

#[test]
fn every_kernel_of_gate_one_is_registered_and_the_others_are_not() {
    let kg = Rig::new(8, 8, [0.1; 3]).kernel();
    let device = device(&kg);
    let ported = [DeviceKernel::IntegratorInitFromCamera, DeviceKernel::IntegratorIntersectClosest, DeviceKernel::IntegratorShadeBackground, DeviceKernel::IntegratorShadeSurface, DeviceKernel::IntegratorMegakernel];
    for kernel in ported {
        assert!(device.kernels().is_ported(kernel), "{kernel:?}");
    }
    for kernel in DeviceKernel::INTEGRATOR.iter().filter(|kernel| !ported.contains(kernel)) {
        assert!(!device.kernels().is_ported(*kernel), "{kernel:?}");
        assert!(matches!(device.queue().enqueue(*kernel, 1, &[]), Err(DeviceError::NotPorted(_))));
    }
}

#[test]
fn a_kernel_given_the_wrong_arguments_says_so() {
    let kg = Rig::new(8, 8, [0.1; 3]).kernel();
    let device = device(&kg);
    let mut queue = device.queue();
    let result = queue.enqueue(DeviceKernel::IntegratorMegakernel, 1, &[KernelArgument::I32(0)]);
    assert!(matches!(result, Err(DeviceError::BadArgument { .. })), "{result:?}");
    // And a block that is not there.
    let result = DeviceKernel::IntegratorIntersectClosest;
    let mut fresh = device.queue();
    assert!(matches!(fresh.enqueue(result, 1, &[KernelArgument::Ptr(DevicePtr(999))]), Err(DeviceError::BadPointer(_))));
}
