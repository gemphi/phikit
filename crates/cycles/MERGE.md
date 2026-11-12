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
## How each conflict was settled
One module for each Cycles header; tests from both sides kept, the
duplicates merged into one test (five).
| Module | Kept | Why |
|---|---|---|
| `KernelGlobals` | TARGET's struct (the scene as `Arc<DeviceScene>`, typed film, integrator, background, objects, shaders), in OTHER's file `kernel/global.rs`, with OTHER's `bvh`, `filter_table_size`, `shutter_table_size` | the CPU device's kernel table holds the globals past any borrow, so they own the scene; Cycles' name `KernelGlobals`; the file name singular. `KernelGlobals::of(&DeviceScene)` keeps OTHER's tests as written |
| `KernelCamera` | OTHER's (`kernel/camera/data.rs`), plus `width` | it reads the motion steps, perspective motion, the shutter table and rolling shutter that TARGET's left out; `kind` is a `CameraType`, not a number |
| `Ray`, `RaySelfPrimitive` | OTHER's (`kernel/ray.rs`) | the ray's `self` as `own` (TARGET called it `kind`); `PRIM_NONE`, `OBJECT_NONE` from the table |
| `Intersection` | OTHER's file, TARGET's `Default` | an empty hit is `PRIM_NONE`, as the walk starts it and TARGET's kernels test it; OTHER's test changed to that |
| `kernel/bvh/` | OTHER's walk, node, data, `scene_intersect` (+ `scene_intersect_shadow`); TARGET's fixed stack, `ray_offset`, `intersection_t_offset` | OTHER's node test uses Cycles' own `min`/`max` (`util/math_base.h:173`), whose order with a NaN is Cycles'; the numbers come from `data/constant.tsv`, not typed in. Cycles' stack is 192 entries on the stack (`BVH_STACK_SIZE`); OTHER's grew a `Vec` for every ray |
| `kernel/camera/` | OTHER's (`mod.rs`, `perspective.rs`, `orthographic.rs`) | more of `camera.h`: polygon blades, the shutter curve, rolling shutter, a moving camera, perspective motion, `camera_position`, `camera_distance`; an unported camera gives black, as Cycles does |
| `kernel/geom/object.rs` | both: OTHER's `object_fetch_transform` and instance push/pop, TARGET's position, direction and normal transforms | OTHER's `ObjectField` is gone: TARGET's `KernelObject` reads the same record once |
| `kernel/geom/triangle` | a folder: TARGET's `triangle.h` reads in `mod.rs`, OTHER's `triangle_intersect` in `intersect.rs` | OTHER's takes `tmax` as Cycles' does |
| `util/intersect.rs` | TARGET's | it adds `ray_triangle_intersect_self` and names the hit's fields; OTHER's tests read them |
| `kernel/sample/mapping.rs` | both: TARGET's cosine hemisphere and orthonormals, OTHER's polygon | `polar_to_cartesian` is `util/projection.h`'s, so `util/projection.rs` (OTHER) keeps it |
| `lookup_table_read` | OTHER's `kernel/util/table.rs` | `kernel/util/lookup_table.h` is its file; a read past the table is zero, not a panic |
| `data/*.tsv` | all regenerated by `tool/*.py` from the Cycles source | every table matched its generator (only line endings differed) |
One test of TARGET's join was stale: since a96b999 a triangle's `shader` is
its slot in `used_shaders` (`mesh.cpp:713`), and the test read it as a
scene shader; it now maps the slot through `used_shaders`.
