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
> **Code under review (current paths, 2026-09-16):**
> - `crates/phiphx/src/importer/blend.rs` + `blend/*` — the native SDNA reader (own workspace crate; phiber re-exports it as `crate::phiphx`). Module map: `blend/README.md`.
> - `crates/phiphx/src/services/scene/{mod,load,import,collection,geometry}.rs` — `SceneModel` and the blend → scene mapping
> - `crates/phiber/src/v2/app/{handle,load,import,seed}.rs`, `src/v2/scene/{keys,source,shading,color}.rs` — the v2 open / progress / settle shell
> - `crates/phiber/src/v2/part/statusbar/list.rs`, `src/v2/part/custom/outliner/{tree,layer,keys}.rs`, `src/v2/view/render/gpui/atom/canvas/view3d/` — consumers
>
> *(Rows marked ✅ are fixed; §0 lists what changed per pass. §3 below describes the code as it was at audit time.)*

Reference Blender version in `REFS`: **5.2.0** (`BKE_blender_version.h`: `BLENDER_VERSION 502`, `BLENDER_FILE_MIN_VERSION 405`).

---

## 0. Status — 2026-09-16 (first fix pass)

Done in this pass (verified with `cargo test -p phiphx` on the moved crate: 71 tests green, incl. Blender-upstream `modeling/operators.blend` with the 5.x header):

- **Workspace cleanup**: 216 unused crate folders deleted from `phiace/crates` (213 Zed copies with mtime 2026-08-01 not in phiber's dependency closure, plus `blenda`, `phikit`, `buikit`); 199 dead `path = "crates/…"` lines pruned from the root `Cargo.toml` (`Cargo.toml.bak` kept beside it until you've built once). 35 crates remain: phiber, phiphx, phical, apple*, and the 28 gpui/ui/theme/util crates phiber actually pulls in.
- **`crates/phiphx` is now the real reader crate.** `crates/phiber/lib/phiphx` moved to `crates/phiphx/src` (all `crate::phx::`/`crate::phiphx::` paths rewritten); phiber depends on it and re-exports it as `crate::phiphx` / `crate::phx`, so no phiber call sites changed. gpui is behind the `bui` feature (on by default) so the reader tests anywhere with `--no-default-features`.
- **#1, #2 fixed** — v1 header (`BLENDER17-01v05xx`) + `BHead4`/`SmallBHead8`/`LargeBHead8` decoders ported from `blo_core_blend_header.cc` / `BLO_core_bhead.hh` (`importer/blend/parse.rs`, `types.rs`). Regression tests use Blender's own `tests/files/modeling/operators.blend`.
- **#6 fixed** — `OldNewMap` hash index (`BlendFile::block_at` / `block_at_code`); all 39 linear `blocks.iter().find(old_address == …)` scans replaced. LAND-TYPE-15-01 (178 MB, 756k verts) now opens in **0.38 s** release; it previously hit the 30 s watchdog.
- **#8 fixed** — `FileGlobal.curscene` from the GLOB block selects the scene (`scene.rs::file_global_curscene`); first-SC fallback only when GLOB is absent.
- **#18 fixed** — `ID.name` read DNA-driven (`id_name_offset`) and used verbatim; the `.001` re-suffix loop is gone.
- **#13 fixed** — `object_transforms.rs` rewritten as a port of `BKE_object_to_mat4` (loc/dloc, rot/drot per `rotmode` with the `eulO_to_mat3` order table, quat/dquat, axis-angle, size·dscale) plus `resolve_world_matrices` (world = parent.world · parentinv · local, PAROBJECT). `SceneObject`/`BObject3D::Loaded` carry `matrix_world`; the viewport transforms raw vertices by it and **no longer per-mesh-normalises** (`fit_vertices` was collapsing every object into a unit box). Verified: curve-parented camera in THYMUS-BOLDROID lands at a different world position than its `loc`.
- **New — #40 fixed**: the `Object.type` table was wrong (10→ARMATURE, 12→CAMERA, 13→LIGHT). Blender: 10=LIGHT, 11=CAMERA, 12=SPEAKER, 13=LIGHT_PROBE, 22=LATTICE, 25=ARMATURE, 26=GPENCIL, 27=CURVES, 28=POINTCLOUD, 29=VOLUME, 30=GREASEPENCIL. Cameras and lights now classify (THYMUS: 9 cameras, 4 lights).
- **#30, #31 fixed** — real progress: `load_blend_scene_with_progress(path, &|fraction, stage| -> bool)` drives stages "Reading file → Indexing blocks (n) → Reading objects (i/N name) → Building scene → Done"; `handle.rs` forwards them over an mpsc channel into `scene.load_progress`/`scene.load_stage` at ~30 Hz. The 30 s watchdog is gone; setting `scene.load_cancel = "true"` cancels (settles silently, prior scene kept — `SceneLoadError::Cancelled`). The statusbar bar still needs a Cancel button wired to that key.
- **#35 fixed** — opening a file writes nothing beside it; the `.phx` sidecar is only written by the asset-import path (`import_blend_native_with_sidecar`).
- Fixtures/examples/docs worth keeping moved from `gemphi/phi/crates/phiphx` (see `crates/phiphx/docs/from-phi/README-phiace.md`); the phi reader itself was the ancestor of this code, nothing to port back.

*(At the time of this pass `crates/phiber` was not yet compiled; it has been since — see §0.1 / §0.2.)*

### 0.1 Second pass — UI parity (file browser, welcome, top menus)

Verified in a clean checkout of the trimmed workspace: `cargo check -p phiber --tests` clean, `cargo test -p phiber` green except one **pre-existing** stale test (`viewport_layout_bug_conditions::c5_activity_list_partitions…` expects the old flat activity list; the rail replaced it — not touched here).

