# Rendering — what Blender does, what Phiber does, and the order to fix it

> Companion to `blend-read-parity.md`, which covers reading the file. This one
> covers turning what was read into pixels.
>
> Blender references, all under `REFS/blender/blender/source/blender/`:
> `draw/engines/workbench/`, `draw/engines/overlay/`, `draw/engines/eevee/`,
> `gpu/shaders/material/`, `draw/intern/draw_manager.cc`,
> `editors/space_image/space_image.cc`, `blenloader/intern/versioning_defaults.cc`.

---

## 0. The finding that matters most

**The viewport has no depth buffer.** It projects triangles, sorts them, and
paints them as vector paths. Everything below follows from that, including the
two symptoms reported: poor quality, and characters rendering without their
clothes.

Until 2026-09-20 the sort was worse than "no depth buffer": it was **two**
sorts. Each object's triangles were sorted among themselves, and then the
objects were ordered *by the depth of their origin* — one number per object,
taken from a point that is usually nowhere near the geometry.

```rust
// view3d/mesh.rs, before
let (_, _, depth) = project3_full(lx, lup, lz, yaw, pitch, pan, zoom);
```

For a rigged character every garment and every body part is a separate object
parented to the same armature, so their origins carry no useful depth relative
to each other. Whichever object lost that comparison disappeared **behind the
other whole**. That is precisely "the clothes aren't showing": a shirt is not
in front of or behind a chest, its triangles interleave with the chest's, and
one number per object cannot express that.

**Fixed — 2026-09-20, in two steps.**

First the two sorts became one: every triangle in the scene sorted against
every other. That was an improvement, not a solution — a single painter sort
still cannot draw two triangles that genuinely intersect, and it still cost an
O(n log n) sort of every triangle every frame.

Then the sorts went away entirely. `mesh/raster.rs` fills triangles into a
colour buffer with a depth buffer beside it, testing each pixel, and the canvas
paints the result as one image (`mesh/paint.rs::fills`). Order stops mattering
once depth decides per pixel. Pinned by
`the_nearer_surface_wins_whichever_is_drawn_first` and
`two_surfaces_that_cross_each_other_each_own_the_half_they_are_nearer_on` —
the second of which no ordering of whole triangles can satisfy.

---

## 1. How Blender produces a viewport frame

`draw_manager.cc` runs three engines over the evaluated depsgraph:

1. **Workbench** (`Solid` mode) — `workbench_prepass_*` writes colour, normal
   and object id to a G-buffer with depth testing; `workbench_composite_*`
   lights it with a three-light studio rig from the active `StudioLight`;
   `workbench_effect_cavity`, `_outline`, `_dof`, `_taa` post-process it;
   `workbench_shadow_*` adds shadow volumes.
2. **Overlay** — grid and axes (`overlay_grid_*`), wireframes, selection
   outlines (`overlay_outline_*`), camera frustums / lights / empties
   (`overlay_extra_*`), armatures, edit-mode cages. Drawn after Workbench,
   against the same depth buffer.
3. **EEVEE** (`Material Preview` / `Rendered`) — the material node tree is
   compiled to GLSL by `gpu_codegen.cc` (103 node shaders in
   `gpu/shaders/material/`) and rendered with real lights, shadows, screen-space
   reflections and an irradiance cache.

Every one of those passes depends on a depth buffer. None of them sorts.

## 2. How Phiber produces a viewport frame today

**Every shaded mode draws on the GPU**, as in Blender. `view3d/draw.rs::gpu_mode`
maps the shading to a phiray kernel:

| Shading | Path |
|---|---|
| Solid | **phiray `Mode::Solid`** — Workbench: one deterministic compute dispatch over the same `GpuBvh`, primary ray + studio rig, finished the frame it is asked for |
| Material Preview, Rendered | **phiray `Mode::Rendered`** — the path tracer, refining while the view is still |
| Wireframe | **phiray `Mode::Wire`** — one primary ray, barycentric edge distance, depth-tested |

