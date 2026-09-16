# .blend File Reading — Blender Parity Gap Analysis

> **Blender references (all under `REFS/blender/blender/source/blender/`):**
> - `blenloader_core/BLO_core_blend_header.hh`, `intern/blo_core_blend_header.cc` — file header (legacy 12-byte and v1 17-byte)
> - `blenloader_core/BLO_core_bhead.hh` — `BHead4` / `SmallBHead8` / `LargeBHead8`, block codes
> - `blenloader/intern/readfile.cc` — `blo_read_file_internal`, `read_global`, `read_libblock`, `read_struct`, `do_versions`, `lib_link_all`
> - `blenloader/intern/writefile.cc` — what Blender 5.x writes by default (line ~770: `LargeBHead8` unless `write_legacy_blend_file_format`)
> - `blenloader/BLO_readfile.hh` — `BlendFileReadReport` (timings + counts), `eBLOReadSkip`
> - `windowmanager/intern/wm_files.cc` — `WM_file_read`, `wm_file_read_pre/post`, `file_read_reports_finalize`, `wm_history_file_update`
> - `makesdna/DNA_fileglobal_types.h` — `FileGlobal` (`curscene`, `curscreen`, `cur_view_layer`, version)
> - `makesdna/DNA_mesh_types.h`, `DNA_object_types.h` — 5.x mesh attribute layout, Object transform fields
> - `editors/space_info/info_stats.cc` — status-bar statistics string
> - `draw/engines/{workbench,overlay,eevee}/shaders/`, `gpu/shaders/material/` — viewport shaders
>
> **Phiber code under review:**
> - `crates/phiber/lib/phiphx/importer/blend.rs` + `blend/*` — the native SDNA reader (declared into phiber via `#[path]` in `src/lib.rs`)
> - `crates/phiber/lib/phiphx/services/scene/{mod,load,import}.rs` — `SceneModel` and the blend → scene mapping
> - `crates/phiber/src/v2/app/{handle,import}.rs`, `src/v2/scene/{keys,source}.rs` — the v2 open/progress/settle shell
> - `crates/phiber/src/v2/part/statusbar/list.rs`, `src/v2/part/custom/outliner/tree.rs`, `src/v2/view/render/gpui/atom/canvas/view3d/` — consumers
> - `crates/phiphx/` (workspace crate) — **currently an empty stub (`lib.rs` is 0 lines)**

Reference Blender version in `REFS`: **5.2.0** (`BKE_blender_version.h`: `BLENDER_VERSION 502`, `BLENDER_FILE_MIN_VERSION 405`).

---

## 1. Goal

Opening a `.blend` in Phiber should show what Blender shows on open — the file's scene, its collection tree, every object placed and oriented as Blender places it, materials as Blender colours them in Solid/Material Preview, the file's saved workspace/viewport state — with better performance than Blender, and with all reading done through one crate: `phiphx`. This document records how Blender does it, how Phiber v2 does it today, where the gaps are, what is wrong, what is already good, and the order to fix things in.

---

## 2. How Blender opens a file (the reference pipeline)

`WM_file_read` (`wm_files.cc` ~L1030–1160) is the whole story:

1. `wm_read_callback_pre_wrapper` → `WM_cursor_wait(true)`. **Blender shows a wait cursor, not a progress bar, while reading a `.blend`.** The UI blocks. Progress bars in Blender come from `WM_jobs` (render, bake, asset/thumbnail loading) and `WM_progress_set` (window-level, used by some importers) — not from file open.
2. `wm_read_exotic(filepath)` — sniff the magic; only `BKE_READ_EXOTIC_OK_BLEND` continues.
3. `BKE_blendfile_read` → `blo_read_file_internal` (`readfile.cc` L4151):
   - decode header (`BLO_readfile_blender_header_decode`) → pointer size, endian, file version, **file-format version 0 or 1** → picks `BHead4` / `SmallBHead8` / `LargeBHead8`.
   - transparently decompress (zstd, and gzip for old files).
   - read `DNA1` → `SDNA`; every struct read goes through `read_struct` → `DNA_struct_reconstruct` when the file's struct layout differs from the running binary's.
   - block loop: `GLOB` → `read_global` (this is where **`FileGlobal.curscene`, `cur_view_layer`, `curscreen`, subversion** come from — Blender uses the *saved active scene*, not "the first SC block"); `TEST` (thumbnail, only kept in background mode); `REND`; `USER` (skipped for normal open via `BLO_READ_SKIP_USERDEF`); `ID_LI` libraries; `ID_LINK_PLACEHOLDER`; every other ID via `read_libblock`.
   - pointer remap uses an `OldNewMap` **hash map** keyed on `BHead.old` — O(1) per lookup.
   - `do_versions` per library, `lib_link_all`, `do_versions_after_linking`, lib-override resync, `blo_read_file_checks`.
   - `BlendFileReadReport` accumulates `duration.{whole,libraries,lib_overrides,...}` and `count.{missing_libraries,missing_linked_id,missing_obdata,...}`.
