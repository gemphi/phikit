# cycles — flow

How a render runs in Cycles, top to bottom, with the Rust types of this
crate. A solid box is a type that exists in `src/` today and is tested
(the tests are in `src/tests`, in the same folders). A
dashed box is still Cycles' C++: it shows the Cycles name and where it is,
and is the next thing to port in that layer, or a trait whose body is not
ported.

## The whole render, from settings to pixels

```mermaid
graph TB
    a1["SessionParam<br/>src/session/param.rs<br/>samples, tile size, time limit"]
    a2["Session::run_main_render_loop<br/>src/session/run.rs<br/>asks for work, renders it, until none is left"]
    a3["TileManager<br/>src/session/tile.rs<br/>effective_tile_size · reset_scheduling · update · next"]
    a4["Tile + BufferParam<br/>src/session/tile.rs · buffer/param.rs<br/>the pixels to render and where they sit in the frame"]
    a45["Session::next_work<br/>src/session/update.rs<br/>the next work; when a tile has none, the next tile"]
    a5["RenderScheduler::render_work<br/>src/integrator/scheduler/schedule.rs<br/>the next unit of work, from the samples left and the clock"]
    a6["RenderWork<br/>src/integrator/work.rs<br/>how many samples, and what else to do this round"]
    a7["PathTrace::render<br/>src/integrator/trace/pipeline.rs<br/>the steps of the work, in the pipeline's order"]
    a8["PathTraceWork::render_samples<br/>src/integrator/trace/work.rs is the trait<br/>its bodies: integrator/path_trace_work_cpu.cpp · path_trace_work_gpu.cpp"]
    a9["DeviceKernel<br/>src/kernel/mod.rs<br/>the 18 kernels a path moves between"]
    a10["RenderBuffer + BufferPass<br/>src/session/buffer/render.rs · pass.rs<br/>flat floats, pass_stride a pixel, laid out from data/pass.tsv"]
    a1 --> a2 --> a3 --> a4
    a2 --> a45 --> a5 --> a6 --> a7 --> a8 --> a9 --> a10
    a45 -->|"a new tile: PathTrace::reset(frame, Session::tile_param)"| a7
    a4 --> a45
    a7 -->|"report_*: how long each step took"| a5
    a6 -->|"any() is false: the tile is done"| a45
    classDef todo stroke-dasharray: 6 4;
    class a8 todo;
```

`Session::next_work` is the part of Cycles' update before each round that
joins the scheduler, the tiles and the path tracer; a test renders a frame
of two tiles through it, tile by tile. `Session::run_main_render_loop`
(`src/session/run.rs`) still reaches what is below it through `RenderHost`,
because the rest of that update, the scene's, is not ported.

## Layer 1, session: a frame becomes tiles

```mermaid
graph TB
    s1["SessionParam { use_auto_tile, tile_size }"]
    s2["TileManager::effective_tile_size(param, width, height)<br/>the whole image when one tile covers it, else a square tile"]
    s3["TileManager::compute_render_tile_size<br/>a multiple of IMAGE_TILE_SIZE 128, at most MAX_TILE_SIZE 8192"]
    s4["BufferParam::full(width, height)<br/>update_offset_stride"]
    s5["TileManager::reset_scheduling(buffer, tile_size)<br/>tiles across, tiles down"]
    s6["TileManager::update(buffer, adaptive_sampling, baking)<br/>4 pixels of overscan for many tiles with adaptive sampling"]
    s7["TileManager::next() until done()"]
    s8["Tile { x, y, width, height, window_x, window_y, window_width, window_height }<br/>left to right, then down"]
    s1 --> s2 --> s5
    s3 --> s2
    s4 --> s5 --> s6 --> s7 --> s8
```

## Layer 2, integrator: one round of work

### The scheduler decides the work

`RenderScheduler::render_work` (`src/integrator/scheduler/`) fills one
`RenderWork` in. Each box is a method and the file it is in.

```mermaid
graph TB
    r0["reset(buffer)<br/>mod.rs · divider.rs<br/>update_start_resolution_divider: a moving view starts smaller"]
    r1["render_work(tile_manager)<br/>schedule.rs"]
    r2["check_time_limit_reached · done<br/>report.rs · schedule.rs"]
    r3["next_resolution_divider<br/>schedule.rs<br/>start divider, then 4, then 1"]
    r4["sample_count_to_path_trace<br/>sample.rs<br/>1 at first, then what fits display_update_interval,<br/>a power of two, more for an idle device"]
    r5["AdaptiveSampling::align_sample<br/>adaptive.rs<br/>end the work on a filtered sample"]
    r6["work_need_rebalance · work_need_adaptive_filter<br/>work_need_denoise · work_need_update_display<br/>need.rs"]
    r7["set_postprocess_render_work<br/>schedule.rs<br/>after the last sample: cryptomatte, denoise, write the tile"]
    r8["set_full_frame_render_work<br/>schedule.rs<br/>after the last tile of many: write the full frame"]
    r9["RenderWork"]
    r10["display_update_interval<br/>interval.rs"]
    r0 --> r1 --> r2
    r2 -->|"samples left"| r3 --> r4 --> r5 --> r6 --> r9
    r10 --> r4
    r6 -->|"that was the last sample"| r7 --> r9
    r2 -->|"done"| r7
    r7 -->|"nothing left for the tile"| r8 --> r9
```