phiray also runs the GPU **skinning** kernel (`scene/skin.rs`).

Until 2026-09-20 Solid fell to the CPU painter below `CPU_PAINTER_TRIS`
(200k triangles) — which is to say, in the common case. A `.blend` opens in
Solid and a character file is well under that, so what you saw when you opened
a file was a single-threaded rasteriser re-shading every pixel inside the paint
pass, every frame, with nothing cached. **That was the slowness.** Blender has
no such path: Workbench is a GPU engine and always has been.

### 2.1 The two deterministic kernels

`phiray/src/trace/wgsl/workbench.wgsl` + `solid.wgsl`, selected by
`Tracer::set_mode(Mode::Solid)`:

- `BKE_studiolight_default` and `workbench_world_light_lib.glsl::get_world_lighting`
  ported verbatim to WGSL — the same four-light rig, wrapped terminators,
  `blinn_specular`, `brdf_approx` and the metallic/dielectric split the CPU
  painter had, so the look is unchanged.
- One primary ray per pixel against the BVH, no jitter, no bounces, no
  accumulation: `Tracer::samples()` is `1` in this mode, so the frame is
  finished on its first dispatch and the viewport stops polling.
- The normal goes into view space through the camera basis, which is what fixes
  the rig to the camera the way Workbench does.
- Pinned by `phiray/tests/solid.rs`: one dispatch, a lit surface brighter than
  the sky, byte-identical frames on repeat, and a pass that re-records when the
  mode changes.

**Wireframe** (`wire.wgsl`, `Mode::Wire`) is the same shape of pass. Where the
primary ray hits, the perpendicular distance from the hit point to each of the
triangle's three edges is `bary_i × 2·area / |edge_i|`; comparing that against
the pixel's world footprint (constant under an orthographic view, growing with
distance under a perspective one) gives a line of constant screen width at any
depth. Because it is a real ray cast it is **hidden-surface correct**: an edge
behind a face does not draw. The CPU stroke path it replaces had no depth test
at all — every edge in the scene drew over every other. The camera's `env`
carries the wire colour in this mode; there is no environment to reflect in a
wireframe. Pinned by `a_wire_behind_a_face_does_not_draw`, which asserts both
that the hidden edge is absent *and* that the same pixel does draw once the
occluder is removed, so the test cannot pass for the wrong reason.

Both modes report `samples() == 1` via `Mode::deterministic()`, so the viewport
paints once and stops polling.

**Why a compute ray-cast and not a raster pipeline.** Blender's Workbench is a
*GPU rasteriser* — a real vertex/fragment pipeline with a hardware depth
buffer. This kernel casts one primary ray per pixel against the BVH instead.
It is not the same architecture, and the honest reason for the difference is
reuse: phiray already owns the GPU, the BVH, the scene upload, the recorded
pass and the skinning kernel, so Solid became ~140 lines of WGSL and a mode
flag rather than a second GPU backend with its own pipeline, render target and
vertex buffers. A hardware rasteriser would still be faster for Solid at high
triangle counts — primary-ray BVH traversal is O(log n) per pixel where a
raster pass is O(1) per fragment — so if Solid ever becomes the bottleneck
again, a wgpu render pipeline writing into the same texture is the next move,
and this kernel is what it would be measured against. The mistake worth naming
is not "raster": it was putting *any* per-pixel loop on the CPU, inside the
paint pass, with no cache.

The CPU painter is now purely the **no-GPU fallback**, not a performance tier. While it was the common path it also carried a silent bug:
`frame_for` computed a reduced supersample factor to stay inside the pixel
budget and then built the buffer at the full factor anyway, so a canvas over
the budget drew nothing at all instead of dropping to 1x. Fixed in the same
change.

`atom/canvas/view3d/`:

