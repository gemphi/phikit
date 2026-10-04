# Animation and Rendering — where each stands (batch 101, revised batch 113)

A full audit of the two workspaces against Blender 5.2 LTS (the source at `refs/blender/blender`, `BKE_blender_version.h` 502), taken from the code and
its tests rather than from the batch notes; where the two disagreed the code
won (two `jobs.md` lines were stale: driving objects and bones from the frame,
and Bézier evaluation with layered actions, are both done). "Inert" means an
entry is on screen and in Blender's place, but picking it, or its shortcut,
only says "not available yet" in the status bar and the Info editor.

| Workspace | Plays / renders what Blender does | What you can edit | Inert menu entries |
|---|---|---|---|
| Shading / ECS Wand / Compositing | Laid out; a placeholder two-node graph, not the file's tree | nothing yet | all |
| Animation | Mostly — objects, NLA, rigs, constraints, IK, rig drivers | Insert keys (I); in the Dope Sheet pick, move and delete keys (batch 98); markers, actions, NLA strips and tracks | about 390 across the Dope Sheet, Graph, Drivers, NLA and Timeline |
| Rendering | Partly — one path tracer for every engine; Principled BSDF only | Resolution (batch 97), colour management (batch 99), samples, bounces, denoise, output format; the Rendering workspace opens on a render (batch 99); any size renders whole and Esc or the status bar's X stops it (batch 100) | about 50 in the Image editor |

---

## Animation

### Playback and evaluation — done, checked against Blender

- **Actions.** Layered (4.4+) and legacy actions; F-curves constant, linear,
  Bézier and all ten easing kinds; auto and auto-clamped handles with
  Blender's smoothing; constant and linear extrapolation
  (`phiphx/blend/anim/`; `tests/playback_moves_the_scene.rs`,
  `handles_truth.rs`).
- **NLA.** Tracks bottom to top, solo, mute, hold, scale and repeat, every
  blend mode and influence, blend in / out; the action on top
  (`anim/nla_eval.rs`, `nla_truth.rs`).
- **Rigs.** Every bone posed through its inheritance, Blender's legacy IK
  ported step for step, and the constraints: Child Of, Track To, Damped
  Track, Locked Track, the Limits, the Copies, Stretch To, Transformation,
  Armature, Action, Pivot, Maintain Volume, Floor, Follow Path, Clamp To.
  THYMUS's 328 bones match Blender at six frames (`rig/tests.rs`).
- **Drivers on a rig** — sums, averages, min / max and simple expressions,
  writing channels, constraint influence and mute, Armature weights and
  properties.
- **In the viewport** — GPU skinning, posing on its own thread ahead of the
  playhead, frame dropping, the five loop modes, play in reverse.

**Not evaluated yet:** Spline IK and Shrinkwrap; drivers on anything but a
rig (objects, materials, shape keys); driver variables other than a single
property (Transform Channel, Rotational Difference, Distance, Context
Property read as 0); F-curve modifiers other than the Generator (Cycles,
Noise, Limits, Envelope, Stepped); shape-key animation; Grease Pencil.

### Timeline — done for playing, partial for its menus

- **Works:** the transport, frame fields, preview range on / off and Clear,
  scrubbing, playhead snapping and its popover, the Playback popover (read
  from and written to the file), keyframe and range jumps, the sidebar,
  markers (add, select, box select, move with G, rename with F2, delete,
  jump; saved).
- **Works since batch 113:** the time axis zooms under the wheel
  (`view2d.zoom_in` / `zoom_out`, 3.75 % a notch about the centre, held
  between 0.01 and 50 pixels a frame), scrolls under Ctrl wheel (forty
  pixels a notch), and Home / View ▸ Frame All frames the shown keys (or
  strips, or the scene range when nothing is keyed) with four diamonds of
  margin; View ▸ Frame Scene Range frames the scene or preview range.
  Each editor keeps its own `View2D.cur`, as each region does in Blender
  (`TimeViewKey`). Ported from `view2d_ops.cc` (`view_zoomstep_apply_ex`,
  `view_pan_apply_ex`), `action_edit.cc` (`actkeys_viewall`,
  `get_keyframe_extents`), `anim_ops.cc` (`scene_range_frame_exec`) and
  `anim_draw.cc` (`ANIM_frame_range_view2d_add_xmargin`).
