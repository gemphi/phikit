# Rendering, saving, exporting and playing — step by step (batches 114–116)

The example file used below:

`C:\Users\phiac\Workspace\gemphi\phiace\crates\phiber\assets\CW-Fashion\Epic-Pods-Female-01\Female-01-Chatty-Whims-Rigged-Lace.blend`

(319 objects, 4.5 M vertices, 9 M triangles with modifiers, rigged, three
lights, Film ▸ Transparent on, Cycles at 1920 × 1920 with 2412 samples,
Filmic). **It is a pose file, not an animated one**: its 69 actions are
one-frame poses (`Eyes-Smile` at frame 2, `Costume-06-Full-Pose-03` at
frame 3, …) on 18 NLA tracks — the keys you see at frames 0–8 — so playing
it steps through poses and then holds.

**The file with a real animation** is

`C:\Users\phiac\Workspace\gemphi\phiace\crates\phiphx\tests\fixtures\THYMUS-BOLDROID-WAVE-003.blend`

(259 objects, a 328-bone rig, `rigAction` keyed over frames 0–132 with
530 curves, scene range 1–140 at 24 fps, camera `Camera.001`): the wave.
It opens in about two seconds and is the file to use for playback and for
rendering a video. `FLORIS_BOLDRIOD_RIG.blend` is a rig in one pose.

## 1. Open

File ▸ Open (Ctrl O), pick the `.blend`. The status bar and the Info editor
show the stages (indexing blocks, reading objects, evaluating modifiers,
reading actions, motion, drivers, building the scene). Female-01 opened in
about 24 s on the GTX 1660 Ti machine before batch 117 (evaluating
modifiers 6.8 s, reading actions 5.2 s, reading motion 8.9 s); the DNA
lookups those stages spent two thirds of their time in are now worked
out once per file, and the cloth subdivisions run on all cores — 28 s →
10 s in the two-core test container (`frame-cost.md`). THYMUS opens in a
third of a second. The first viewport image lands 2 s after the open.

## 2. Settings before a render (Properties ▸ Render and Output tabs)

| Setting | Where | For Female-01 |
|---|---|---|
| Resolution X, Y, % | Output ▸ Format | the file says 1920 × 1920 at 100 %; 50 % for a quick look |
| Max Samples | Render ▸ Sampling ▸ Render | the file says **2412** — that is why the render took 3 min 59 s. 128 is a quick preview, 512 a clean frame |
| Denoise | Render ▸ Sampling ▸ Render ▸ Denoise | on (the file's) |
| Film ▸ Transparent | Render ▸ Film | **on in this file**: the background is cut away, so the render shows black where Blender would show a checkerboard, and a PNG saves with alpha 0 there. Turn it off for the world colour |
| View transform, exposure, gamma | Render ▸ Color Management | the file's Filmic |
| Output path, format | Output ▸ Output | the file says `/tmp/` and PNG RGBA 8-bit — set a Windows folder, e.g. `C:\Users\phiac\Renders\shot_` (a `//` path is beside the `.blend`) |
| Render Engine | Render tab, top | shows the file's choice (Cycles here) but **every engine renders with phiray** — the one gap in this table |

All of these are read from the file, drive the render, and save with Ctrl S
(batches 97–104).

## 3. Render a still

1. Render ▸ Render Image (**F12**). The Rendering workspace opens on the
   Image Editor with the Render Result; the header shows progress; Esc or
   the X beside the status bar's "Rendering" stops it and keeps what was
   traced.
2. The info line over the image says `Frame | Time | Mem | Scene, ViewLayer
   | Finished`, as Blender's does.
3. Image ▸ Save As… (**Shift Alt S**) in the Image Editor — the file browser
   opens; the format is the Output panel's (PNG, JPEG, BMP, TIFF, Targa,
   WebP, EXR half / float / multi-layer, Radiance HDR); Image ▸ Save (Alt S)
   writes to the Output path directly.
4. Pass and Channels selectors in the header switch between Combined,
   Depth, Normal and Diffuse Color; a multi-layer EXR saves them all.

## 3a. Stopping a render