4. `wm_file_read_pre` → swap `Main` (`BKE_blendfile_read_setup_readfile`) → `wm_file_read_setup_wm_finalize` (honours **Load UI**: the file's `wmWindowManager`/`WorkSpace`/`bScreen` become the live UI unless `G_FILE_NO_UI`).
5. `WM_check` (opens windows), `wm_history_file_update` (recent files list).
6. `wm_file_read_post`: `BKE_CB_EVT_LOAD_POST`, **`wm_event_do_depsgraph(C, true)` — the depsgraph evaluates every object (modifiers, constraints, parenting, drivers) before anything is drawn**, `ED_editors_init`, `NC_WM|ND_FILEREAD` notifier, undo stack reset, `WM_toolsystem_init`.
7. `file_read_reports_finalize` → `CLOG_INFO "Blender file read in %.0fm%.2fs"` (+ libraries / overrides timings); `wm_file_read_report` → **report banner** for missing render engines, missing libraries etc. (`WM_report_banner_show`).
8. `WM_cursor_wait(false)`.

What the user then *sees*: the saved workspace tabs and active workspace; each area's saved editor (View3D with its saved `RegionView3D` view quaternion/distance/offset and shading mode, Outliner with saved expand state via `TreeStoreElem`, Properties tab, Timeline frame range and current frame); the Outliner's View Layer tree (Scene Collection → collections → objects → object data, with exclude/hide/select flags from `LayerCollection`/`Base`); the viewport drawn by the **Workbench** engine (studio light, matcap, cavity, object outlines, shadow) plus the **Overlay** engine (grid, axes, wireframes, selection outline, camera frustums, light gizmos, empties, bones, relationship lines); the status bar with `Verts | Faces | Tris | Objects` stats (`info_stats.cc`), memory and version.

---

## 3. How Phiber v2 opens a file today

`BPhiberApp::handle_action` (`src/v2/app/handle.rs`):

1. `should_import(prev_loaded_path, file.selected)` — pure predicate, tested. Good.
2. Sets `scene.loading = true`, seeds `LoadStage::Reading` (0.15) into `scene.load_stage` / `scene.load_progress`. **These are the only progress writes until settle — the "progress bar" jumps 0.15 → 1.0 with nothing in between.** The comment admits it: "the opaque background parse can't call back mid-flight".
3. `cx.background_spawn(load_blend_scene(path))` raced against a **30-second watchdog** → `ImportOutcome::TimedOut`. A large file that would take 40 s is discarded with `timeout: blend import exceeded 30s`. Blender never times out a file read.
4. `load_blend_scene` (`services/scene/load.rs`): metadata gate → **`.phx` sidecar fast-path** (if `foo.phx` next to `foo.blend` is newer, it is loaded instead and the `.blend` is never parsed) → `BlendImporter::import_blend_native` under `catch_unwind` → `scene_from_blend`.
5. `import_blend_native` (`importer/blend.rs`): `std::fs::read` → `BlendFile::parse_bytes` (zstd/gzip sniff, legacy header, block table, DNA) → `extract_scene` → **writes a `.phx` sidecar next to the user's file as a side effect of opening**.
6. `settle_import` clears the loading keys, sets `scene.loaded_path` / `scene.error`, caches `SceneModel` in `scene_cache`; `render()` puts it in a thread-local (`scene/source.rs`) for the Outliner and Viewport.

The viewport (`view/render/gpui/atom/canvas/view3d/mesh/`) is a **CPU painter's-algorithm rasteriser**: triangles are projected, flat-lit by a single normal, depth-sorted and painted as gpui paths. No GPU shaders exist in phiber (`grep wgsl|wgpu|glsl` in `src/` matches only naming in host/space specs).

---

## 4. Gap table

Legend — **GOOD**: at parity or better. **WRONG**: implemented but produces a result Blender would not. **MISSING**: not implemented. **BLOCKER**: prevents current Blender files from opening at all.

### 4.1 Container / low-level reading (`parse.rs`, `dna.rs`, `dna_layout.rs`)

| # | Blender behaviour | Phiber today | Status | Fix |
|---|---|---|---|---|
| 1 | Header decode handles both the legacy 12-byte `BLENDER-v305` header **and the v1 17-byte `BLENDER17-01v0502` header** (`blo_core_blend_header.cc`). **Blender 5.0+ writes the v1 header + `LargeBHead8` by default** (`writefile.cc` L770, L1564–1573). | `read_header` only knows the legacy layout: byte 7 must be `_`/`-`, byte 8 `v`/`V`, bytes 9–11 a 3-digit version. On a 5.x file it reads `'1'` as pointer size 8, `'7'` as **big-endian**, and version `"-01"` → 0, then mis-parses every block header. | **BLOCKER** | Port `BLO_readfile_blender_header_decode` verbatim: return `{pointer_size, endian, file_version, file_format_version}`; select the BHead type from it. Reject `BlenderHeaderUnknown` with a "newer Blender file format" error, as Blender does. |
| 2 | Three BHead layouts: `BHead4 {code,len:i32,old:u32,SDNAnr,nr}`, `SmallBHead8 {code,len:i32,old:u64,SDNAnr,nr}`, `LargeBHead8 {code,SDNAnr,old:u64,len:i64,nr:i64}` (`BLO_core_bhead.hh`). | `read_block` hard-codes the `SmallBHead8`/`BHead4` field order (`code,len,old,SDNAnr,nr`) with 32-bit `len`/`nr`. `LargeBHead8` (field order and 64-bit sizes differ) is unsupported. | **BLOCKER** | Add `BHeadType` enum + three decoders into a common `BHead { code, sdna_nr, old, len: i64, nr: i64 }`. |
| 3 | Zstd (5.x default "compress") and gzip (≤2.7x) transparently decompressed. | `parse_bytes` sniffs both magics. | GOOD | Keep. Consider streaming decode instead of `decode_all` for memory on multi-GB files. |
| 4 | Endianness: 5.x only writes little-endian; big-endian files from old PPC Blenders are byte-swapped on read. | Reads the endian flag; readers honour it for headers. Struct field reads in `dna_layout.rs` are mostly LE. | GOOD (for real-world files) | Low priority; document as LE-only. |
| 5 | `DNA1` decode → `SDNA`; field offsets from the *file's* DNA; `DNA_struct_reconstruct` when the runtime struct differs. | DNA parsed; offsets computed at runtime from the file's DNA (`struct_field_offset`). No reconstruct needed because phiphx has no compiled-in structs. | GOOD | This is the right design. Keep offsets DNA-driven everywhere; **remove** the hard-coded "fallback offsets" in `object_transforms.rs::extract_transforms_fallback` — they are what goes wrong silently on layout changes. |
| 6 | Pointer remap via `OldNewMap` hash (O(1)). | Every dereference is `self.blocks.iter().find(|b| b.old_address == addr && b.code_str == ...)` — a **linear scan per pointer** (`material.rs` L37/70/147/228, `scene.rs`, `mesh_block.rs`). The code's own comments call it "O(n²)" and it is the reason for the 30 s watchdog and the `.phx` sidecar workaround. | WRONG (perf) | Build `HashMap<u64 /*old*/, usize /*block idx*/>` once after the block loop (Blender's `oldnewmap_insert`). This alone removes the need for the sidecar-on-open hack. |
| 7 | Whole-file read is a single pass; blocks are memory-mapped/streamed (`FileReader`), not copied per block. | `read_block` allocates a fresh `Vec<u8>` per block (tens of thousands of allocations on real files). | WRONG (perf) | Read the file once into one buffer (or mmap) and store `(offset, len)` slices per block. |
| 8 | `GLOB` → `FileGlobal`: **`curscene`**, `cur_view_layer`, `curscreen`, `subversion`, `minversion`, `filepath`, colorspace. | `GLOB` is never read. `extract_scene` takes **the first `SC` block**, which is wrong for any multi-scene file. Subversion is not exposed. | WRONG | Read `FileGlobal` first; use `curscene`/`cur_view_layer` as the root; surface version+subversion in the status bar Version segment. |
| 9 | `REND` block: cheap `RenderInfo` (frame range, scene name) for external tools; `TEST` block: thumbnail for the file browser. | Neither read. File browser shows no `.blend` thumbnails. | MISSING | Add `read_render_info()` and `read_thumbnail()` header-only readers (stop at first `DNA1`/ID block) so the file browser can preview without a full parse — Blender's file browser does exactly this (`BLO_thumbnail_from_file`). |
| 10 | Linked libraries (`ID_LI`), `ID_LINK_PLACEHOLDER`, library overrides, resync; `BlendFileReadReport.count.missing_*` drives the report banner. | No library handling. Objects whose data lives in a linked `.blend` silently come out empty. | MISSING | Phase 3: resolve `Library.filepath` (`//`-relative), open linked files, map placeholders; count and report missing ones exactly as `file_read_reports_finalize` does. |
| 11 | `do_versions*` for old files (`versioning_250…520.cc`). | None. phiphx instead keeps *two* mesh layouts (`mvert/mpoly/mloop` legacy and 4.x attribute). | Acceptable | Reading is DNA-driven, so versioning is only needed where *semantics* changed (e.g. `smooth` flag → `sharp_face` attribute, `MLoopUV` → float2 attribute). Keep the dual paths but gate them on `file_version`, not on "modern path returned nothing". |