| Step | Where | Notes |
|---|---|---|
| Camera | `scene/camera.rs` | Orbits a pivot on Blender's back axis; ortho, or perspective with `dist × 36 mm × 2 / lens` (`BKE_camera_params_from_view3d`). Matches Blender — §0.7 of the read-parity doc. |
| Project | `mesh/kernel.rs::primitive_vertex` | Per-vertex, CPU. |
| Cull | `mesh.rs` | Screen-space winding; backfaces dropped. |
| Shade | `mesh/studio.rs`, via `kernel.rs::lit`, called from `raster.rs` | `get_world_lighting` in full: Blender's four-light Default rig with wrapped terminators, plus Blinn specular split by the material's roughness and metallic — **per pixel**, from a normal interpolated across the triangle. |
| Colour | `kernel.rs::loaded_fill`, `raster.rs` | Solid: `Material.r/g/b`. Material/Rendered: Principled Base Color, or the image texture sampled **bilinearly per pixel** from a UV interpolated across the triangle. |
| Order | — | None. The depth buffer decides per pixel. |
| Raster | `mesh/raster.rs` | Scanline fill with a depth test, 2× supersampled and averaged down, painted as one image. Wireframe still strokes a path per triangle: there is nothing for a depth buffer to resolve in a wireframe. |

### 2.2 What was actually slow on the Female-01 character (2026-09-20)

Measured against Blender on `Female-01-Chatty-Whims-Rigged-Lace.blend`
(305 objects, 8.97 M faces in the file):

| | Blender | Phiber before | Phiber now |
|---|---|---|---|
| Faces the viewport draws | 1,444,810 | 1,444,810 (visibility was already right) | 1,444,810 |
| Faces the overlay *reports* | 1,444,810 | **8,970,212** — it counted the file, not the view layer | 1,444,810 |
| Images uploaded in Solid | none (Workbench reads viewport colour) | every base, normal, roughness map at full size | none |
| Image size cap (Material) | `U.glreslimit`, off by default | none | 2048 (`TEXTURE_LIMIT`) |
| When the GPU declines a scene | n/a | CPU painter took the whole scene → seconds per frame, window stopped answering | painter refuses above `CPU_MAX_TRIS` (400k) and the overlay says why |

The last row is the freeze. The tracer packs all images into one storage
binding (`data = [table…, atlas…]`), so a character's texture set alone could
push the scene past the device limit; the render thread declined it and the
CPU painter inherited millions of triangles inside the paint pass. Now: Solid
and Wireframe upload no images at all (`uses_viewport_color`), Material
downsamples to 2048 on upload (`Atlas::add_bounded`), the shading rides in
the scene key so the two builds never mix, and a declined scene shows a line
in the viewport overlay instead of a frozen window.

### 2.3 The body coming through the clothes

Not an offset. The garments in that file carry
`SOLIDIFY(-0.004, offset -1) → SUBSURF(2) → LATTICE → ARMATURE`, and two of
those were wrong here:

- **Lattice was not implemented** (`eModifierType_Lattice` = 2). Blender
  moves the garments **3.5–4.7 cm on average, 6.3 cm max** with it; without
  it they sit where the skin is. `phiphx/src/blend/modifier/lattice.rs` ports
  `lattice_deform.cc` and `key_curve_position_weights` verbatim, and matches
  Blender's evaluated garment to under 1 mm
  (`tests/lattice.rs::the_lattice_moves_the_garment_where_blender_does`, run
  against the real file).
- **Solidify assigned its two shells the wrong way round for a negative
  thickness.** Blender decides by which offset is larger
  (`INIT_VERT_ARRAY_OFFSETS`), not by name: without FLIP the original
  vertices take `max(ofs_orig, ofs_new)`. Ours always gave them
  `(1+offset)/2·thickness`, so for `-0.004` the correctly-wound surface stayed
  on the body and only the reversed copy moved out.

### 2.4 Measured against Blender's Workbench (2026-09-20)

