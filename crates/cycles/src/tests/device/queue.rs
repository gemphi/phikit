// SPDX-License-Identifier: Apache-2.0
//! Tests of `device/queue.rs` and `device/cpu/kernel.rs`: a kernel runs
//! over a work size with its arguments, an unported one is a typed error.
use std::sync::Arc;

use crate::device::{CpuDevice, CpuKernel, Device, DeviceError, DeviceType, DeviceVector, KernelArgument, MemoryType};
use crate::kernel::DeviceKernel;

/// A stand-in kernel: `out[i] = in[i] * scale` for `i` in `0..work_size`.
fn scale_kernel() -> CpuDevice {
    let mut kernels = CpuKernel::new();
    kernels.set(
        DeviceKernel::IntegratorInitFromCamera,
        Arc::new(|memory, work_size, args| {
            let bad = |index| DeviceError::BadArgument { kernel: DeviceKernel::IntegratorInitFromCamera, index };
            let input = args.first().and_then(KernelArgument::as_ptr).ok_or(bad(0))?;
            let output = args.get(1).and_then(KernelArgument::as_ptr).ok_or(bad(1))?;
            let scale = args.get(2).and_then(KernelArgument::as_f32).ok_or(bad(2))?;
            let source: Vec<f32> = memory.slice::<f32>(input)?.to_vec();
            let target = memory.slice_mut::<f32>(output)?;
            for i in 0..work_size as usize {
                target[i] = source[i] * scale;
            }
            Ok(())
        }),
    );
    CpuDevice::with_kernels(kernels)
}

#[test]
fn a_cpu_device_describes_itself() {
    let info = CpuDevice::new().info().clone();
    assert_eq!(info.kind, DeviceType::Cpu);
    assert_eq!(info.id, "CPU");
    assert!(!info.has_gpu_queue);
    assert!(info.cpu_threads >= 1);
}

#[test]
fn an_enqueued_kernel_runs_over_its_work_size() {
    let device = scale_kernel();
    let mut input = DeviceVector::<f32>::new("in", MemoryType::ReadOnly);
    let mut output = DeviceVector::<f32>::new("out", MemoryType::ReadWrite);
    input.resize(4);
    input.host_mut().copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
    output.resize(4);
    input.copy_to_device(&device).unwrap();
    output.zero_to_device(&device).unwrap();
    let mut queue = device.queue();
    // Three of the four items only.
    let args = [KernelArgument::Ptr(input.ptr()), KernelArgument::Ptr(output.ptr()), KernelArgument::F32(10.0)];
    queue.enqueue(DeviceKernel::IntegratorInitFromCamera, 3, &args).unwrap();
    queue.synchronize().unwrap();
    output.copy_from_device(&device).unwrap();
    assert_eq!(output.host(), &[10.0, 20.0, 30.0, 0.0]);
}

#[test]
fn kernels_run_in_the_order_they_were_enqueued() {
    let device = scale_kernel();
    let mut buffer = DeviceVector::<f32>::new("b", MemoryType::ReadWrite);
    buffer.resize(2);
    buffer.host_mut().copy_from_slice(&[1.0, 2.0]);
    buffer.copy_to_device(&device).unwrap();
    let mut queue = device.queue();
    let args = [KernelArgument::Ptr(buffer.ptr()), KernelArgument::Ptr(buffer.ptr()), KernelArgument::F32(3.0)];
    queue.enqueue(DeviceKernel::IntegratorInitFromCamera, 2, &args).unwrap();
    queue.enqueue(DeviceKernel::IntegratorInitFromCamera, 2, &args).unwrap();
    queue.synchronize().unwrap();
    buffer.copy_from_device(&device).unwrap();
    assert_eq!(buffer.host(), &[9.0, 18.0]);
}

#[test]
fn an_unported_kernel_is_a_typed_error_never_a_panic() {
    let device = CpuDevice::new();
    let mut queue = device.queue();
    for kernel in DeviceKernel::INTEGRATOR.into_iter().chain([DeviceKernel::IntegratorMegakernel]) {
        assert!(!device.kernels().is_ported(kernel));
        assert_eq!(queue.enqueue(kernel, 1, &[]), Err(DeviceError::NotPorted(kernel)));
        assert_eq!(queue.synchronize(), Err(DeviceError::NotPorted(kernel)));
    }
}

#[test]
fn a_failed_queue_skips_what_follows_until_synchronized() {
    let device = scale_kernel();
    let mut queue = device.queue();
    // Wrong arguments for a ported kernel.
    let first = queue.enqueue(DeviceKernel::IntegratorInitFromCamera, 1, &[]);
    assert!(matches!(first, Err(DeviceError::BadArgument { index: 0, .. })));
    let second = queue.enqueue(DeviceKernel::IntegratorInitFromCamera, 1, &[]);
    assert_eq!(second, first);
    assert_eq!(queue.synchronize(), first);
    // The error was taken; the queue is usable again.
    assert_eq!(queue.synchronize(), Ok(()));
}

#[test]
fn two_queues_share_the_devices_memory() {
    let device = scale_kernel();
    let mut buffer = DeviceVector::<f32>::new("b", MemoryType::ReadWrite);
    buffer.resize(1);
    buffer.host_mut()[0] = 2.0;
    buffer.copy_to_device(&device).unwrap();
    let args = [KernelArgument::Ptr(buffer.ptr()), KernelArgument::Ptr(buffer.ptr()), KernelArgument::F32(2.0)];
    let (mut a, mut b) = (device.queue(), device.queue());
    a.enqueue(DeviceKernel::IntegratorInitFromCamera, 1, &args).unwrap();
    b.enqueue(DeviceKernel::IntegratorInitFromCamera, 1, &args).unwrap();
    buffer.copy_from_device(&device).unwrap();
    assert_eq!(buffer.host(), &[8.0]);
}

#[test]
fn a_device_is_used_through_the_trait_object() {
    let device: Box<dyn Device> = Box::new(scale_kernel());
    let mut buffer = DeviceVector::<f32>::new("b", MemoryType::ReadWrite);
    buffer.resize(1);
    buffer.host_mut()[0] = 1.5;
    buffer.copy_to_device(device.as_ref()).unwrap();
    let args = [KernelArgument::Ptr(buffer.ptr()), KernelArgument::Ptr(buffer.ptr()), KernelArgument::F32(2.0)];
    let mut queue = device.queue();
    queue.enqueue(DeviceKernel::IntegratorInitFromCamera, 1, &args).unwrap();
    queue.synchronize().unwrap();
    buffer.copy_from_device(device.as_ref()).unwrap();
    assert_eq!(buffer.host(), &[3.0]);
}