### The path tracer carries it out

`PathTrace::render` (`src/integrator/trace/pipeline.rs`) takes the steps
below in this order, leaving out a step whose flag is off
(`integrator/path_trace.cpp:184-256`); `RenderWork::step_list` names the
same steps. A dashed box is reached through a trait: `PathTraceWork` for
what a device does, `TraceHost` for the denoiser, the display and the tile
file. Nothing implements those traits yet outside the tests.

```mermaid
graph TB
    w0["RenderWork"]
    w1["init_render_buffers<br/>update_work_buffer_param_if_needed, trace/buffer.rs<br/>slice_buffer_param · scale_buffer_param, trace/slice.rs<br/>PathTraceWork::zero_render_buffer"]
    w2["rebalance<br/>WorkBalanceInfo::rebalance, balance.rs<br/>copy the tile out, slice by the new weights, copy it back"]
    w3["path_trace<br/>PathTraceWork::render_samples on every device,<br/>which fills its path states from WorkTileScheduler::work, tile/scheduler.rs<br/>report_path_trace_time · report_path_trace_occupancy"]
    w4["adaptive_sample<br/>PathTraceWork::adaptive_sampling_converge_filter_count_active<br/>reschedule_on_converge · reschedule_on_idle, reschedule.rs"]
    w5["cryptomatte_postprocess<br/>PathTraceWork::cryptomatte_postprocess"]
    w6["denoise<br/>TraceHost::denoise · report_denoise_time"]
    w7["write_tile_buffer<br/>TraceHost::write_tile: to the host, or to the tile file"]
    w8["update_display<br/>TraceHost::update_display · report_display_update_time"]
    w9["finalize_full_buffer_on_disk<br/>TraceHost::finish_tile_file"]
    w0 --> w1 --> w2 --> w3 --> w4 --> w5 --> w6 --> w7 --> w8 --> w9
    w4 -->|"no pixel open, or few: a lower threshold, filter again"| w4
    classDef todo stroke-dasharray: 6 4;
    class w5,w6,w7,w8,w9 todo;
```

An instant cancel (`PathTrace::request_cancel`) ends the round after path
tracing, adaptive sampling, cryptomatte or denoising, whichever is running.

## The scene: from sockets to what a kernel reads

Everything a scene holds is a `Node` whose settings are sockets. A manager
writes what the sockets say into the device scene: the constant data and
the flat arrays. The tables are generated from Cycles, never typed in.

```mermaid
graph TB
    t1["data/node.tsv<br/>tool/node.py from the NODE_DEFINE blocks<br/>183 node types, 1,164 sockets with their flags, 627 enum choices"]
    t2["data/kernel.tsv<br/>tool/kernel.py from kernel/data_template.h, types.h<br/>11 structs, 342 members, each with its word"]
    t3["data/constant.tsv<br/>tool/constant.py from kernel/types.h, features.h, kernel/svm/types.h, scene/object.h, geometry.h, bvh/*.h, graph/node_type.h<br/>905 flags and limits by name"]
    t4["data/film.tsv<br/>tool/film.py from Film::device_update<br/>the member each pass type's offset goes to"]
    n1["NodeType · SocketType · NodeEnum<br/>src/graph/schema.rs · socket.rs"]
    n2["Node<br/>src/graph/node.rs<br/>a Value for each socket, and which were set"]
    n3["Integrator { node } · Film { node } · Camera { node }<br/>src/scene/integrator/ · film/ · camera/"]
    n4["device_update of each<br/>Integrator: bounces, clamps, closures, sampling pattern, seed, jitter<br/>Film: Film::update_pass, Film::pass_layout, the pixel filter table<br/>Camera: Camera::update, the matrices, the lens, the shutter table, its places over the shutter"]
    n5["KernelData<br/>src/kernel/data.rs<br/>words laid out by the table: kernel_data.integrator.max_bounce"]
    n6["DeviceScene { data, lookup_table, sample_pattern_lut, object, tri_vert, bvh_node, ... }<br/>src/scene/device.rs<br/>53 arrays in Cycles, listed in data/array.tsv"]
    n9["LookupTableSet<br/>src/scene/table.rs<br/>the filter and shutter tables, packed into lookup_table"]
    n7["Integrator::adaptive_sampling · denoise_param<br/>src/scene/integrator/param.rs"]
    n8["Session::update_integrator<br/>src/session/update.rs<br/>to the RenderScheduler"]
    t1 --> n1 --> n2 --> n3 --> n4 --> n5 --> n6
    t2 --> n5
    t3 --> n4
    t4 --> n4
    n4 --> n9 --> n6
    n3 --> n7 --> n8
```