**To run the comparison yourself** (PowerShell, from the workspace root; put
Blender on PATH or use its full path):

```powershell
$f = "crates\phiber\assets\CW-Fashion\Epic-Pods-Female-01\Female-01-Chatty-Whims-Rigged-Lace.blend"
blender -b -P crates\phiphx\tests\truth\blender_front.py -- $f blender_front.png
cargo run --release -p phiber --example workbench_render -- $f phiber_front.png
cargo run --release -p phiber --example render_compare  -- blender_front.png phiber_front.png side_by_side.png
```

`render_compare` prints the mean difference on the figure, each render's
mean brightness, and *edge softness* — transition pixels per silhouette
pixel, 0 = hard, contrast factored out — and writes the two images side by
side. Today: difference 14.7 / 255, softness Blender 1.02 vs Phiber 1.05
(both one-sample hard edges; equally crisp).


`examples/workbench_render.rs` renders a file the way the viewport does —
phiphx stack, view-layer visibility, `Mode::Solid` — from a fixed orthographic
front view; `blender_front.py` renders the same view through Blender's
Workbench (studio light, material colour, no cavity / shadow / outline /
AA, Standard view transform). On Female-01, after the Lattice, Solidify and
view-transform fixes: **mean absolute pixel difference on the figure 14.7 /
255**, mean brightness 139 (Blender) vs 125 (Phiber). The garments sit over
the body in both and their seams land in the same places
(`docs/female01_blender_vs_phiber.png`). What remains visible: Blender's
edges are a touch softer, and the hair reads lighter in Blender because its
specular comes from material roughness / metallic — which the viewport path
passes and the example does not.

### 2.5 Material Preview in colour, measured against Blender (2026-09-22)

`examples/color_render.rs` renders Material Preview through phiray from the
front view of `phiphx/tests/truth/blender_color.py` (Blender's Material
Preview setup: studio HDRI, no scene lights, flat grey background), and both
print their timings. Fixed on the way, each checked against Blender's own
numbers on Female-01:

- **Solidify read the wrong flag bits** (`RIM` is bit 0, `FLIP` bit 5 —
  `DNA_modifier_types.h`), so garments lost their rim, and it cleared UVs on
  every solidified mesh. Now: rim quads wound against their owning face,
  shell and rim carry UVs. Top and pants match Blender's evaluated vertex and
  triangle counts exactly (8,800 / 17,600 and 36,734 / 73,472).
- **Procedural colour graphs** folded to nothing when they passed through
  Ambient Occlusion, Texture Coordinate or Mapping, and the hair fell back to
  light grey on a 0.97-metallic surface (silver). Those three nodes now fold.
- **Missing images** folded to white. Blender draws a texture it cannot find
  as magenta, and this file's garment masks live in a folder that is not
  shipped — so Blender shows the clothes pink. Phiber now does the same
  (`image_present`, resolved against the `.blend`'s folder).
- **View transform.** The tracer used an ACES curve Blender does not have.
  `Camera::view` now carries the scene's `view_settings` (read from
  `Scene.view_settings`): Standard, Filmic (Blender's curve, sampled — exact
  per channel) and AgX (Blender's curve between fitted inset/outset matrices,
  mean error 1.6/255), plus exposure and gamma. The accumulation buffer keeps
  scene-linear sums for `Tracer::radiance` and display sums for the pixel.

Open: MC grain at 32 samples (Blender's EEVEE preview is raster, noise-free —
needs a denoiser or more samples for a still), studio HDRI lighting (Phiber
lights Material Preview with a flat grey environment and a view sun), and the
two hair materials built from nodes current Blender also does not recognise.

## 3. Where the quality is lost, in order of how much it costs

1. ~~**No depth buffer.**~~ Fixed (§0).
2. ~~**Per-triangle normals.**~~ Fixed: `BTriangle` carries a normal per corner
   and the rasteriser interpolates them, so a smooth mesh reads smooth.
   Pinned by `shading_varies_across_a_face_whose_corners_face_different_ways`
   and `a_flat_face_shades_evenly`.