### 4.2 Scene graph extraction (`scene.rs`, `object.rs`, `object_transforms.rs`, `object_meta.rs`)

| # | Blender | Phiber today | Status | Fix |
|---|---|---|---|---|
| 12 | Outliner "View Layer" mode shows `ViewLayer.layer_collections` (Scene Collection → children), with `LayerCollection.flag` exclude / hide_viewport / holdout / indirect_only, and per-object `Base.flag` (selected, hidden). | `extract_collection_hierarchy` walks `Scene.master_collection` → `Collection.children` / `gobject` (good), **but `scene_from_blend` throws the tree away**: `SceneModel` has no `collections` field, and `outliner/tree.rs::flatten` builds a flat, alphabetically sorted object list under one root. View-layer flags are never read. | WRONG | Carry `collections: Vec<SceneCollection { name, children, object_indices, flags }>` into `SceneModel`; read `ViewLayer`/`LayerCollection`/`Base` for the flags; make the Outliner render the tree (the expand/hide plumbing in `tree.rs` already exists). |
| 13 | Object placement = `BKE_object_where_is_calc`: `loc/rot(or quat, by rotmode)/size` → local matrix, × `parentinv`, × parent's world matrix (recursively), plus constraints. **In 4.x/5.x DNA there is no `obmat` field** — `object_to_world` is runtime only (`DNA_object_types.h` L675). | `object_transforms.rs` reads `loc`, `rot`, `quat`, `size` and looks for `"obmat"` (absent in 4.x+ files → falls back). Then `object_from_blend` copies **only `location` and `scale`** into `SceneObject` — **rotation and `matrix_world` are dropped** (`services/scene/import.rs` L45–46; `SceneObject` has no rotation field). Parenting (`*parent`, `parentinv`) is never read. | WRONG | Compute `matrix_world` in phiphx exactly as Blender does (loc/rot/quat/`rotmode`/`size` → `parentinv` → parent chain; `parent_type` OBJECT at minimum). Store a full `[[f32;4];4]` on `SceneObject` and have the viewport transform by it. |
| 14 | Object types drawn: MESH, CURVES/CURVE/SURF/FONT (evaluated to mesh), META, ARMATURE (bones), LATTICE, EMPTY (draw type: plain axes/arrows/cube/image), CAMERA (frustum), LIGHT (gizmo per type), SPEAKER, LIGHTPROBE, GPENCIL, VOLUME, POINTCLOUD. | Mesh vertices/faces only; CAMERA/LIGHT partitioned into `cameras`/`lights` with a few fields; everything else appears in the outliner (if at all) and draws nothing. | MISSING | Add typed data readers per `Object.type`: `Camera` (lens, sensor, clip, type), `Light` (type, color, energy, spot size), `Empty` (`empty_drawtype`, `empty_drawsize`), then Curves/Text/Armature in later phases. Draw the Blender overlay shapes for them. |
| 15 | Instancing: `Object.instance_collection` + `transflag & OB_DUPLICOLLECTION`, plus geometry-nodes instances via depsgraph. | `object_meta.rs::extract_instance_info` flags `is_instance` and `instance_targets` names. Nothing is instanced in the viewport. | MISSING | Expand collection instances into transformed copies at scene-build time (Blender does this in the depsgraph). |
| 16 | Modifiers are **evaluated** (subsurf, mirror, array, boolean, solidify, bevel, geometry nodes…) before drawing. | `extract_modifiers` lists modifier names; geometry is drawn unmodified. A subdivided character shows as its cage. | MISSING (biggest visual gap after transforms) | Phase 4: evaluate the common non-destructive modifiers in Rust (Mirror, Array, Solidify, Subdivision via OpenSubdiv-equivalent or Catmull-Clark, Triangulate, Weld, Displace-less). Show the modifier stack in Properties → Modifiers meanwhile so the user sees *why* the shape differs. |
| 17 | Active object & selection from `ViewLayer.basact` / `Base.flag & BASE_SELECTED`. | `BlendScene` has `active_object` / `selected` fields but they are never populated. | MISSING | Read `ViewLayer.basact` and `Base.flag`; drive outliner highlight + viewport outline. |
| 18 | Object unique names come from `ID.name` (already unique per type in Blender). | `extract_scene` re-derives `.001` suffixes with a `HashMap` counter, which can rename objects that were legitimately unique or that were **already** `Cube.001` in the file. | WRONG | Use `ID.name[2..]` verbatim. |