### A mesh, and the objects that place it

A mesh keeps its triangles in sockets and everything that has a value for
each vertex, corner or face in attributes: its positions are the attribute
`P`, its normals the attribute `N`. An object places a geometry by its
transform. The two managers flatten them into the device's arrays.

```mermaid
graph TB
    a0["data/attribute.tsv<br/>tool/attribute.py from scene/attribute.cpp<br/>each standard attribute's name, type and element"]
    a1["Attribute { name, std, kind, element, center, motion }<br/>src/scene/attribute/mod.rs<br/>AttributeData: Float, Float2, Float3, Float4, Byte, Normal, Matrix"]
    a2["AttributeSet<br/>src/scene/attribute/set.rs"]
    a3["Geometry { node, kind, attribute, used_shader, bound }<br/>src/scene/geometry/mod.rs"]
    a4["Mesh { geometry, subd_attribute }<br/>src/scene/mesh/<br/>Mesh::add_vertex_normal, compute_bound, apply_transform"]
    a5["Object { node, geometry, bound }<br/>src/scene/object/mod.rs<br/>Object::update_motion, compute_bound"]
    a6["Shader · AttributeRequestSet<br/>what the shaders ask a geometry for, handed in"]
    b1["ObjectManager::device_update<br/>src/scene/object/transform.rs<br/>each object as a KernelObject, its flags, its motion"]
    b2["GeometryManager::device_update<br/>src/scene/geometry/manager.rs<br/>normals, offsets, static transforms, bounds, the trees"]
    b3["GeometryManager::device_update_attribute<br/>src/scene/geometry/attribute.rs · map.rs<br/>the attributes asked for, by type, and the map to find them"]
    b4["GeometryManager::device_update_mesh<br/>src/scene/geometry/mesh.rs<br/>Mesh::pack_shader, Mesh::pack_triangle"]
    b5["ObjectManager::device_update_geom_offset · device_update_flag<br/>src/scene/object/offset.rs · flag.rs"]
    c1["DeviceScene.object: RecordArray of KernelObject<br/>object_flag · object_motion_pass · object_motion<br/>layout: data/record.tsv, src/kernel/record.rs"]
    c2["DeviceScene.tri_vert · attribute_float, float2, float3, float4, uchar4, normal<br/>attribute_map: RecordArray of AttributeMap"]
    c3["DeviceScene.tri_vindex · tri_shader"]
    a0 --> a1 --> a2 --> a3 --> a4
    a4 --> b2
    a5 --> b1 --> c1
    a6 --> b2 --> b3 --> c2
    b2 --> b4 --> c3
    b3 --> b5 --> c1
```

### The tree a ray walks

A ray finds what it hits by walking a tree of boxes. Each instanced mesh
has a tree over its triangles; the scene has one over its objects, where
an instanced object is a single primitive that leads into its mesh's
tree. Only Cycles' own binary layout, `BVH2`, is ported.