3. ~~**Vector paths are the wrong primitive.**~~ Fixed: one image per frame
   instead of a path per triangle.
4. ~~**One texel per triangle.**~~ Fixed: `BTriangle` carries a UV per corner
   and the texture it addresses; `SceneTexture::sample_smooth` samples
   bilinearly per pixel. Pinned by `a_textured_face_varies_across_itself`.
5. ~~**One light, flat.**~~ Fixed: `mesh/studio.rs` is Blender's four-light
   Default rig (`BKE_studiolight_default`) applied through
   `wrapped_lighting` from `workbench_world_light_lib.glsl`.
6. ~~**No specular.**~~ Fixed: `blinn_specular`, `brdf_approx`, the
   metallic/dielectric split and the wrapped `spec_env` blend are all ported.
   `BTriangle` carries `roughness` and `metallic`, and `lit` shades in linear
   RGB so a metal's highlight keeps the surface's own tint.
7. **No shadows, no cavity, no ambient occlusion.** Workbench's
   `workbench_effect_cavity` and `workbench_shadow_*` carry a lot of what
   makes Blender's Solid mode readable on dense geometry. This is now the top
   remaining defect. Antialiasing is 2× supersampling, which is adequate.
8. ~~**Exposure is hand-set.**~~ Fixed 2026-09-20: the rig's scene-linear
   output goes through the sRGB transfer curve — Blender's *Standard* view
   transform — in both the WGSL kernel (`util.wgsl::srgb`) and the CPU
   fallback. `STUDIO_GAIN` is gone; there is no non-Blender number left in
   the shading path. Blender renders the same rig
   through Standard/Filmic. Porting a view transform would remove the last
   magic number in the shading path.

Note what is *not* on this list: the geometry itself. The reader matches
Blender exactly on vertex and face counts, UVs, material indices and smooth
flags across 1,276 meshes in 8 files (`phiphx/tests/blender_truth.rs`), and the
modifier stack's silhouettes match Blender's own renders at IoU ≥ 0.999
(`phiber/tests/viewport_vs_blender.rs`). **The data is right; the shading is
not.**

## 4. The order to fix it

Each step is independently shippable and independently measurable.

**Step 1 — Measure interiors, not just silhouettes.**
`viewport_vs_blender.rs` compares *silhouettes* (alpha masks), which cannot see
a garment ordering error at all — FLORIS scored 1.000 while its clothes were
vanishing. `phiphx/tests/truth/blender_object_map.py` (new) renders each mesh
in a flat emission colour keyed to its index, so the ground truth says **which
object owns each pixel**. Comparing that against a software z-buffer of our own
scene gives a per-object visibility score. Until this exists, no rendering
change can be honestly called an improvement.
*Blocked on:* framing the test views so the subject fills the frame, and
excluding `WGT-` rig widgets (Blender hides them; the map currently includes
them).

**Step 2 — Software z-buffer for the CPU path.** ✅ *Done 2026-09-20.*
`mesh/raster.rs`: a BGRA colour buffer with a depth buffer, scanline fill with
a per-pixel depth test, 2× supersampled and box-downsampled for antialiasing,
bounded at 8M pixels, painted through `window.paint_image`.

**Step 3 — Per-pixel shading on that buffer.** ✅ *Done 2026-09-20.*
Normals and UVs both interpolate across the triangle; textures sample
bilinearly per pixel; lighting is Blender's four-light Default studio rig with
wrapped terminators, ported from `BKE_studiolight_default` and
`workbench_world_light_lib.glsl`.
Specular landed too: `get_world_lighting` is ported whole — Blinn on the half
vector, `brdf_approx` fresnel, the metallic/dielectric split and the wrapped
`spec_env` blend for soft lights. Shading runs in linear RGB.
Remaining in this area: cavity and shadow (§3.7), and a view transform to
replace the hand-set exposure (§3.8).

