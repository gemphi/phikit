# reference

Where what `cycles` is ported from and judged by is on disk (from
`gemphi/`). None of it is built here. Surveyed 2026-11-10.

- **Cycles:** `ext/phiber-refs/blender/blender/intern/cycles`, Blender 5.2.
  872 files, 221,440 lines of `.cpp`, `.h`, `.osl` and `.mm`. Apache-2.0
  (`doc/license/readme.txt`; 891 files carry `SPDX-License-Identifier: Apache-2.0`,
  29 BSD-3-Clause, 3 MIT, 2 Zlib).
- **Its tests:** `ext/phiber-refs/blender/blender/tests/files/render`: 791
  scenes in 45 categories, 768 reference images under
  `<category>/cycles_renders`. The runner is
  `tests/python/cycles_render_tests.py` with `modules/render_report.py`.
  The licence of the test files was not checked.
- **Blender's side of the engine:** `intern/cycles/blender` (15,080 lines):
  not ported, since phiphx reads the file, but it is where to look for how a
  Blender setting becomes a Cycles one.

## Cycles by part

| Part | Files | Lines | What it is |
|---|---|---|---|
| `kernel/` | 366 | 78,733 | everything that runs per path: integrator 14,098, svm 14,204, osl 14,696, device 8,691, closure 7,162, geom 4,435, light 4,296, bvh 2,518, film 2,402, camera 1,046, sample 997 |
| `scene/` | 79 | 41,443 | scene objects, and their flattening into kernel arrays; `shader_nodes.cpp` alone is 8,424 |
| `util/` | 123 | 22,579 | maths, containers, threads |
| `device/` | 80 | 22,056 | cpu, cuda, hip, hiprt, metal, oneapi, optix |
| `blender/` | 36 | 15,080 | the bridge from Blender's data |
| `integrator/` | 43 | 11,464 | the host loop, the scheduler, denoisers, pass access |
| `bvh/` | 29 | 6,477 | tree building and the volume octree |
| `hydra/`, `app/`, `subd/` | 64 | 12,545 | USD delegate, standalone app, subdivision |
| `session/` | 15 | 4,484 | the render session, tiles, buffers |
| `graph/` | 7 | 2,419 | node and socket reflection |

## What we already own that it meets

- `phikit/crates/phiray/src`: `gpu/` and `kernel.rs` (the device layer),
  `bvh/` (SAH trees, a GPU radix sort), `trace/wgsl/bvh.wgsl` (two-level
  walk), `trace/wgsl/light.wgsl` (GGX, sheen, coat, light sampling with
  MIS), `trace/wgsl/adapt.wgsl` (Cycles' convergence check).
- `phikit/crates/phiphx/src/blend/shade`: Blender's material node graphs,
  read and folded to one material.
- `phikit/crates/phiber/src/scene/render_engine.rs`: the Engine row that
  already names Cycles.

## Other references

- `phikit/crates/phiray/reference`: HVVR, DRTK (visibility as a buffer),
  mvp (a tree rebuilt on the GPU each frame), volumetric_primitives (media
  without marching), HairMSNN (a hair BSDF to compare with).
- `phikit/crates/phiray/docs/APPLE.md`: Apple's Metal standards against the
  tracer, which Cycles' own Metal backend (`device/metal`) also follows.