```mermaid
graph TB
    p0["data/param.tsv<br/>tool/param.py from the constructors of BVHParams and SceneParams<br/>what each setting starts as"]
    p1["SceneParam { bvh_layout, bvh_type, use_bvh_spatial_split, bvh_time_step_count }<br/>src/scene/param.rs"]
    p2["BvhParam · BvhLayout · BvhType · BvhLimit<br/>src/bvh/param.rs<br/>BvhLayout::best: the widest layout the device can walk"]
    g1["Geometry::compute_bvh<br/>src/scene/geometry/bvh.rs<br/>each instanced mesh's own tree: built, or refitted when only vertices moved"]
    g2["GeometryManager::device_update_bvh<br/>src/scene/geometry/bvh.rs<br/>the scene's tree, over the objects"]
    b1["Bvh2::build · Bvh2::refit<br/>src/bvh/bvh2/mod.rs · refit.rs"]
    r1["BvhBuild::add_reference<br/>src/bvh/build/reference.rs<br/>BvhReference: a triangle, or an instanced object, and its box"]
    r2["BvhBuild::build_node<br/>src/bvh/build/mod.rs<br/>BvhObjectBinning: bins along each axis, the cut that costs least"]
    r3["BvhBuild::build_spatial_node<br/>BvhMixedSplit: BvhObjectSplit or BvhSpatialSplit, src/bvh/split/<br/>a triangle may be cut, and is then in two leaves"]
    r4["BvhBuild::create_leaf_node<br/>src/bvh/build/leaf.rs<br/>a leaf for each type of primitive"]
    n1["BvhNode { bound, visibility, time_from, time_to, kind }<br/>src/bvh/node.rs<br/>Inner { child } or Leaf { lo, hi }"]
    b2["Bvh2::pack_node · pack_leaf · pack_instance<br/>src/bvh/bvh2/mod.rs · instance.rs<br/>each geometry's tree after the scene's"]
    k1["PackedBvh { node, leaf_node, object_node, prim_type, prim_index, prim_object, prim_visibility, prim_time, root_index }<br/>src/bvh/mod.rs"]
    d1["DeviceScene.bvh_node · bvh_leaf_node · object_node<br/>prim_type · prim_index · prim_object · prim_visibility · prim_time<br/>kernel_data.bvh.root · bvh_layout · use_bvh_steps"]
    p0 --> p1
    p0 --> p2
    p1 --> g1
    p1 --> g2
    p2 --> b1
    g1 --> b1
    g2 --> b1
    b1 --> r1
    r1 --> r2 --> r4
    r1 --> r3 --> r4
    r4 --> n1 --> b2 --> k1
    g1 -->|"Geometry.bvh"| b2
    k1 --> d1
```

### A shader's graph

A shader is a graph of nodes: textures and maths feed closures, and the
closures feed the output node's surface, volume and displacement. Before
it is compiled the graph is made simpler and is given what it left
unsaid. So far only the graph is ported, not what a node compiles to.

```mermaid
graph TB
    s0["data/node.tsv<br/>every shader node's sockets and their flags; the 64 convert nodes"]
    s1["data/shader.tsv<br/>tool/shader.py from scene/shader_nodes.h<br/>each node class's special type, and what it says of itself"]
    s2["ShaderNode { node, input, output, id, bump, special_type }<br/>src/scene/shader/graph/node.rs<br/>ShaderInput { link } · ShaderOutput { link }, socket.rs"]
    s3["ShaderGraph::connect · disconnect_input · relink_node<br/>link.rs<br/>a convert node, or an emission node, where two types meet"]
    s4["ShaderGraph::remove_proxy_node<br/>proxy.rs<br/>the sockets of node groups, passed through"]
    s5["ShaderGraph::simplify<br/>mod.rs"]
    s6["default_input<br/>input.rs<br/>coordinates, normals and tangents nobody linked"]
    s7["clean<br/>clean.rs<br/>deduplicate_node, optimize_volume_output, break_cycle, unused nodes dropped"]
    s8["refine_bump_node<br/>bump.rs<br/>what a height depends on, three times: ShaderBump::Center, Dx, Dy"]
    s9["ShaderGraph::finalize<br/>mod.rs"]
    s10["bump_from_displacement<br/>bump.rs"]
    s11["transform_multi_closure<br/>closure.rs<br/>mixes and adds become a weight on each closure"]
    s12["compute_displacement_hash<br/>hash.rs<br/>Node::hash into Md5Hash"]
    c1["constant folding, simplify_settings, expand<br/>each node class's own: scene/shader_nodes.cpp, constant_fold.cpp"]
    c2["SVMCompiler: the graph into svm_nodes<br/>scene/svm.cpp"]
    s0 --> s2
    s1 --> s2
    s2 --> s3 --> s4 --> s5
    s5 --> s6 --> s7 --> s8 --> s9
    s9 --> s10 --> s11
    s3 --> s12
    c1 -.-> s7
    s11 -.-> c2
    classDef todo stroke-dasharray: 6 4;
    class c1,c2 todo;
```

### The order a scene is written to the device

`Scene::device_update` (`scene/scene.cpp:176-411`) runs the managers in this
order, because each uses what the ones before it worked out. A solid box
is ported; a dashed one is still Cycles' C++.

