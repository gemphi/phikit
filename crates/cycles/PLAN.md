# cycles — plan

The folders to build and the gates each must pass. Steps 1 to 4 below are
ported as their rows say; no kernel is, so the crate renders no image yet.
Diagrams are in [ARCHITECTURE.md](ARCHITECTURE.md), requirements in
[SPECS.md](SPECS.md). The folders and the order are suggested, not decided.

## Steps, top to bottom

The port follows Cycles' own hierarchy, from the session down to the
kernels, and the crate's folders are Cycles' directories. The render as
these types run it is drawn in [FLOW.md](FLOW.md).

| Step | Layer | Cycles | In `src/` | State |
|---|---|---|---|---|
| 1 | session | `SessionParams`, `Tile`, `TileManager` scheduling | `session/param.rs`, `tile.rs` | done |
| 1 | integrator, the two types the flow needs | `RenderWork`, the order of `render_pipeline` | `integrator/work.rs` | done |
| 1 | kernel, names only | `DeviceKernel` | `kernel/mod.rs` | done |
| 2 | session | the `Session` loop, `BufferParams`, `BufferPass`, `RenderBuffers` | `session/run.rs`, `session/buffer/` | done |
| 2 | scene, what the buffer needs | `PassType`, `PassMode`, `PassInfo`, `Pass`; the pass table generated from Cycles | `scene/pass.rs`, `data/pass.tsv`, `tool/pass.py` | done |
| 2 | session, left over | tile files on disk; the session thread, pause and status | `session/` | - |
| 3 | integrator | `RenderScheduler` in full, `AdaptiveSampling`, `TileSize`, the work balancer; the host loop of `PathTrace` (`render_pipeline` and its steps) over the `PathTraceWork` trait | `integrator/scheduler/`, `adaptive.rs`, `tile.rs`, `balance.rs`, `integrator/trace/` | done |
| 3 | device, scene, util: what the scheduler needs | `DenoiseParams`, `Integrator::MAX_SAMPLES`, `time_dt` as a clock handed in | `device/denoise.rs`, `scene/integrator.rs`, `util/time.rs`, `data/denoise.tsv`, `data/report.tsv` | done |
| 3 | integrator and session, the join | `WorkTileScheduler`, `KernelWorkTile`; the big tile sliced between devices and scaled by the resolution divider; `PathTrace::reset`; the scheduler, tile and path tracer part of `Session::run_update_for_next_iteration` | `integrator/tile/`, `kernel/tile.rs`, `integrator/trace/slice.rs`, `buffer.rs`, `session/update.rs` | done |
| 3 | integrator, left over | what needs a thing not ported yet: `PassAccessor` (the film kernels), the output and display drivers with `PathTraceTile` and `PathTraceDisplay` (the pass accessor), the tile and full-frame files (an image writer), `Denoiser` (a denoiser), the device reports (a device), path guiding | `integrator/` | - |
| 4 | graph: what a scene is made of | `Node`, `NodeType`, `SocketType`, `NodeEnum`; every node type of Cycles generated from its `NODE_DEFINE` blocks | `graph/`, `data/node.tsv`, `tool/node.py` | done |
| 4 | kernel: what a scene is written into | `KernelData` laid out from `kernel/data_template.h`; the kernel's flags and limits; the list of device arrays | `kernel/data.rs`, `kernel/constant.rs`, `scene/device.rs`, `data/kernel.tsv`, `data/constant.tsv`, `data/array.tsv`, `tool/kernel.py`, `tool/constant.py` | done |
| 4 | scene, the first manager | `Integrator`: its sockets, `device_update`, and the settings the scheduler and the denoiser take from it | `scene/integrator/`, `session/update.rs` | done |
| 4 | scene, the film and the camera | `Film` in full: the passes it adds, their layout and flags, the pixel filter; `LookupTables`; `Camera`: its matrices, lens, motion vectors, shutter table and motion over the shutter; the transforms and the CDF tables they need | `scene/film/`, `scene/camera/`, `scene/table.rs`, `util/transform.rs`, `projection.rs`, `cdf.rs`, `data/film.tsv`, `tool/film.py` | done |
| 4 | scene, the objects and the meshes | `Attribute` and `AttributeSet`; `Geometry` and `Mesh`: triangles, normals, bounds, applied transforms; `Object` and `ObjectManager`: transforms, motion, flags; `GeometryManager`: the attribute arrays, the attribute map, the triangle arrays; the bounding box, motion decomposition, packed normals and MurmurHash3 they need | `scene/attribute/`, `scene/geometry/`, `scene/mesh/`, `scene/object/`, `kernel/record.rs`, `util/bound.rs`, `motion.rs`, `normal.rs`, `murmur.rs`, `vector.rs`, `data/attribute.tsv`, `data/record.tsv`, `tool/attribute.py` | done |
| 4 | bvh: the tree a ray walks | `BVHParams`, `BVHReference`, `BVHNode`; `BVHBuild` with object binning and spatial splits; `BVH2` packed, refitted, and with each instanced geometry's tree after the scene's; `SceneParams`; `Geometry::compute_bvh` and `GeometryManager::device_update_bvh` | `bvh/`, `scene/geometry/bvh.rs`, `scene/param.rs`, `util/param.rs`, `data/param.tsv`, `tool/param.py` | done |
| 4 | scene, the shader graph | `ShaderGraph`, `ShaderNode` and their sockets: links with convert nodes, proxies, default inputs, the clean-up, bump samples, bump from displacement, closure weights, the displacement hash; each node class's traits generated; MD5 | `scene/shader/graph/`, `graph/hash.rs`, `util/md5.rs`, `data/shader.tsv`, `tool/shader.py` | done |
| 4 | scene, left over | curves and points in the tree, with their unaligned nodes; tessellation, displacement and tangents of a mesh; hair, point clouds and volumes; the other managers, in the order of `Scene::device_update`: background, the rest of the shaders (`Shader` and its manager, the SVM compiler, what each node class compiles to and folds), lights, images, baking; the camera's volume test; the tabulated Sobol table | `scene/`, `bvh/` | - |
| 5 | kernel | the path state, then the kernels one by one, the CPU megakernel first | `kernel/` | - |
| 6 | kernel on the GPU | the WGSL twin of each kernel, the wavefront loop | `kernel/`, `wgsl/` | - |

