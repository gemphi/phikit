# cycles — architecture

Cycles in Rust. Nothing is implemented: this is the architecture another
session builds from. Every figure is from Blender's source on disk
(`gemphi/ext/phiber-refs/blender/blender/intern/cycles`, read 2026-11-10,
nothing built or run). The crate is `cycles`; Cycles with a capital is
Blender's.

Three rules shape it. **Blender is the specification and the judge**: a
part is done when Blender's own test scenes render within Blender's own
tolerance. **Nothing phiray or phiphx owns is written again.** **Rust
first, GPU second**: each kernel exists in Rust, where it is tested, before
it exists in WGSL.

## Cycles by size, and what becomes of each part

221,440 lines in 872 files.

```mermaid
graph TB
    subgraph port["port: about 58,000 lines of kernel and shader code, and parts of scene and integrator"]
        k_int["kernel/integrator 14,098<br/>the path state machine"]
        k_svm["kernel/svm 14,204 + scene/svm.cpp 1,281<br/>shader bytecode: compile and run"]
        k_clo["kernel/closure 7,162<br/>the BSDFs"]
        k_geom["kernel/geom 4,435 · kernel/light 4,296"]
        k_film["kernel/film 2,402 · sample 997 · camera 1,046"]
        s_scene["scene/ 41,443, in part<br/>scene objects to kernel arrays, shader_nodes 8,424"]
        h_int["integrator/ 11,464, in part<br/>the wavefront loop, adaptive, passes"]
    end
    subgraph reuse["already ours"]
        r_blend["blender/ 15,080<br/>phiphx reads the .blend itself"]
        r_dev["device/ 22,056 + kernel/device 8,691<br/>phiray gpu and Kernel, on wgpu"]
        r_bvh["bvh/ 6,477 + kernel/bvh 2,518<br/>phiray bvh and bvh.wgsl, to extend"]
        r_shade["the Principled lobes<br/>phiray light.wgsl, phiphx blend/shade"]
    end
    subgraph leave["not ported"]
        l_osl["OSL 16,349"]
        l_guide["path guiding: OpenPGL, CPU only"]
        l_den["OIDN and OptiX denoisers: outside libraries"]
        l_misc["hydra · app · subd 12,545"]
        l_bvh["Embree · OptiX · MetalRT · HIPRT trees"]
    end
```

## Why Metal's road is closed to WGSL

```mermaid
graph TB
    m1["Cycles writes each kernel once, in C++ headers"]
    m2["CUDA, OptiX, HIP, oneAPI: C++ compilers"]
    m3["Metal: the shading language is a C++ dialect, so the same<br/>headers compile as they are<br/>device/metal/device_impl.mm:441, kernel/device/metal/compat.h:48-135"]
    w1["WGSL has no templates, classes, pointers, includes or macros:<br/>the headers cannot be fed to it"]
    w2["so: each kernel is written in Rust, the reference,<br/>and again in WGSL, its twin, checked against the Rust one"]
    m1 --> m2
    m1 --> m3
    m1 --> w1 --> w2
```

## Rust first, GPU second

```mermaid
graph TB
    a["a .blend from Blender's tests"]
    b["phiphx: the scene"]
    c["cycles data:<br/>the kernel arrays"]
    d["Rust megakernel, on the CPU<br/>one path at a time, as Cycles' CPU path does<br/>kernel/integrator/megakernel.h:22-119"]
    e["compared with Blender's reference image,<br/>by Blender's rule"]
    f["WGSL twin of each kernel"]
    g["wavefront on wgpu:<br/>state in arrays, queues, sort by shader"]
    h["compared with the Rust megakernel,<br/>then with Blender's image"]
    a --> b --> c --> d --> e
    d --> f --> g --> h
```

The CPU path comes first because it can be judged against Blender before
any GPU code exists, and it then judges every WGSL kernel on small inputs.

## The modules, and what each stands on