- **Inert:** View ▸ Show Seconds, Sync Visible Range; Set Preview Range
  (P); Marker ▸ Duplicate, Lock, Bind Camera to Markers (Ctrl B).
- **Missing:** panning with the middle mouse (`view2d.pan`), Shift wheel
  over the channel list, Zoom to Mouse Position (off in Blender's defaults,
  so the zoom is about the centre here too), the Graph editor's value axis
  under Frame All (`graphkeys_viewall` frames both axes; x only here), and
  the file's own `ARegion.v2d.cur` — the editors open on `action_create`'s
  default rather than where the file left them.

### Dope Sheet — display done, key editing started (batch 98)

- **Works since batch 98:** click, Shift click and box selection of key
  columns, A / Alt A / Ctrl I, moving the picked keys by a drag or G (one
  undo step, Escape puts them back), X / Delete; saved, and Blender reads
  every result back with the same handles. Only the scrub band scrubs.
- **Works:** the summary row and a row per object with its keys; the Action
  Editor's action and slot selector (browse, New, Unlink); the sidebar's
  Action panels (Manual Frame Range, Cyclic — saved); Push Down and Stash;
  Insert Keyframes (I) on every shown curve; filters and search; N, Alt P.
- **Inert:** about 95 entries — Select (Box from the keyboard, Circle,
  Lasso, Columns, Before / After Current Frame), Channel (Delete, Group,
  Mute, Protect, Extrapolation, Bake), Key (Extend, Scale, Slide, Copy /
  Paste, Duplicate, Interpolation, Handle Type, Easing, Keyframe Type, Snap,
  Mirror, Clean, Bake), Action (Merge, Separate Slots, Move Slots).
- **Missing:** a row per F-curve and per group (rows stop at the object, so
  a whole column is picked at once; no expanders or channel toggles); X's
  confirmation; the Shape Key, Grease Pencil, Mask and Cache File modes draw
  the Dope Sheet's content; key edits cannot be saved into a file older than
  Blender 4.4 (THYMUS).

### Graph Editor — display done, editing not started

- **Works:** every curve drawn as Blender draws it (axis colours, handles,
  selected and active, muted grey), its own view and scrub, Insert ▸ All
  Channels.
- **Inert:** about 130 entries, among them Key ▸ Transform, Snap, Mirror,
  Density, Blend, Smooth and Add F-Curve Modifier.
- **Missing:** selecting and moving keys and handles, handle and
  interpolation types on existing keys, the per-curve channel list, the
  F-Curve and Modifiers sidebar tabs, zoom and pan.

### Drivers editor — display done

- **Works:** the driver curves of the shown objects with Blender's names,
  the input along X.
- **Inert:** about 120 entries. **Missing:** the Drivers panel (variables,
  expression, type), adding or copying a driver.

### NLA — editing started

- **Works:** drawing; click select of strips and tracks; Select All / None /
  Invert; Delete; Toggle Muting; Tracks ▸ Add, Add Above Selected, Delete;
  Add ▸ Action (Shift A); all saved and read back by Blender.
- **Inert:** about 50 entries — Split, Duplicate, Transform, Snap, Make
  Meta, Bake, Sync Length, Start Tweaking (Tab), box select, Add ▸
  Transition, Sound, Selected Objects.
- **Missing:** tweak mode, dragging strips, the strip's sidebar properties.

### Keying

- **Works:** I in the viewport keys Location, Rotation and Scale with
  Blender's handles, making the action and slot when needed; saved, and
  Blender reads every key back.
- **Works since batch 99:** auto keying — a G, R or S kept with the record
  button on keys the transform's channels at the shown frame, Only Insert
  Needed and Replace as Blender has them; Delete Keyframes (Alt I) and Clear
  Keyframes.
- **Missing:** keying sets (K), per-property I, the auto-keying warning.

---

## Rendering

### F12 and Ctrl F12

- **Works:** F12 renders on a worker through the file's active camera at the
  shown frame — depth of field, orthographic and focus object included —
  with progress in the header and the info bar; objects hidden for render
  are left out. Ctrl F12 renders the range and writes numbered frames to the
  Output path.