### 4.3 Mesh data (`mesh.rs`, `mesh_block.rs`)

| # | Blender | Phiber today | Status | Fix |
|---|---|---|---|---|
| 19 | 4.x/5.x mesh: `verts_num/edges_num/faces_num/corners_num`, `face_offset_indices`, `position` (float3), `.corner_vert`, `.corner_edge`, `.edge_verts`, `material_index`, `sharp_face`, `sharp_edge`, custom normals, UV float2 layers, colour attributes — stored in `CustomData vert/edge/face/corner_data` and (5.x) `AttributeStorage attribute_storage`. | Reads counts, `face_offset_indices`, `position`, `.corner_vert`, UV layer, loop normals from both `CustomData` and `attribute_storage`. Bounds-checked, never panics. | GOOD (core geometry) | Keep; add fuzz tests using `tests/fixtures`. |
| 20 | `material_index` (per-face int attribute) selects the material slot per face. | Not read. `SceneObject.material_indices` is **always empty** (`import.rs` L136, L356), so every face gets slot 0 — multi-material objects render single-colour. | WRONG | Read the `material_index` int attribute (the helper `read_customdata_int` already exists) and carry per-triangle indices through triangulation. |
| 21 | Smooth vs flat shading from `sharp_face` (5.x) / `ME_SMOOTH` poly flag (legacy); custom split normals. | Loop normals read when present; otherwise `build_normals` synthesises per-face normals → everything flat-shaded. | WRONG | Honour `sharp_face`; compute vertex-averaged normals for smooth faces exactly like `BKE_mesh_normals_loop_split`. |
| 22 | Edges (loose edges drawn in wireframe/overlay), vertex groups, shape keys (`Key`), colour attributes. | Not read. | MISSING | Phase 4. |
| 23 | Triangulation: Blender uses `BKE_mesh_calc_corner_tris` (ear-clipping for n-gons via `BLI_polyfill_calc`). | Fan triangulation in `scene_from_blend`. Concave n-gons render wrong. | WRONG | Port `BLI_polyfill_2d` (ear clipping) for n>4. |

