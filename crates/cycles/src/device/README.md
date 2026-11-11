# device

Ported from `intern/cycles/device/`. A device is memory plus a queue of
kernels.

| File | Cycles | Holds |
|---|---|---|
| `info.rs` | `device.h` | `DeviceType`, `DeviceInfo` |
| `device.rs` | `device.h` | the `Device` trait, `DeviceError` |
| `memory.rs` | `memory.h` | `MemoryType`, `DevicePtr`, `DeviceVector<T>` (`device_vector<T>`) |
| `argument.rs` | `kernel.h` | `KernelArgument` (`DeviceKernelArguments`) |
| `queue.rs` | `queue.h` | the `DeviceQueue` trait |
| `cpu/` | `cpu/` | `CpuDevice`, `CpuKernels` (`CPUKernels`), `CpuMemory` |
| `denoise.rs` | `denoise.h` | `DenoiseParam` |

## The contract

`Device` is `Send + Sync` and every method takes `&self`. Memory is bytes
behind a `DevicePtr` (a handle, never an address): `mem_alloc` (zeroed),
`mem_zero`, `mem_copy_to`, `mem_copy_from`, `mem_free`. `queue()` makes a
`DeviceQueue`: `enqueue(kernel, work_size, &[KernelArgument])`, then
`synchronize()`, which returns the first error. `DeviceVector<T>` is the typed
host side, as `device_vector<T>` is: it keeps a host copy and a `DevicePtr`.

## One kernel, two devices

The same call runs on `CpuDevice` and on the `WgpuDevice` phiray's gpu layer
will provide, because nothing in the contract names either:

- A kernel is `DeviceKernel` (in `kernel/`) plus a work size plus arguments.
  An argument is a `DevicePtr`, an `i32`, an `f32` or a `bool`: what
  Cycles' `DeviceKernelArguments` carries, and what a WGSL uniform and a bind
  group can carry.
- The CPU body of a kernel is a Rust function registered in `CpuKernels`.
  The wgpu body is a WGSL entry point of the same name in `wgsl/`, found by
  `DeviceKernel`.
- Data layouts are the ones in `kernel/` (`KernelData`, `RecordLayout`,
  generated from Cycles' headers), so a block means the same bytes to both.

To compare a kernel: allocate the inputs with `DeviceVector`, copy them to
each device, enqueue the kernel with the same arguments on each queue,
`synchronize`, `copy_from_device`, and compare the host copies (equal for
integer data, within a tolerance for floats). `tests/device/queue.rs` does
this on the CPU with a stand-in kernel; the wgpu run joins it when
`WgpuDevice` exists, as one more `Box<dyn Device>` in the same test.

## Not ported kernels

A kernel with no body returns `DeviceError::NotPorted(kernel)` from
`enqueue`. A render that reaches one stops with that error
(`PathTraceWorkCpu::error`), and never panics.

## Not ported from Cycles

Multi devices, the denoiser list in `DeviceInfo`, texture and image memory
(`MEM_IMAGE_TEXTURE` is named, not used), peer access, and the graphics
interop. The CPU has no queue in Cycles; `CpuQueue` runs a kernel as it is
enqueued, so `PathTraceWorkCpu` (`integrator/trace/cpu.rs`) uses the same
calls as a GPU work will.