**Step 4 — Overlay pass.**
Port `overlay_grid_frag` first — the grid defines the look of an opened file
more than anything else — then `overlay_outline_*` for selection, then
`overlay_extra_*` shapes. These draw against the depth buffer from step 2.

**Step 5 — GPU path.** ✅ *Shading done 2026-09-20.*
All four shading modes render through phiray over the same BVH: `Mode::Wire`,
`Mode::Solid`, and the tracer for Material and Rendered (§2.1). The CPU painter
is reached only when there is no GPU adapter. Remaining here: the **overlay
pass** (step 4) — grid, selection outlines, gizmos and extras — still draws on
the CPU, over the GPU image.

**Step 6 — Rendered mode.** ✅ *Already built.*
`crates/phiray` is a WebGPU compute path tracer over a `GpuBvh`
(`phiray::Tracer`, `Bvh::build_compact`), and `view3d/rendered/` already feeds
it the scene, camera and lights on a render thread. Material Preview uses a
studio rig; Rendered uses the file's own lights and world. It falls back to the
mesh painter when there is no GPU adapter.

What is *not* done: the `PHIRAY` entry in Properties → Render → Engine is a
selectable value with nothing behind it. The tracer is wired to the viewport's
**shading mode**, not to the scene's render engine. Connecting them — and
rendering a frame at the scene's resolution into the Rendering workspace's
Image Editor — is the open work.

## 5. Phiray, and a straight answer about NVIDIA Warp

**Phiray** is a real, working GPU path tracer: `crates/phiray`, a WebGPU
compute kernel over a compacted BVH, already driving Material Preview and
Rendered mode in the viewport and the GPU skinning path.

What is not connected is the *engine enum*. `PHIRAY` in
Properties → Render → Engine (`part/panel/render/set.rs`) is a selectable value
with nothing behind it: the tracer keys off the viewport's shading mode
instead. A `.blend` never names PHIRAY, so it can only ever be chosen there.

**NVIDIA Warp is not a renderer.** It is a Python framework for writing GPU
kernels for simulation — Newton physics builds on it. There is no Warp renderer
to borrow, and no way to embed a Python GPU framework inside a Rust/wgpu
application. If the appeal is "NVIDIA's GPU power", the honest routes are:

- **WGSL compute** in our own wgpu pipeline (step 5/6). Runs on NVIDIA, AMD,
  Apple and in the browser. This is what phiray's path tracer already is.
- **OptiX / RTX** would mean a CUDA dependency, NVIDIA-only builds, and no web
  target. Blender's own Cycles does this behind a device abstraction; it is a
  large, separate project.

If what you want is Warp's *simulation* (cloth, soft bodies, particles), that is
a real and separate thing worth discussing — but it feeds geometry into a
renderer, it does not replace one.

## 6. The Rendering workspace

Blender's `Rendering` workspace is a **single full-width Image Editor** on
Render Result — `versioning_defaults.cc:283` hides the image tool header for it
by name, which is how you can tell it is a `SPACE_IMAGE` workspace. Ours
declared View3D beside an Image Editor; it is now one Image Editor, pinned by
`rendering_is_one_full_width_image_editor`.

What it shows is still the Image Editor's placeholder checkerboard. It becomes
useful at step 2, when there is a rendered buffer to put in it.

## 7. Open items from the read side that affect the picture

- Subdivision **creases** and UV smoothing (`crease_edge` / `crease_vert`).
- Armature **envelopes**, dual quaternions, B-Bones, `PARSKEL`.
- Modifiers still missing: Bevel (display), Array caps / merge / fit, Solidify
  complex mode, Displace with a texture.
- The full material **node graph** (§25 of the read-parity doc): only Principled
  Base Color and the first Image Texture are read today.