### 4.4 Materials, textures, shaders (`material.rs`, `services/scene/mod.rs`)

| # | Blender | Phiber today | Status | Fix |
|---|---|---|---|---|
| 24 | Solid mode colour comes from `View3DShading.color_type`: MATERIAL uses `Material.r/g/b` (viewport display colour) — **not** the Principled Base Color; OBJECT uses `Object.color`; RANDOM hashes the name; SINGLE/VERTEX/TEXTURE as named. Default is MATERIAL. | `read_material_fields` **prefers the Principled "Base Color" socket over `Material.r/g/b`**. That is Material-Preview behaviour, not Solid-mode behaviour, so colours differ from what Blender shows on open (Blender opens in the file's saved shading mode, usually Solid). | WRONG | Read both; expose `viewport_color` (`Material.r,g,b,a`, `roughness`, `metallic`) and `base_color` separately; let the View3D shading mode pick. |
| 25 | Material Preview / Rendered: the full shader node tree is compiled to GLSL (`gpu/shaders/material/gpu_shader_material_*.glsl`, 103 node shaders; codegen in `gpu/intern/gpu_codegen.cc`, `gpu_node_graph.cc`) and drawn by EEVEE. | Only Principled `Base Color` (+ `roughness`/`metallic` scalars from the `Material` struct, not the node socket) and the first `TexImage` node's `Image.filepath`. No node graph is retained. | MISSING | Phase 5: read `bNodeTree.nodes/links` + `bNodeSocket.default_value` into a `MaterialGraph`; render a Principled-only fast path first (WGSL PBR), then port node shaders incrementally. |
| 26 | Images: `Image.filepath` (`//` relative to the blend), `ImagePackedFile` packed bytes, colour space (`Image.colorspace_settings`), UDIM tiles. | `image_texture_path` returns the path string; nothing loads, unpacks or samples it (README "Known limitations" confirms). | MISSING | Decode packed images from the `.blend` (`Image.packedfiles` list → `PackedFile.data`), resolve `//` against the file dir, upload as textures once the GPU path exists. |
| 27 | `World` (viewport background in Rendered mode, `World.horr/horg/horb` legacy, node tree for 4.x+), `Scene.r` (engine, `xsch/ysch/size`, `cfra/sfra/efra`, fps), `Scene.camera`. | `SceneModel.timeline/world/render` exist but `scene_from_blend` fills them with `Default::default()` (`import.rs` L50–52). The Timeline/Rendering/Shading hosts therefore always show factory values. | WRONG | Read `Scene.r.{cfra,sfra,efra,frs_sec,frs_sec_base,xsch,ysch,size,engine}` and `Scene.world` → colour; trivial with the existing offset helpers. |

### 4.5 UI state from the file ("Load UI")

| # | Blender | Phiber today | Status | Fix |
|---|---|---|---|---|
| 28 | With **Load UI** on (default), `wmWindowManager` → `wmWindow.workspace_hook` → active `WorkSpace`, `bScreen` → `ScrArea` (spacetype, v1..v4 verts) → `SpaceLink` list; `View3D.shading.type`, `RegionView3D.viewquat/dist/ofs/persp/is_persp`; `SpaceOutliner.treestore` expand flags; `SpaceProperties.mainb` tab; `Scene.r.cfra` for the timeline. | None of `WM`, `WS`, `SR` blocks are read. Phiber always opens in its own default Layout with its own camera. The user does not see "the same things Blender sees" — they see phiber's default layout with the file's meshes in it. | MISSING | Phase 6: read the active workspace name + its screen's area grid + each area's space type and map onto `BSpaceType`/`BHostType`/zone fractions (the v2 area/zone model already stores fractions); apply `RegionView3D` to the viewport camera and `View3D.shading` to the shading dropdown; apply outliner expand state. Offer the same `load_ui` toggle in the Open dialog. |
| 29 | Recent files list updated on successful open (`wm_history_file_update`), kept even on failure since 4.3 (see comment "#127825"). | `v2/modal/splash/recent.rs` keeps 10 slots in state; check that a v2 open pushes into it (Blender writes `recent-files.txt` in the config dir). | Partial | Persist to disk the way Blender does; update on every open. |

### 4.6 Progress, reporting, failure handling (`handle.rs`, `import.rs`, `statusbar/list.rs`)

| # | Blender | Phiber today | Status | Fix |
|---|---|---|---|---|
| 30 | Wait cursor + blocking read; **no progress bar for file open**; `CLOG` timing line on completion; report banner on problems. | Background thread + status-bar progress bar + stage label. **This is better than Blender** in principle — but the bar is fake: only `Reading (0.15)` is ever written before settle. | WRONG (fake progress) | Give `BlendImporter` a `&dyn Fn(f32, &str)` / channel progress sink (the `BlendImporter::import` asset path *already* takes one; `load_blend_scene` does not). Emit real fractions: bytes read / decompressed, blocks indexed, IDs read per type, meshes built, materials resolved, scene mapped — and forward via `cx.background_executor()` updates to `scene.load_progress`. Keep the bar (parity-plus), and also show the wait cursor over the viewport for exact Blender feel. |
| 31 | No timeout; a 2 GB production file simply takes as long as it takes. | 30 s watchdog → `TimedOut`, prior scene retained. | WRONG | Remove the timeout once #6/#7 land (a hashed remap makes a 178 MB file sub-second to index). Replace with a **Cancel** button on the progress bar (Blender's jobs have one). |
| 32 | `catch_unwind`-free; readers are bounds-safe by construction. | `catch_unwind` around the parser in two places (`blend.rs`, `load.rs`) hides bugs as `Malformed`. | Acceptable as a safety net | Keep, but log the panic payload + block code/offset so fuzz cases can be turned into fixtures. |
| 33 | Timing report: "Blender file read in 0m1.23s" (+ libraries/overrides). Counts of missing libs/IDs → banner. | No timing, no counts. | MISSING | Return a `BlendReadReport { duration, blocks, ids_by_type, missing_libraries, warnings }` from phiphx; print to the Info editor (`host/info`) and status bar the way Blender does. |
| 34 | Status bar: `Verts | Faces | Tris | Objects` (scene stats from `info_stats.cc`), memory, version. | Segments `Scene / Frame / Stats / Memory / Version` exist in `statusbar/set.rs`; they need to resolve from the loaded `SceneModel` + header version rather than placeholders. | Partial | Compute `SceneStats` in `scene_from_blend`; Version = `"{major}.{minor}.{patch}"` from `FileGlobal`/header. |
| 35 | Opening a file never writes next to it. | `import_blend_native` **writes `foo.phx` beside the user's `foo.blend` on every first open**, and later opens silently read the sidecar instead of the `.blend` (stale-cache risk if mtimes are odd, e.g. cloud sync). | WRONG (side effect) | Move the cache to the app's cache dir, key it by (path, size, mtime, header hash), and make it opt-in. Once #6/#7 land the cache is unnecessary for open; keep it only for the asset-import pipeline. |

### 4.7 Viewport rendering — "shaders"

| # | Blender | Phiber today | Status | Fix |
|---|---|---|---|---|
| 36 | Solid mode = **Workbench** engine: `draw/engines/workbench/shaders/` (34 files: `workbench_prepass_*`, `workbench_composite_*`, `workbench_effect_cavity/outline/dof/taa`, `workbench_shadow_*`, studio-light + matcap in `workbench_world_light_lib`). | CPU flat Lambert (`mesh/kernel.rs::lit`) painted as gpui paths; no depth buffer (painter's sort), no studio light, no matcap, no cavity, no outline, no shadow, no X-ray. | MISSING | Phase 5: GPU viewport via `gpui_wgpu` (already in the workspace) with a depth buffer. Port order: `workbench_prepass_vert/frag` → `workbench_composite_frag` (studio light: three-light rig from `StudioLight`, default "Default" studio light) → `workbench_effect_outline` → `workbench_effect_cavity` → matcaps (ship Blender's `release/datafiles/studiolights/matcap/*.exr`). |
| 37 | **Overlay** engine: `draw/engines/overlay/shaders/` (122 files): `overlay_grid_*` (infinite grid with fade, axes), `overlay_wireframe_*`, `overlay_outline_*` (selected object outline), `overlay_extra_*` (camera frustum, lights, empties, force fields), `overlay_armature_*`, `overlay_edit_mesh_*`. | Grid/axes painted as 2D lines in `bui/core/spatial.rs`; no selection outline; no extras. | MISSING | Port `overlay_grid_frag` first (it defines the look of an open file more than anything), then `overlay_extra` shapes for camera/light/empty, then `overlay_outline_*`. |
| 38 | Material Preview / Rendered = **EEVEE** (`draw/engines/eevee/shaders/`, 138 files) + material node codegen (`gpu/shaders/material/*.glsl`, 103 files). | None. | MISSING | Phase 6+: Principled BSDF WGSL first; port `gpu_shader_material_principled.glsl`, `_tex_image`, `_mix`, `_math`, `_color_ramp`, `_normal_map` as the most-used nodes; world lighting from `World` node tree or a default HDRI. |
| 39 | Shaders are GLSL with Blender's `gpu_shader_create_info` metadata (`*_info.hh`). | gpui renders via wgpu/blade (WGSL). | — | Translate GLSL→WGSL with `naga` at build time where possible (Blender's GLSL uses its own preprocessor macros and `#pragma BLENDER_REQUIRE`; expect hand-porting for the `_lib.glsl` includes). Keep each ported shader beside its Blender source path in a `phiphx::gpu::shaders` table so parity can be diffed. |