- **Fixed in batch 101:** a cancelled render keeps the image as far as it
  got, as Blender keeps its finished tiles, and an animation render stops
  at that frame; a running render draws the window when a sample lands
  rather than every vsync (the viewport's ticker, batch 99, serves both).
- **Fixed in batch 100:** a frame larger than the device binds as one image
  is rendered in bands (phiray `Camera.tile`) and joined, so a 4K frame
  renders whole on a 128 MiB software GPU; Esc in the render window and the
  X beside the status bar's "Rendering" progress stop a render, which says
  how far it got.
- **Fixed in batch 99:** the Rendering workspace renders the scene when it
  opens (and when a file opens in it); a render thread that dies on the GPU
  clears the job instead of leaving it "running".
- **Fixed in batch 97:** the size. F12 rendered the file's resolution
  whatever Output ▸ Format said; it now renders the panel's Resolution X, Y
  and %, the camera's frame in the viewport takes its shape from them, and
  Ctrl S writes them (`r.xsch / ysch / size`).
- **Batch 116:** FFmpeg Video writes one movie per Ctrl F12 — the
  frames gathered and encoded with the file's `ffcodecdata` (container,
  codec, CRF, preset, keyframe interval; the Output ▸ Encoding panel
  shows and overrides them) through the `ffmpeg` program when present,
  else a hand-written Motion-JPEG AVI (`scene/video`), named
  `BKE_movie_filepath_get`'s way (`shot_0001-0140.mkv`). Esc anywhere
  cancels a render (115); the Image editor's header carries Cancel and
  the progress (116).
