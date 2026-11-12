# cycles — the merge of two copies
TARGET is this branch, `cycles` at a96b999. OTHER is `cycles-main` at
8ca9c2b. Both grew from the crate of 9996115 (`cycles: the crate as it
stands`), which is the base of the three-way comparison below.
## Classification, file by file
| | Files |
|---|---|
| in TARGET | 426 |
| in OTHER | 331 |
| only in TARGET | 127 (the device, `truth/`, `blender/`, gate 1 kernels, `examples/gate1.rs`, four data tables) |
| only in OTHER | 32 (kernel `global`, `ray`, `intersection`, `bvh/{data,node}`, `camera/{data,orthographic,perspective}`, `geom/triangle/`, `util/`, and their tests) |
| in both, identical | 255 |
| in both, different | 44 |
Of the 44 that differ, against the base:
| Who changed it | Files |
|---|---|
| TARGET only (taken from TARGET) | 26: `ARCHITECTURE.md`, `Cargo.toml`, `data/record.tsv`, `tool/kernel.py`, `src/lib.rs`, `src/device/mod.rs`, `src/integrator/{mod,trace/mod}.rs`, `src/scene/{device,mod,integrator/mod,shader/mod}.rs`, `src/util/{hash,transform}.rs`, and the tests `mod.rs`, `device/mod.rs`, `integrator/trace/mod.rs`, `scene/{mod,integrator/mod,shader/mod}.rs`, `util/hash.rs` |
| OTHER only (taken from OTHER) | 7: `FLOW.md`, `data/constant.tsv`, `tool/constant.py`, `src/kernel/data.rs`, `src/util/projection.rs`, tests `kernel/data.rs`, `util/projection.rs` |
| both (merged by content) | 4: `PLAN.md`, `src/kernel/mod.rs`, `src/util/mod.rs`, tests `kernel/mod.rs` |
| added by both (conflicting ports) | 12: `src/kernel/bvh/{mod,traversal,util}.rs`, `src/kernel/camera/mod.rs`, `src/kernel/geom/{mod,object}.rs`, `src/kernel/sample/{mod,mapping}.rs`, `src/util/intersect.rs`, tests `kernel/sample/{mod,mapping}.rs`, `util/intersect.rs` |
Ports of the same Cycles header under different file names:
| Cycles | TARGET | OTHER |
|---|---|---|
| `kernel/device/cpu/globals.h` `KernelGlobalsCPU` | `kernel/globals.rs` | `kernel/global.rs` |
| `kernel/types.h` `Ray`, `RaySelfPrimitives`, `Intersection` | `kernel/types.rs` | `kernel/ray.rs`, `kernel/intersection.rs` |
| `kernel/types.h` `KernelCamera` | in `kernel/globals.rs` | `kernel/camera/data.rs` |
| `kernel/types.h` `KernelBVH` | in `kernel/globals.rs` (`bvh_root`) | `kernel/bvh/data.rs` |
| `kernel/bvh/nodes.h` | `kernel/bvh/nodes.rs` | `kernel/bvh/node.rs` |
| `kernel/bvh/bvh.h` `scene_intersect` | `kernel/bvh/scene.rs` | `kernel/bvh/mod.rs` |
| `kernel/camera/camera.h` | `kernel/camera/camera.rs` | `kernel/camera/{mod,perspective,orthographic}.rs` |
| `kernel/geom/triangle_intersect.h` | `kernel/geom/triangle.rs` | `kernel/geom/triangle/intersect.rs` |
| `kernel/util/lookup_table.h` | `KernelGlobals::lookup_table_read` | `kernel/util/table.rs` |
| `util/projection.h` `polar_to_cartesian` | `kernel/sample/mapping.rs` | `util/projection.rs` |