```mermaid
graph LR
    subgraph cycles["cycles"]
        p_truth["truth<br/>Blender's scenes, references, tolerance"]
        p_data["data<br/>the kernel arrays from a scene"]
        p_svm["svm<br/>compile a node graph to words · run them"]
        p_closure["closure<br/>the BSDFs, BSSRDFs, phase functions"]
        p_path["path<br/>the state of one path, the 18 kernels"]
        p_light["light<br/>the light tree, sampling, MIS"]
        p_sample["sample<br/>Sobol-Burley, tabulated Sobol"]
        p_film["film<br/>passes, AOVs, adaptive sampling"]
        p_wave["wave<br/>the wavefront loop on the GPU"]
    end
    subgraph ours["ours already"]
        o_phx["phiphx<br/>SceneModel · blend/shade node graphs"]
        o_gpu["phiray gpu · kernel<br/>Gpu · GpuArray · Pass · Kernel"]
        o_bvh["phiray bvh<br/>SAH trees · radix sort"]
        o_adapt["phiray adapt.wgsl<br/>Cycles' convergence check, already ported"]
        o_ecs["phiecs<br/>a Readout role"]
    end
    p_data --> o_phx
    p_svm --> o_phx
    p_path --> p_svm
    p_path --> p_closure
    p_path --> p_light
    p_path --> p_sample
    p_path --> p_data
    p_path --> o_bvh
    p_film --> o_adapt
    p_wave --> p_path
    p_wave --> o_gpu
    p_wave --> o_bvh
    p_truth --> p_film
    p_wave --> o_ecs
```

## One sample batch on the GPU, as Cycles runs it

```mermaid
graph TB
    w1["enqueue camera paths when active paths run low<br/>integrator/path_trace_work_gpu.cpp:812-863"]
    w2["pick the kernel with the most queued paths<br/>:414"]
    w3{"a shading kernel"}
    w4["sort its paths by shader: count, prefix sum, write<br/>:627-672 · phiray bvh/radix.rs has the sort"]
    w5["list its paths: QUEUED_PATHS_ARRAY<br/>:546"]
    w6["run the kernel: each path names its next kernel<br/>kernel/integrator/state_template.h:33"]
    w7["drain shadow kernels first when their slots run short<br/>:483-501"]
    w8["compact the live paths<br/>:692"]
    w9["no path left: the batch is done"]
    w1 --> w2 --> w3
    w3 -->|yes| w4 --> w6
    w3 -->|no| w5 --> w6
    w6 --> w7 --> w8 --> w2
    w2 --> w9
```

Cycles runs a million paths or more at once on a GPU
(`device/cuda/queue.cpp:34`); on the CPU it runs one, through a switch on
the same `queued_kernel` field. The state is one definition that expands
both ways (`kernel/integrator/state.h:69, 140`), and that is the shape to
keep: one Rust definition, an array of structs on the CPU, a struct of
arrays on the GPU.

## Device and truth

`device/` is Cycles' `device/` as a contract: `Device` (memory by handle:
alloc, zero, copy to and from, free), `DeviceQueue` (enqueue a `DeviceKernel`
over a work size with `KernelArgument`s, synchronize) and `DeviceVector<T>`.
`CpuDevice` runs a kernel as a Rust function from a table, and a kernel with
no body returns `NotPorted`. phiray's gpu layer will implement the same trait
as a `WgpuDevice`, so one kernel is run on both and the buffers compared; the
recipe is in [src/device/README.md](src/device/README.md).

`truth/` is gate 0: a scene and its reference found by Blender's layout, a
render through the session, `oiiotool --diff` reproduced from
`render_report.py:125-205` (a pixel fails past T, a render fails when more than
P per cent of pixels do, T and P from `cycles_render_tests.py:337-357`), and a
verdict with its diff images. `blender/` is the join: phiphx's `SceneModel`,
through one file (`blender/model.rs`), into a `Scene`.
## Pages

The render as the Rust types run it: [FLOW.md](FLOW.md) · Folders, order and gates: [PLAN.md](PLAN.md) · What each part must do, with
Cycles' file and line: [SPECS.md](SPECS.md) · Where the source, the tests
and the other references are: [reference/README.md](reference/README.md)