- **File / Asset browser = one implementation.** The modal dialog and the `File` / `Asset` hosts used two divergent sidebars (different headings, keys and accordion styles). Now `part/custom/file/{header,path_bar,sidebar,execute}.rs` are the only implementation; `modal/file_browser/*` re-exports them; `BFileItem` assembles them in Blender's `space_file` region order (UI path bar → TOOLS | WINDOW → EXECUTE). The redundant generic `Toolbar` Tools-region was removed from the `file`/`asset` hosts.
- **Sidebar = Blender's four `Bookmarks` panels** (`space_filebrowser.py` L221–358): Volumes, System, Bookmarks (with `+ Add Bookmark` → `file.bookmark_add`, ten state-backed slots `ctx.file.bookmark.<n>`), Recent (shared with the welcome list). Each is a rounded `Card` with the same accordion header the Properties editor uses, keyed `ctx.file.sidebar.<label>.expanded`. Asset mode shows Library (Current File / Essentials / All) + Catalogs (All / Unassigned), per Blender's asset tools region.
- **Header menus = Blender's**: Files → View (Source List, File Path, Frame Selected, Display Size, Recursion Level, Area), Select (All/None/Invert, Box Select); Assets adds Library (Refresh, Refresh Remote Listing) and Catalog (Undo, Redo, Save Catalogs, New Catalog). Same tables drive the host header (`menubar/set.rs` `FILE_HOST`/`ASSET`) and the modal.
- **Path bar** gained Blender's filter popover (`FILEBROWSER_PT_filter`: Show Hidden + the type flags) next to the filter toggle.
- **Welcome window** restructured to `WM_MT_splash` (`bl_operators/wm.py:3431`): banner (Phiber + version) → split (New File | Recent Files ×5 + More…, or Getting Started links when empty) → split (Open…, Recover Last Session | What's New, Support). Every block is one `Card`. Labels are Blender's ("Recent Files", "Getting Started"); branding is Phiber's.
- **Top menus open their windows.** `BMenuItem` carries declarative `purpose` (file-browser job) and `url` fields; `leaf_action` routes them. Open… / Save As… / Save Copy… / Link… / Append… / every Import & Export entry / Export All Collections / Save Screenshot… now open the file browser **with the right purpose** (the reducer raises the modal when `ctx.file.purpose` is set); Render ▸ Render Image / Animation / View Render / View Animation open a new **Render** window (`BModalType::Render` — the Image host in a window-sized card, as Blender's "Blender Render" window is an Image Editor); Help ▸ Support / Communities / Get Involved / Release Notes / Report a Bug and the Phiber ▸ Donate / What's New / Credits / License / Website items open the system browser (`ctx.link` → `cx.open_url`, Blender's `wm.url_open`); **About Phiber** finally has a body (`WM_MT_splash_about`: Version / Date / Hash / Branch / Windowing Environment + Release Notes · Credits · License · Website · Donate). Menu "Blender" → "Phiber".
- Still `op(...)` with no window: New ▸ templates, Open Recent ▸ list, Revert, Save (direct write), Recover ▸, Data Previews ▸, External Data ▸, Clean Up ▸, Defaults ▸, Quit, Edit ▸ Undo/Redo/Repeat/Rename/Batch Rename, Window ▸ New Window / Fullscreen / workspace cycling. These need reducer logic rather than a window.
- File-size rule: every new/rewritten file is ≤ 200 lines. Pre-existing offenders in the touched area: `part/menubar/set.rs` (~470), `phiphx/importer/blend/mesh_block.rs` (1211) — split candidates, not done here.

### 0.2 Third pass — Outliner, selection, materials, scene props, file-size rule

Verified: `cargo check -p phiber --tests` clean; `cargo test --no-fail-fast -p phiber -p phiphx` → **451 passed, 0 failed** (the stale `c5_activity_list…` test was rewritten against the Rail API it now uses).

- **#12 ✅ Outliner shows the file's collection tree.** phiphx reads the opened `ViewLayer` (`view_layer.rs`: `FileGlobal.cur_view_layer` → `object_bases`, `basact`, `layer_collections`); `SceneModel` gained `collections: Vec<SceneCollection>` (with `exclude` / `hide_viewport`), `selection` and `object_types`. `outliner/layer.rs` renders Scene Collection → child collections → objects → mesh data, recursive, collections before objects, sorted, search keeps ancestors. Cameras / lights / empties / curves get their own icons (`outliner/type_set.rs`). Default scene keeps the old flat path.
- **#17 ✅ Selection & visibility.** On settle, `app/seed.rs` makes the file's active object Phiber's active object and closes the eye (`hide_get`) on objects that are `BASE_HIDDEN` or live in an excluded / hidden collection — the viewport already honours that key.
- **#20 ✅ Multi-material meshes.** `material_index` is read from `face_data` (4.x/5.x) / `pdata`, carried through fan triangulation into `SceneObject.material_indices`.
- **#24 ✅ Solid-mode colour.** Material slots carry `viewport_color` (`Material.r/g/b/a`) and `base_color` (Principled). New `scene/shading.rs` (`ctx.view3d.shading`: wireframe / solid / material / rendered, default solid) picks which one the viewport paints (`scene/color.rs`). *The header dropdown does not write that key yet.*
- **#27 ✅ Scene properties.** `scene_props.rs` reads `Scene.r` (cfra/sfra/efra, frs_sec/frs_sec_base, xsch/ysch/size, engine) and `World.horr/g/b`; `scene_from_blend` fills `SceneTimeline` / `SceneRender` / `SceneWorld` instead of factory defaults. THYMUS fixture: frames 1–140, 1440×1440, CYCLES.
- **#14 ◐** object types are classified and shown with Blender icons; camera / light / empty viewport shapes are still not drawn, and non-mesh objects no longer have their data block scanned as a mesh (cameras used to report 123 "vertices").
- **Regression test on a real file:** `crates/phiber/tests/outliner_view_layer.rs` opens `crates/phiphx/tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend` and checks the tree, icons, hidden collection, active object and props; `examples/inspect_blend.rs` prints the same tree for any file.
- **≤ 200-line rule enforced on everything touched.** Split along Blender seams: `parse.rs` → `header.rs` + `bhead.rs`; `object_transforms.rs` → `rotation.rs` + `object_type.rs`; `scene.rs` → `collection.rs` + `names.rs` + `world.rs`; the 1,212-line `mesh_block.rs` → `mesh_offsets` / `mesh_block` / `mesh_positions` / `mesh_faces` / `customdata_float` / `mesh_legacy` / `mesh_uv` / `mesh_normals`; `material.rs` → `material_slots.rs` + `node_tree.rs`; `types.rs` → `scene_types.rs`; `blend.rs` → `asset_import.rs`; `services/scene/import.rs` → `geometry.rs` + `import_extras.rs` + `import_phx.rs` + `material.rs`; `app/handle.rs` → `app/load.rs`; `outliner/tree.rs` → `keys.rs`; `file/sidebar.rs` → `asset_sidebar.rs`; tests moved to `*_tests.rs`. Remaining >200 in the tree are pre-existing files outside this work (e.g. `part/menubar/set.rs`, `outliner/set.rs`, `outliner/item.rs`).
- **Icons:** 12 icon names used by the new file-browser / welcome code did not exist in `assets/icons`; all now map to real SVGs.
- **Docs updated:** `modals.md` (8 modals, triggers, §7–10), `spaces/modals/README.md`, new `spaces/modals/welcome.md` and `spaces/modals/file_browser.md`, `MANUAL.md` (§11 modal table, §16 run-on-Windows checklist now all done, cross-refs), `spaces/layout/tasks.md` (marked historical), `phiphx/src/importer/README.md`, `phiphx/src/importer/blend/README.md` (module map + status), `phiphx/docs/from-phi/BLENDER_API_COVERAGE.md` (historical banner).

### 0.3 Fourth pass — overlay extras, shading modes, smooth shading, triangulation

Verified: `cargo test --no-fail-fast -p phiber -p phiphx` → **461 passed, 0 failed** (incl. the repo's own `renderer_decomposition` architecture test).

- **Shading radio in the View3D header** (`part/modebar/shading.rs`, Blender `VIEW3D_HT_header` `shading.type`): Wireframe · Solid · Material Preview · Rendered write `ctx.view3d.shading`. Solid paints `Material.r/g/b`, Material/Rendered paint the Principled Base Color, **Wireframe strokes triangle edges** instead of filling.
- **#14 ✅ Cameras, lights, empties.** phiphx reads their DNA (`object_data.rs`: `Camera.type/lens/ortho_scale/sensor_x/clip_*/drawsize`, `Light.type/r,g,b/energy/spotsize/radius/area_size`, `Object.empty_drawtype/empty_drawsize`). `SceneCamera` / `SceneLight` gained world matrices and those fields; new `SceneEmpty` + `SceneModel.empties`. `scene/extras` builds Blender's overlay wire shapes — camera frame per `BKE_camera_view_frame_ex` (perspective pyramid or ortho box + up triangle, aspect from the scene resolution), light ring + sun ray / spot cone / area square, empties from `overlay_shape.cc` (plain axes, arrows, single arrow, cube, circle in XZ, sphere, cone). `canvas/view3d/extras.rs` projects and strokes them; hidden objects are skipped. THYMUS: 5 ortho + 4 perspective cameras, 2 spot / 1 area / 1 point light with their real energies.
- **#21 ✅ Smooth shading.** Per-face smooth flag from `sharp_face` (3.5+; absent ⇒ smooth, as Blender), or the legacy `MPoly.flag & ME_SMOOTH`. Smooth triangles take corner-angle-weighted vertex normals (`services/scene/geometry.rs`); flat ones keep the face normal; custom split normals still win.
- **Lighting fix:** loaded normals were object-local and Z-up but lit as if world / Y-up; the painter now rotates them by `matrix_world` and remaps axes, so rotated and parented objects shade correctly.
- **#23 ✅ Triangulation** follows `mesh_calc_tessellation_for_face` (`services/scene/polyfill.rs`): quads (0,1,2)+(0,2,3), n-gons ear-clipped on the Newell-normal dominant plane. Concave L-shape test covers the area exactly once. (The asset-pipeline `.phx` exporter still fans.)
- **New finding — #41 ✅ `CustomData.totlayer` offset.** Six readers assumed `totlayer` follows the `layers` pointer; Blender puts `typemap[]` in between. Layer loops only worked when `typemap[0]` happened to be large. Now DNA-driven (`dna_layout::customdata_totlayer_off`) and loops stop at the end of the written layer block. Side effect: UVs are actually read now (LAND-TYPE-15-01: 2.8 M corners) — open time unchanged at 0.41 s.
- **New finding — #42 ✅ Blender ≤ 3.x names the light struct `Lamp`** in DNA; lights in older files read as factory defaults until the fallback was added.
- **New finding — #43 ✅ Legacy `MPoly` layer.** Blender 3.x still writes faces as `CD_MPOLY` (type 25) for backward compatibility; `material_index` and smooth flags come from it when the named attributes are absent. Per-face slots now work on THYMUS (7 multi-material meshes). The int-layer lookup also no longer accepts an arbitrary layer by type when the name doesn't match.
- New tests: `tests/viewport_extras.rs` (camera / light / empty DNA, overlay lines, face slots, shading radio), `scene/extras/tests.rs` (frustum depth per Blender formula, circle plane, display size), `polyfill.rs` tests.

### 0.4 Fifth pass — file-browser thumbnails, read report, Info log, Cancel

Verified: `cargo test --no-fail-fast -p phiber -p phiphx` → **466 passed, 0 failed**.

- **#9 ✅ `TEST` thumbnail + `REND` quick reader** (`phiphx/importer/blend/quick.rs`, Blender `BLO_thumbnail_from_file` / `BLO_read_render_info`). `BlendFile::read_quick_info(path)` streams raw / gzip / zstd, reads `REND` (`sfra, efra, scene_name`) and `TEST` (`w, h`, bottom-up RGBA → flipped top-down), and stops at the first other block — ~0.1 ms warm even on a 178 MB file, no DNA parse. `examples/peek_blend.rs` dumps it.
- **File browser Thumbnails display** (`part/custom/file/thumbs.rs`, `preview.rs`): new `BView::Thumbnail` / `BView::Grid` primitives (`view/prim/thumb.rs`, gpui `atom/thumb.rs` with an RGBA→BGRA `RenderImage` cache). `.blend` tiles show the embedded preview and a subtitle like `Scene · 1–140 · 3.6`; previews are cached by (path, mtime). Test: `tests/file_thumbnails.rs`.
- **#33 ✅ Read report** (`phiphx/services/scene/report.rs`): `SceneReadReport { scene, file_version, duration_ms, objects, verts, faces, tris, cameras, lights, empties, warnings }`. `settle_import` → `app/seed.rs::report_loaded` writes Blender-style lines to the **Info editor** (`part/custom/text/info.rs`, 32-entry ring: `Read blend: "…"`, `Blender file read in 0m0.41s`, stats, warnings) and sets status `Scene` / `Frame` / `Stats` (`Verts:… | Faces:… | Tris:… | Objects:…`). Failures log as `ERROR`.
- **#34 partial → mostly ✅**: Stats and Version resolve from the loaded file; the Memory segment is still a placeholder.
- **Cancel** ✅ — an `x` button next to the status-bar progress bar sets the load's cancel key; the prior scene stays.

### 0.5 Sixth pass — Load UI (#28)

Verified: `cargo test --no-fail-fast -p phiber -p phiphx` → **473 passed, 0 failed**.

- **Reader** (`phiphx/importer/blend/ui_state.rs`): `wmWindowManager.windows` → `wmWindow.workspace_hook` → `WorkSpaceInstanceHook.active` (workspace name) / `.act_layout` → `WorkSpaceLayout.screen`; falls back to `FileGlobal.curscreen`. On that screen the **largest** `SPACE_VIEW3D` area wins; its `RGN_TYPE_WINDOW` region gives `RegionView3D.viewinv / ofs / dist / persp`, and its `View3D.shading.type` gives the saved shading mode. Stored as `BlendSceneProps.ui` plus `Scene.camera`.
- **Reduction** (`phiphx/services/scene/ui.rs`): `SceneModel.ui = SceneUi { workspace, view: SceneView { yaw, pitch, perspective, camera_view, pivot, width, shading } }`. Yaw and pitch come from the view's back axis `viewinv[2]`. With `persp == CAMOB` the scene camera's matrix is used instead, and an ortho camera sets `width = ortho_scale`. Otherwise `width = dist × 36·2/50`, Blender's viewport sensor × initial zoom over the default lens.
- **Apply** (`phiber/app/load_ui.rs`, called from `settle_import`):
  - Sets the workspace tab (matched by slug or by label, so "UV Editing" maps to `UvEditing`).
  - Sets `view3d.yaw / pitch / zoom / perspective / shading`.
  - Sets the pan so the saved pivot sits at the viewport centre.
- **Open dialog option** (`part/custom/file/options.rs`): a declarative options table. For Open it adds a **Load UI** checkbox panel (`WM_OT_open_mainfile.load_ui`, default on; key `file.load_ui`). With the box off, the current layout is kept.
- **Checked against real files:**
  - THYMUS opens in **Layout**, level, through its front camera.
  - Blender's `tests/files/modeling/operators.blend` opens in **Scripting** at yaw 103° / pitch 45°. Its `viewinv` translation equals `−ofs + back·dist`, which confirms the matrix layout.
- **Still open under #28:** the saved area layout (zone sizes and each area's editor type), outliner expand state, and the Properties tab. The viewport's own projection (`project3_full`) is an oblique approximation: its depth sort and on-screen vertical direction disagree about which side is near. Restored angles match Blender's direction, but the picture only matches exactly once Phase 5 draws with `viewinv` directly (new finding **#44**).

### 0.6 Seventh pass — image textures (#26) and a Blender ground-truth sweep

Verified: `cargo test --no-fail-fast -p phiber -p phiphx` → **479 passed, 0 failed**. The new `phiphx/tests/blender_truth.rs` compares against **Blender 5.0.1 itself**, run headless here via `pip install bpy`. Every mesh in the scene each file opens with — **1,276 meshes across 8 files** from Blender 3.4, 3.6, 4.2, 5.0 and 5.2, including the production files LAND-TYPE-15-01 and Mannequins-Female--Pose-05 — matches Blender exactly. The checks cover vertex and face counts plus sums of positions, face indices, active-UV values, material indices and smooth flags.

- **#26 ✅ Image textures in Material Preview.**
  - `importer/blend/image.rs` reads each image's key (`Image.filepath`, stored in DNA as `name[1024]`, or the image name) and the packed bytes (`Image.packedfiles` → `ImagePackedFile.packedfile` → `PackedFile.size/data`; legacy `Image.packedfile`).
  - `services/scene/texture.rs` decodes them once per load, trying Blender's order: packed bytes, then `//`-relative to the `.blend`, then the bare file name next to it. Images are reduced to ≤ 256 px.
  - Result: `SceneMaterial.texture`. Missing images are listed in the Info log as `Image "…" not found`.
  - Material Preview / Rendered sample the texture at each triangle's UV centroid (sRGB → linear; nearest pixel with wrap-around). Solid shading keeps `Material.r/g/b`.
  - New fixture `TEXTURED-PACKED.blend`, saved by Blender 5.0.1: a plane with a packed 2×2 image and a cube using `//tex/magenta.png`.
  - Limit: one colour per triangle. Real per-pixel sampling comes with the GPU path (Phase 5).
- **Reader bugs the sweep found and fixed:**
  - **#45 Node trees were never read.** The exact-name lookup asked for `idname` / `name`, but DNA spells them `idname[64]` / `name[64]`. The Principled Base Color and Image Texture lookups silently fell back to defaults. A bare name now also matches its array spelling.
  - **#46 Pointer arrays were sized as one pointer.** `*gputexture[3][2]` counted as 8 bytes instead of 48, which shifted every later field of `Image` and any struct like it. New test: every DNA struct's summed field sizes must equal its declared size in all fixtures.
  - **#47 Blender 5.x meshes read as garbage.**
    - 5.x writes geometry to `Mesh.attribute_storage` and leaves `CustomData` empty. New `attr_storage.rs` reads `Attribute { name, data_type, domain, storage_type, data }`. It handles array storage and the **single-value** storage 5.2 uses for constant attributes: position, `.corner_vert`, UV maps (the active one via `active_uv_map_attribute`), `material_index`, `sharp_face`.
    - The face-offset pointer keeps its 3.6 DNA spelling `*poly_offset_indices`.
  - **#48 Repeated saved addresses.**
    - Blender 5.x writes generated pointers, and 3.6 writes temporary compatibility arrays whose addresses get reused, so one address can appear in several IDs' data.
    - New `scope.rs` mirrors Blender's per-ID data map: inside `scoped(owner, …)` a repeated address resolves to the first copy written after that ID. Mesh, material, image and smooth-flag reads are scoped.
  - **#49 Invented or dropped vertices.**
    - Removed the byte-pattern scanner, which invented ~430 vertices for empty meshes.
    - Removed the "at least 3 vertices" and range filters, which dropped 2-vertex widget meshes.
    - Fixed the `MVert` stride guess (a 4-vertex mesh read as 3). The stride now comes from the block's own element count.
  - **#50 Wrong UV layer and stride.**
    - The old reader accepted any layer whose name contained "uv", including the `.vs.` / `.pn.` selection layers. It also read `CD_MLOOPUV` (type 16, 12-byte `MLoopUV`) as 8-byte float pairs.
    - It now follows `CustomData_get_active_layer_index`: `CD_PROP_FLOAT2` (49) first, then `CD_MLOOPUV`, using the active offset.
- **Speed, same machine, warm, release build:** LAND-TYPE-15-01 opens in **0.52 s vs Blender 2.3 s**; Mannequins-Female--Pose-05 in **0.47 s vs Blender 7.0 s**. LAND was 0.41 s before this pass; the extra time comes from now reading UVs and smooth flags correctly.

### 0.7 Eighth pass — the 3D viewport shows what Blender shows (#44, #16)

Verified: `cargo test --no-fail-fast -p phiber -p phiphx` → **488 passed, 0 failed**. The new `phiber/tests/viewport_vs_blender.rs` opens a file end to end (reader → modifier stack → Load UI → camera), fills every triangle and compares the result with **Blender 5.0.1's own render of the same saved view** (`phiphx/tests/truth/views/*.png`, made by `tests/truth/blender_view_mask.py`). Silhouette overlap (IoU):

| File | Saved view | Before this pass | Now |
|---|---|---|---|
| `operators.blend` (Blender 5.2) | perspective user view, Scripting | — | **1.000** |
| THYMUS (3.6) | through the scene camera; Subdivision + Armature | 0.705 | **0.999** |
| FLORIS (3.6) | perspective user view; Mirror + Subdivision + Armature | 0.902 (mirror missing) | **1.000** |

- **#44 ✅ Real viewport camera** (`phiber/scene/camera.rs`).
  - The old oblique projection is gone. The view now orbits a **pivot** along Blender's back axis: yaw 0 / pitch 0 is Front (numpad 1) and pitch 90 is Top. It is orthographic, or perspective with Blender's viewport lens: visible width = `dist × 36 mm × 2 / View3D.lens`, per `BKE_camera_params_from_view3d`.
  - Canvas aspect is applied so pixels stay square.
  - New state keys in `state/key.rs`: `K_VIEW_PIVOT` and `K_VIEW_FOV`.
  - Load UI sets the pivot and lens. In camera view, the eye is the camera itself with the camera's lens.
  - Zoom range widened to 0.02–50, because real views saved in files fall outside the old 0.4–3.
- **Floor grid in 3D** (`view3d/floor.rs`): Blender's Z = 0 grid and red X / green Y axes, with powers-of-ten spacing. They replace the flat 2D guide lines.
  - Removed the two decorative camera/light icons that sat at fixed screen spots; real camera and light wires are already drawn in 3D.
- **Studio lighting follows the view**, like Blender's Solid mode, and is two-sided.
- **#16 ✅ (first three) Modifier stack** (`phiphx/importer/blend/modifier/`).
  - Declarative: `KINDS` maps `eModifierType` → reader → `BModifier` trait object; the stack runs in file order on a `BlendEvalMesh`. Only modifiers enabled for the viewport (`eModifierMode_Realtime`) run.
  - **Armature** (`armature_deform.cc`): vertex-group linear blend skinning with `pose_mat · arm_mat⁻¹`. Blender saves the evaluated `pose_mat`, so the saved pose shows without evaluating animation. Weights come from `CD_MDEFORMVERT` or legacy `Mesh.dvert` (`vgroups.rs`); group names from `Mesh.vertex_group_names` or `Object.defbase`.
  - **Subdivision Surface**: Catmull-Clark or Simple at the viewport `levels`. Boundaries are smoothed, or corners kept when set. Vertices are projected to the limit surface unless "Use Limit Surface" is off. Vertex-group weights are carried through, so Subdivision → Armature stacks (the most common in these rigs) still deform.
  - **Mirror** (`mesh_mirror.cc`): X/Y/Z about the origin or a mirror object; copies within tolerance are merged; winding is reversed; `.L`/`.R` groups are flipped (`flip_side`); UVs are optionally flipped and offset.
  - The rest mesh stays untouched for the mesh ground-truth test; the viewport draws `BlendSceneProps.evaluated`. Evaluated meshes recompute their normals, since saved split normals describe the rest pose.
  - Speed: flat edge topology (sorted corners, no hashing), and objects evaluated in parallel (rayon). Mesh building (triangulation and normals) is also parallel now.
  - New progress stage: "Evaluating modifiers".
  - Open times, 2-core cloud machine (release, warm), now including evaluated modifiers: LAND-TYPE-15-01 1.7 s, 3.7 M verts. Mannequins 2.6 s, 3.4 M verts (Blender: 7.0 s just to load, before evaluating).
- **Sortable headers** (`prim/sort.rs`): the label sits left, the arrow is pinned to the far end, and the arrow slot is always reserved so headers line up with their columns.

**Next up (in order):** see §0.9.

### 0.8 Ninth pass — window chrome, activity bar, modals, playback; renderer and USD survey

Verified: `cargo test --no-fail-fast -p phiber -p phiphx` → **493 passed, 0 failed** (new: `tests/chrome_wiring.rs`, timeline step test).

- **Top bar follows the theme.**
  - The window opened with gpui's default title bar, which on Windows is the native white one. It now opens with `appears_transparent` (title "Phiber") and client decorations. The Topbar zone is the title bar: `WindowControlArea::Drag`, so empty space drags the window.
  - New declared parts `CaptionLead` / `CaptionTrail` (`part/caption/`, renderer `atom/caption.rs`). On Windows and Linux they are minimize / maximize / close, with Windows snap layouts handled by the OS. On macOS they reserve space for the traffic lights.
  - Vendored gpui change (`window.rs`): a window-control area counts only when it is the **topmost** hitbox under the mouse, so menus and tabs inside the drag area stay clickable.
  - Not yet seen on your Windows machine; check dragging, double-click to maximize, and the three buttons.
- **Activity bar wired.** Every entry records its tab, and a declarative reducer rule opens its destination:
  - Home → Welcome.
  - Spaces → new **Spaces** picker, built from the same set as the workspace tabs; picking one closes it.
  - Chat and Stream → new **Collaboration** modal. It shows the real `collab.status` / `collab.server` / `collab.peer.<n>` state, and says "offline, no collaboration server configured" until a service writes those keys. It invents nothing.
  - Settings and Profile → as before.
- **Preferences resolved.** The Settings modal showed only its title. It now *is* the Preferences host, elevated: one implementation for the workspace editor and the dialog. **Help** lists the web links (open in the browser). **Profile** shows its fields ("Not signed in" until sign-in exists). **Command palette** is Blender's F3 Menu Search: it walks every Topbar menu leaf recursively and fires the same action as the menu item. Typing into the field still needs the text-input entity (task 16).
- **State.**
  - New `state/follow.rs`: a table of "when key X is set, also do Y" rules, which is where cross-key behaviour lives now.
  - New keys in `state/key.rs`: `K_LINK`, `K_COMMAND_QUERY`, `K_COLLAB_*`, `K_FRAME_STEP`, `K_FRAME_START / END`, `K_FPS`, plus `int_or` / `float_or` readers.
- **Playback.** The timeline used private `time.*` keys and Play only flipped a flag. It now uses the global `K_FRAME` / `K_PLAYING`. Step / start / end go through `K_FRAME_STEP`, wrapping inside the file's frame range (set on load). A ticker (`app/playback.rs`) advances the frame at the file's fps while playing.
- **Menus.** Every dropdown already goes through the one `BView::popover` primitive (menubar, sub-menus, file header, filter, mode, switcher, property enums). Remaining differences are in the *rows*: filter popovers use buttons with check icons, while menus use `MenuItem` rows (task 18).

#### Apple crate and phiray (survey)

- `crates/apple` (in phiace) is a large toolkit that mirrors Apple's frameworks. It contains appkit (NSView-style UI, a **wgpu renderer** with G-buffer, deferred lighting, shadows, occlusion, post-process and EXR loading), puikit / swiftui (declarative UI), uikit, RealityKit / reality-foundation (**USD / USDZ loading**), iosar / xrosar (AR, RoomPlan), coreml, gameplay_kit and vision_kit. It also has its **own copies** of `phiphx` and `buikit` under `src/`, which duplicate `crates/phiphx` and phiber.
- `apps/phiray` is the Blender-Game-Engine-style runtime/app: a Blender-like shell, transport playback and movie export via ffmpeg. Its README and session notes describe `runtime/`, `anim/`, `render/`, `physics/`, `logic/` and `convert/` as "relocated to the phi_phx crate", but **those modules exist in neither phiace nor phi**. The animation, skinning and conversion code it depends on is missing, so the example cannot build as described. What is real and reusable is the appkit wgpu renderer.
- **Three renderers** (Blender's Workbench / EEVEE / Cycles):
  1. **Solid** — phiber's canvas (camera, modifiers and Load UI now match Blender; §0.7). Moving it to the GPU gives a depth buffer and per-pixel textures.
  2. **Material Preview** — the appkit wgpu deferred PBR renderer (phiray), fed from `SceneModel`. gpui already runs on wgpu (`gpui_wgpu`), so it can render into a texture shown in the View3D area.
  3. **Rendered** — a GPU path tracer: WebGPU compute over a BVH (phiray already has `Bvh` / `nearest_hit`). The same WGSL runs natively and in the browser.
  4. The same `viewport_vs_blender` masks then check all three.

#### OpenUSD / fast web rendering (research)

- **NVIDIA:** the OpenUSD tooling is Omniverse (Kit, RTX renderer, streaming), which is not an embeddable web library. **Warp** is a Python GPU simulation framework (Newton physics builds on it), not a renderer. Useful references: `NVIDIA-Omniverse/OpenUSD-Code-Samples`, `LearnOpenUSD`.
- **Pure Rust USD:** `mxpv/openusd` (MIT). It reads and writes usda / usdc / usdz, has a full composition engine (LIVRPS, variants, instancing) and schema features (geom, lux, shade, skel, physics), with **no C++ dependencies**. It is the best fit for phiphx's USD path and works in WASM. Also `FloatyMonkey/openusd-rs` (work in progress) and `pxr_rs` (bindings to Pixar's C++).
- **Web:** Autodesk's WASM OpenUSD bindings plus `needle-tools/usd-viewer`, a three.js Hydra delegate over WebGL / WebGPU with OpenSubdiv and MaterialX. The viewer is **PolyForm Noncommercial**, so it is reference only unless licensed. Pixar's Hydra Storm now has a Vulkan backend. The Khronos talk "USD and MaterialX on the Web" covers the WebGPU approach.
- **WebGPU path tracing references:** `gnikoloff/webgpu-raytracer`, `re-ovo/wgpu-path-tracing`, `pema99/rust-path-tracer` (all Rust or WGSL).
- **Recommendation:** read USD with `openusd` in phiphx, keep one `SceneModel`, and render with our wgpu passes. Native and web (`gpui_web`) then share code, with no C++ in the web build.

### 0.11 Tenth pass — the rest of the modifier list, and layout rules that live in one place

Verified: `cargo test -p phiber` → **all suites green**; `cargo test -p phiphx` green except the six tests that read Blender's upstream `tests/files/*.blend`, which are not present in this checkout (pre-existing).

**Modifiers (§0.9 item 2).** `KINDS` now has five more rows, each with unit tests over a plain quad (`blend/modifier/tests/stack.rs`, `tests/merge.rs`):

- **Array** (`MOD_array.cc`, type 12) — `count` copies displaced by the sum of the enabled offsets: a constant vector, a fraction of the mesh's own bounding box, and/or another object's transform relative to this one. UVs, material slots, smooth flags and vertex weights travel with each copy. Not yet: start/end caps, merge, fit-to-curve and fit-to-length.
- **Triangulate** (`MOD_triangulate.cc`, type 44) — faces of `min_vertices` or more corners split through `scene::polyfill::triangulate`, the same routine the renderer already uses for n-gons, so a triangulated mesh and an untriangulated one shade identically. `polyfill` was promoted to `pub(crate)` for this rather than copied.
- **Solidify**, simple mode (`MOD_solidify_extrude.cc`, type 33) — an inner shell offset along the vertex normals plus a rim closing the open boundary; `offset_fac` slides the pair (−1 inside, +1 outside, 0 centred) and `MOD_SOLIDIFY_FLIP` reverses the inner winding. Not yet: complex (non-manifold) mode, per-vertex-group thickness, rim material offsets.
- **Displace**, textureless (`MOD_displace.cc`, type 14) — every vertex moves `strength × (1 − midlevel)` along X / Y / Z or the vertex normal, which is what Blender shows for a Displace with no texture assigned. `RGB_XYZ` needs a texture, so it is a no-op here exactly as it is in Blender.
- **Weld** (`MOD_weld.cc`, type 55) — vertices closer than `merge_dist` merge through a union-find, so the result is order-independent like Blender's. `ALL` mode buckets candidates in a grid of the merge distance; `CONNECTED` only considers pairs that already share an edge. Collapsed corners and faces left with fewer than three are dropped.

New `blend/modifier/geom.rs` holds the area-weighted vertex normals and the open-boundary walk that Solidify and Displace both need, instead of a copy in each.

Still open on this item: Bevel (display), Array caps / merge / fit modes, Solidify complex mode, Displace with a texture, and subdivision creases / UV smoothing.

**Layout rules moved to one place.** Six reported defects all had the same shape — a rule stated per site, so the sites drifted:

- **A canvas rounds its own corners.** GPUI's content mask is a plain rectangle, so a rounded parent cannot clip a square child; only the 3D viewport painted its arcs, so the node editors, the image editor and the fallback canvas squared off the zone they sat in. `atom/canvas/surface.rs` is now the one opaque base every canvas starts from, and `view3d/corner.rs::round_surface` delegates to it.
- **A side area scrolls only when its host stacks content.** `BAreaType::is_scrollable` said Left and Right, regardless of host, so an editor canvas in a side column took its natural height and left a dead band under it (the gap below the Image Editor in Texture Paint). The area walker now also asks `host.fills_content()`.
- **Preferences declares its categories as a NavBar region**, the same `RGN_TYPE_NAV_BAR` Properties uses and Blender's `space_userpref.cc` declares. It had them as a Left *pane* of the Center region, which is a different treatment: no rail fill, Row axis, content width.
- **A menu takes its submenus with it when it closes.** `state/menu.rs` set only the menu's own key to false, so a submenu left open stayed `V_TRUE` and its popover reappeared stacked over the next menu opened. Closing is now a subtree operation (`menu::close`).
- **Menu rows are strip controls, not pills.** They were painted through the *floating* chrome path, which gives a resting fill; they are hover-only now, and every popover frames its rows with one gutter on all four sides so a fill stops short of the panel's border radius — the same rule the toolbar and sidebar follow.
- **One window-control component.** The OS caption and the modal title bar each drew their own minimise / maximise / close with different icons, sizes, radii and hover. `atom/caption.rs` now owns one `WINDOW_CONTROLS` table and one `control()` treatment; the modal title bar builds from it and attaches its own actions.
- **An expanded rail entry fills its rail.** A labelled cap had no width, so expanding the toolbar or sidebar left small centred pills in a wide strip. `BCapSide::in_rail()` distinguishes a rail cap from a header cap; rail caps stretch across the rail, lead with their icon and keep one height whether or not the label shows.

Tests: `crates/phiber/tests/central_reuse.rs` pins each of these; `chrome_geometry.rs`'s menu test now pins the gutter rule instead of the edge-to-edge one it replaced.

### 0.9 Outstanding tasks

Ordered. "3D" items first, as requested. Each is sized for a quick turnaround with no sculpting or modelling. Every sub-module must stay declarative: a static table of sets, a trait per behaviour, recursion over the tree, and at most 200 substantive lines per file.

**3D (continue)**

1. **Progressive open.** Show the rest meshes as soon as they are read, then swap in evaluated meshes when the modifier stack finishes, the way Blender draws while the depsgraph evaluates. Removes most of the wait on big files.
2. **More modifiers** — Array, Triangulate, Solidify, Displace and Weld landed in §0.11. Remaining: Bevel (display); Array caps / merge / fit modes; Solidify complex mode; Displace with a texture; subdivision **creases** (`crease_edge` / `crease_vert`) and UV smoothing.
3. **Armature completeness:** envelopes (`ARM_DEF_ENVELOPE`), dual quaternions, B-Bone segments, multi-modifier, and parent type `PARSKEL`.
4. **Unsupported-modifier report:** list each object's skipped modifiers in the Info log (names from `supported_modifiers()`).
5. **Viewport interaction on the real camera:** drag orbits yaw *and* pitch around the pivot; Shift-drag pans the pivot; wheel dollies; numpad views (1/3/7, Ctrl for opposite); frame-selected.
6. **Phase 5 GPU path** (was §8): per-pixel textures, Workbench / Overlay shaders, depth buffer instead of painter's sorting.
7. **#25** keep the full material node graph; **#28 remainder** (area grid, outliner expand state, Properties tab).

**Loading window (status zone)**

8. While a file loads, the **status bar zone expands** into a full-width panel: a progress bar spanning the whole bottom edge, the current stage (Reading file → Indexing blocks → Reading objects n/N → Evaluating modifiers → Building scene), a short running log of finished stages with timings (Blender's report lines), and Cancel. When done, the panel **collapses back** into the normal status bar and the last report line stays visible. Declarative: the stage list comes from phiphx's progress labels, and the panel is a `BViewList` over them.

**State**

9. **Resume from the last state.** Persist `BState`, meaning everything except transient keys (progress, hover, open menus, modal), to a user-config file on change (debounced). Restore it on launch, including reopening the last file with Load UI off, so the saved session wins over the file's UI.
10. **A handful of verbs.** Reduce `BState`'s public surface to `get`, `has`, `set` (plus `remove`): fold `set_encoded`, `get_encoded` and `iter_keys` into those, and route `host()` scoping through `BPath`. Everything else stays in the `apply_action` reducer.
11. **Keys belong to state.** Move every key constructor out of views and parts into `state/key/` as declarative tables, one module per domain: scene / view3d, file, outliner, info, menus, zones. Views only read keys. Today 23 non-state files build `BPath`s; `scene/keys.rs`, `part/custom/file/keys.rs` and `part/custom/outliner/keys.rs` are the big ones. Add a test that fails when a `BPath` literal appears outside `state/`.
12. **Everything configured declaratively.** Audit views, stores and sets against the trait + table + recursion pattern (the modifier `KINDS` table and the file-browser options table are the model). Replace ad-hoc builders.

**Hosts, headers, modals**

13. **File and Asset browser as reusable hosts.** Any area can switch to them. They are resizable like other areas and share one state per host instance (not one global `file.*`).
14. **Sortable headers** ✅ spacing and arrow placement (this pass). Remaining: make end-aligned numeric columns (Size) put their label flush with the numbers, with the arrow still last.
15. **Modals are spaces, elevated.** (Settings done this pass: it renders the Preferences host.) A modal renders the same Space → Zone → Area → Host tree as a workspace, inside an elevated, rounded card with a shadow. Reuse the area and host code rather than bespoke modal layouts (file browser, preferences, render, about).

**Added this pass**

16. **Text input entity** for search and text fields (command palette, outliner and file filters, profile). This is gpui's `EntityInputHandler`; today fields only clear on click.
17. **Animation, staged and each step checked against Blender at several frames** (`phiphx/src/blend/anim/`, truth `tests/truth/thymus_anim.txt` from `truth/blender_anim.py` at frames 1 / 20 / 40 / 66 / 100 / 132):
    (a) ✅ object keyframes — `read.rs` reads layered actions (4.4+/5.x: `layer_array → strip_array → strip_keyframe_data_array → channelbag_array → fcurve_array`, curves grouped by `slot_handle`) and legacy `bAction.curves`; `Object.adt → AnimData.action / slot_handle` binds an object to its slot. `fcurve.rs` evaluates constant / linear / Bézier segments (`BKE_fcurve_correct_bezpart`, root of the x cubic, y at that t) with constant extrapolation. `channels.rs` writes `location / rotation_euler / rotation_quaternion / rotation_axis_angle / scale` over the same `ObjectTransformRaw → BKE_object_to_mat4` the object reader uses. The metarig's world matrix matches Blender at every frame.
    (b) ◐ pose-bone keyframes — `pose.rs` reads every `bPoseChannel` (name, parent, `Bone.arm_mat`, saved loc/rot/scale/rotmode) and builds `matrix_basis` per bone (`BKE_pchan_to_mat4`) and the plain `BKE_pose_where_is` chain (`pose_mat = parent_pose · parent_arm⁻¹ · arm · basis`). **On THYMUS, all 271 non-driven bones' bases match Blender at all six frames (< 2e-3), and `root`'s pose matrix matches.** Not yet: `inherit_scale_mode` variants, `BONE_NO_LOCAL_LOCATION`, re-running the stack on frame change. **Deliberately not wired to the viewport yet:** a Rigify rig's DEF bones are placed by constraints, so replacing the saved `pose_mat` with this chain would snap the character to rest — the saved pose stays until (c).
    (c) ☐ constraints and drivers. THYMUS needs Copy Transforms (123), Stretch To (47), Copy Scale (33), Copy Location (24), Copy Rotation (21), Damped Track (16), Armature (9), IK (8), Limit Rotation (6), Limit Distance (4), and 101 drivers (57 bones have a driver on their own channels — tagged `PBD` in the truth). This is what makes the animation visible.
    THYMUS ("WAVE", `rigAction` frames 0–132) is the acceptance file. The timeline's channel list becomes the real F-Curves.
18. **One menu look.** Every popover body uses `MenuItem` rows (check state as the item's icon slot), including the file filter and view options.
19. **One zone look.** Every zone, area and host frame comes from the same surface treatment (radius, gap, background) through the style resolver; no per-part backgrounds. Audit `region_bg` and area backgrounds against `workspace_background`.
20. **Collaboration service.** Choose the transport (e.g. a CRDT such as Automerge or Yjs over WebSocket/WebRTC; Zed's `livekit` crate is already patched in the workspace). It writes `collab.*` and applies remote `BAction`s through `apply_action`, which is already the single place state changes.
21. **Renderer 2** (appkit wgpu into a gpui texture) and **renderer 3** (WebGPU path tracer). **USD via `openusd`.** Clean up the duplicate `apple/src/phiphx` and `apple/src/buikit` copies once phiray uses `crates/phiphx` / phiber.
22. **phiray build.** Either restore the missing engine modules from history or rebuild them on phiphx (animation from task 17, conversion = `SceneModel`), then fix the README.


### 0.10 Heavy files — rendering 20M-triangle scenes (phiray)

Measured in a 2-core, 7 GB test machine with a software GPU (128 MB per storage binding):

| File | Triangles | Open | Rendered view |
|---|---|---|---|
| FLORIS_BOLDRIOD_RIG | 155k | 0.14 s | ✅ |
| THYMUS-BOLDROID-WAVE-003 | 436k | 0.2 s | ✅ smooth |
| LAND-TYPE-15-01 | 7.2 M | 2.5 s | ✅ |
| Mannequins-Female--Pose-05 | 6.9 M | 3–4 s | ✅ |
| FLOWER-CITY-BUILDINGS-… | 7.6 M | 2.2 s | ✅ |
| HEXA-COMBINED-T-48 | 20.8 M | 7.6 s | ✅ (was out of memory) |
| Smile&Driveaway-Animation | 20.2 M | 6.5–9 s | ✅ (was out of memory) |

What changed:

- **phiphx `SceneCorners`:** stores a 4-byte normal id per triangle corner plus a table of unique normals (was 12 bytes per corner).
- **phiphx `SceneObject`:** `mesh`, `uvs` and `material_indices` are `Arc`s, so the per-frame model clone in phiber no longer copies geometry.
- **phiray `Scene`:**
  - `vertex` / `material` / `face` build the scene without a global vertex hash map.
  - phiber shares vertices per mesh (`rendered/verts.rs`).
  - `Bvh::build_compact` builds the compacted tree directly.
  - `Scene::seal` frees the de-duplication maps.
- **phiray buffer splitting:**
  - Large arrays span 4 storage bindings when the device allows 14 or more (`Trace4`, `wgsl/binds4.wgsl`), otherwise 2 (`Trace`).
  - A scene may use up to 4× the per-binding limit per array; larger scenes still fall back to the mesh painter.
- **Test any file:** `PHIBER_TEST_FILES="a.blend;b.blend" cargo test --release -p phiber --lib fixture_renders -- --nocapture` prints open time, peak memory, triangle count and frame time.

Next: per-object streaming for scenes beyond 4× the limit, lower peak memory while opening (2.6 GB for HEXA), textures on roughness / metallic / emission.


### 0.11 Workspace layout and shader maps

**Layout.** `crates/` now holds only our own crates:

| Path | What it is |
|---|---|
| `crates/phiber` | the app (and `assets/`, where every asset lives) |
| `crates/phiphx` | the reader; entry point `src/phx.rs` (the `src/phx/` folder is gone, tests in `src/phx_tests.rs`) |
| `crates/phiray` | the renderer |
| `crates/phical`, `crates/apple*` | unchanged, outside the workspace graph |
| `crates/phigpu/` | the 29 vendored UI crates phiber is built on — `gpui*`, `ui*`, `theme`, `menu`, `util*`, `collections`, `z*` and the rest (see `crates/phigpu/README.md`) |

Paths live in the root `Cargo.toml` only (`members` and `[workspace.dependencies]`),
so moving a crate is two edits there. Assets moved from `phi/crates/phiphx/assets`
into `crates/phiber/assets/production` (924 MB, git-ignored); tests read them from there.

**Shader maps.** Images linked to Principled `Roughness`, `Metallic` and
`Emission Color` are read (`blend/shade/maps.rs` → `SceneMaps`), decoded with the
other textures, and sampled per hit (`Tri::with_maps`; a material record is now 17
words). A linked image replaces the socket value, so phiber sets the scalar to 1
and the shader multiplies. Emitter sampling reads the emission image too, so a
textured glowing surface lights the scene with its own colours. Verified against
Blender 5.0.1 (`phiphx/tests/maps_truth.rs`, fixture `SHADE-MAPS.blend`) and on the
GPU (`phiray/tests/maps.rs`).


### 0.12 One view layer, one theme bridge

The app ran entirely on `v2` while the original `bui` layer still compiled beside
it. Nothing in `v2` called into it, so it is retired to `_unused/phiber-v1/`
(git keeps the history): `bui`, `state`, `store`, `services`, the three v1 app
files (`handle.rs`, `interactive.rs`, `render.rs`) and `tests/store_blender_tests.rs`
— about 17,600 lines. `crates/phiber/src` is now `app`, `v2`, `lib.rs`, `main.rs`.

The vendored widget set went with it. phiber draws its own elements, and the one
call that still reached into the vendored `ui` crate (a tooltip) is now
`v2`-independent, so `ui`, `ui_macros`, `component`, `icons` and `menu` left the
workspace (`_unused/`). What is left of the vendored tree, and why:

| Folder | Crates | Why it stays |
|---|---|---|
| `crates/phigpu` | `gpui*` (10) | the window, input and GPU drawing; phiber depends on `gpui` and `gpui_platform` directly |
| `crates/phiui` | `theme`, `syntax_theme` | gpui's theme globals; reached only through `v2::theme::vendor::install` |
| `crates/phicore` | `collections`, `sum_tree`, `util`, `path`, `util_macros`, `http_client`, `scheduler`, `media`, `refineable`, `zlog`, `ztracing*` | listed inside gpui's and theme's own Cargo.toml — not ours to remove without forking gpui |

`cargo tree -p phiber` is the test of whether a vendored crate is really needed.


### 0.13 File browser typing, resizable sidebar, one chrome grid

**Typing.** `state::edit` gives the window one caret: clicking a text field puts
it there (`ctx.edit` names the key), the root's key handler types into that key,
Backspace deletes, Return commits through the normal action path (so a typed
folder navigates) and Escape restores the value the field started with. The file
browser's search box and directory field are now real text fields, as is the
command palette's. Covered by `state::edit` unit tests.

**Resizable sidebar.** Every `BPanelRole::Sidebar` carries a grab strip on its
inner edge (`prim/panel/resize.rs`); dragging writes `ctx.sidebar.width`, clamped
between the sidebar minimum and three panel widths.

**One grid.** Measured on the painted UI (`tests/chrome_geometry.rs`,
`tests/outliner_tree_geometry.rs`, GPUI `debug_bounds`):

| Rule | Where |
|---|---|
| A menu cap or labelled tab fills its strip apart from a 2 px gap above it, and its bottom edge is the strip's — so the popover or pane below continues the same surface | `atom/anchor.rs`, `atom/tab.rs`, `pane.rs` (`items_end` in headers), `dimensions.menu_top_gap` |
| Every icon rail (toolbar, sidebar, navbar, activity) is exactly one strip height wide — 30 px | `theme/style/metrics.rs` |
| The file and asset browser headings are rounded on top like every host header | `prim/panel/header.rs` |
| Modals hold no padding of their own; their rows run edge to edge like Blender's File View | `prim/panel/modal.rs` |
| A workspace has no padding around its zones — they tile to the window edge, the gap between them being the resize seam | `space.rs` |

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
| 1 ✅ | Header decode handles both the legacy 12-byte `BLENDER-v305` header **and the v1 17-byte `BLENDER17-01v0502` header** (`blo_core_blend_header.cc`). **Blender 5.0+ writes the v1 header + `LargeBHead8` by default** (`writefile.cc` L770, L1564–1573). | `read_header` only knows the legacy layout: byte 7 must be `_`/`-`, byte 8 `v`/`V`, bytes 9–11 a 3-digit version. On a 5.x file it reads `'1'` as pointer size 8, `'7'` as **big-endian**, and version `"-01"` → 0, then mis-parses every block header. | **BLOCKER** | Port `BLO_readfile_blender_header_decode` verbatim: return `{pointer_size, endian, file_version, file_format_version}`; select the BHead type from it. Reject `BlenderHeaderUnknown` with a "newer Blender file format" error, as Blender does. |
| 2 ✅ | Three BHead layouts: `BHead4 {code,len:i32,old:u32,SDNAnr,nr}`, `SmallBHead8 {code,len:i32,old:u64,SDNAnr,nr}`, `LargeBHead8 {code,SDNAnr,old:u64,len:i64,nr:i64}` (`BLO_core_bhead.hh`). | `read_block` hard-codes the `SmallBHead8`/`BHead4` field order (`code,len,old,SDNAnr,nr`) with 32-bit `len`/`nr`. `LargeBHead8` (field order and 64-bit sizes differ) is unsupported. | **BLOCKER** | Add `BHeadType` enum + three decoders into a common `BHead { code, sdna_nr, old, len: i64, nr: i64 }`. |
| 3 ✅ | Zstd (5.x default "compress") and gzip (≤2.7x) transparently decompressed. | `parse_bytes` sniffs both magics. | GOOD | Keep. Consider streaming decode instead of `decode_all` for memory on multi-GB files. |
| 4 | Endianness: 5.x only writes little-endian; big-endian files from old PPC Blenders are byte-swapped on read. | Reads the endian flag; readers honour it for headers. Struct field reads in `dna_layout.rs` are mostly LE. | GOOD (for real-world files) | Low priority; document as LE-only. |
| 5 ✅ | `DNA1` decode → `SDNA`; field offsets from the *file's* DNA; `DNA_struct_reconstruct` when the runtime struct differs. | DNA parsed; offsets computed at runtime from the file's DNA (`struct_field_offset`). No reconstruct needed because phiphx has no compiled-in structs. | GOOD | This is the right design. Keep offsets DNA-driven everywhere; **remove** the hard-coded "fallback offsets" in `object_transforms.rs::extract_transforms_fallback` — they are what goes wrong silently on layout changes. |
| 6 ✅ | Pointer remap via `OldNewMap` hash (O(1)). | Every dereference is `self.blocks.iter().find(|b| b.old_address == addr && b.code_str == ...)` — a **linear scan per pointer** (`material.rs` L37/70/147/228, `scene.rs`, `mesh_block.rs`). The code's own comments call it "O(n²)" and it is the reason for the 30 s watchdog and the `.phx` sidecar workaround. | WRONG (perf) | Build `HashMap<u64 /*old*/, usize /*block idx*/>` once after the block loop (Blender's `oldnewmap_insert`). This alone removes the need for the sidecar-on-open hack. |
| 7 | Whole-file read is a single pass; blocks are memory-mapped/streamed (`FileReader`), not copied per block. | `read_block` allocates a fresh `Vec<u8>` per block (tens of thousands of allocations on real files). | WRONG (perf) | Read the file once into one buffer (or mmap) and store `(offset, len)` slices per block. |
| 8 ✅ | `GLOB` → `FileGlobal`: **`curscene`**, `cur_view_layer`, `curscreen`, `subversion`, `minversion`, `filepath`, colorspace. | `GLOB` is never read. `extract_scene` takes **the first `SC` block**, which is wrong for any multi-scene file. Subversion is not exposed. | WRONG | Read `FileGlobal` first; use `curscene`/`cur_view_layer` as the root; surface version+subversion in the status bar Version segment. |
| 9 ✅ | `REND` block: cheap `RenderInfo` (frame range, scene name) for external tools; `TEST` block: thumbnail for the file browser. | Neither read. File browser shows no `.blend` thumbnails. | MISSING | Add `read_render_info()` and `read_thumbnail()` header-only readers (stop at first `DNA1`/ID block) so the file browser can preview without a full parse — Blender's file browser does exactly this (`BLO_thumbnail_from_file`). |
| 10 | Linked libraries (`ID_LI`), `ID_LINK_PLACEHOLDER`, library overrides, resync; `BlendFileReadReport.count.missing_*` drives the report banner. | No library handling. Objects whose data lives in a linked `.blend` silently come out empty. | MISSING | Phase 3: resolve `Library.filepath` (`//`-relative), open linked files, map placeholders; count and report missing ones exactly as `file_read_reports_finalize` does. |
| 11 | `do_versions*` for old files (`versioning_250…520.cc`). | None. phiphx instead keeps *two* mesh layouts (`mvert/mpoly/mloop` legacy and 4.x attribute). | Acceptable | Reading is DNA-driven, so versioning is only needed where *semantics* changed (e.g. `smooth` flag → `sharp_face` attribute, `MLoopUV` → float2 attribute). Keep the dual paths but gate them on `file_version`, not on "modern path returned nothing". |

### 4.2 Scene graph extraction (`scene.rs`, `object.rs`, `object_transforms.rs`, `object_meta.rs`)

| # | Blender | Phiber today | Status | Fix |
|---|---|---|---|---|
| 12 ✅ | Outliner "View Layer" mode shows `ViewLayer.layer_collections` (Scene Collection → children), with `LayerCollection.flag` exclude / hide_viewport / holdout / indirect_only, and per-object `Base.flag` (selected, hidden). | `extract_collection_hierarchy` walks `Scene.master_collection` → `Collection.children` / `gobject` (good), **but `scene_from_blend` throws the tree away**: `SceneModel` has no `collections` field, and `outliner/tree.rs::flatten` builds a flat, alphabetically sorted object list under one root. View-layer flags are never read. | WRONG | Carry `collections: Vec<SceneCollection { name, children, object_indices, flags }>` into `SceneModel`; read `ViewLayer`/`LayerCollection`/`Base` for the flags; make the Outliner render the tree (the expand/hide plumbing in `tree.rs` already exists). |
| 13 ✅ | Object placement = `BKE_object_where_is_calc`: `loc/rot(or quat, by rotmode)/size` → local matrix, × `parentinv`, × parent's world matrix (recursively), plus constraints. **In 4.x/5.x DNA there is no `obmat` field** — `object_to_world` is runtime only (`DNA_object_types.h` L675). | `object_transforms.rs` reads `loc`, `rot`, `quat`, `size` and looks for `"obmat"` (absent in 4.x+ files → falls back). Then `object_from_blend` copies **only `location` and `scale`** into `SceneObject` — **rotation and `matrix_world` are dropped** (`services/scene/import.rs` L45–46; `SceneObject` has no rotation field). Parenting (`*parent`, `parentinv`) is never read. | WRONG | Compute `matrix_world` in phiphx exactly as Blender does (loc/rot/quat/`rotmode`/`size` → `parentinv` → parent chain; `parent_type` OBJECT at minimum). Store a full `[[f32;4];4]` on `SceneObject` and have the viewport transform by it. |
| 14 ✅ | Object types drawn: MESH, CURVES/CURVE/SURF/FONT (evaluated to mesh), META, ARMATURE (bones), LATTICE, EMPTY (draw type: plain axes/arrows/cube/image), CAMERA (frustum), LIGHT (gizmo per type), SPEAKER, LIGHTPROBE, GPENCIL, VOLUME, POINTCLOUD. | Mesh vertices/faces only; CAMERA/LIGHT partitioned into `cameras`/`lights` with a few fields; everything else appears in the outliner (if at all) and draws nothing. | MISSING | Add typed data readers per `Object.type`: `Camera` (lens, sensor, clip, type), `Light` (type, color, energy, spot size), `Empty` (`empty_drawtype`, `empty_drawsize`), then Curves/Text/Armature in later phases. Draw the Blender overlay shapes for them. |
| 15 | Instancing: `Object.instance_collection` + `transflag & OB_DUPLICOLLECTION`, plus geometry-nodes instances via depsgraph. | `object_meta.rs::extract_instance_info` flags `is_instance` and `instance_targets` names. Nothing is instanced in the viewport. | MISSING | Expand collection instances into transformed copies at scene-build time (Blender does this in the depsgraph). |
| 16 ◐ | Modifiers are **evaluated** (subsurf, mirror, array, boolean, solidify, bevel, geometry nodes…) before drawing. | `extract_modifiers` lists modifier names; geometry is drawn unmodified. A subdivided character shows as its cage. | MISSING (biggest visual gap after transforms) | Phase 4: evaluate the common non-destructive modifiers in Rust (Mirror, Array, Solidify, Subdivision via OpenSubdiv-equivalent or Catmull-Clark, Triangulate, Weld, Displace-less). Show the modifier stack in Properties → Modifiers meanwhile so the user sees *why* the shape differs. |
| 17 ✅ | Active object & selection from `ViewLayer.basact` / `Base.flag & BASE_SELECTED`. | `BlendScene` has `active_object` / `selected` fields but they are never populated. | MISSING | Read `ViewLayer.basact` and `Base.flag`; drive outliner highlight + viewport outline. |
| 18 ✅ | Object unique names come from `ID.name` (already unique per type in Blender). | `extract_scene` re-derives `.001` suffixes with a `HashMap` counter, which can rename objects that were legitimately unique or that were **already** `Cube.001` in the file. | WRONG | Use `ID.name[2..]` verbatim. |

### 4.3 Mesh data (`mesh.rs`, `mesh_block.rs`)

| # | Blender | Phiber today | Status | Fix |
|---|---|---|---|---|
| 19 ✅ | 4.x/5.x mesh: `verts_num/edges_num/faces_num/corners_num`, `face_offset_indices`, `position` (float3), `.corner_vert`, `.corner_edge`, `.edge_verts`, `material_index`, `sharp_face`, `sharp_edge`, custom normals, UV float2 layers, colour attributes — stored in `CustomData vert/edge/face/corner_data` and (5.x) `AttributeStorage attribute_storage`. | Reads counts, `face_offset_indices`, `position`, `.corner_vert`, UV layer, loop normals from both `CustomData` and `attribute_storage`. Bounds-checked, never panics. | GOOD (core geometry) | Keep; add fuzz tests using `tests/fixtures`. |
| 20 ✅ | `material_index` (per-face int attribute) selects the material slot per face. | Not read. `SceneObject.material_indices` is **always empty** (`import.rs` L136, L356), so every face gets slot 0 — multi-material objects render single-colour. | WRONG | Read the `material_index` int attribute (the helper `read_customdata_int` already exists) and carry per-triangle indices through triangulation. |
| 21 ✅ | Smooth vs flat shading from `sharp_face` (5.x) / `ME_SMOOTH` poly flag (legacy); custom split normals. | Loop normals read when present; otherwise `build_normals` synthesises per-face normals → everything flat-shaded. | WRONG | Honour `sharp_face`; compute vertex-averaged normals for smooth faces exactly like `BKE_mesh_normals_loop_split`. |
| 22 | Edges (loose edges drawn in wireframe/overlay), vertex groups, shape keys (`Key`), colour attributes. | Not read. | MISSING | Phase 4. |
| 23 ✅ | Triangulation: Blender uses `BKE_mesh_calc_corner_tris` (ear-clipping for n-gons via `BLI_polyfill_calc`). | Fan triangulation in `scene_from_blend`. Concave n-gons render wrong. | WRONG | Port `BLI_polyfill_2d` (ear clipping) for n>4. |

### 4.4 Materials, textures, shaders (`material.rs`, `services/scene/mod.rs`)

| # | Blender | Phiber today | Status | Fix |
|---|---|---|---|---|
| 24 ✅ | Solid mode colour comes from `View3DShading.color_type`: MATERIAL uses `Material.r/g/b` (viewport display colour) — **not** the Principled Base Color; OBJECT uses `Object.color`; RANDOM hashes the name; SINGLE/VERTEX/TEXTURE as named. Default is MATERIAL. | `read_material_fields` **prefers the Principled "Base Color" socket over `Material.r/g/b`**. That is Material-Preview behaviour, not Solid-mode behaviour, so colours differ from what Blender shows on open (Blender opens in the file's saved shading mode, usually Solid). | WRONG | Read both; expose `viewport_color` (`Material.r,g,b,a`, `roughness`, `metallic`) and `base_color` separately; let the View3D shading mode pick. |
| 25 | Material Preview / Rendered: the full shader node tree is compiled to GLSL (`gpu/shaders/material/gpu_shader_material_*.glsl`, 103 node shaders; codegen in `gpu/intern/gpu_codegen.cc`, `gpu_node_graph.cc`) and drawn by EEVEE. | Only Principled `Base Color` (+ `roughness`/`metallic` scalars from the `Material` struct, not the node socket) and the first `TexImage` node's `Image.filepath`. No node graph is retained. | MISSING | Phase 5: read `bNodeTree.nodes/links` + `bNodeSocket.default_value` into a `MaterialGraph`; render a Principled-only fast path first (WGSL PBR), then port node shaders incrementally. |
| 26 ✅ | Images: `Image.filepath` (`//` relative to the blend), `ImagePackedFile` packed bytes, colour space (`Image.colorspace_settings`), UDIM tiles. | `image_texture_path` returns the path string; nothing loads, unpacks or samples it (README "Known limitations" confirms). | MISSING | Decode packed images from the `.blend` (`Image.packedfiles` list → `PackedFile.data`), resolve `//` against the file dir, upload as textures once the GPU path exists. |
| 27 ✅ | `World` (viewport background in Rendered mode, `World.horr/horg/horb` legacy, node tree for 4.x+), `Scene.r` (engine, `xsch/ysch/size`, `cfra/sfra/efra`, fps), `Scene.camera`. | `SceneModel.timeline/world/render` exist but `scene_from_blend` fills them with `Default::default()` (`import.rs` L50–52). The Timeline/Rendering/Shading hosts therefore always show factory values. | WRONG | Read `Scene.r.{cfra,sfra,efra,frs_sec,frs_sec_base,xsch,ysch,size,engine}` and `Scene.world` → colour; trivial with the existing offset helpers. |

### 4.5 UI state from the file ("Load UI")

| # | Blender | Phiber today | Status | Fix |
|---|---|---|---|---|
| 28 ◐ | With **Load UI** on (default), `wmWindowManager` → `wmWindow.workspace_hook` → active `WorkSpace`, `bScreen` → `ScrArea` (spacetype, v1..v4 verts) → `SpaceLink` list; `View3D.shading.type`, `RegionView3D.viewquat/dist/ofs/persp/is_persp`; `SpaceOutliner.treestore` expand flags; `SpaceProperties.mainb` tab; `Scene.r.cfra` for the timeline. | None of `WM`, `WS`, `SR` blocks are read. Phiber always opens in its own default Layout with its own camera. The user does not see "the same things Blender sees" — they see phiber's default layout with the file's meshes in it. | MISSING | Phase 6: read the active workspace name + its screen's area grid + each area's space type and map onto `BSpaceType`/`BHostType`/zone fractions (the v2 area/zone model already stores fractions); apply `RegionView3D` to the viewport camera and `View3D.shading` to the shading dropdown; apply outliner expand state. Offer the same `load_ui` toggle in the Open dialog. |
| 29 | Recent files list updated on successful open (`wm_history_file_update`), kept even on failure since 4.3 (see comment "#127825"). | `v2/modal/splash/recent.rs` keeps 10 slots in state; check that a v2 open pushes into it (Blender writes `recent-files.txt` in the config dir). | Partial | Persist to disk the way Blender does; update on every open. |

### 4.6 Progress, reporting, failure handling (`handle.rs`, `import.rs`, `statusbar/list.rs`)

| # | Blender | Phiber today | Status | Fix |
|---|---|---|---|---|
| 30 ✅ | Wait cursor + blocking read; **no progress bar for file open**; `CLOG` timing line on completion; report banner on problems. | Background thread + status-bar progress bar + stage label. **This is better than Blender** in principle — but the bar is fake: only `Reading (0.15)` is ever written before settle. | WRONG (fake progress) | Give `BlendImporter` a `&dyn Fn(f32, &str)` / channel progress sink (the `BlendImporter::import` asset path *already* takes one; `load_blend_scene` does not). Emit real fractions: bytes read / decompressed, blocks indexed, IDs read per type, meshes built, materials resolved, scene mapped — and forward via `cx.background_executor()` updates to `scene.load_progress`. Keep the bar (parity-plus), and also show the wait cursor over the viewport for exact Blender feel. |
| 31 ✅ | No timeout; a 2 GB production file simply takes as long as it takes. | 30 s watchdog → `TimedOut`, prior scene retained. | WRONG | Remove the timeout once #6/#7 land (a hashed remap makes a 178 MB file sub-second to index). Replace with a **Cancel** button on the progress bar (Blender's jobs have one). |
| 32 | `catch_unwind`-free; readers are bounds-safe by construction. | `catch_unwind` around the parser in two places (`blend.rs`, `load.rs`) hides bugs as `Malformed`. | Acceptable as a safety net | Keep, but log the panic payload + block code/offset so fuzz cases can be turned into fixtures. |
| 33 ✅ | Timing report: "Blender file read in 0m1.23s" (+ libraries/overrides). Counts of missing libs/IDs → banner. | No timing, no counts. | MISSING | Return a `BlendReadReport { duration, blocks, ids_by_type, missing_libraries, warnings }` from phiphx; print to the Info editor (`host/info`) and status bar the way Blender does. |
| 34 ◐ | Status bar: `Verts | Faces | Tris | Objects` (scene stats from `info_stats.cc`), memory, version. | Segments `Scene / Frame / Stats / Memory / Version` exist in `statusbar/set.rs`; they need to resolve from the loaded `SceneModel` + header version rather than placeholders. | Partial | Compute `SceneStats` in `scene_from_blend`; Version = `"{major}.{minor}.{patch}"` from `FileGlobal`/header. |
| 35 ✅ | Opening a file never writes next to it. | `import_blend_native` **writes `foo.phx` beside the user's `foo.blend` on every first open**, and later opens silently read the sidecar instead of the `.blend` (stale-cache risk if mtimes are odd, e.g. cloud sync). | WRONG (side effect) | Move the cache to the app's cache dir, key it by (path, size, mtime, header hash), and make it opt-in. Once #6/#7 land the cache is unnecessary for open; keep it only for the asset-import pipeline. |

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

---

## B-Bone segments — the deformation gap behind the render artifacts (2026-09-22)

`modifier/armature.rs` has always carried the line *"Not yet: envelopes, dual
quaternions, B-Bone segments, multi-modifier."* On Female-01 that last item is
not a corner case: the rig is Rigify, and **139 of its 204 deform bones are
B-Bones** — `DEF-spine.*` and `DEF-ear.*` at 8 segments, `DEF-thigh.*`,
`DEF-shin.*` and most face bones at 10. We treat every bone as one rigid
transform, so wherever a segmented bone is actually bent, our vertices land
about a millimetre away from Blender's.

A millimetre is more than a garment's thickness, which is why the body pokes
through the leggings as dark speckles and why the gap between the legs renders
as skin instead of fabric.

### How it was pinned down

`crates/phiber/examples/vertex_delta.rs` compares our evaluated vertices
against `phiphx/tests/truth/blender_vertex_dump.py`, index by index and — when
the two vertex orders differ, which Solidify and Subsurf both cause — as a
nearest-neighbour point set. `PHIPHX_MODIFIERS=<n>` truncates every stack to
its first `n` modifiers and the truth script's `--keep N` mutes the same ones
in Blender, so a stack can be walked one stage at a time.

`Female-Mannequin-Body-01.002` (the leggings), stack
`SOLIDIFY → SUBSURF → LATTICE → ARMATURE`, mean world-space deviation:

| Stage | Modifiers applied | Vertices | Mean | p99 |
|---|---|---|---|---|
| 1 | Solidify | 2 294 | **0.034 mm** | 0.32 mm |
| 2 | + Subsurf | 36 734 | **0.025 mm** | 0.19 mm |
| 3 | + Lattice | 36 734 | **0.026 mm** | 0.19 mm |
| 4 | + Armature | 36 734 | **1.361 mm** | 3.31 mm |

So Solidify, Subsurf (including its vertex-group propagation) and the 7×3×5
B-spline Lattice are all exact to well under a tenth of a millimetre, and the
whole error enters at the armature. Objects bound to bones that are *not* bent
in this pose agree exactly, which is why the diagnosis is the segment blend and
not skinning in general:

| Object | Stack | Deviation |
|---|---|---|
| `Earring pearl.001` | Mirror → Armature | 0.0003 mm |
| `Teeth.003` | Armature ×2 | 0.0001 mm |
| `Eyebrows.001` | Subsurf → Armature ×2 | 0.0002 mm |
| `Nose.002` | Subsurf → Armature ×2 | 0.0003 mm |
| `Full-body .001` | Subsurf → Armature | **0.62 mm** (worst 12.5 mm) |
| `Female-Mannequin-Body-01.002` | Solidify → Subsurf → Lattice → Armature | **1.36 mm** (worst 4.1 mm) |

The worst deviations on the leggings sit at a mean height of 0.94 m — the hips
and upper thighs, where `DEF-thigh.*` is bent. Every modifier option is plain:
`use_vertex_groups` on, envelopes off, `use_deform_preserve_volume` off, one
armature, no vertex-group mask. Nothing else is left to explain it.

### Implemented (2026-09-22)

`modifier/bbone.rs` ports `BKE_pchan_bbone_spline_params_get`,
`BKE_pchan_bbone_handles_compute`, `BKE_pchan_bbone_spline_compute`,
`BKE_pchan_bbone_segments_cache_compute` and
`find_bbone_segment_index_straight` from `armature.cc` (the checkout in
`REFS/blender`, v5.2.0), plus the `vec_roll_to_mat3` / `mat3_vec_to_roll` /
`equalize_cubic_bezier` / `BKE_curve_forward_diff_bezier` maths they need.
`armature.rs` now carries a `BDeformBone` — the rigid `pose · rest⁻¹` matrix
and, when the bone has segments, its `segments + 1` deform matrices.

The deform matrix for segment `a` is `pose_mat · bbone[a] · rest[a]⁻¹ ·
arm_mat⁻¹`, which is the posed segment times the inverse rest segment; with a
flat spline it collapses to `pose · rest⁻¹`, the one-segment case. A vertex's
segment comes from its Y in bone-local space, and `b_bone_deform` adds the two
neighbouring segments with weights `1 − blend` and `blend` — both affine in
the matrix, so the skinner can lerp the two matrices instead and keep one
multiply per influence. `SkinPair` therefore grew a `next` slot and a `blend`,
and the phiray `skin` kernel lerps the pair; a rigid bone sets `next == bone`
and `blend = 0`.

Not ported: `BBONE_MAPPING_CURVED` (Blender's BSP boundary planes). Every
segmented bone in this rig uses `BBONE_MAPPING_STRAIGHT`; a curved-mapping bone
falls back to the straight lookup.

Result, same measurement as above:

| Object | Before | After |
|---|---|---|
| `Female-Mannequin-Body-01.002` (leggings) | 1.361 mm mean, 3.31 mm p99 | **0.025 mm mean, 0.19 mm p99, 0 vertices over 0.5 mm** |
| `Full-body .001` | 0.62 mm mean, 12.5 mm worst | **0.038 mm mean, 0.41 mm p99**, 135 of 13 983 still over 0.5 mm |

The leggings now agree as closely after the armature as they did before it
(0.026 mm at the lattice stage), which is the definition of the armature
contributing no error of its own. The 135 remaining body outliers sit at
z 1.18–1.66 — shoulders and head, not the legs — and are still open.

### What it did *not* fix

The render artifacts this was chased for are still there, so the attribution
above was wrong and is corrected here. Rendering the leggings with the body
dropped (`PHIBER_DROP=Full-body`, which now applies after the camera bounds so
the frame does not move) separates them:

- **The dark speckles on the thigh are the body**, and they survive the fix.
  With our surface now within 0.025 mm of Blender's and the body-to-garment
  gap of the same order, which surface a ray hits first is decided by float
  precision — the speckle is that coin flip, not a deformation error. Blender's
  watertight traversal does not have it.
- **The dark wedge between the legs is not the body at all** — it is still
  there with the body dropped, one shade lighter. It is the leggings' own far
  leg, in a crevice our 2-bounce GI leaves far darker than Cycles' 12.

Both are rendering gaps (`rendering.md` §3 rows 4 and 5), not reading gaps.


Blender ref: `BKE_armature.hh`, `armature.cc` — `b_bone_spline_setup()` builds
a bone's Bezier from its head/tail handles (`bbone_handle_type_start/end`,
`bbone_curveinx/z`, `bbone_roll`, `ease_in/out`, `scale_in/out`) and produces
`bbone_segments` matrices; `BKE_pchan_bbone_deform_segment_index()` picks the
segment a vertex falls in from its position along the bone, and
`b_bone_deform()` blends the two neighbouring segment matrices.

For us that means `armature.rs` gains a per-pose-channel segment table, and
`SkinJob` grows from one matrix per bone to `bbone_segments` matrices per bone
plus a per-vertex segment index and blend factor. The GPU skinner's buffer
layout changes with it. `vertex_delta` is the check: the leggings and the body
should both come back under 0.05 mm.