The tests are in `src/tests`, apart from the code and in the same folders,
as Blender keeps its unit tests in a `tests` folder beside the code. A file
name is one word: related files share a folder (`session/buffer/`).

Where the folder table below names a folder Cycles does not have (`truth`,
`data`, `svm`, `path`, `wave`), read it as the Cycles directory that holds
that work: `data` is `scene/`, `svm`, `closure`, `path`, `light`, `sample`
and `film` are under `kernel/`, and `wave` is `integrator/`.

## The folder tree (suggested)

Names are singular; no file passes 250 lines; a table Cycles keeps in a
header becomes a data file generated from that header, never typed in.

| Path | Holds | Ported from (`intern/cycles`) | Stands on |
|---|---|---|---|
| `src/truth/` | find a test scene and its reference, render, compare by Blender's rule | `tests/python/cycles_render_tests.py`, `modules/render_report.py:138-145` | - |
| `src/data/` | the kernel arrays: object, triangle, attribute, light, shader | `kernel/data_arrays.h`, `kernel/types.h:1367-1665`, `scene/scene.cpp:176-405` | phiphx `SceneModel` |
| `src/svm/compile/` | a node graph to words | `scene/svm.cpp:845-1174`, `scene/shader_nodes.cpp` | phiphx `blend/shade` graph |
| `src/svm/run/` | the interpreter: one file a family of nodes | `kernel/svm/svm.h:102-117`, `kernel/svm/*.h` | - |
| `src/closure/` | the BSDFs, BSSRDFs and phase functions | `kernel/closure/` | - |
| `src/path/state.rs` | one path's state, defined once | `kernel/integrator/state_template.h`, `shadow_state_template.h` | - |
| `src/path/<kernel>.rs` | the kernels: init, intersect closest, intersect shadow, shade surface, shade light, shade background, and on | `kernel/integrator/*.h`, enum `kernel/types.h:1744-1778` | phiray `bvh` |
| `src/path/mega.rs` | the CPU loop: a switch on the queued kernel | `kernel/integrator/megakernel.h:22-119` | rayon |
| `src/light/` | the light tree, the fallback distribution, MIS | `scene/light_tree.cpp`, `kernel/light/tree.h:125-743`, `kernel/sample/mis.h:26` | - |
| `src/sample/` | Sobol-Burley, tabulated Sobol | `kernel/sample/` | - |
| `src/film/` | passes, AOVs, light groups, the adaptive check | `kernel/film/`, `scene/pass.cpp` | phiray `adapt.wgsl` |
| `src/wave/` | the wavefront loop, queues, sort by shader, compaction | `integrator/path_trace_work_gpu.cpp:348-692` | phiray `gpu`, `kernel`, `bvh/radix.rs` |
| `wgsl/` | the WGSL twin of each kernel, closure and node | the same headers | phiray `gpu` |
| `data/` | node kinds, closure ids, the Sobol table: generated | `kernel/svm/node_types_template.h`, `kernel/svm/types.h:464-529`, `kernel/types.h:1730` | - |
| `reference/` | where the source and the tests are | - | - |

