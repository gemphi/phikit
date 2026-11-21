# cycles — where the merged crate came from
On 12 November 2026 two copies of this crate, grown apart from 9996115
(`cycles: the crate as it stands`), became one: `cycles` at a96b999
(TARGET) and `cycles-main` at 8ca9c2b (OTHER), merged in 319637b.
- From TARGET: `device/`, `truth/` (gate 0), `blender/` (the join), the
  gate 1 kernels (`kernel/{closure,device,film,integrator,svm}`, the
  samplers of `kernel/sample/` and its cosine hemisphere),
  `examples/gate1.rs`, `data/{sobol,state,svm,tabulated}.tsv`; the owned
  `KernelGlobal` with its typed film, integrator, background, objects and
  shaders; `util/intersect.rs` (with `ray_triangle_intersect_self`);
  `ray_offset` and `intersection_t_offset`; Cycles' fixed BVH stack.
- From OTHER: the camera (`kernel/camera/`: motion, rolling shutter,
  blades, perspective motion), `KernelCamera`, `Ray` and `Intersection` in
  files of their own, the BVH walk and node test (`kernel/bvh/`, numbers from
  `data/constant.tsv`, Cycles' NaN order of `min`/`max`), `KernelBvh`,
  `triangle_intersect` with Cycles' `tmax`, `object_fetch_transform` and the
  instance push and pop, `kernel/util/` (lookup table, differentials),
  `regular_polygon_sample`, and the edits to `FLOW.md`, `data/constant.tsv`,
  `tool/constant.py`, `kernel/data.rs` and `util/projection.rs`.
- Both sides' tests were kept (960 pass); five that tested the same thing
  were made one. Every `data/*.tsv` matched its `tool/*.py`, run on the
  Cycles source.
