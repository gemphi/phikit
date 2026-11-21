# cycles — specs

What each part must do. One row is one requirement, with the Cycles source
it is ported from (paths under `intern/cycles`) and what we already own.
Nothing is met today.

## Sources

| Short | What | Where (from `gemphi/`) |
|---|---|---|
| **CY** | Cycles, 221,440 lines, Apache-2.0 | `ext/phiber-refs/blender/blender/intern/cycles` |
| **TS** | Blender's render tests: 791 scenes in 45 categories, 768 reference images | `ext/phiber-refs/blender/blender/tests/files/render`, `tests/python/cycles_render_tests.py` |
| **PR** | phiray | `phikit/crates/phiray/src` |
| **PX** | phiphx's node-graph evaluation | `phikit/crates/phiphx/src/blend/shade` |
| index | the survey of Cycles by part | [reference/README.md](reference/README.md) |

## Rules for the port

| Rule | Why |
|---|---|
| Every ported file names the Cycles file it comes from and keeps the Apache-2.0 notice (`doc/license/readme.txt`) | the licence asks for it |
| A kernel exists in Rust before it exists in WGSL, and the WGSL is tested against the Rust on small inputs | WGSL cannot take Cycles' headers; the Rust is what Blender's images judge |
| One definition of the path state, an array of structs on the CPU and a struct of arrays on the GPU | as `kernel/integrator/state.h:69, 140` |
| Tables are generated from Cycles' headers into `data/` | no data in code; the node list and closure ids must match Blender's exactly |
| Nothing phiray or phiphx owns is written again | one owner a concept |
| `match`, not `else if`; singular names; no file over 250 lines | the workspace's rules |

## truth

| Part | Must | Cycles | Ours |
|---|---|---|---|
| scene and reference | pair `<category>/<name>.blend` with `<category>/cycles_renders/<name>.png` | `cycles_render_tests.py:175` | - |
| verdict | fail a pixel past 0.016 and a render past 1 per cent of pixels; honour the per-category allowances | `render_report.py:138-145, 287-288`; `cycles_render_tests.py:337-357` | phiray's tests compare images their own way |
| settings | render as the test does: the file's own samples and resolution, frame 1 | `cycles_render_tests.py:197-243` | PX reads the file's render settings |

## data

| Part | Must | Cycles | Ours |
|---|---|---|---|
| arrays | fill the kernel arrays a feature needs, and only those | `kernel/data_arrays.h` (53 arrays) | PR packs one word buffer (`trace/wgsl/layout.wgsl`) |
| `KernelData` | hold camera, film, integrator, background and tree settings as one block | `kernel/types.h:1367`, `kernel/data_template.h` | PR `Camera` |
| update order | build in Cycles' order: background, camera, object, geometry and tree, image, light, integrator, film | `scene/scene.cpp:268-405` | - |
| attributes | carry per-vertex, per-corner and per-face attributes by name | `attributes_map`, `attributes_float*` | PR has a fixed vertex record |

## svm and closure

| Part | Must | Cycles | Ours |
|---|---|---|---|
| compile | turn a node graph into the word array Cycles would | `scene/svm.cpp:1174, 1036, 845` | PX folds a graph to one material; it does not compile |
| node kinds | cover the 94 kinds, in the order the test scenes need them | `kernel/svm/node_types_template.h` | PX and PR cover noise, voronoi, gabor, mix, mapping and a few more |
| interpreter | loop over the words with a float stack of 255 | `kernel/svm/svm.h:102-117`, `svm/types.h:41` | - |
| closures | hold up to 64 weighted closures at a hit | `kernel/types.h:830` | PR has one fixed lobe set |
| Principled | build its lobes in Cycles' order: sheen, coat, thin film, metallic, transmission, specular, subsurface, diffuse, emission | `kernel/svm/closure.h:94-520` | PR `light.wgsl:56-292`, PX `principled.rs`: GGX, sheen, coat, diffuse |

## path, light, sample

| Part | Must | Cycles | Ours |
|---|---|---|---|
| state | carry pixel, sample, the bounce counters, the queued kernel, throughput, the ray, the hit, the volume stack | `kernel/integrator/state_template.h:7-151` | PR keeps it in locals of one kernel |
| kernels | be the 18 integrator kernels, each naming its successor | `kernel/types.h:1744-1778` | PR has one megakernel |
| megakernel | run one path to its end by a switch on the queued kernel | `kernel/integrator/megakernel.h:22-119` | - |
| intersection | answer closest, shadow, shadow-all, local and volume queries over two levels | `kernel/bvh/bvh.h:114-394` | PR `bvh.wgsl:107-168`: closest and any |
| roulette | continue with probability min(sqrt(max throughput), 1) past the minimum bounce | `kernel/integrator/path_state.h:277` | PR `light.wgsl:815-819`, capped at 0.95 |
| direct light | sample a light, cast a shadow path, weight by the power heuristic | `kernel/integrator/shade_surface.h:319`, `kernel/sample/mis.h:26` | PR `light.wgsl:345-670` |
| light tree | pick an emitter by bounding box, cone and energy | `kernel/light/tree.h:125, 593, 743` | PR picks one lamp by reservoir |
| samplers | Sobol-Burley and tabulated Sobol | `kernel/sample/pattern.h:42-131`, `types.h:1730` | PR `hash.wgsl` |

## film and wave

| Part | Must | Cycles | Ours |
|---|---|---|---|
| passes | accumulate flat floats, `pass_stride` a pixel | `session/buffers.cpp:122-130`, `kernel/types.h:350` | PR packs its own planes |
| adaptive | stop a pixel under the noise threshold; default 0.01, or max(0.001, 1/samples) when 0 | `kernel/film/adaptive_sampling.h:29`, `scene/integrator.cpp:142, 485` | PR `adapt.wgsl`, already ported from it |
| pick | run the kernel with the most queued paths | `integrator/path_trace_work_gpu.cpp:414` | - |
| sort | sort shading paths by shader key before they run | `:627-672`, `state_template.h:73` | PR `bvh/radix.rs`: a GPU radix sort |
| compact | pack live paths so new ones fill the tail | `:692` | - |
| many at once | keep on the order of a million paths in flight | `device/cuda/queue.cpp:34` | PR traces one path a pixel a dispatch |

## Later, by the same rules

Curves and hair (`kernel/geom/curve_intersect.h`, 958 lines), points, motion
(`bvh/params.h:100-102`), subsurface (`subsurface_random_walk.h:202`),
volumes with null scattering over an octree of majorants
(`shade_volume.h:1939`, `bvh/octree.cpp`), caustics (`mnee.h:947`), shadow
catcher (`shadow_catcher.h:12`), baking (`init_from_bake.h`).