## Gates

A gate is passed when the named test categories render within Blender's
tolerance: a pixel fails past 0.016, and a render fails when more than 1 per
cent of pixels do (`render_report.py:287-288`), with the per-category
allowances Blender itself sets (`cycles_render_tests.py:337-357`).

| Gate | On | What must work | New in it |
|---|---|---|---|
| 0 | - | the harness: a scene in, a verdict out, against an image Blender made | `truth` |
| 1 | CPU | camera, triangles, instances, a diffuse and an emission closure, background | `data`, `path`, `mega` |
| 2 | CPU | node graphs: the node kinds the test scenes use most, first; the Principled lobes | `svm`, `closure` |
| 3 | CPU | lamps, next-event estimation, MIS, the light tree; the samplers | `light`, `sample` |
| 4 | CPU | passes, AOVs, adaptive sampling, shadow catcher | `film` |
| 5 | GPU | the same scenes on wgpu: state in arrays, queues, the WGSL twins, sort, compact | `wave`, `wgsl` |
| 6 | both | curves and hair, points, motion blur, subsurface, volumes with null scattering, caustics | more of `path`, `closure` |

```mermaid
graph LR
    g0["0 truth"] --> g1["1 geometry and two closures"] --> g2["2 node graphs"] --> g3["3 lights and samplers"] --> g4["4 film"]
    g2 --> g5["5 the GPU"]
    g3 --> g5
    g4 --> g6["6 hair, volumes, skin, motion"]
    g5 --> g6
```

Gate 2 starts with a census: which of the 94 node kinds and which closures
each of the 791 test scenes uses, so the order of porting follows what the
scenes need. phiphx's parity plan was driven the same way.

## How each reference is used

| Reference | Used for |
|---|---|
| Cycles source | the specification: every kernel, node and closure is ported from its file, and says which |
| Blender's 791 test scenes and 768 reference images | the judge, from gate 0 on |
| Cycles' CPU path (`path_trace_work_cpu.cpp:69-180`) | the shape of the Rust reference: one path, one switch |
| Cycles' GPU path and its Metal backend | the shape of the wavefront loop; kernel data baked into specialised pipelines (`device/metal/device_impl.mm:429`) |
| phiray | the GPU layer, the trees, the radix sort, the adaptive check, a megakernel to compare images with |
| phiphx | the scene and the node graphs; its fold of a graph to one material stays phiray's path |
| Meta's renderers (`phiray/reference`) | HVVR and DRTK for keeping visibility as a buffer; mvp for rebuilding the top tree on the GPU; volumetric_primitives for media without marching, as a second option beside null scattering |
| Apple's Metal headers | the acceleration-structure and counter vocabulary phiray's plan already maps |

## Not ported

OSL (16,349 lines; only CPU and OptiX run it), path guiding (OpenPGL, CPU
only), the OIDN and OptiX denoisers (outside libraries: phiray's filter
stands in), `blender/` (phiphx reads the file), `device/` (wgpu), Embree and
the vendor trees, hydra, the standalone app.

## Two engines, one dropdown

phiray and `cycles` are two different engines. They share what comes before
the light (the scene from phiphx, the GPU layer, the trees) and nothing
after it: each has its own kernels, its own materials and its own image.

| | phiray | `cycles` |
|---|---|---|
| Goal | fast and interactive, its own look | match Blender's Cycles image for image |
| Materials | a node graph folded to one fixed material | the node graph itself, compiled and run for every hit |
| Path | one kernel traces a whole path | many small kernels, paths sorted by shader |
| Judged by | its own tests | Blender's reference images |

Phiber's Engine dropdown already lists both names
(`phiber/src/scene/render_engine.rs`). Today choosing "Cycles" there still
runs phiray, because no other engine exists. Once `cycles` passes a gate,
choosing "Cycles" runs `cycles` for the scenes that gate covers, and
choosing "Phiray" runs phiray as before. The dropdown is the only thing
they have in common in the app: each engine registers itself the same way
(a `Readout` role in the phiecs plan) so the app has one switch to flip,
not two code paths to maintain.
