# Jobs - `phiphx/export/usd`
| State | Open | Done |
|---|---|---|
| in progress | 8 | 4 |
## Todo
- [ ] USDC (binary) writer: `export_usdc` is a stub, `UsdStage` writes text only
- [ ] USDZ packing (zip of the layer and its images; packed `.blend` images are not extracted yet, a material names them by key)
- [ ] A real USD reader (`import`): nothing reads a layer back; round-trip tests wait on it
- [ ] UsdSkel: armatures and skin weights (`SceneModel::rig`) are not written
- [ ] `timeSamples`: the layer is one frame; `SceneModel::motion`, actions and the timeline are not written
- [ ] Convert on load: USD, Alembic and OBJ in, `.blend`-free, once the reader exists
- [ ] Per-corner UVs (`SceneUvMap` is per face of the base mesh; the layer has the per-vertex UV the tracer uses) and extra UV sets
- [ ] Check a layer in `pxr`: no usd-core on the dev machine, so only the structural `UsdCheck` has run (`tests/usd_stage_truth.rs` loads it in pxr when `import pxr` works)
## Done
- [x] USDA for meshes and point clouds (`UsdExporter`, `UsdaWriter`)
- [x] `UsdStage`: one `SceneModel` writer: hierarchy, collections, meshes, normals, UVs, materials, instancing, cameras, lights, world (`stage/`, `tests/usd_stage_truth.rs`)
- [x] `BlenderScene::to_usda` goes through `UsdStage` (it wrote the first mesh as a point cloud)
- [x] `UsdCheck`: balanced blocks, valid and unique prim names, counts that agree
Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.