```mermaid
graph TB
    u1["shaders: shader.cpp, svm.cpp; their graph is src/scene/shader/graph/"] --> u2["film passes: Film::update_pass, src/scene/film/pass.rs"] --> u3["kernel features, load kernels"] --> u4["background"] --> u5["camera: src/scene/camera/"]
    u5 --> u6["geometry preprocess: GeometryManager::device_update_preprocess"] --> u7["objects: src/scene/object/"] --> u8["particles"] --> u9["meshes: src/scene/geometry/, src/scene/mesh/"] --> u19["the trees: src/bvh/, src/scene/geometry/bvh.rs"] --> u10["object flags, primitive offsets: src/scene/object/flag.rs"]
    u10 --> u11["images"] --> u12["volumes"] --> u13["lookup tables: src/scene/table.rs"] --> u14["lights: light.cpp"] --> u15["integrator: src/scene/integrator/"]
    u15 --> u16["film: src/scene/film/"] --> u17["baking"] --> u18["the constant data to the device"]
    classDef todo stroke-dasharray: 6 4;
    class u1,u3,u4,u8,u11,u12,u14,u17,u18 todo;
```

## Layer 3, kernel: one path

Every name below is a variant of `DeviceKernel` (`src/kernel/mod.rs`). The
arrows are the usual routes between them and are dashed because the kernels
themselves are not ported: each one's body is still in
`kernel/integrator/*.h`, and the routes have not been read line by line.

```mermaid
graph TB
    k1["IntegratorInitFromCamera<br/>a camera ray for a pixel and a sample"]
    k2["IntegratorIntersectClosest<br/>the nearest hit"]
    k3["IntegratorShadeBackground<br/>nothing hit"]
    k4["IntegratorShadeLightForward<br/>a lamp hit"]
    k5["IntegratorShadeSurface<br/>run the shader, sample a light and a direction"]
    k6["IntegratorShadeVolume<br/>inside a medium"]
    k7["IntegratorIntersectShadow<br/>is the light visible"]
    k8["IntegratorShadeShadow<br/>add the light if it is"]
    k9["the path ends: its radiance is in the buffer"]
    k1 -.-> k2
    k2 -.-> k3 -.-> k9
    k2 -.-> k4 -.-> k9
    k2 -.-> k5
    k2 -.-> k6
    k5 -.->|"the next bounce"| k2
    k6 -.-> k2
    k5 -.->|"a shadow path"| k7 -.-> k8
    cpu["on the CPU: IntegratorMegakernel, one path at a time,<br/>a switch on the kernel the path is queued for<br/>kernel/integrator/megakernel.h:22-119"]
    gpu["on the GPU: many paths at once, the fullest kernel runs next,<br/>shading paths sorted by shader<br/>integrator/path_trace_work_gpu.cpp:414, 627-672"]
    k9 --- cpu
    k9 --- gpu
```

## The types, and what each is ported from

