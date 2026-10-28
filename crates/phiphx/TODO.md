# phiphx — Blender parity plan

The goal: every `.blend` the user owns opens in Phiber with the geometry,
placement and shading Blender 5.2 evaluates for it. Not modelling or
sculpting; everything a saved file *shows*.

The measure is Blender itself. A census script
(`scratchpad/coverage/coverage.py`, run by Blender over all 230 files) lists
every feature each file uses and every drawn copy with its triangle count
and world box; `tri_count --objects --posed --bounds` prints the same for
phiphx; `diff.py` / `boxdiff.py` compare them. A port counts as done when
its fixture test (`tests/*_truth.rs`, truth dumped by Blender) passes **and**
the real files that use it diff clean.

## Census (230 files, 2026-10-27)

| Feature | Files | State |
|---|---|---|
| SUBSURF / ARMATURE / SOLIDIFY / MIRROR / LATTICE | 62 / 60 / 47 / 44 / 42 | done |
| Collection instances | 2 (city scenes) | done: CRYSTAL-CAVES-CITY matches exactly (1,309 objects, 70.8M tris) |
| Vertex parenting (`PARVERT1`/`PARVERT3`) | 50 | done: exact on a posed parent, ~1 mm on a subdivided one |
| Cloth (viewport) | 12 | done: baked cache read, in-file and on disk; `Dress-08` 4.8e-7 |
| Geometry Nodes (only "Smooth by Angle") | 15 | **open** |
| SHRINKWRAP / CURVE / CORRECTIVE_SMOOTH / MULTIRES / MASK | 20 / 5 / 5 / 4 / 7 | done |
| FONT and CURVE objects | 5 / 7 | done |
| SIMPLE_DEFORM | 1 | done: all 20 city uses within 2.9e-6 |
| SURFACE_DEFORM | 2 | done: fixture 3e-4; both real files have lost their target, so Blender disables it |
| Grease Pencil | 8 | nothing to draw: every GP object is an empty `Annotations` layer (0 strokes) |
| Rigify pose (IK, Stretch To, Copy Transforms, Damped Track, Limit *) | 60 | done: 0/404 bones over 1e-4; the Limit Rotation legacy clamp was not versioned in |
| Scene Simplify cap | several | done |

## Done (each with its Blender source and test)

- Curve deform — `curve_deform.cc`; `curve_deform_truth`
- Corrective Smooth — `MOD_correctivesmooth.cc`; `corrective_smooth_truth`
- Shrinkwrap, all modes, target evaluated — `shrinkwrap.cc`; `shrinkwrap_truth`
- Multires with MDisps — `subdiv_displacement_multires.cc`; `multires_truth`
- Text objects, Type 1 + TrueType — `vfont.cc`, `vfontdata_freetype.cc`; `text_truth`
- Curve objects: bevel, profile, object, taper, fill, caps — `displist.cc`; `curve_mesh_truth`
- Mask, group + armature + smooth — `MOD_mask.cc`; `mask_truth`
- Array merge — `MOD_array.cc`, `mesh_merge_verts`; `array_merge_truth`
- Solidify creases and angle-weighted normals — `MOD_solidify_extrude.cc`; `solidify_creases_truth`
- Mirror merge rule — `mesh_mirror.cc`
- Subdivision: `boundary_smooth` read right way round; Simplify cap (`get_render_subsurf_level`)
- Collection instances share the member's evaluated mesh; progressive open swaps them in
- Edge and object-to-mesh reads scoped to their owner (repeated addresses in big files)
- 3.x face reader no longer drops faces of 100+ corners
- Text/Curve `offset` read under its DNA name `width`

## In progress

1. **Rigify pose** — compare every bone's `pose_bone.matrix` with Blender's at the saved frame, fix the first diverging constraint, repeat. Files: `blend/rig/`, `blend/anim/`, `scene/rig.rs`. Test: a Rigify-like leg fixture.
2. **Vertex parenting** — `ob_parvert3` / `give_parvert` on the parent's evaluated mesh; `object/transform.rs`, world-matrix resolution, progressive open, per-frame poser.
3. **Simple Deform** — register in `KINDS`, fixture + truth test, check CRYSTAL-CAVES-CITY.
4. **Surface Deform** — `MOD_surfacedeform.cc`, saved binds on the target's evaluated mesh.
5. **Cloth** — pass-through before the cache starts; read `PTCacheMem` and `blendcache_*` at the frame. No simulation.
6. **Smooth by Angle** — a field evaluator for the eight node types the files use; sharp edges carried through `BlendEvalMesh`; split normals checked against Blender.
7. **phiray draws no geometry** (blocking, 2026-10-28). The instancing
   agent's uncommitted phiray edits (23 files, `trace/instance.rs`,
   `trace/mod.rs`, `scene.rs`, `gpu/raster.rs`, `wgsl/{bvh,layout,raster,solid,shade}`)
   leave every viewport mode drawing only the world. Repro:
   `phiber.exe -b "Claude outputs/phiber-walkthrough/cube-sphere-anim.blend" -o <dir>/frame_ -f 1`.
   The patch is saved at `_temp/phiray-instancing-wip/`. Fix forward or back
   the layout/BVH change out; nothing else rebuilds phiray until it draws.