- **Esc**, anywhere, while a render runs (batch 115 — Blender's
  `G.is_break`: a plain Esc cancels a render whatever else has the
  keyboard). Before 115 only Esc *in the render window* (F11) did; the
  Rendering workspace needed the status bar.
- The **Cancel button in the Image Editor's header** (batch 116), beside
  "Render Result" with the render's progress bar, while it runs.
- The **X beside "Rendering …"** in the status bar, while it shows
  (Blender's job bar, `WM_jobs_stop`).
- A cancelled render keeps what it traced (as Blender keeps its finished
  tiles); an animation render stops at the frame it reached.
- The header's Time is the render's own clock. Frame 207 of Female-01 at
  2412 samples took 18 min 39 s with the viewport playing the animation
  at the same time — both on the one GPU. Stop playback (Esc, or the
  pause button) before a long render, and set Max Samples first.

## 3b. Blender's render controls, and which exist here

| Blender (Cycles) | Phiber | Notes |
|---|---|---|
| Render Engine (Eevee / Workbench / Cycles) | read, saved, **ignored** | one tracer renders them all |
| Device (CPU / GPU) | Preferences ▸ System ▸ Render Devices; the Render tab shows it | picks the wgpu adapter |
| Sampling ▸ Viewport: Max Samples, Denoise, start sample | applied, saved | the viewport's Rendered mode |
| Sampling ▸ Render: Max Samples | applied, saved | the only knob that sets render time |
| Noise Threshold / Adaptive, Min Samples, Time Limit | read, saved, **not applied** | every pixel takes Max Samples |
| Denoise, denoiser choice, passes | our a-trous denoiser applied; the choice read and saved | not OpenImageDenoise |
| Light Paths ▸ Max Bounces (Diffuse) | applied, saved | Glossy / Transmission / Volume bounces read, saved, not applied; Clamping not applied |
| Caustics, Fast GI | read, saved, not applied | |
| Volumes | — | no volumes |
| Motion Blur | panel only | no |
| Film ▸ Transparent | applied, saved | Pixel Filter width read, saved, not applied |
| Performance (threads, tiles, memory) | — | the tracer tiles by the GPU's limit on its own |
| Color Management: view transform, exposure, gamma | applied, saved | Standard, Filmic, AgX; Look not read |
| Output: resolution, %, frame range, path, format, color depth | applied, saved | PNG, JPEG, BMP, TIFF, Targa, WebP, EXR, HDR |
| Output ▸ Encoding: container, codec, quality (CRF), speed, keyframe interval, bitrate | applied (batch 116), read from the file; not saved back yet | one movie per Ctrl F12 through ffmpeg; MJPEG AVI without it; no audio |
| View Layer passes | Combined, Depth, Normal, Diffuse Color | the rest listed, not rendered |

**What limits "Blender quality" today is not the sampler** — a 512-sample
frame here converges like Cycles' (`rendering.md` §7, §18) — but what the
materials carry into it: only the first Principled BSDF of a material is
read, so a Mix Shader, an Emission, Glass, Transparent, a Noise or
Voronoi texture, or a Bump node collapses to a flat colour; and a
texture the file does not pack, and whose path is not found, is a flat
base colour (§8 below). Those two — the shader-node port from
`intern/cycles/kernel/svm/` and Find Missing Files — are the rendering
items in front of everything else.

## 4. Render the animation — stills or a video

Render ▸ Render Animation (**Ctrl F12**) renders Start..End (the Timeline's
fields) through the camera. With an image format it writes one file per
frame to the Output path: `shot_0001.png`, or `shot_##` for two digits
(`BLI_path_frame`'s rule). Esc stops at the frame it reached.

**A video (batch 116):** Properties ▸ Output ▸ Output ▸ File Format:
**FFmpeg Video**, then the Encoding panel below it — Container (Matroska,
MPEG-4, WebM, QuickTime, AVI, …), Video Codec (H.264, H.265, AV1, VP9,
ProRes, PNG, …), Output Quality (constant rate factor: Perceptually
Lossless … Lowest, or Constant Bitrate), Encoding Speed, Keyframe Interval
— all read from the file's `ffcodecdata` and the same values Blender
passes to libavcodec (`movie_write.cc`). Ctrl F12 then gathers the frames
and writes **one movie**, named as Blender names it:
`shot_0001-0140.mkv` (the frame range and the container's extension,
`BKE_movie_filepath_get`). The Info editor says `Saved movie: '…'`.

Phiber encodes through the `ffmpeg` program when the machine has one
(Blender ships its own libavcodec; Phiber does not yet). **If ffmpeg is
not installed**, the movie is still written — as a Motion-JPEG AVI
(`shot_0001-0140.avi`, every player opens it) — and the Info editor says
so; install ffmpeg (`winget install ffmpeg`, then restart Phiber) to get
the H.264 / Matroska the panel asks for. A cancelled movie render encodes
the frames it has.

**Step by step, THYMUS to a video:**

1. File ▸ Open → `crates\phiphx\tests\fixtures\THYMUS-BOLDROID-WAVE-003.blend`.
2. Properties ▸ Render ▸ Sampling ▸ Render ▸ Max Samples: 64 (a clean
   enough frame in a few seconds each; 128 for better).
3. Properties ▸ Output ▸ Format: Resolution 1280 × 720, 100 %. Frame
   Start 1, End 140 (the Timeline's fields).
4. Properties ▸ Output ▸ Output: Output Path `C:\Users\phiac\Renders\thymus_`;
   File Format **FFmpeg Video**; Encoding: Container Matroska, Codec
   H.264, Output Quality Medium.
5. Render ▸ Render Animation (Ctrl F12). The Rendering workspace shows
   each frame as it lands; the header's Cancel or Esc stops it.
6. `C:\Users\phiac\Renders\thymus_0001-0140.mkv` — or `.avi` without ffmpeg.

Not yet: Overwrite / Placeholders, the frame step, audio, the stamp
metadata.

## 5. Export

File ▸ Export ▸ glTF 2.0 (.gltf), Universal Scene Description (.usda),
Wavefront (.obj), Stanford PLY (.ply), STL (.stl) — the file browser
opens for the path. What goes out is the evaluated scene (modifiers
applied, the pose at the current frame) as triangles with materials'
base colours; textures and the rig itself do not export yet.

## 6. Save the file

Ctrl S writes the `.blend` (its render and output settings, keys you
inserted or moved, markers, the view). Shift Ctrl S Save As… and Ctrl Alt
S Save Incremental keep the original. Closing with unsaved changes asks
(batch 108).

## 7. Play the animation

- **Space** plays, Shift Ctrl Space plays backwards, Esc stops; the
  Timeline's transport buttons do the same; the frame field and the
  Start / End fields set the range; the magnet snaps the playhead.
- Shift ← / Shift → jump to the range ends, ↑ / ↓ to the next or previous
  keyframe.
- The Animation workspace shows the Dope Sheet over the viewport; the
  wheel zooms its time axis, Ctrl wheel scrolls it, Home frames the keys
  (batch 113). I inserts keys on the active object, G / R / S with the
  record button on auto-keys.
- Female-01: the file's rigs (`metarig`, `WGT-rig`) and actions play
  through the rig evaluator (bones, constraints, drivers — `rig/tests.rs`
  against Blender); frames step at the file's fps, dropping frames rather
  than slowing down, as Blender's AV-sync does.

## 8. What your Female-01 render shows, and why

The frame you rendered is **what Blender would render from this file as
it stands on this machine**:

- **Black background** — Film ▸ Transparent is on in the file. Blender's
  Image Editor shows a checkerboard there; Phiber shows black (the alpha is
  0 either way, and the saved PNG has it).
- **Flat purple, blue and magenta patches instead of cloth and skin** —
  345 of the file's 346 material slots point at image textures that are
  **not in the file**: `//..\..\..\..\Downloads\Colour-Options\…`,
  `//..\..\..\..\..\..\Textures\Female-01-Texture-File\Eyes_Iris.png`,
  `…\Highlights.<UDIM>.png` — relative paths into the artist's Downloads
  and a `Textures` folder six levels up, none of which exist beside the
  `.blend` here. The eye materials' base colour is (1, 0, 1): that magenta
  *is* Blender's missing-texture colour, and Blender renders the same.
  Only one slot (DRESS-08, a 256² pattern) is packed, and it renders.
- **To see the real look**: in Blender, File ▸ External Data ▸ Find
  Missing Files (pointing at the Textures folder), then File ▸ External
  Data ▸ Pack Resources and save — Phiber reads packed images (the
  TEXTURED-PACKED fixture renders its texture). Or put the `Textures`
  folder where the paths expect it (`C:\Users\phiac\Textures\…` for the
  six-up paths). Phiber has no Find Missing Files yet.
- **3 min 59 s** — 2412 samples at 1920 × 1920 on a 1660 Ti. Max Samples
  128 is about 15 s.

## 9. Where rendering and animation stand

`animation-rendering-status.md` has the full tables; the short form:

**Rendering — works:** F12 / Ctrl F12 through the file's camera (depth of
field, orthographic), any size (tiled past the GPU's binding), cancel,
progress; Principled BSDF with image textures, normal maps, alpha, SSS,
sheen, coat; sun / point / spot / area lights with soft shadows; HDRI
worlds with importance sampling; four view transforms; denoising; Depth /
Normal / Diffuse Color passes; PNG, JPEG, TIFF, EXR (multi-layer), HDR
saves; Film Transparent; the Render and Output panels read, applied and
saved. **Not done:** the engine choice (everything renders with phiray —
no Cycles, EEVEE or Workbench *final* renders), shader nodes beyond the
first Principled BSDF (Mix / Add Shader, Emission, Glass, Transparent,
procedural textures, Bump fold to a colour), Find Missing Files, render
slots beyond the first, motion blur, video output, Mist / AO / Position /
Vector / Cryptomatte passes, adaptive sampling and time limit (read and
saved, not applied).

**The node editors (Shading, ECS Wand — formerly Geometry Nodes,
Compositing):** laid out as Blender lays them out, with a placeholder
graph of two cards (Principled BSDF → Material Output); **they do not yet
draw the file's node tree**, though phiphx reads it (`blend/shade/nodes.rs`,
`links.rs`) to fold a material's Principled values. Drawing the real
shader tree from that data is the Shading space's next step; ECS Wand has
no node evaluation at all (`GeometryNodeTree` is not read) — that is a
large piece, after the shader nodes in the tracer.

**Viewport — works:** Solid (Workbench shading; since batch 114 its
surfaces are rasterized on hardware, as Workbench does; since batch 118
anti-aliased by Workbench's eight jittered samples, settling while the
view rests; since batch 119 with Workbench's object outline, and the
curvature effect where the file's Cavity asks), Wireframe (the same
anti-aliasing), X-ray, Material Preview (deterministic, studio light),
Rendered (progressive, denoised), objects as instances (a move is a
matrix); the Shading popover (the chevron after the shading buttons,
batches 120–121): Lighting Studio / Flat, Color Material / Object /
Random / Single, Background Theme / World / Viewport, Outline and its
colour, X-Ray and its alpha, Cavity with its type, ridge / valley factors
and — World / Both — the SSAO settings (Samples, Distance, Attenuation,
batch 122), drawn by the kernel as Blender's `cavity_compute`; Shadow with
its intensity and settings (Direction, Offset, Focus, batch 123), one
shadow ray per pixel with `get_shadow`'s terminator. **Not done:** SMAA on
the first frames of Solid; MatCap, Attribute and Texture colour (they
draw as Studio / Material and the popover says so); Workbench's depth of
field; an object's Shadow switch (`OB_DRAW_NO_SHADOW_CAST`); Material
Preview on EEVEE's model; `Scene.display.shading` for a Workbench F12.

**Animation — works:** playback of objects, NLA, rigs, constraints, IK,
rig drivers; keying (I, auto-key, Alt I); Dope Sheet key select / move /
delete; markers; actions, slots, NLA tracks and strips; the time axis
zoom / scroll / Frame All. **Not done:** Graph editor editing, channel
rows per F-curve, NLA tweak mode and strip dragging, drivers beyond rigs,
shape keys, F-curve modifiers, about 390 inert menu entries.