| Rust type | File | Cycles | State |
|---|---|---|---|
| `SessionParam`, `ShadingSystem` | `src/session/param.rs` | `SessionParams`, `session/session.h:36-112` | ported, tested |
| `BufferParam` | `src/session/buffer/param.rs` | `BufferParams`, `session/buffers.h`, `buffers.cpp` | ported, tested |
| `BufferPass`, `PASS_UNUSED` | `src/session/buffer/pass.rs` | `BufferPass` and the pass half of `BufferParams`, `session/buffers.cpp` | ported, tested |
| `RenderBuffer` | `src/session/buffer/render.rs` | `RenderBuffers`, `render_buffers_host_copy_denoised` | ported, tested; copies to a device not yet |
| `PassType`, `PassMode`, `PassInfo`, `Pass` | `src/scene/pass.rs`, `data/pass.tsv` | `kernel/types.h` (`PassType`), `scene/pass.h`, `pass.cpp` | ported, tested; the table is generated by `tool/pass.py` |
| `Session`, `RenderHost`, `SessionEnd` | `src/session/run.rs` | `Session::run_main_render_loop`, `session/session.cpp:163-234` | the loop ported, tested; the thread, pause and status not yet |
| `Tile`, `TileManager` | `src/session/tile.rs` | `session/tile.h`, `tile.cpp`; `Session::get_effective_tile_size`, `session.cpp:499-523` | scheduling ported, tested; tile files on disk not yet |
| `RenderWork`, `WorkPathTrace`, `WorkAdaptive`, `WorkTile`, `WorkDisplay` | `src/integrator/work.rs` | `RenderWork`, `integrator/render_scheduler.h` | ported, tested |
| `RenderStep` | `src/integrator/work.rs` | the order of `PathTrace::render_pipeline`, `integrator/path_trace.cpp:184-256` | ported, tested |
| `RenderScheduler`, `SchedulerState`, `FirstRenderTime`, `TimeAverage`, `TaskTime` | `src/integrator/scheduler/` (11 files) | `RenderScheduler`, `integrator/render_scheduler.h`, `render_scheduler.cpp`, all 1,328 lines | ported, tested; Cycles' own 2 tests pass |
| `AdaptiveSampling` | `src/integrator/adaptive.rs` | `integrator/adaptive_sampling.h`, `.cpp` | ported; Cycles' own 3 tests pass |
| `TileSize` | `src/integrator/tile/size.rs` | `integrator/tile.h`, `.cpp` | ported; Cycles' own 2 tests pass |
| `WorkTileScheduler` | `src/integrator/tile/scheduler.rs` | `integrator/work_tile_scheduler.h`, `.cpp` | ported, tested; no device asks it for tiles yet |
| `KernelWorkTile` | `src/kernel/tile.rs` | `kernel/types.h:1688-1701` | ported |
| `WorkBalanceInfo` | `src/integrator/balance.rs` | `integrator/work_balancer.h`, `.cpp` | ported, tested |
| `PathTrace`, `TraceState` | `src/integrator/trace/mod.rs`, `pipeline.rs` | `PathTrace::render`, `render_pipeline` and its steps, `integrator/path_trace.cpp:162-256, 400-523, 606-692, 739-908` | the host loop ported, tested against a scripted device; the works run one after another, not in parallel |
| `PathTrace::reset`, `update_work_buffer_param_if_needed`, `copy_to_render_buffer` | `src/integrator/trace/buffer.rs` | `path_trace.cpp:121-143, 319-398, 1034-1050` | ported, tested |
| `slice_buffer_param`, `scale_buffer_param` | `src/integrator/trace/slice.rs` | `foreach_sliced_buffer_params`, `scale_buffer_params`, `path_trace.cpp:272-352` | ported, tested |
| `Session::next_work`, `Session::tile_param` | `src/session/update.rs` | `Session::run_update_for_next_iteration`, `update_buffers_for_params`, `session/session.cpp:311-430, 554-580` | the scheduler, tile and path tracer part ported, tested end to end on two tiles; the scene's part not |
| `PathTraceWork`, `RenderStatistic`, `TraceHost` | `src/integrator/trace/work.rs` | `PathTraceWork`, `integrator/path_trace_work.h`; the denoiser, display and tile file calls of `path_trace.cpp` | the traits ported; nothing implements them yet outside the tests |
| `DenoiseParam`, `DenoiserType`, `DenoiserPass`, `DenoiserPrefilter`, `DenoiserQuality` | `src/device/denoise.rs`, `data/denoise.tsv` | `device/denoise.h`, `denoise.cpp` | ported, tested; no denoiser yet |
| `Node`, `NodeType`, `SocketType`, `SocketKind`, `NodeEnum`, `Value` | `src/graph/`, `data/node.tsv` | `graph/node.h`, `node_type.h`, `node_enum.h`; every `NODE_DEFINE` of `scene/*.cpp` | ported, tested; all 119 node types, the 64 convert nodes and their sockets with their flags are read; links between nodes, transforms and arrays of nodes hold no value yet |
| `KernelData`, `Field`, `DataStruct` | `src/kernel/data.rs`, `data/kernel.tsv` | `KernelData`, `kernel/types.h:1367-1408`; `kernel/data_template.h` | ported as words laid out by the generated table, tested; the camera is in it, the light link sets not yet |
| `constant`, `flag` | `src/kernel/constant.rs`, `data/constant.tsv` | the enums and defines of `kernel/types.h`, `kernel/features.h`; the SVM's enums of `kernel/svm/types.h`; the managers' flags; the limits and node sizes of `bvh/*.h`; `SocketType::Flags` | generated, tested |
| `DeviceScene` | `src/scene/device.rs`, `data/array.tsv` | `scene/devicescene.h`, `kernel/data_arrays.h` | the constant data and 26 arrays of 53; the others come with their managers. An update writes an array whole: Cycles' tags for what to copy or reallocate are not ported |
| `RecordArray`, `RecordLayout`, `RecordField` | `src/kernel/record.rs`, `data/record.tsv` | `KernelObject`, `AttributeMap`, `kernel/types.h:819-825, 1412-1459` | an array of structs as words laid out by the generated table, tested |
| `Attribute`, `AttributeData`, `AttributeSet`, `AttributeRequest`, `AttributeRequestSet`, `AttributeDescriptor`, `AttributeStandard`, `AttributeElement`, `AttributeType`, `AttributePrimitive`, `KernelDataType` | `src/scene/attribute/` (5 files), `data/attribute.tsv`, `tool/attribute.py` | `scene/attribute.h`, `attribute.cpp`; the attribute enums of `kernel/types.h:730-816` | ported, tested. Not ported: voxel attributes, values shared with the host application |
| `Geometry`, `GeometryKind`, `Shader`, `AttributeIdMap` | `src/scene/geometry/mod.rs`, `shader.rs` | `Geometry`, `scene/geometry.h`, `geometry.cpp:35-222`, `geometry_attributes.cpp:24-87`; `ShaderManager::get_shader_id`, `get_attribute_id` | ported, tested; its own tree is in `bvh.rs`, below. What it reads of a shader is handed in |
| `Mesh`, `MeshSize`, `Triangle`, `SubdFace` | `src/scene/mesh/` (6 files) | `scene/mesh.h`, `mesh.cpp:180-640, 641-817, 873-928` | triangles, bounds, applied transforms, vertex normals, generated and undisplaced attributes, packing: ported, tested. Not ported: tessellation, displacement, tangents |
| `Object` | `src/scene/object/mod.rs` | `scene/object.h`, `object.cpp:75-303, 396-495` | ported, tested. Not ported: a volume's step size and nudge, an object's own attribute values |
| `ObjectManager`, `ObjectInput` | `src/scene/object/` (5 more files) | `ObjectManager`, `object.cpp:497-1313` | `device_update`, the transforms, motion vectors and motion blur, the flags, static transforms, the offsets, the Cryptomatte manifests: ported, tested, for meshes. Hair, points, volumes and the world's object wait for those kinds |
| `GeometryManager`, `GeometryInput`, `find_attribute` | `src/scene/geometry/` (4 more files) | `GeometryManager`, `geometry.cpp:224-427, 792-1277`, `geometry_mesh.cpp`, `geometry_attributes.cpp:89-731`; `kernel/geom/attribute.h:42-92` | offsets, the update's order, the attribute arrays and the attribute map, the triangle arrays: ported, tested. Not ported: tessellation, displacement, curves and points, OSL's map entries |
| `BvhParam`, `BvhLayout`, `BvhType`, `BvhLimit` | `src/bvh/param.rs`, `data/param.tsv`, `tool/param.py` | `bvh/params.h:17-180`, `BVHParams::best_bvh_layout` of `bvh.cpp:65-86`; the layouts `Geometry::need_build_bvh` names | ported, tested; the defaults and limits are generated. Not ported: `bvh_layout_name` |
| `BvhReference`, `BvhRange`, `BvhSpatialBin`, `BvhSpatialStorage` | `src/bvh/reference.rs` | `bvh/params.h:182-323` | ported, tested; the numbers Cycles hides in a box's fourth part are fields |
| `BvhNode`, `BvhNodeKind`, `BvhStat` | `src/bvh/node.rs` | `bvh/node.h`, `node.cpp:17-132` | ported, tested, with two children to a node. Not ported: aligned spaces, `print`, `dump_graph` |
| `BvhObjectBinning` | `src/bvh/binning.rs` | `bvh/binning.h`, `binning.cpp` | ported, tested |
| `bvh_reference_sort`, `bvh_reference_compare` | `src/bvh/sort.rs` | `bvh/sort.cpp` | the order ported, tested; the standard sort stands in for Cycles' threaded one |
| `BvhMixedSplit`, `BvhObjectSplit`, `BvhSpatialSplit` | `src/bvh/split/` (4 files) | `bvh/split.h`, `split.cpp` | ported, tested, for triangles and objects |
| `BvhBuild`, `PrimitiveBit` | `src/bvh/build/` (3 files) | `BVHBuild`, `bvh/build.h`, `build.cpp:57-137, 322-346, 378-568, 633-1200` | references, both builders and the leaves: ported, tested. On one thread; no progress or cancelling; `rotate`, which Cycles has switched off, is not ported |
| `Bvh2`, `PackedBvh`, `bvh::create` | `src/bvh/bvh2/` (3 files), `src/bvh/mod.rs` | `BVH2`, `bvh/bvh2.h`, `bvh2.cpp`; `PackedBVH`, `BVH::create` of `bvh/bvh.h`, `bvh.cpp:97-148` | packing, refitting and the instances' trees: ported, tested by walking the packed tree. Not ported: every layout but `BVH2`; curves, points and their unaligned nodes |
| `SceneParam` | `src/scene/param.rs`, `data/param.tsv` | `SceneParams`, `scene/scene.h` | ported, tested |
| `Geometry::compute_bvh`, `need_build_bvh`, `tag_update`; `GeometryManager::device_update_bvh` | `src/scene/geometry/bvh.rs` | `scene/geometry_bvh.cpp`; `geometry.cpp:164-172, 203-221` | ported, tested. The scene's tree is handed in until the scene itself is ported; the trees are built one after another; `tag_update` does not tag the lights |
| `Film`, `PassLayout`, `FilmPassInput`, `FilmPassChange`, `filter_table` | `src/scene/film/` (4 files), `data/film.tsv` | `scene/film.h`, `film.cpp`, all of it | ported, tested: the passes the film adds and their order, the offsets, the flags, the pixel filter, the kernel features |
| `LookupTableSet` | `src/scene/table.rs` | `LookupTables`, `scene/tables.h`, `tables.cpp` | ported, tested |
| `Camera`, `MotionType` | `src/scene/camera/` (3 files) | `scene/camera.h`, `camera.cpp:198-562, 606-610, 884-917` | the matrices, differentials, frustum, lens, panorama and stereo settings, motion vectors, a moving field of view, the shutter table, the camera's places over the shutter for motion blur (`camera_motion`), `update_interactive_motion`: ported, tested. Not ported: `device_update_volume`, the bounds and sizes for adaptive subdivision, an OSL camera |
| `Transform`, `ProjectionTransform`, `BoundBox2D`, `BoundBox` | `src/util/transform.rs`, `projection.rs`, `bound.rs` | `util/transform.h`, `projection.h`, `projection_inverse.h`, `boundbox.h` | what the camera, the objects, the meshes and the trees need of them, tested |
| `DecomposedTransform`, `motion_decompose`, `motion_array_interpolate`, `compose` | `src/util/motion.rs` | `util/transform.cpp:22-186`, `transform.h:467-660` | ported, tested |
| `PackedNormal` | `src/util/normal.rs` | `packed_normal`, `util/types_normal.h` | ported, tested |
| `ShaderGraph`, `ShaderNode`, `ShaderInput`, `ShaderOutput`, `InputId`, `OutputId`, `ShaderBump`, `ShaderNodeSpecialType` | `src/scene/shader/graph/` (11 files), `data/shader.tsv`, `tool/shader.py` | `scene/shader_graph.h`, `shader_graph.cpp:20-527, 599-1189`; `is_linear_operation` of nine node classes, `shader_nodes.cpp` | links and convert nodes, proxies, default inputs, deduplication, the volume check, cycles, unused nodes, bump samples, bump from displacement, closure weights, the displacement hash: ported, tested. Not ported: constant folding, `simplify_settings` and `expand` (the node classes' own), `get_num_closures`, `dump_graph`, the image nodes' `equals` |
| `Md5Hash` | `src/util/md5.rs` | `MD5Hash`, `util/md5.h` | written from RFC 1321, tested against its digests; `append_file` not |
| `Node::hash`, `copy_value`, `socket_list` | `src/graph/hash.rs`, `node.rs` | `Node::hash`, `Node::copy_value`, `graph/node.cpp` | ported, tested; a string is hashed as its text, where Cycles hashes its address |
| `murmur_hash3`, `hash_to_float` | `src/util/murmur.rs` | `util/murmurhash.cpp` | ported, tested against MurmurHash3's published values |
| `cdf_evaluate`, `cdf_invert`, `cdf_inverted` | `src/util/cdf.rs` | `util/math_cdf.h`, `math_cdf.cpp` | ported, tested |
| `Integrator`, `IntegratorInput`, `HaltonSequence`, `GuidingParam` | `src/scene/integrator/` (4 files) | `scene/integrator.h`, `integrator.cpp` | settings, `device_update`, `get_adaptive_sampling`, `get_denoise_params`, `get_guiding_params`, `get_kernel_features` ported, tested; the tabulated Sobol samples are not generated |
| `hash_uint`, `hash_uint2`, `hash_uint3` | `src/util/hash.rs` | `util/hash.h:80-166` | ported, tested against the same mix written apart in Python |
| `Clock`, `SystemClock`, `ManualClock` | `src/util/time.rs` | `time_dt`, `util/time.h` | a clock handed in, not a global one |
| `DeviceKernel` | `src/kernel/mod.rs` | `DeviceKernel`, `kernel/types.h:1744-1763` | names and numbers ported; no kernel body yet |
| - | - | tile files on disk, `session/tile.cpp` | left in layer 1 |
| - | - | the rest of `PathTrace`: reading and writing tiles and the full frame on disk, the output and display drivers, the denoiser, the reports (`path_trace.cpp`, about 700 lines); `PathTraceDisplay`, `PathTraceTile`, `PassAccessor`, `Denoiser`, path guiding; `PathTraceWorkCPU` and `PathTraceWorkGPU`, which come with the kernels | left in layer 2 |
| - | - | the other managers of `Scene::device_update`, in its order: the shaders (`Shader`, `ShaderManager`, the SVM compiler, what each of the 97 node classes compiles to and folds), background, hair and point clouds, images, volumes, lights, baking (`scene/*.cpp`) | left in the scene |
| - | - | `IntegratorState`, `kernel/integrator/state_template.h` | first in layer 3 |

Cycles' plural type names are singular here: `SessionParams` is
`SessionParam`, `BufferParams` is `BufferParam`, and fields such as
`samples` and `threads` are `sample_count` and `thread_count`.