---

## 5. What is good and should be kept

- The **two-pass design** (block table + DNA, then semantic extraction with runtime-computed offsets) is exactly Blender's model and is why old and new files both partially work. Do not replace it with compiled-in structs.
- **Zstd/gzip sniffing**, bounds-safe readers, the 200-line module split, and the `README.md`/`flow.md` in `importer/blend/` are solid.
- **4.x/5.x attribute-storage mesh reading** (`position`, `.corner_vert`, `face_offset_indices`, UVs, loop normals) is correct and the hardest part to get right.
- The v2 shell — pure `should_import` / `settle_import`, off-thread load, error categorisation (`Inaccessible/Malformed/Unsupported`), retaining the prior scene on failure, the thread-local `scene_source` — is well-tested and a better UX than Blender's blocking read. Only its *inputs* (real progress, no timeout) need fixing.
- Outliner plumbing (`tree.rs`: expand/hide/sort/search state) is ready to receive a real collection tree.

## 6. What is wrong (highest impact first)

1. **Any file saved by Blender 5.0+ with default settings cannot be opened** (header v1 + `LargeBHead8`, #1, #2). Everything else is moot until this lands.
2. **Objects are placed wrong**: rotation and parenting are discarded (#13). A rigged/parented scene collapses.
3. **Outliner is flat** — the collection tree, view-layer flags and selection are dropped (#12, #17).
4. **Progress is fake and loads are killed at 30 s** (#30, #31); the cause is the O(n²) pointer remap (#6, #7).
5. **Solid-mode colours are wrong** (Principled Base Color instead of viewport display colour, #24) and **multi-material objects are single-colour** (#20).
6. **Opening writes a `.phx` next to the user's file** (#35).
7. **Wrong scene chosen** in multi-scene files (#8); names re-suffixed (#18).
8. The workspace `crates/phiphx` is an empty stub while the real reader lives inside `crates/phiber/lib/phiphx` via `#[path]` — so "phiber reads through phiphx" is true in name only, and nothing else in the workspace can use the reader.

## 7. Target architecture — make `crates/phiphx` the one reader

Move `crates/phiber/lib/phiphx/**` into `crates/phiphx/src/**`, add it to the workspace, depend on it from phiber (`phiphx = { workspace = true }`), delete the `#[path]` re-export in `src/lib.rs`. Then mirror `blenloader` inside phiphx so a Blender file path maps to a phiphx module:

```
crates/phiphx/src/
├── blend/
│   ├── header.rs        # BLO_core_blend_header: legacy + v1 decode          (#1)
│   ├── bhead.rs         # BHead4 / SmallBHead8 / LargeBHead8 → BHead        (#2)
│   ├── file.rs          # single buffer / mmap, block index, OldNewMap      (#6, #7)
│   ├── dna.rs, dna_layout.rs                                                (keep)
│   ├── global.rs        # FileGlobal (curscene, cur_view_layer, version)    (#8)
│   ├── quick.rs         # REND + TEST readers for the file browser          (#9)
│   ├── report.rs        # BlendReadReport: durations, counts, warnings      (#33)
│   ├── progress.rs      # BlendProgress sink trait (fraction, stage, cancel)(#30)
│   ├── id/              # one reader per ID type: scene, collection, view_layer,
│   │                    # object, mesh, material, node_tree, image, camera, light,
│   │                    # world, workspace, screen, view3d, outliner …
│   ├── eval/            # where_is_calc (transforms), collection instancing,
│   │                    # modifiers (Phase 4), polyfill triangulation
│   └── library.rs       # ID_LI, placeholders, //-relative resolution        (#10)
├── scene/               # SceneModel (+ collections, matrix_world, per-face
│                        # material indices, stats, ui state)
└── gpu/shaders/         # ported workbench / overlay / eevee (Phase 5+)
```

`BlendProgress` is the one new seam the v2 shell needs: `handle.rs` passes an `mpsc` sender wrapped in the trait; the receiver on the UI side updates `scene.load_progress` / `scene.load_stage` and the `Cancel` action sets an `AtomicBool` the reader polls between blocks.

## 8. Next steps — phased, with acceptance criteria (for the implementing agent)

**Phase 0 — Fixtures & harness (½ day).** Save `tests/fixtures/*.blend` from Blender 5.2 with default settings (v1 header, zstd) *and* with `write_legacy_blend_file_format` on, plus one 3.6 and one 2.79 file; include a multi-scene file, a parented + rotated hierarchy, a multi-material mesh, a collection-instance file, a subsurf'd cube, packed image texture. Add a `cargo test -p phiphx` golden test that dumps `{version, scene name, collection tree, object name→matrix_world, material slots}` and diffs against JSON exported from Blender via a tiny `bpy` script (`tools/dump_blend_truth.py`). Every later phase adds to this diff.

**Phase 1 — Open 5.x files, fast (2–3 days).** #1, #2, #6, #7, #8, #18, #35. Acceptance: all fixtures parse; a 178 MB file indexes in < 1 s and fully loads in < 5 s release; no file written beside the source; multi-scene fixture opens `curscene`.

**Phase 2 — Real progress, no timeout, reports (1 day).** #30, #31, #33, #34. Acceptance: bar advances monotonically through at least 8 real stages; Cancel returns to the prior scene within 100 ms; Info editor shows `Blender file read in 0m0.83s` and object/vert counts; Version segment shows `5.2.0`.

**Phase 3 — Scene parity (3–4 days).** #12, #13, #14 (camera/light/empty), #15, #17, #20, #21, #23, #24, #27. Acceptance: the truth-diff for matrices, collection tree, slot indices, selection and frame range is empty on every fixture; Outliner tree matches Blender screenshot for each fixture; Solid-mode colour of every object equals `Material.r/g/b`.

**Phase 4 — Evaluated geometry (1–2 weeks).** #16, #22. Mirror, Array, Solidify, Triangulate, Weld, Subdivision Surface (Catmull-Clark, matching Blender's `subdivision_type`/levels/`use_limit_surface` off) and collection instancing. Acceptance: vertex/face counts of the evaluated mesh match Blender's status bar for each fixture; visual diff of the viewport within tolerance.

**Phase 5 — GPU viewport with Blender's shaders (2–3 weeks).** #36, #37, #39. wgpu pipeline with depth; port `overlay_grid`, `workbench_prepass/composite` (studio light), `workbench_effect_outline`, `overlay_extra` for camera/light/empty. Keep a `SHADER_PARITY.md` table: Blender shader path → phiphx WGSL path → status.

**Phase 6 — Load UI + materials (2–3 weeks).** #25, #26, #28, #29, #38. Workspace/screen/area mapping, `RegionView3D` camera, shading mode, outliner expand state; Principled + Image Texture material path; packed image decode.

**Phase 7 — Libraries & overrides (later).** #10.

## 9. Verification checklist for every PR touching the reader

- `cargo test -p phiphx` truth-diff is empty for all fixtures.
- Open each fixture in Blender 5.2 and Phiber side by side: same active workspace (after Phase 6), same outliner tree, same object placement, same solid colours, same status-bar counts.
- Time the open of the largest fixture in both; Phiber must be faster and must never time out.
- `grep -rn 'blocks.iter().find' crates/phiphx/src` returns nothing (no linear pointer scans).
- No file is created or modified beside the opened `.blend`.