## Rendering and the spatial world (phiray, phiber)

The same census lists every shader, world and compositor node, light and
camera the files use (`aggregate.py ... shader world light camera`).
phiray's own audit is `crates/phiray/docs/ROADMAP.md`.

- **Shader nodes** (in progress): `blend/shade` bakes a fixed Principled
  record and knows 29 node types; the files use 55. Not handled: node groups
  (88 files), Map Range (88), Separate/Combine XYZ (89/72), Vector Rotate
  (72), Vertex Color (72), Separate/Combine Color (68), White Noise (60),
  Displacement (25), Vector Math (24), Fresnel (16), Normal (13), Layer
  Weight (8), volumes (5, not planned). Translate from Cycles
  (`intern/cycles/kernel/svm/*.h`, `scene/shader_nodes.cpp`). Proof: 8 files
  rendered in Blender (own engine, low samples) and in phiber, mean error
  before/after and side-by-sides.
- **Shared-geometry instancing in phiray** (in progress): ROADMAP §4.4 step 1.
  One mesh, N placements; the 65,536-object cap goes; collection instances
  from phiphx share one `Arc<SceneMesh>`. Proof: BC-15 and CRYSTAL-CAVES-CITY
  open time, GPU memory, pixel-identical renders.
- **EEVEE files** (90 of 230): Phiber renders them with the path tracer; the
  visible differences to list after the node work are EEVEE's soft shadows,
  bloom, SSR and AO settings.
- **Basketball demo** (`Downloads/phiber/basketball.py`, patched for Blender
  5's layered actions): BASKETBALL.blend, 218 frames, 1280×720. Phiber
  renders a frame in 7 s at 4096 samples, 3 s at 512. 157 frames rendered and
  encoded to `scratchpad/ball/basketball_preview.mp4` with Blender's own
  sequencer (`scratchpad/ball/encode.py`, which handles Blender 5's `strips`
  and `media_type`); the rest are rendering. Then a high-resolution still of
  the shot. A lost device under load panicked the headless frame assembly
  (`phiber/src/view/draw/canvas/view3d/rendered/still.rs`'s `lay`, which
  guarded only the colour plane); every plane is now checked.
- **The game** (`crates/phiplay`, Phiber's Play host, `phiber play:`): render
  the default match headless, screenshot the host at high resolution, list
  what the sets declare that the scene does not show, fix.
- **Physics** (ROADMAP §2, XPBD): not started; the ball's bounce is keyed, not
  simulated. `blend/physics` reads every setting; 3 of its 6 truth tests fail
  on flag names (being fixed).

## Open after that

- Position census (`boxrun.sh`) over all 230 files; every OFF line gets a cause. 32 files ran before the restart: all clean except the Rigify foot.
- Re-run the count census over all 230 files once the open ports land (`diffall.sh`; `diff.py`'s dedup key now includes the instancer, so earlier per-file diffs under-report coincident copies).
- Armature-deformed members of collection instances take the pose.
- Mirror and Weld carry creases; Subdivision keeps part-sharp creases past `quality`.
- `BlendSkinSource::run` (modifiers after an Armature) runs with no world table, so a Curve/Lattice/Shrinkwrap after an Armature is a no-op on the re-pose path.
- Drivers for the new modifiers (`driver.rs` `SETTINGS` has no CORRECTIVE_SMOOTH / SHRINKWRAP / MASK rows).
- Geometry Nodes beyond Smooth by Angle, particles, Bevel modifier: none of the 230 files use them.

## Committed so far (2026-10-28, nothing pushed)

- phiber `1cbde46` — the em dash written as a hyphen through the reader and
  its eleven stores. Text literals only.
- phikit `b4ca396bf` — the caption-button fix (`gpui_windows/src/events.rs`):
  Windows 11's Snap Layouts swallows the non-client button-up over maximize,
  so the press was never finished. Checked by hand in the app.

Held back on purpose, each for a stated reason:

- The phiray shared-geometry instancing work (23 files). The cube-sphere
  repro must draw geometry first; the patch is in
  `_temp/phiray-instancing-wip/`.
- `phiber/src/scene/play/stage.rs` (broadcast lens worked out from where the
  camera stands), `phiber/src/view/draw/canvas/view3d/rendered/still.rs`
  (every readback plane guarded) and `phiplay/src/build/venue/light.rs`
  (2 m ceiling fixtures, same total power, so players cast a shadow). All
  three compile; none has been seen in a render, because that needs a phiber
  build and phiray is mid-regression.
- The phiphx ports wait on a full suite run, which is queued behind other
  sessions' builds.

Not mine, left alone: `phiber/src/modal/command/*` (the top-bar session's),
`phiber/tests/welcome_refinement.rs` and `file_browser_accordion_refinement.rs`
(owner being found), `phigpu/gpui_windows/directx_*` (the viewport session's),
`phigpu/gpui/src/app/test_context.rs`.

## Rules

`else if` is banned (use `match`); files under ~250 lines; every port cites
the Blender function in its doc comment; nothing deleted (clutter goes to
`gemphi/_temp` with an INVENTORY line); builds under
`CARGO_TARGET_DIR=target/mods`.