- **Gaps:** the engine choice is ignored (Eevee, Workbench and Cycles all
  render with phiray); no bundled encoder (ffmpeg is the machine's), no
  audio, the Encoding panel is not written back to the file; an
  animation render leaves the playhead on the last frame and ignores
  Overwrite, Placeholders and the frame step.

### Image editor (Render Result)

- **Works:** the Render Result at fit, zoom 1:8 – 8:1, Frame All, the
  render-info bar; the Slot, View Layer, Pass and Channels selectors; Image ▸
  Save, Save As, Save a Copy in PNG, JPEG, BMP, TIFF, Targa, WebP, EXR (half
  and float, and multi-layer with Combined, Depth, Normal, Diffuse Color) and
  Radiance HDR.
- **Inert:** about 50 entries — Image ▸ New, Open, Replace, Reload, Save All,
  Resize; Render Region (Ctrl B); Render Slot cycle (J).
- **Missing:** render slots beyond the first (eight labels, one image),
  reading a pixel's value, the compositor, metadata.

### Properties ▸ Render and Output

| Setting | Read from the file | Changes the render | Saved |
|---|---|---|---|
| Resolution X, Y, % | yes | **yes (batch 97)** | **yes (batch 97)** |
| Samples, Diffuse bounces, Denoise | yes (a 5.0 file's since batch 102) | yes | **yes (batch 102)** |
| Preview samples | yes | viewport | **yes (batch 102)** |
| Output path, file format, colour mode, depth | yes | how it is saved | **yes (batch 102)** |
| View transform, exposure, gamma | yes | **yes (batch 99)** — the viewport and F12 | **yes (batch 99)** |
| Render engine | yes | no (always phiray) | **yes (batch 102)** |
| Other Cycles settings (adaptive, time limit, other bounces, caustics, filter width, denoiser, filter type) | yes (the enumerations since batch 104) | no | **yes (batches 102, 104)** |
| Film Transparent | yes (batch 103) | **yes (batch 103)** — F12's background at alpha 0, RGBA saves cut away | **yes (batch 103)** |
| Motion blur | panel only | no | no |
| Pass toggles | yes | Combined, Depth, Normal, Diffuse Color only | no |

### What the tracer can and cannot draw

- **Can:** sun, point, spot and area lights with soft shadows; emissive
  faces; world colour or HDRI with importance sampling; the Principled BSDF
  (base, roughness, metallic, IOR, transmission, emission, subsurface as a
  wrap, sheen, coat, anisotropy), image textures, normal maps, alpha
  cut-outs; bladed bokeh; Standard, Filmic and AgX with exposure and gamma;
  a guided denoiser; Depth, Normal and Diffuse Color passes.
- **Cannot yet:** a Mix Shader's factor per pixel (its two closures are
  two lobes since batch 141; the closure nodes fold to Principled
  parameters since 140; volumes fold to nothing); procedural textures
  (Noise, Voronoi fold to one colour);
  Bump; Sky Texture; Film Transparent (alpha is always opaque); motion
  blur; volumes; caustics; holdout and shadow catcher; adaptive sampling;
  Mist, AO, Position, Vector, UV, index and light passes; Cryptomatte.

---

## The viewport, and Blender's way of drawing it (batch 113)

Blender's Solid, Wireframe and Material Preview are rasterizers — Workbench
(`source/blender/draw/engines/workbench/`: `workbench_mesh_passes.cc`,
`workbench_world_light_lib.glsl`, `workbench_matcap_lib.glsl`,
`workbench_effect_antialiasing.cc`, `workbench_effect_outline.cc`,
`workbench_effect_cavity.cc`) and EEVEE — over the draw manager's one GPU
batch per mesh (`DRW_cache_mesh_surface_get`), placed by the object's
matrix. Only Cycles' Rendered mode path traces. Phiber runs all four modes
through its path tracer (phiray): one pipeline, but a ray tracer doing a
rasterizer's job for the three modes people spend their day in — the cost
grows with pixels × scene depth and every frame is read back from the GPU
and uploaded as a picture. Batches 108–112 made that path faster
(deterministic Material Preview, samples in slices, the ticker, objects as
instances); it remains the wrong engine for Solid.

**Batch 114 — done: the prepass.** Solid and Wireframe rasterize their
primary hits through a wgpu render pipeline over the tracer's own buffers
(the face's vertices pulled by id, placed by batch 112's instance table —
the batch cache is the scene's storage), the depth test keeping the
nearest, and the kernels shade what it found (`rendering.md` §24):
Workbench's prepass + composite, pixel for pixel the traced image. On by
default on hardware; a software adapter traces (its rasterizer is the
slower of the two). **Batch 118:** Workbench's TAA — eight jittered
frames (`BLI_jitter_init`, recentred), the 3 × 3 Blackman-Harris splat
(`workbench_effect_antialiasing.cc`), ported step for step; Solid and
Wireframe settle in eight samples, the first the sharp one, bands overlap
a row so they join (`rendering.md` §25); SMAA on the first frames and
the counts: batch 139 (§39). **Batch 119:** the object outline
(`workbench_effect_outline.cc`) and the curvature effect
(`workbench_curvature.bsl.hh`) in an `Fx` pass before the gather, the
settings from the file's `View3DShading` (`rendering.md` §26). **Batch
120:** the Shading popover's Options and Cavity panels switch them in the
app. **Batch 121:** Lighting (Studio / Flat), Color (Material / Object /
Random / Single) and Background (Theme / World / Viewport) in the popover
and the kernel (`rendering.md` §27). **Batch 122:** Cavity ▸ World — the
screen-space ambient occlusion, EEVEE's blue noise and the Hammersley
disk (§28). **Batch 123:** Shadow — the directional light's shadow ray
with `get_shadow`'s terminator (§29). **Batch 124:** Backface Culling and
Specular Lighting (§30). **Batch 125:** Color ▸ Attribute (§31). **Batch
126:** MatCap lighting with Blender's 26 matcaps (§32). **Batch 127:**
Color ▸ Texture (§33). **Batch 128:** F12 follows the engine — a
Workbench file renders Solid with `Scene.display.shading` (§34). **Batch
129:** the user's own matcaps from Blender's config folder (§35). **Batch
130:** the colour attribute through the modifier stack. **Batch 131:** an
object's Shadow switches — Viewport Display's for Solid, Ray Visibility's
for the ray-traced modes (§36). **Batch 132:** their checkboxes in the
Properties editor, saved to the file (§37). **Batch 133:** Depth of Field
through the camera, Workbench's five-shader gather (§38). **Batch 134:**
the Render menu entire — F11, Ctrl F11, Render Audio…, Lock Interface,
and F12 on a fresh session. **Batch 135:** the eight render slots. **Batch 136:** images in the Image
editor — New, Open, Invert, Flip, Rotate, Copy, Reload, Save As. **Batch
137:** Render Region (Ctrl B, Crop). **Batch 138:** the viewport's own
render region outside the camera view (View ▸ Render Region), traced
alone under Rendered shading. **Batch 139:** SMAA on Solid's first
frames and the anti-aliasing sample count — Preferences ▸ Viewport ▸
Quality, `render_aa` for a Workbench render (§39). **Batch 140:** the
surface shader from the Material Output — Mix / Add Shader and the
closure nodes folded to the Principled parameters, Light Path for a
camera ray (§40). **Batch 141:** a Mix Shader's two closures as two lobes
— a coin per hit in the path tracer, both blended in Material Preview
(§41). **Next:** per-pixel Noise / Voronoi and Bump; then Material
Preview on EEVEE's model.

## What to do next, in order

**Animation**

1. ~~Select, move (G) and delete (X) keys in the Dope Sheet~~ — batch 98.
   The Graph editor's keys and handles next: the same steps
   (`scene/action/retime.rs`) with a hit-test on its curves. Large.
2. ~~Auto keying on transform confirm, and Delete Keyframe (Alt I)~~ — batch 99.
3. Channel rows per F-curve and group, with mute and lock. Large.
4. ~~Zoom, scroll and Frame All for the animation editors~~ — batch 113.
   Still: middle-mouse pan, Set Preview Range (P), the file's `v2d.cur`.
   Small.
5. The Key menu over a selection: Copy / Paste, Interpolation, Handle Type,
   Easing, Snap, Mirror (needs 1). Medium.
6. F-curve modifiers: Cycles first, then Noise, Limits, Stepped. Medium.
7. Drivers beyond rigs, and the other variable types. Large.
8. Shape-key animation and the Shape Key editor. Large.
9. NLA tweak mode and strip dragging. Large.
10. Spline IK and Shrinkwrap. Large.

**Rendering**

0. ~~Workbench's prepass for Solid and Wireframe~~ — batch 114; ~~its
   TAA~~ — batch 118; ~~the object outline and curvature~~ — batch 119.
   ~~The Shading popover (Options, Cavity)~~ — batch 120; ~~Lighting,
   Color, Background~~ — batch 121; ~~Cavity ▸ World (SSAO)~~ — batch 122;
   ~~Shadow~~ — batch 123; ~~Backface Culling, Specular Lighting~~ —
   batch 124; ~~Attribute colour~~ — batch 125; ~~MatCap~~ — batch 126; ~~Texture colour~~ — batch 127.
   ~~`Scene.display.shading` for F12~~ — batch 128; ~~Depth of Field~~ —
   batch 133; ~~SMAA on the first frames~~ — batch 139. Then Material Preview on
   EEVEE's model. Medium each.
1. ~~The panel's resolution reaches F12 and is saved~~ — batch 97.
2. ~~The colour-management panel seeded from the file, read by F12 and the
   viewport, and saved~~ — batch 99 (Look is still not read).
3. ~~Cancel a render (Esc, the render window's button), and tiles for frames
   larger than the GPU binds~~ — batch 100.
4. ~~Save the rest of Render and Output (samples, bounces, denoise, engine,
   output path and format)~~ — batches 102 and 104.
5. ~~Film Transparent through the tracer and RGBA saves~~ — batch 103
   (transparent glass under it still to do).
6. Shader coverage: Mix / Add Shader, Emission, Diffuse, Glossy, Glass,
   Transparent; Bump; procedural Noise and Voronoi per pixel — each from
   `intern/cycles/kernel/svm/` (`svm_noise.h`, `svm_voronoi.h`,
   `svm_closure.h`) and the node graph compiled as `shader/graph.cpp`
   folds it. Large.
7. Render slots, Image ▸ Open / New / Reload. Medium.
8. Sampling controls: adaptive threshold, time limit, the other bounces,
   clamping. Medium.
9. More passes: Mist, AO, Position, Vector, UV, Emission, index. Medium.
10. ~~Video output~~ — batch 116 (through ffmpeg, or MJPEG AVI). Motion
    blur. Large.
