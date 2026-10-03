# Rendering — samples, controls, and what it takes to ship fashion work

> Companion to `render-parity.md` (what Blender does vs what we do) and
> `blend-read-parity.md` (reading the file). This one answers three questions:
> **what is actually being rendered today**, **which knobs exist**, and **what
> is still missing before a frame is good enough to hand a client.**
>
> Short answer to "did you enable the highest quality": **not yet — but the
> two biggest gaps are now closed.** Material lobes (subsurface, sheen, coat,
> anisotropy) are read and shaded, and the world is read from its node tree
> instead of a stale 2.7x field, so an HDRI actually lights the scene. What is
> still open is in §3 and §5.
>
> **2026-09-22.** Rendered mode now matches a Cycles render of the same file
> closely enough to compare frame to frame — see §7.

---

## 1. What renders a frame today

| Shading | Kernel | Passes per frame | Noise |
|---|---|---|---|
| Wireframe | `phiray` `Mode::Wire` | 1 (deterministic) | none |
| Solid (Workbench) | `Mode::Solid` | 1 (deterministic) | none |
| Material Preview | `Mode::Preview` — deterministic: prefiltered studio-light environment, hard-shadowed lamps, mipmapped textures (`preview.wgsl`, since batch 108) | 1 (deterministic) | none |
| Rendered | `Mode::Rendered` → path tracer, the file's lights and world | refines to **32** (viewport: `preview_samples`) | the shown frame is denoised from the first sample (Viewport Denoising, on by default since batch 109); the samples beneath converge untouched |

The still-image path for checks against Blender is
`cargo run --release -p phiber --example color_render -- FILE OUT.png
[material|solid|rendered] [samples]`.

**Samples are the single biggest quality control we have today.** On Female-01
at 720 × 1080, software GPU:

| Samples | Render time (software GPU) | Look |
|---|---|---|
| 32 (default) | 8.1 s | grain on cloth and in shadow |
| 256 | 58.5 s | clean; no visible grain |

On a real GPU both numbers drop by one to two orders of magnitude. Blender's
own defaults for comparison: Cycles **viewport 1024** samples with the
denoiser on, **render 4096** with adaptive sampling (noise threshold 0.01) and
denoising; EEVEE viewport 16 / render 64 TAA samples (raster, so no noise).

---

## 2. Every control that exists today

### 2.1 Frame and camera — `phiray::Camera`

| Field | Meaning | Where it comes from |
|---|---|---|
| `w`, `h` | Pixels traced | Viewport: canvas size, capped at `MAX_SIDE` 1600 px on the long side. Still: 720 × 1080 in the example |
| `persp`, `dist`, `scale`, `units` | Orthographic or perspective, focal distance, zoom | The file's saved 3D view (`BKE_camera_params_from_view3d`) |
| `right`, `up`, `back`, `pivot`, `pan` | Orbit basis | The view |
| `aspect` | Pixel aspect | Canvas |
| `sun` | Key direction for the studio rig | Fixed to the view |
| `sky`, `env` | Background colour, and the world radiance every ray sees | Theme, or the file's Background node (colour × strength) |
| `envmap`, `env_rot` | Equirectangular environment image (atlas id + 1; 0 = none) and the world Mapping node's Z rotation | `World.nodetree` → Environment Texture + Mapping |
| `grid`, `grid_color` | Floor grid | Viewport only |
| `ao` | Reach of the short-range occlusion term | Auto: 8 % of the scene diagonal |
| `view` | **View transform, exposure, gamma** — Standard / Filmic / AgX | `Scene.view_settings` |
| `lights[16]`, `count` | Up to 16 scene lights: sun / point / spot / area, colour × power, radius, cone | The file's lights (Rendered); empty in Material Preview, which uses the studio rig |
| `glow` | Emissive triangle count; > 0 turns on emitter sampling | Built from the scene |

### 2.2 Surfaces — `phiray::Tri` (what a material can say)

Base colour + alpha, roughness, metallic, IOR, transmission, emission colour,
**subsurface weight and scatter radius, sheen weight / roughness / tint, coat
weight / roughness, anisotropy and its rotation**, and five image slots: base
colour, normal map (with strength), roughness, metallic, emission. A material
record is 30 words on the GPU. Textures are capped at **2048 px** on upload in the
viewport (`TEXTURE_LIMIT`) and carry a mip chain (`Atlas::add`); Material
Preview reads the level a pixel's footprint asks for, the path tracer the
base level (it supersamples).

### 2.3 Integrator (fixed in the shader today, not exposed)

- **8 path events** for alpha cut-outs and glass (not 8 GI bounces).
- **1 diffuse bounce** for indirect light, blended with short-range occlusion.
- **1 glossy sample** for reflection, from the environment or one further hit.
- **Light sampling**: up to 4 lights per sample, picked at random and
  reweighted when there are more; area and spot shapes sampled by radius.
- **Emitting meshes**: sampled directly and combined with the diffuse bounce
  (multiple importance sampling, power heuristic).
- Pixel jitter for antialiasing; samples average in display space.

### 2.4 What the Properties → Render tab shows

The full Cycles panel tree is drawn — Sampling, Light Paths, Volumes, Motion
Blur, Film, Performance, Curves, Colour Management — because it mirrors
Blender's UI. **Four rows now drive the render** (§11): Sampling → Viewport →
Max Samples, Sampling → Render → Max Samples, Denoise, and Light Paths → Max
Bounces → Diffuse. Colour Management was already wired. Clamping, Film,
Motion Blur and the `PHIRAY` engine entry are still drawn without effect.

---

## 3. What a high-end fashion frame needs, and where we stand

Fashion renders live or die on skin, fabric and light. Ranked by what is
missing most:

| # | Needed | Blender | Phiber today |
|---|---|---|---|
| 1 | **Subsurface scattering** (skin, ears, nose) | Principled `Subsurface`, random-walk | **Read and shaded** — `Subsurface Weight / Radius / Scale` reach the kernel as a wrapped diffuse lobe tinted by the scatter radius, plus a back-scatter term for thin parts. Not a random walk, so thick translucency (a hand held to a lamp) is still approximate |
| 2 | **Fabric response**: Sheen (velvet, cotton fuzz), anisotropic highlights (silk, satin), thin-film / iridescence | Principled `Sheen`, `Anisotropic`, `Coat`, `Iridescence` | **Sheen and Coat shaded** (Charlie sheen lobe, energy-conserving clearcoat). `Anisotropic` / `Anisotropic Rotation` are read and carried but not shaded — that needs per-triangle tangents. Iridescence not read |
| 3 | **HDRI environment lighting** + real softboxes | Environment Texture world, area lights with shape | **Done for the world**: `World.nodetree` → Background colour × strength, Environment Texture (EXR / HDR / LDR) and the Mapping node's Z rotation, sampled equirectangularly as RGBE. Area lights are still discs of a single radius |
| 4 | **Denoising** | OpenImageDenoise / OptiX, albedo + normal guided | **Implemented** — a five-pass edge-avoiding a-trous filter guided by albedo, normal and depth (§10). Not a learned denoiser, so it is weaker than OIDN on fine detail |
| 5 | **Full GI** — multiple diffuse bounces, colour bleed | `max_bounces` 12 by default, diffuse 4 | **Implemented**, `Camera::bounces`; the default is now **4** (Blender's diffuse default) whenever the world has a sampling table, and 1 without one |
| 5b | **Environment next-event estimation** — sample the world image by luminance and shadow-ray it | Cycles builds a CDF over the environment and samples it directly | **Implemented** — `Atlas::add_env_cdf` + `env_light` in `light.wgsl`, MIS'd against the cosine sampler (see §9) |
| 6 | **Texture fidelity**: 4–8 K maps, mipmaps, anisotropic filtering | Full-resolution, GPU-filtered | 2048 cap; mip chains since batch 108 (Material Preview samples them trilinearly, Rendered still reads the base level and resolves aliasing by sampling) |
| 7 | **Depth of field** | Camera aperture, bokeh shape | **Not implemented** — the camera itself is read and framed correctly (§12), only the aperture is missing |
| 8 | **Shadow catcher / holdout / film transparency** for compositing onto a backdrop | Object flags + `film_transparent` | **Not implemented** |
| 9 | **Output**: resolution %, 16-bit PNG / EXR, passes (AOVs), colour-managed save | Output panel | **Not implemented** — the still example writes 8-bit PNG at a fixed size |
| 10 | **Adaptive sampling + clamping + light tree** | Cycles defaults | **Not implemented** — fixed sample count, no firefly clamp |
| 11 | **Motion blur, volumetrics, caustics** | Cycles | **Not implemented** |
| 12 | **Looks** (Filmic High Contrast, AgX Punchy) | OCIO looks | Base curve only; exposure and gamma do apply |

Also open from the read side, and visible in the picture: the two hair
materials built from node types current Blender does not recognise either, and
a texture folder this file references that is not shipped (Blender and Phiber
both draw those patches magenta now — matching, but neither is right).

---

## 4. Recommended settings, once the knobs exist

| Purpose | Samples | Denoise | Bounces | Texture cap | Output |
|---|---|---|---|---|---|
| Viewport look-dev | 32 → refine to 256 while idle | on, from 8 samples | 4 | 2048 | screen |
| Client preview | 256 | on | 8 | 4096 | 16-bit PNG |
| Final deliverable | 1024–4096 adaptive (threshold 0.01) | on, albedo + normal guided | 12 (Blender's default), 4 transmission, 12 glossy | full resolution | EXR (linear) + 16-bit PNG, AgX or Filmic |

Blender's equivalents, for the same three rows: `cycles.preview_samples`,
`cycles.samples` + `use_adaptive_sampling` + `adaptive_threshold`,
`use_denoising`, `max_bounces`, and Output → Resolution / File Format.

---

## 5. The pipeline that has to exist

Today there is no render *pipeline* — there is a viewport tracer and a
command-line example. A deliverable needs:

1. **Engine wiring.** Samples, diffuse bounces and denoise now reach
   `Camera` / `Tracer` (§11). Still open: `PHIRAY` in Properties → Render →
   Engine actually selecting the renderer, and Clamping.
2. **Render job.** F12 renders at the scene's resolution × percentage on a
   worker thread, with progress and a cancel — the same progress surface the
   file loader uses — and the result goes to the Rendering workspace's Image
   Editor (which today shows a placeholder checkerboard).
3. **Result buffer and saving.** Keep the frame in scene-linear, show it
   through the view transform, save 16-bit PNG and EXR, remember the last
   render, and hold render passes (combined, depth, normal, albedo, object
   index) for compositing.
4. **Quality work**, in the order of §3. Subsurface, fabric lobes, the HDRI
   world, multi-bounce GI, environment importance sampling and the denoiser
   are done; **texture fidelity, depth of field, film transparency and the
   output formats are what is left**, along with wiring the Render panel to
   the knobs that now exist.
5. **Camera**: the file's camera object is used (§12); depth of field is
   still open.

Steps 1–3 are plumbing and can land quickly. Step 4 is the part that decides
whether the picture is sellable, and subsurface plus fabric lobes are the two
that change it most.

---

## 6. How to check quality against Blender

```powershell
$f = "$PWD\crates\phiber\assets\CW-Fashion\Epic-Pods-Female-01\Female-01-Chatty-Whims-Rigged-Lace.blend"
# Blender's Material Preview (EEVEE) of the same front view
Start-Process blender-launcher -Wait -NoNewWindow -ArgumentList @("-b","`"$f`"","-P","`"$PWD\crates\phiphx\tests\truth\blender_color.py`"","--","`"$f`"","`"$PWD\blender_color.png`"","eevee")
# Phiber, 256 samples
cargo run --release -p phiber --example color_render -- $f phiber_color.png material 256
cargo run --release -p phiber --example render_compare -- blender_color.png phiber_color.png color_side_by_side.png
```

`render_compare` prints the mean difference on the figure, each image's mean
brightness and an edge-softness figure, and writes the pair side by side.


---

## 7. Where Rendered mode stands against Cycles (2026-09-22)

Same file, same frame, same sample count, both on CPU (Cycles CPU vs phiray on
a software Vulkan device — a real GPU changes only our number):

| | Blender Cycles | Phiber Rendered |
|---|---|---|
| Samples | 64 | 64 |
| Read | 9.2 s | 5.7 s |
| Render | 56.1 s | 82.7 s |
| Total | 65.2 s | 88.5 s |

Framing, background, light direction and colour now agree. What is still
visibly different:

- **Noise.** Ours is grainier at the same sample count and has no denoiser;
  Cycles' sampling is smarter per ray. This is gap #4 and is now the first
  thing to fix.
- **Dark patches** on the thigh and between the legs — overlapping garment and
  body geometry, or shadow-ray self-intersection at the current `eps`. Needs a
  look.
- **Slight over-contrast** in the hair.

### Two parity facts this comparison turned up

- **`World.horr/horg/horb` is not what a 2.8+ render uses.** We were reading
  it; Blender reads the Background node. On Female-01 the legacy field says
  a dark green-grey, while the file is actually lit by a 4K sky EXR at
  strength 0.5, rotated 0.80 rad. That one field was the whole "our renders
  look flat" problem.
- **Blender does not look for a world image beside the `.blend`.** It resolves
  the `//`-relative path and nothing else; a missing environment renders the
  frame magenta, the same rule it uses for missing surface textures. Our
  `candidates()` additionally tries the bare file name next to the `.blend`,
  which is more forgiving than Blender — worth keeping, worth knowing.
- **None of Female-01's materials set Subsurface, Sheen or Coat** (one nose
  material has subsurface 0.015). Blender renders that skin without subsurface
  too, so the lobes change this asset barely at all. They matter for client
  files that do use them.

Reference renders:

```powershell
$f = "$PWD\crates\phiber\assets\CW-Fashion\Epic-Pods-Female-01\Female-01-Chatty-Whims-Rigged-Lace.blend"
# Blender, the file's own world and lights, orthographic front view
Start-Process blender-launcher -Wait -NoNewWindow -ArgumentList @("-b","`"$f`"","-P","`"$PWD\crates\phiphx\tests\truth\blender_world.py`"","--","`"$f`"","`"$PWD\blender_world.png`"","64","cycles")
# Phiber, same frame
cargo run --release -p phiber --example color_render -- $f phiber_world.png rendered 64
cargo run --release -p phiber --example render_compare -- blender_world.png phiber_world.png world_side_by_side.png
```


---

## 8. Multi-bounce GI, and why it is capped at one bounce (2026-09-22)

`indirect()` in `light.wgsl` is now a loop rather than a single sample: it
sends a cosine-weighted ray, gathers direct light and any emitter where it
lands, scales the throughput by that surface's albedo and carries on, up to
`Camera::bounces`. A path that is cut off inside the scene is closed with the
world seen from the last surface, *unoccluded* — the calibrated one-bounce
ambient this code used to be.

**The default is 1, not Blender's 4.** Raising it makes the picture worse, and
the reason is worth recording because it names the next piece of work.

Measured on the leggings alone, against a Cycles render of the same object and
frame (pixels more than 120/765 darker than Blender, in the cavity between the
legs):

| Diffuse bounces | Mean \|diff\| in that region | Pixels much darker than Cycles |
|---|---|---|
| 1 | 17.2 | **0** |
| 2 | 19.8 | 82 |
| 6 | 22.8 | 1417 |

More bounces should only ever *add* light. They subtract here because the
unoccluded tail at depth 1 is an over-estimate that happens to land on Cycles'
answer, and each extra bounce replaces more of it with real, occluded
transport — while a cosine-sampled ray inside a cavity almost never finds the
opening, so the light that should replace it is never sampled.

The fix is **environment next-event estimation**: build a luminance CDF over
the world image, sample a direction towards the bright parts at every bounce
and shadow-ray it, exactly as Cycles does. With that, a cavity sees the sky
through its opening on nearly every sample instead of by luck, and the bounce
depth can go to Blender's 4 without losing energy.

Until then `Camera::bounces` exists and `PHIRAY_BOUNCES=<n>` on `color_render`
sweeps it, so the change can be measured the moment the sampling lands.

### What this ruled out

The dark cavity is **not** a geometry error. Rendering the leggings alone in
both renderers and comparing silhouettes gives an IoU of **0.9976** (157
differing pixels out of 66 454), and every one of the object's 36 734
evaluated vertices is within 0.025 mm of Blender's. `PHIBER_ONLY=<name>` and
`PHIBER_DROP=<name>` on `color_render` isolate an object without moving the
camera, which is how that was established.


---

## 9. Environment importance sampling (2026-09-23)

§8 ended on the finding that raising the bounce depth made cavities darker,
because a cosine-sampled ray has to find a cavity's opening by chance. That is
now fixed the way Cycles fixes it.

`Atlas::add_env_cdf` builds a sampling table from the world image — a
128 × 64 grid of luminance × sin θ, stored as one conditional CDF over columns
per row plus a final row holding the marginal CDF over rows. `env_sample` in
`light.wgsl` draws a direction from it, `env_pdf` reports the density of any
direction, and `env_light` shadow-rays the sample and adds
`f · L / pdf`. Both estimators — the table and the cosine ray that `indirect`
already sent — see the same world, so each carries a power-heuristic MIS
weight against the other.

With that in place two things change: the unoccluded tail that used to close a
truncated path is gone (the light it stood in for is now sampled directly, so
the tail would double it), and the default diffuse depth goes from 1 to
**Blender's 4** whenever a table is present.

### Why the tests are worth trusting

`tests/environment.rs` does not check a picture. The estimator divides by the
table's own density, so a wrong density is a *bias*: guided and unguided
sampling converge to different numbers. The test runs a diffuse quad under a
sky with a small bright patch, once cosine-sampled at 4096 samples and once
table-sampled at 512, and requires them within 6 %. Deliberately dropping the
`sin θ` term from the density makes it fail at **9.6 % apart**, so the test
does bite.

A second test requires the guided estimator to settle more between 64 and 512
samples than the unguided one — that the table buys convergence, not just
correctness.

### Known approximation

The MIS weight is computed against the diffuse lobe only. Cycles weights each
lobe separately, so a sharp specular reflection of a small bright light in the
environment is still under-counted here.


---

## 10. The denoiser (2026-09-23)

`Tracer::denoise` runs five edge-avoiding a-trous wavelet passes over the
finished frame (Dammertz et al. 2010), with taps 1, 2, 4, 8 and 16 pixels
apart so a wide radius is reached without an enormous kernel. The tracer now
records, per pixel and averaged over the samples like the radiance itself, the
first hit's **albedo, normal and depth** (`guide` in `util.wgsl`); a tap is
weighted down when any of those disagrees with the centre's, which is what
stops a silhouette or a seam from smearing.

Radiance is filtered with the **albedo divided out**, so a texture is put back
after filtering instead of being blurred away with the noise.

Everything is filtered in display space, because that is what the tracer
averages and therefore what the image shows — the mean of the transformed
samples, not the transform of the mean. Filtering scene-linear values and
transforming once at the end is a visibly different (brighter) picture; that
mistake cost a first attempt, which measured *worse* than not denoising.

### What it buys

On the test scene — two quads of different albedo under a sky with a small
very bright patch, so most rays miss the light — mean per-byte difference from
a 1024-sample reference:

| | 12 samples |
|---|---|
| No denoise | 6.49 |
| Denoised | **0.98** |

### The tests

`tests/denoise.rs` guards the two halves separately, and the split is
deliberate: replacing the edge-stopping weight with 1 (blur everything) still
passes the edge test, because albedo demodulation alone keeps a texture edge —
but it fails the convergence test, because blurring across geometry moves the
picture away from the reference. So one test is named for what it really
guards (demodulation) and the other covers edge stopping.

### Using it

`Tracer::denoise(&gpu)` after a render has settled; `pixels` then returns the
filtered image, and tracing another sample overwrites it. In phiber,
`PHIRAY_DENOISE=1` turns it on for `still()` — a placeholder until the Render
panel's Denoise checkbox drives it, which is part of §5's plumbing.

### Cost

The sample buffer went from 7 to 20 words per pixel (the guides and two
scratch images), so 720 x 1080 now holds 62 MB rather than 22 MB. Worth
revisiting for the viewport, where the guides could be packed.


---

## 11. The Render panel drives the render (2026-09-23)

`scene/render_set.rs` reads Properties → Render and hands it over as one
`BRenderSet`:

| Panel row | RNA key | Reaches |
|---|---|---|
| Sampling → Viewport → Max Samples | `cycles.preview_samples` | `Tracer::set_samples` |
| Sampling → Render → Max Samples | `cycles.samples` | `still`'s sample target |
| Sampling → Render → Denoise | `cycles.use_denoising` | `Tracer::denoise` once the frame settles |
| Light Paths → Max Bounces → Diffuse | `cycles.diffuse_bounces` | `Camera::bounces` |

It rides to the worker on a new `BRayShot` (camera, size, kernel, samples,
denoise) rather than five positional arguments, and the sample target and
denoise flag join the view key by hand — neither lives in `Camera`, and
changing either has to restart the refinement rather than carry on.

### Properties now have RNA defaults

`BPropSpec` gained a `default`, because the panel used to show **0 samples**:
a control with no stored value fell back to its *kind's* factory value, and an
integer's is its minimum. Blender's RNA carries a per-property default, so the
spec does too — `.at(4096)`, `.at(1024)`, `.at(4)`, `.on()` on the four rows
above. Everything else keeps the old behaviour (`default: None`).

### Read from the file (2026-09-23)

`phiphx/src/blend/idprop.rs` reads ID properties — `IDProperty` /
`IDPropertyData` from `DNA_ID.h`, the types from `DNA_ID_enums.h` — so
`scene["cycles"]` now reaches `SceneRender::cycles`, and
`app/seed.rs::seed_render_props` writes it into `ctx.buts.*` on load.

Blender keeps no separate "session" copy: a file's values *are* the session's
until someone edits them. And it stores **only what differs from the RNA
default** — Female-01's whole Cycles group is three entries:

```
scene["cycles"]: 3 entries
  preview_samples = Int(20)
  samples = Int(20)
  use_light_tree = Int(0)
```

So the panel now opens that file showing **20** samples, while
`diffuse_bounces`, which the file never wrote, stays at Blender's 4. The
absent-vs-zero distinction is exactly why §11's RNA defaults had to land
first; the two halves only work together.

Seeding writes each value in the category the *control* reads, not the one
the file used — an add-on may store a flag as an integer while the checkbox
parses `true` / `false` — and under the panel's own `'static` key, so the
state key stays a `ButsKey` rather than a path spelt by hand. The Render
tab's `spec_of` supplies both.

### Tests

`tests/render_panel_drives_the_render.rs` checks the defaults are Blender's,
that an edit reaches `BRenderSet`, and that a value the panel could not have
written (empty, or `"lots"`) falls back to the default rather than to zero.
That the bounce count then reaches `Camera::bounces` is checked in-crate,
where `BRayLook` lives.


---

## 12. Rendering through the file's camera (2026-09-23)

F12 does not look through the viewport. `scene/render_job.rs` builds the
render from the file instead: `Scene.camera` (now read — it was not before),
`resolution_x / y` scaled by `resolution_percent`, and the Render panel's
samples, bounces and denoise.

### The framing

phiray's screen is `u, v ∈ [-0.5, 0.5]`, and the kernel takes `u` to
`u / scale` across and `v` to `v / (aspect · scale)` up. So the half-width the
frame spans at the eye is `0.5 / scale` over a distance `dist`, while
Blender's camera spans `sensor_width / 2` over `lens`:

```
tan(fov/2) = (sensor_width / 2) / lens        Blender
           = (0.5 / scale) / dist             phiray
```

`dist` only decides where the eye sits along a ray that is normalised
afterwards, so any positive value works. Blender's `sensor_fit = AUTO` puts
the sensor across the **longer** side, which is why a portrait frame divides
by the aspect where a landscape one does not — and the vertical half-extent
already carries an `aspect`, so it *divides* rather than multiplies.

That last detail was wrong in the first version, and invisible: the test file
renders 1920 × 1920, where the two are identical. The unit test's own helper
had the same error, so the two cancelled and the angle assertions passed. What
caught it was a plain sanity assertion — "a portrait frame must not be as wide
as a landscape one" — which is why it is still in the test with a comment
saying so.

### Measured

Cycles against phiber, both through Female-01's own `Camera.001`, silhouettes
compared:

| Frame | IoU |
|---|---|
| 480 × 480 (the file's shape, quartered) | **0.9981** |
| 270 × 480 (portrait, to exercise the sensor fit) | **0.9975** |

`examples/camera_render.rs` renders it; `PHIBER_SCALE=<n>` divides the
resolution and `PHIBER_RES=WxH` reshapes the frame, which is how the second
row was measured.

### One world, two cameras

The viewport's orbit camera and this one differ only in framing — they stand
in the same world — so the sky, the environment image, the lamps and the
bounce count now come from one place, `rendered/world.rs`'s `lit`. Before
this, `camera_of` built a bare camera and the first render came out black,
lit only by the studio rig.

That move also pulled `BRayLook`, `light`, `view_words` and `environment` out
of `rendered/scene.rs`, which the decomposition guard had just pushed over its
200-line cap — the world was a second concern living in the geometry file.

## 13. The render runs in the background (2026-09-23)

Until now F12 opened a Render window that showed a checkerboard, and the only
way to get an image out was an example binary. The render now runs the way
Blender's does: on a worker, with progress, cancellable, and the result lands
in the Image Editor as **Render Result**.

### The job

`v2/scene/render_job.rs` holds one process-wide job, the way Blender holds one
`Render` per scene:

- `BRenderShot { w, h, bgra, taken, target, done }` — the newest frame, its
  sample count `taken` out of `target`, and whether the render finished.
  `fraction()` is `taken / target`, `has_image()` says whether there is
  anything to paint yet.
- `BJob` — a `Mutex` slot for the shot plus `running` / `cancel` atomics and a
  sequence number, so a late frame from a cancelled render cannot overwrite a
  newer one.
- `start(objects, plan, plain)` publishes the target **before** the first
  sample, so the window reads "sample 0 of 4096" immediately rather than
  showing nothing for the first few seconds. It refuses to start a second
  render while one is running — Blender does the same.
- `cancel()` sets the flag; the sampler checks it between samples and stops,
  keeping the partial image. Blender keeps the partial too.
- `shot()` / `rendering()` are what the UI reads. Neither blocks the render.

### Sampling one at a time

`view3d/rendered.rs` gained `still_stepwise(..., step)`, the same render as
`still` but yielding after every sample: it calls `step(taken, pixels)` and
stops early when that returns `false`. Pixels are only materialised when the
caller asks for them (the job asks roughly ten times a second, not every
sample), so progressive display costs almost nothing.

### What the window shows

`view/draw/canvas/image.rs` paints the newest frame letterboxed inside the
editor, keeping the checkerboard only when there is nothing to show yet. The
caption follows Blender's:

- while running — `Rendering — sample 137 of 4096`
- when done — `Render Result — 1920 × 1080, 4096 samples`

`app/handle.rs::ensure_render()` starts the render when the render modal opens,
skipping when one is already running or a finished result is already on screen
— so F12 renders, and F11 (re-open) shows the last result rather than
re-rendering, which is Blender's behaviour.

### The guard

`tests/render_job_runs_in_the_background.rs` runs one sequential test covering
progress publication, the refusal to start twice, completion with an image, and
cancel. Mutation-tested: making `cancel()` a no-op makes it fail with
"cancel was ignored: 4000 samples taken".

### Still missing in this area

- EXR and 16-bit PNG output; the job holds BGRA8 only.
- Render passes (AOVs) — combined only.
- Depth of field: the camera is read and framed correctly, but the aperture is
  ignored, so everything is in focus.
- Film transparency and shadow catcher.

## 14. Saving a render: EXR, 16-bit PNG, and the Output panel (2026-09-23)

A render that only exists on screen is not a deliverable. This adds the two
halves of getting one off the screen: reading Properties → Output off the
`.blend`, and writing the finished render in the format it names.

### Reading the panel (`phiphx`)

`blend/scene/output.rs` reads `Scene.r.pic` and the embedded
`Scene.r.im_format`, producing `BlendOutput { path, format, depth, channel,
quality, compress, exr_codec }`. Three things about that struct are easy to
get wrong and are worth naming:

- **`depth` is not a bit count.** `eImageFormatDepth` is a one-of-many flag:
  8 bits is `1 << 1`, 16 is `1 << 4`, 32 is `1 << 6`. Reading it as a number
  gives 2, 16 and 64.
- **`planes` became `color_mode` in 5.x** with the same values, because
  `ImColorMode` is the old total-bit-count (BW 8, BW+A 16, RGB 24, RGBA 32).
  Either name answers, so both are tried.
- **Blender turns separators native as it reads.** A path authored on Windows
  (`D:\Graphigem\Rig\Rig_ROM2`) is reported with forward slashes everywhere
  else, so the reader normalises rather than handing back backslashes. This
  was found by the truth test, not by reading the source.

`BlendOutput::file_of` is the `do_makepicstring` path rule: `//` resolves
against the `.blend`'s own folder, a bare folder gets Blender's "Untitled",
and the format's extension is *replaced* when the name already ends in some
image extension (`shot.png` saved as EXR is `shot.exr`, not `shot.png.exr`)
and appended otherwise. Frame numbering is deliberately not applied — that is
the animation path, and a still render has one frame.

### Writing it (`phiber`)

`v2/scene/render_save.rs`. The rule that matters is which buffer a format
reads, and it is the thing most likely to be got backwards:

| Format | Buffer | Why |
| --- | --- | --- |
| 8-bit anything | the bytes on screen | already transformed and quantised |
| 16-bit PNG | the **display** floats | a PNG is display-referred at any depth |
| EXR, Radiance HDR | **scene-linear** radiance | baking AgX into an EXR destroys the point of it |

So the three cases are separate functions rather than one with a flag. EXR is
written through the `exr` crate rather than `image`, whose encoder is
float-only: a half file asked for in the `.blend` would otherwise come back
silently at twice the size. Blender's default for OpenEXR *is* half, so that
is not a corner case.

A depth the format does not offer (`BKE_imtype_valid_depths`) falls back to
the format's first rather than being refused — an 8-bit EXR from an older
file is written half, as Blender does.

### What had to change underneath

- `Tracer::display` (phiray) — the display-space image as floats, read from
  the display sums at words 4n..7n. A 16-bit PNG quantises this rather than
  stretching the 8-bit image, so the extra bits carry real precision.
- The last à-trous pass now writes its filtered result **back into the display
  sums**, not only into the packed 8-bit image. Without that, a 16-bit save
  would have handed over the noisy image while the screen showed the filtered
  one — a silent mismatch between what you see and what you get.
- `BStill { rgba, display, linear }` replaces the bare `Vec<u8>` that
  `still_stepwise` handed back, and `BRenderShot` carries all three. The
  floats cost one more readback at the end, not a second render.

### The guards

- `phiphx/tests/output_truth.rs` against `truth/output_format.txt`, written by
  Blender 5.0.1 (`truth/blender_output.py`, which also writes the
  `OUTPUT-FORMAT.blend` fixture — a scene with *none* of the Output panel left
  at its default, so a field read at the wrong offset cannot pass by luck).
  Two production fixtures in the list are set to FFMPEG with absolute Windows
  paths, which is what a real animation project's file looks like.
  Mutation-tested: dropping the 32-bit depth case reports `…|8|…` against
  `…|32|…`; mapping imtype 23 to PNG reports `PNG` against `OPEN_EXR`.
- `phiber/tests/render_saves_in_the_file_format.rs` reads its own output back
  and compares numbers, because a view transform is close enough to a gamma
  curve that a linear/display mix-up survives eyeballing. Mutation-tested:
  feeding the 16-bit PNG the linear buffer fails; ignoring the half flag fails
  (the half file is no longer smaller); dropping the BGRA swap fails.

### Still missing in this area

- Nothing calls the writer yet: Image → Save As in the Rendering space is the
  next piece, and Blender's "Save" on an unsaved Render Result opens that
  dialog rather than writing straight away.
- Render passes (AOVs), so `OPEN_EXR_MULTILAYER` saves as a single combined
  part.
- Film transparency, so alpha is opaque everywhere and an RGBA save says so
  honestly rather than carrying a meaningful matte.
- Depth of field: the camera is read and framed correctly, but the aperture is
  ignored.

## 15. Image → Save As, and the Output panel showing the file (2026-09-23)

§14 built the writer but nothing called it. This connects both ends: the
Output tab now shows what the file actually says, and Image → Save As writes
the Render Result through it.

### The Output tab reads the file

`app/seed.rs::seed_output_props` writes the file's resolution, resolution %,
output path, file format, colour mode and colour depth into `ctx.buts.*`
under the RNA keys the Output tab already declared
(`part/props/output.rs`) — the same trick `seed_render_props` uses for the
Render tab, and for the same reason: Blender keeps no separate session copy,
so opening a file that says OpenEXR/32 has to *show* OpenEXR/32.

One difference from the Render tab worth stating: these come out of DNA, not
ID properties, so every one of them is present in every file. There is no
"absent means the RNA default" case to honour here.

`scene/output_set.rs::output_set` is the way back — the panel's strings read
as a `BlendOutput`, the same type the reader produces. That is what makes the
round trip work: switch File Format to OpenEXR in the panel, save, and you
get an EXR rather than the PNG the file was saved with.

### Save As

- `BFilePurpose::SaveImage` (`IMAGE_OT_save_as`) — the one purpose that does
  not name a `.blend`, so it never touches the recent-files list.
- The Image menu's **Save** (Alt S) and **Save As…** (Shift Alt S) both open
  the browser. A Render Result has no file of its own, so Blender's Save
  opens Save As too rather than writing straight away.
- `seed_image_target` opens the browser *where Properties → Output points*,
  with the name it would use, as `IMAGE_OT_save_as` seeds itself from the
  render output settings — not in the user's home folder.
- Committing a name writes the file and reports it in the status bar and the
  Info log. A name typed in the browser wins; the panel's format still
  decides the extension, because in Blender changing the name does not change
  the format.
- Nothing rendered yet reports "No render to save. Render an image first
  (F12)." A render still in flight is refused by the writer itself with its
  own message — the guard is not repeated in the operator, so one place
  decides whether a shot can be written. (This was found by mutation testing:
  with the check in both places, removing either one left the test passing.)

### The guards

`phiber/tests/save_as_writes_the_render.rs` drives the real reducer
(`apply_action`, not a bare store write, so the rules actually run): it
checks the panel round trip, that the browser opens pre-filled at the Output
folder, that a commit mid-render writes nothing, and that a commit after a
render produces a real 16 × 16 float EXR that is not black — read back with
the decoder, not just stat'd.

Mutation-tested, four for four: no pre-fill fails; letting the typed
extension win fails; not clearing the purpose fails; removing the writer's
"has an image" guard fails.

Suite: 704 passed, 6 failed — the six being phiphx tests that need production
`.blend` files kept outside the repo.

### Still missing in this area

- Render passes (AOVs), so `OPEN_EXR_MULTILAYER` saves as one combined part.
- The Output panel's remaining rows are read but not acted on: Overwrite,
  Placeholders, File Extensions, metadata stamping, Post Processing.
- Film transparency, so alpha is opaque everywhere.
- Depth of field.

## 16. Depth of field (2026-09-23)

Until now the camera was read and framed correctly but its aperture was
ignored, so every file rendered as a pinhole. A file that uses depth of field
— which most portrait and product work does — came out wrong in a way nothing
flagged. That is now closed end to end.

### Reading it (`blend/object/dof.rs`)

`Camera.dof` is a `CameraDOFSettings`. Two things in it are not what they look
like:

- **The f-stop is a ratio, not a length.** The radius a renderer needs is
  `(lens in metres) / (2 × f-stop)` for a perspective camera, and
  `1 / (2 × f-stop)` for an orthographic one — Cycles drops the lens factor
  there rather than treating the lens as 1 mm (`blender_camera_sync`,
  `intern/cycles/blender/camera.cpp`). Getting that wrong shrinks an
  orthographic aperture by a factor of a thousand.
- **`flag` is a `short`**, not an int, so `CAM_DOF_ENABLED` is read two bytes
  wide. Depth of field off means a pinhole whatever f-stop the file carries,
  which is what stops a camera with a wide aperture saved on it from softening
  every render made through it.

A camera can also focus on an *object* rather than a distance. What counts is
that object's distance **along the camera's own axis**, not the straight-line
distance to it (`blender_camera_focal_distance`). `render_job::focus_on`
resolves that where the scene's transforms are known, and returns nothing —
rather than guessing a number — when the camera focuses by distance.

### Tracing it (`phiray`)

The `Camera` gained `aperture`, `focus`, `blades`, `blade_spin` and
`aperture_ratio`; zero aperture is a pinhole, so depth of field costs nothing
when it is off. The primary ray in `shade.wgsl` now:

1. finds the aim point on the plane of focus,
2. offsets the eye over the aperture,
3. re-aims at the point it found.

Step 3 is the one that matters. Jittering the eye *without* re-aiming blurs
everything, including what should be sharp — that is the classic bug, and the
guard test is built to catch exactly it.

The plane of focus is a **plane** perpendicular to the view axis, not a sphere
around the lens, so the distance along a given ray is `focus / cos` (Cycles:
`ft = focaldistance / D.z`). Being honest about the test: this is correct per
Cycles but the guard does not resolve it — at the field of view used, the
difference between plane and sphere is well under a pixel, and mutating it
leaves the test passing. It would take a very wide lens to show.

Bokeh shape follows Cycles exactly: Shirley's concentric disk mapping for a
round iris, the corner-then-triangle sample for a bladed one
(`sample_uniform_disk` / `regular_polygon_sample`, `kernel/sample/mapping.h`),
with the anamorphic squeeze applied across only.

### The guards

- `phiray/tests/depth_of_field.rs` renders one edge three ways — pinhole,
  open aperture focused on it, open aperture focused elsewhere — and only the
  third may be soft. It also checks the blur *grows with the aperture*, which
  is what makes the f-stop mean anything. Mutation-tested: removing the
  re-aim blurs the focused card and fails.
- `phiphx/tests/dof_truth.rs` against `truth/depth_of_field.txt` from Blender
  5.0.1 (`truth/blender_dof.py`, which writes `DEPTH-OF-FIELD.blend`: one
  camera focusing on an Empty with a hexagonal anamorphic iris, one focusing
  by distance, one with depth of field off). Mutation-tested: ignoring the
  focus object fails; using the lens factor for an orthographic aperture
  fails. One mutation survives honestly — reading `aperture_blades` as a
  short instead of an int gives the same answer for any real blade count.
- `phiber/tests/render_camera_focuses_like_blender.rs` puts the focus target
  deliberately **off** the camera axis, so axial distance (10) and
  straight-line distance (10.77) are different numbers. A target on the axis
  could not tell them apart, and the first version of this fixture had it on
  the axis — the mutation survived until the fixture was fixed.

Suite: 709 passed, 6 failed (the six being phiphx tests that need production
`.blend` files kept outside the repo).

### Still missing in this area

- Render passes (AOVs).
- Motion blur — the shutter is not read at all.
- Panoramic cameras: `PANO` is read but traced as a perspective camera.

## 17. Render passes (2026-09-23)

`OPEN_EXR_MULTILAYER` saved a single combined image, which made the format a
lie. It now writes one layer per render pass.

### What a pass is called

The names matter more than the data, because a compositor matches on them,
and several are not guessable. Rather than read them out of the source I had
Blender **render a 16 × 16 multi-layer EXR with every pass switched on**
(`phiphx/tests/truth/blender_pass.py`) and took the channel list from the
file. That turned up:

- Mist's single channel is `Z`, not `V`.
- Both index passes use `X`.
- Ambient Occlusion is three channels, not one.
- UV is `U`, `V`, `A`.
- Cycles 5 has no Shadow pass at all, so the entry was dropped.

`ViewLayer.passflag` is read off the view layer Blender opens. The bitfield
has holes in it — several bits are `SCE_PASS_UNUSED_*` from passes Blender
removed — so nothing about the numbering can be derived; every value is
copied. A guard checks no two passes share a bit.

Combined is always in the list even when the flag omits it: Blender's UI does
not let it be switched off, and an EXR with no image would be a strange thing
to write.

### What is actually written

The tracer already recorded albedo, normal and depth as denoiser guides, so
three passes came almost free — `Tracer::albedo`, `normal` and `depth`. Two
details:

- The **normal** is re-normalised on readback. What is stored is a *sum* over
  samples, and averaging unit vectors does not give a unit vector.
- The **depth** of a ray that met nothing reads `1e10`, which is what Cycles
  writes (`film_get_pass_pixel_depth`, `kernel/film/read.h`). Zero would read
  as "touching the lens".
- The **albedo** pass is the raw linear buffer. The denoiser applies a
  transfer curve to the same data before comparing neighbours; a pass must
  not.

A pass that is asked for but cannot be traced yet — Mist, Position, the
indices, UV, the light-path splits — is **left out** rather than written as
black. An empty layer in a compositor reads as a render that came out wrong,
which costs more to chase than a missing one does. `written_pass` says which
passes a save will actually contain, and the status bar reports the count.

### The bug the file format nearly hid

The first writer put every channel in one EXR part with fully-qualified names
(`ViewLayer.Depth.Z`). Every channel name read back correctly and the naming
test passed — but a reader groups a single part into *one* layer, so all the
passes collapsed into one and effectively vanished.

Blender writes one part per pass (`R_IMF_EXR_FLAG_MULTIPART`), each part
named `ViewLayer.<Pass>` with its channels still fully qualified. Reading
Blender's own rendered EXR back through the same decoder is what showed the
difference; the fix was structural, and there is now a test on the *parts*,
not only the names. A second near-miss along the way: naming the part **and**
qualifying the channels produced `ViewLayer.ViewLayer.Depth.Z`.

### The guards

- `phiphx/tests/pass_truth.rs` against `truth/pass_channels.txt` — every
  channel phiber names must exist in the EXR Blender rendered, with the same
  channel count.
- `phiber/tests/render_saves_the_passes.rs` — layer names and order, one part
  per pass, untraced passes absent, and each layer holding its own data (the
  fixture's albedo is deliberately unlike the image, so a layer that is
  secretly a copy of Combined cannot pass for it). Mutation-tested: writing
  one part for everything fails; writing untraced passes anyway fails.

Suite: 718 passed, 6 failed (the six being phiphx tests that need production
`.blend` files kept outside the repo).

### Still missing in this area

- Position, Mist, UV, Vector and the object / material index passes: asked
  for, read, and deliberately not written.
- The light-path splits (Diffuse / Glossy / Transmission, Direct / Indirect),
  which need the integrator to keep them apart rather than a readback.
- Cryptomatte.
- Motion blur, and panoramic cameras.

## 18. Rendering quality against Cycles: the fixes that measured (2026-09-26)

§4 of `assessment.md` reported the tracer's brightness "within 3% of Cycles"
on THYMUS. That number was a coincidence — a too-dark metal and a too-bright
mis-read rubber averaging to Blender's mean — and it is withdrawn here. What
follows is the measurement that replaced it and the four changes it drove.

### The method: isolate lighting from materials

A whole-figure mean says nothing when one material is read wrong. So both
renderers were given the **same white diffuse override** —
`bpy` `view_layer.material_override` on Blender's side (`PHIBER_OVERRIDE` in
`truth/blender_color.py`), a white-diffuse swap in `rendered/scene.rs` on
ours — and the view transform was pinned to Standard on both
(`PHIBER_VIEW`). That leaves nothing but light transport and the camera in
the picture. Separately, `examples/env_probe.rs` renders a white card under
a uniform sky of radiance 6 and prints linear / display / packed values: the
analytic answer is 2.4 linear, so a transport bug shows as a number, not a
look.

| measure (Standard view, white override) | Blender | Phiber before | Phiber now |
| --- | --- | --- | --- |
| figure mean | 226.9 | 198 | **225.4** |
| shin metal patch (real materials, Filmic) | 24.2 | 15 | **24.9** |
| head (real materials) | 105 | 70 | 91 |

### Fix 1 — Material Preview lights with Blender's studio light

`crates/phiber/assets/studiolight/forest.hdr` is Blender's own `forest.exr`
(CC0, Poly Haven) as RGBE at full range, 1024 × 512, maximum radiance ≈ 1432.
The first conversion went through `save_render` and came back clamped at 1.0
— the view transform had been applied — which is why the metal was flat;
the guard now asserts a maximum above 500. As in Blender
(`studiolight_background_alpha = 0`) the HDRI lights the model but is not
seen: `Camera::env_hidden` makes a camera ray that misses show the flat
theme background while every bounce sees the sky.

### Fix 2 — the studio rig is a fallback, not a supplement

`lights()` in `light.wgsl` used to add the three-point rig on top of an HDRI
world. That was ~20% extra on every lamp-less scene in Rendered mode
(236 vs 198 on the probe). The rule is now: **the rig lights a scene only
when it has no light of any kind** — no lamp and no environment.
`tests/hidden_environment.rs` holds it.

### Fix 3 — accumulate linear, transform once

Cycles averages linear radiance and applies the view transform to the
converged buffer. phiray transformed *each sample* and averaged the results.
A concave curve with a clamp averaged that way is always darker, and a
highlight can never reach white. This was the whole "contrast gap": the
override render read 198 against Blender's 227, with p5 at 61 against 149.

The buffer is now `PX_WORDS = 24` per pixel: linear sums at `n..4n`, the
shown value (transformed mean, grid composited) at `4n..7n`, guides and
denoise scratch after, and the grid overlay as premultiplied sums at
`20n..24n` so the floor grid is averaged in linear space too. `emit()` takes
linear radiance and an overlay; the denoiser reads and writes the linear
sums and composites once at the end. `Tracer::display()` returns the shown
buffer as-is and `radiance()` the linear mean, which is what the DoF test
now measures — its emitter clips in display space.

### Fix 4 — Cycles' GGX in place of a cone sample

The glossy lobe was a cone jitter damped by an ad-hoc `(1 − rough × 0.7)`.
It is now the microfacet port from `kernel/closure/bsdf_microfacet.h`:
`ggx_sample_vndf` (Heitz 2018 visible-normal sampling, `alpha = roughness²`),
Smith `Λ` (eq. 72), throughput `F · (1 + Λi) / (1 + Λi + Λo)`, and the
`brdf()` used for light sampling carries the matching `D · G2 / (4 · n·v)`.
Fresnel uses the material's `f0`, or `((ior−1)/(ior+1))²` for glass. The
shin metal patch went from 15 to 24.9 against Blender's 24.2.

### What the numbers still say

- **Head 91 vs 105, chest 75.8 vs 46.** Both patches include the rubber
  lining, which Blender 5.0.1 renders **black** because it reads that 3.6
  file's `ShaderNodeMix` as `NodeUndefined`, and phiber renders **white**
  because the Noise / Musgrave / Gradient → ColorRamp → Mix network is not
  evaluated and falls back to the socket default. Neither is right; the
  oracle is not trustworthy for that material, and the fix is node
  evaluation (assessment §8, item 5), not chasing black.
- **Background.** Blender's Material Preview background is flat; phiber
  still draws the viewport's sky gradient behind the figure, which also
  pollutes figure masks in the comparison scripts. Next.
- **GI depth.** Still 4 with an environment table and 1 without; the cavity
  measurement in §8 pre-dates environment NEE and has to be redone before
  adopting Blender's 4 everywhere.

Suite: phiray 36/0; phiber + phiphx 726 passed, 6 failed — the six being
phiphx tests that need Blender's `tests/files` checkout beside the repo.

## 19. Three things the THYMUS comparison was measuring instead of the tracer (2026-09-26)

§18 left the figure mean at 87.8 against Cycles' 54 and blamed a mis-read
material. Chasing that number down found three errors, only one of them in
the tracer. In order of discovery:

### The rubber lining is black in Blender for a reason, and now here too

`mat_rubber_lining`'s Mix nodes are stored in the file with the idname
**`Undefined[ShaderNodeMix]`** — the file was last saved by a Blender that
did not have the Mix node, and that is what Blender writes for a node type it
cannot register. Every Blender since reads them as `NodeUndefined`, so the
link into Base Color is dead and Cycles renders the socket's **own default:
black**, with the roughness socket's 0.845 for the same reason. phiber's
colour fold returned nothing for the unknown node and fell back to the
Principled default of 0.8 grey — white rubber, and the "white body panels"
of every comparison so far.

`BGraph::input_or_default` now does what Blender does: a link the fold
cannot follow leaves the socket on its stored value. The Blender-side fold in
`truth/blender_bsdf.py` was making the same mistake, so the truth file said
0.8 too; it now says black, and `bsdf_truth` holds it — 17 materials match.
On the way, `shade/scalar.rs` ports Bright/Contrast, Gamma, the Math node's
forty-one operations and Clamp from Blender's GLSL, with the safe helpers
(`compatible_pow`, `compatible_mod`, `safe_divide`) so a division by zero
folds to zero as Blender's does.

### The measuring tool was leaving the lamps on

`examples/color_render.rs` — the still that every Material Preview
comparison came from — kept the file's four lamps in Material mode, where
Blender switches them off (and `blender_color.py` hides them for the
reference). The app itself was right (`world.rs::lit` drops them for the
studio look); only the example was wrong, so every "phiber is brighter"
number since §7 had a lamp in it. Off: figure mean 71.0.

### Specular light from the sky was counted twice

`env_light` sampled the environment table and evaluated the *whole* BRDF,
specular lobe included, weighted against the cosine sampler only; the GGX
reflection ray then gathered the same sky with weight 1. For a metal under
a flat sky that is `1 + pe²/(pe² + pb²)` of the truth — measured +5 %
(`examples/metal_probe.rs`: 0.0611 for an F0 of 0.058) — and under the studio
HDRI it is far worse where the table is dense, which is exactly the sky the
torso reflects: chest 103 against Cycles' 73.

`ggx_pdf` (Cycles' `bsdf_microfacet_eval` density, `G1 · D / (4 n·v)`) now
weights the specular lobe in `env_light` against the reflection ray and the
reflection ray against the table, the same power heuristic the diffuse lobe
already used; sheen and coat, which only light sampling gathers, keep weight
1, and a secondary hit in `indirect()` — which takes no reflection ray —
passes `mis_spec = false`. The probe reads 0.0573 for 0.058.

### Where it stands

| Material Preview, 32 samples, Filmic | Blender | before | now |
| --- | --- | --- | --- |
| figure mean | 56.5 | 87.8 | **61.4** |
| figure p50 | 43 | 68 | **53** |
| figure p95 | 139 | 218 | **130** |
| shin metal patch | 32.5 | — | **33.7** |
| chest patch | 73.0 | 102.7 | 85.8 |
| head patch | 92.5 | — | 78.7 |

What is left is smaller than what was fixed, and named: Cycles' Principled
defaults to **Multiscatter GGX**, so a rough metal returns ≈ F0 where our
single-scattering lobe loses energy (0.049 for 0.058 at roughness 0.5 in the
probe); the dielectric specular layer does not yet take its energy from the
diffuse beneath it (a white diffuse card reads 0.83 for Cycles' ≈ 0.81); and
the Fresnel is Schlick rather than Principled v2's F82 tint. The head's
highlight and the chest's shading are where those three show.

Suite: phiray 36/0; the `metal_probe` and `forest_probe` examples print the
analytic checks above.

---

## 20. Tracking — the rendering space (2026-09-26)

Status: **done**, **partial**, **todo**. Blender's Rendering workspace, read
from `startup.blend` with `bpy` (`phiber/tests/truth/blender_workspaces.py`):
an Image Editor showing `Render Result` with its sidebar open (1574 × 954 of
1920 × 1080), a Timeline across the bottom (74 px), and a Properties editor
down the whole right side (339 px) — no Outliner.

| # | Part | Blender source | Status | Notes |
| --- | --- | --- | --- | --- |
| 1 | Rendering workspace layout | `startup.blend` | **done** | Image Editor + Properties down the right + Timeline strip, from the `bpy` truth (`tests/workspace_truth.rs`); the side columns run to the status bar in every workspace now |
| 2 | Image editor header: mode, View / Image menus, image selector, slot, view layer, pass, display channels, gizmo / overlay toggles | `IMAGE_HT_header`, `template_image_layers` | **done** | §21; View and Image menus ported from `space_image.py` (Select and UV only in the UV editor), New / Open until a render exists, then Render Result with Slot / ViewLayer / pass / channels dropdowns |
| 3 | Render Result in the window at `sima->zoom`, centred, Home fits | `image_main_region_set_view2d`, `image_view_all` | **done** | §21; wheel zooms by 1.25, middle-drag pans, View ▸ Zoom ratios and Frame All write the view; a fitting render shows at 1:1, a larger one at the power-of-two fit |
| 4 | Render info bar (`Frame:1 \| Time: … \| Mem: … \| Scene, ViewLayer \| Sample n/N`) | `draw_render_info`, `ED_region_info_draw`, `make_renderinfo_string` | **done** | §21; a quarter-black bar `UI_UNIT_Y` tall across the top, `Finished` once done |
| 5 | Display channels and passes | `sima->display_channels`, `iuser->pass` | **done** | Color and Alpha (checker behind), Color, Alpha, Red, Green, Blue; Combined, Depth, Normal, DiffCol as raw, clipped values — what Blender shows |
| 6 | Sidebar: Image / Tool / View / Scopes | `IMAGE_PT_image_properties`, `IMAGE_PT_view_display`, scopes | **todo** | the tab rail exists; no panels |
| 7 | Render slots (8) and pass switching in the header | `template_image_layers` | **todo** | passes render (§17) and save; the header cannot switch what the window shows |
| 8 | Render border | `image.render_border` | **todo** | |
| 9 | Render, save, passes, camera, DoF, background job | §11–§17 | **done** | |
| 10 | Rendered mode against Cycles: linear accumulation, GGX, MIS, env NEE, denoiser | §7–§10, §18–§19 | **partial** | THYMUS figure mean 61.4 vs Cycles 56.5 |
| 11 | Multiscatter GGX, dielectric specular energy conservation, F82 Fresnel | `microfacet.h`, `principled.h` | **todo** | §19 |
| 12 | Node-network evaluation (procedural textures) or baking | `svm_*.h` | **todo** | §5 |
| 13 | Motion blur, panoramic cameras, volumes, adaptive sampling, clamping | Cycles integrator | **todo** | |
| 14 | Real GPU timing against Cycles | — | **todo** | never measured where it counts |

---

## 21. The Image editor is Blender's (2026-09-26)

`IMAGE_HT_header` (`bl_ui/space_image.py`) is now declared as the Image
host's header (`host/image/set.rs`): the editor switch, mode and menus on
the left; the image data-block after the first spacer (`template_ID` — New
and Open until a render has been asked for, then `Render Result`); after
the second spacer the gizmo and overlay toggles, and with an image the
render slot (`Slot 1`…`Slot 8`), view layer and pass dropdowns
(`template_image_layers`) and the display channels
(`part/image/header`). The menus are `IMAGE_MT_view` with its Zoom
submenu (`image.view_zoom_ratio` 1:8 … 8:1 write `image.zoom`; Frame All is
`image.view_all`) and `IMAGE_MT_image` in full; Select and UV appear only
in the UV editor, as `IMAGE_MT_editor_menus` shows them.

The window (`canvas/image/`) is `image_main_region_set_view2d`: the picture
centred at `sima->zoom` and moved by `xof / yof`, a wheel step scaling by
1.25 and the middle button panning. `image_view_all`'s rule is kept: a
picture that fits shows at 1:1, a larger one at the largest power-of-two
fraction that fits (a 1920 × 1080 render in a 1268 × 486 window is 1:4).
The pass and channels choose the bytes: Depth is grey, raw and clipped;
Normal and DiffCol raw; Color and Alpha draws the checker behind the
picture. Over it, `draw_render_info` — `ED_region_info_draw`'s quarter-black
bar, one `UI_UNIT_Y` tall — carries Blender's own text:
`Frame:1 | Time:00:12.34 | Mem:12M, Peak: 12M | Scene, ViewLayer | Sample
n/N`, then `Finished`. The header and the bar read a copy-free
`render_status` rather than cloning the shot's buffers every frame.

Colours: the window is the theme's editor back (the same slot the Timeline
uses), the bar's fill is Blender's literal quarter-black. Not yet: the
sidebar's panels (Image / Tool / View / Scopes — the tab rail exists),
render slots beyond the first holding their own image, the render border,
scrollers, and the pixel-info footer. Guarded by
`rendering_workspace_shows_the_render.rs`, which switches to the Rendering
workspace, checks the header before and after a 64 × 32 render, and finds
the picture at 1:1 with the bar over it.

## 22. Material Preview is deterministic, and a sample yields to the hand (batch 108, 2026-10-03)

The user's session journal (`app/journal.rs`, batch 107) from a release
build on a GTX 1660 Ti said where the slowness was, and it was not the UI
thread (1.8–5 ms a frame): every orbit step waited 0.3–1.2 s for its first
image, because a full-size sample of the previous view — 2–6 s per view on
Female-01's 4.5 M vertices — had to finish first; and Material Preview was
one noisy sample per frame of even a plain cube, because it was path traced
like Rendered.

### Material Preview as EEVEE draws it

`Mode::Preview` is its own kernel now (`trace/wgsl/preview.wgsl`), listed
among the deterministic modes: one dispatch is the frame, and the frame is
the same every time. What was precomputed for it (`trace/ibl.rs`, on the
CPU when the scene is built, into the atlas beside the studio HDRI):

- **irradiance** as nine spherical-harmonic coefficients over π
  (Ramamoorthi & Hanrahan), so a diffuse surface's radiance is
  `albedo · Σ cᵢ Yᵢ(n)`;
- **prefiltered copies** of the environment at roughness 0.2, 0.4, 0.6,
  0.8 and 1 (the image itself stands for 0): the GGX lobe convolved over
  the sphere (Karis's split sum, `N = V = R`), 128 × 64 each, with filtered
  importance sampling (each of 64 samples reads the image at the level its
  lobe covers) so 64 samples are smooth where plain sampling speckles.

The kernel then shades a surface as `albedo · irradiance(n) · (1 − F) +
prefiltered(R, rough) · envBRDF(f0, rough, n·v) + lamps + emission`, with
Karis's analytic fit for the split sum's second term, Lagarde's
roughness-aware Fresnel, one shadow ray per lamp (every lamp, hard shadows),
alpha blended deterministically through up to six layers, glass continued
as one refracted ray, and the three-point rig only when there is no light
of any kind — the same rule as the path tracer's. Textures are read
trilinearly at the mip level the pixel's footprint asks for (`mip_level`:
the triangle's texel density times the footprint, widened at a grazing
angle); the footprint is `t · (1 / (scale · w)) / dist` in perspective and
one pixel's world width in orthographic. What EEVEE has and this does not,
yet: soft shadows, screen-space reflections of other objects, ambient
occlusion.

The atlas head's width carries the level count in its top byte
(`head_levels`); `Atlas::add` builds the chain; `BRayKey` now tells a
Material scene from a Rendered one, since they hold different images.
`Tracer::denoise` is a no-op for a deterministic mode (it was filtering
Solid frames that wrote no guides).

Guarded by `phiray/tests/preview.rs`: one pass is the frame and the same
frame again; a flat card's neighbouring pixels differ by under 1/255 where
the path tracer's first sample differs by more than four times that; a
64 × 64 single-texel checker tiled eight times over a card reads as one
grey across it (no moiré); a lamp casts a shadow where the blocker is and
not beside it.

### A sample in slices

`Tracer::sample_in(gpu, cam, w, h, slices, go)` dispatches a sample as
`slices` pixel ranges (`Camera::slice`, `Pass::add_over` / `run_step`), the
device drained after each and `go()` asked whether to carry on; `None`
abandons the sample and the next one starts the image over, since a frame
half of whose pixels have one sample more is no frame. The render thread
(`rendered/worker.rs`, with `pace.rs`, `posted.rs`, `job.rs` and `build.rs`
split out of it) cuts a full-size sample into enough slices for one to take
about 25 ms, from how long the last full sample took per pixel, and `go` is
"no newer view or scene has been posted" (`BPosted`, two atomics the UI
side writes as it posts). And while views keep arriving — a drag — the next
sample stays at the coarse size (an eighth) rather than doubling, as Cycles
keeps a navigated viewport at its start size; it grows once the hand rests.
`phiray/tests/slices.rs`: seven slices are the same pixels as one; a sample
abandoned after its first slice leaves the next at sample 0.

### The tracer accounts for itself

The journal now carries the tracer's lines: `scene built — N triangles, M MB
of images: triangles X ms, tree and upload Y ms` (the 184 s the user waited
after opening Female-01 will be named on the next run), `scene posed in X
ms`, `tree rebuilt over the pose in X ms`, and every 64 samples a tally —
how many, over how long, at what size, the average and slowest trace, the
average readback, how many were abandoned for a newer view.

## 23. Objects are instances: a move is a matrix (batch 112, 2026-10-03)

Blender's viewport (and Cycles' BVH) keeps each object's mesh once, in the
object's space, and places it with `object_to_world`; a transform touches
nothing but that matrix. The tracer baked everything into one world-space
triangle soup, so a grab, a turn or a scale meant every vertex of the scene
transformed on the CPU, uploaded again, and the one tree refitted and then
rebuilt — the "objects too slow to move" of the user's list.

### Two levels of tree

`phiray/trace/instance.rs` — an `Instance` is a world matrix (Blender's
`m[col][row]`) and the ranges of faces and vertices it owns.
`Scene::instance(world)` begins one; the vertices and faces added next are
its, in its own space; a face packs `material | instance << 16` in its
fourth word (`face_mat`, `face_inst`; 65 536 instances, the rest share the
last). A scene built from world-space triangles has one instance at the
identity, so every test and caller from before draws exactly as it did.

`phiray/trace/trees.rs` — `Trees::build` makes a tree per instance over its
faces in object space (in parallel) and a top tree over the instances'
placed bounds (`Instance::world_box` of each root). The kernel sees one
node array: the top tree first in `2n − 1` reserved slots (so a top tree
built again with a node fewer never moves what follows), then each
instance's tree with its children and face slots shifted into place. A
top-tree leaf carries the `INST` bit and ranges over slots of
`inst_order`; the instance table behind the atlas in `data`
(`Camera.insts = [first word, count]`) holds each instance's world rows,
inverse rows, tree root, face range and an identity flag.

`bvh.wgsl` — `trace` walks the top tree; at a leaf it calls `trace_blas`
for each instance named, which takes the ray into object space by the
inverse (direction left unnormalised, so `t` means the same along both
rays), walks that instance's tree, and writes world `t` into the shared
best hit. `tri_at` places a hit face's corners by the world rows and its
normals by the inverse-transpose, so every shader — Solid, Wire, Preview,
Rendered, the light table's emitters — sees world-space triangles as
before. An instance at the identity skips both transforms.

### What a move costs now

`Tracer::reinstance(worlds)` sets the matrices, rebuilds the top tree
(`retlas`) and writes it and the instance table in place
(`GpuArray::write_at`) — a few kilobytes for hundreds of objects, nothing
of the meshes — and starts the image over. `repose` (vertices moved: a
rig's frame) refits each instance's tree and the top tree; `retree`
rebuilds each instance's tree once the motion stops, the image keeping
its samples. phiber's `rendered/place.rs` picks: the very same meshes,
materials and textures (`Arc` identity) under other matrices are placed;
otherwise posed; otherwise built.

`phiray/tests/instances.rs`: a scaled, turned instance draws what the same
wall baked in world space draws (within edge rounding); a moved instance
draws as a scene built at the new place and starts at sample 0; a new
tree after a move keeps the samples. Measured on the software GPU in the
container, the tests only; the user's journal will carry `Tracer: N
objects placed in X ms` from the next run.

### Still open

Instancing the *same* mesh twice (Blender's linked duplicates, particle
instances) shares nothing yet — each object still uploads its own copy of
the vertices; the instance table is ready for it (two instances naming
one face range) once the scene builder dedups by `Arc`. The top tree is
rebuilt whole on every move (cheap at hundreds of objects, not at tens of
thousands); a refit would do when only one moved.

## 24. Solid finds its surfaces the way Workbench does: a visibility prepass (batch 114, 2026-10-03)

Blender's Solid and Wireframe viewports are Workbench, a rasterizer: the
draw manager keeps one GPU batch per mesh (`DRW_cache_mesh_surface_get`),
`workbench_mesh_passes.cc` draws them through the raster pipeline with a
depth test (the prepass), and the composite pass shades what each pixel
sees. Until this batch Phiber's Solid and Wireframe cast a ray per pixel
through the BVH to find that surface — a ray tracer doing a rasterizer's
job. The shading was already Workbench's (`workbench.wgsl`); the finding
was not.

### The prepass

`phiray/src/gpu/raster.rs` (`Visibility`) and `trace/wgsl/raster.wgsl`: a
wgpu render pipeline draws every face of the scene as three vertices
**pulled from the tracer's own buffers** — the vertex shader reads the
face's vertex ids (`face`), the vertex (`vert`) and the instance's world
rows (`inst_at`, batch 112's table), so no second copy of the mesh exists
on the GPU; the batch cache is the scene's own storage. The projection is
`solid.wgsl`'s ray model inverted, so the pixel a point lands on is the
pixel whose ray would have hit it:

- perspective: the eye at `pivot + back · dist / units`; with `wv` the
  depth along the view axis, `x_clip = 2 (cx · dist · scale + pan.x · wv)`,
  `y_clip = 2 · aspect (cy · dist · scale − pan.y · wv)`, `w = wv`;
- orthographic: `x_ndc = 2 (cx · units · scale + pan.x)`, `y_ndc = 2 ·
  aspect (cy · units · scale − pan.y)`.

Depth is reversed — `near / wv` in perspective, the scene's reach either
side of the pivot in orthographic, compared *greater* — for float precision
far from the eye. The fragment writes four words into an `Rgba32Uint`
target: the face, the barycentrics `u, v` (the hardware's
perspective-correct interpolation of the corner weights is the tracer's
Möller–Trumbore `u, v`) and the distance along the ray; the target is
copied into a buffer the kernels bind after the image (`vis2.wgsl`,
`primary`). `solid.wgsl` and `wire.wgsl` call `primary(o, d, i)` where they
called `trace`; X-ray (every surface along the ray) and the tiles of a
still larger than one binding still trace.

`phiray/tests/raster.rs`: Solid and Wireframe rasterized against the same
frames traced, orthographic and perspective, over three placed instances
(one turned and scaled): **0 of 5184 pixels differ** in each of the four
cases — the same hit, the same shading, pixel for pixel. X-ray is traced
either way and identical; a moved instance (`reinstance`) is drawn where it
went.

### What it costs, and where it is on

| THYMUS (259 objects, 436 k faces), 1200 × 700, lavapipe (software) | Solid | Wireframe |
|---|---|---|
| Primary hits traced through the BVH | 83 ms | 82 ms |
| Primary hits rasterized (1.3 M vertices pulled from storage) | 378 ms | 377 ms |

A software rasterizer is slower than a software BVH walk for a scene of
this size, so the prepass is **off on software adapters** and **on for
hardware** (`Tracer::raster`, from `GpuAdapter::is_software`), where a
GPU rasterizes 436 k triangles in well under a millisecond and the frame
is left with the shading alone. The journal's `scene built` line says
which path a session took. On the user's GTX 1660 Ti the traced Solid
frame was already 0–18 ms (batch 111's journal); the first measurement of
the rasterized one is the next journal's.

### Still to port from Workbench

- ~~Anti-aliasing~~ — §25, batch 118. Still: SMAA on the first frames,
  `Scene.display.viewport_aa` / `render_aa` from the file.
- ~~The object outline and the curvature effect~~ — §26, batch 119.
  Still: Cavity ▸ World (SSAO, `workbench_cavity.bsl.hh`), shadows
  (`workbench_shadow.cc`), matcaps, `Scene.display` from the file
  (`viewport_aa`); ~~`shading.light`, `color_type`, the popover's
  Lighting and Color~~ — §27, batch 121 (MatCap, Attribute and Texture
  still draw as Studio / Material).
- Material Preview on EEVEE's model (a raster prepass too; the prefiltered
  environment shading of batch 108 stays).

## 25. Solid and Wireframe are anti-aliased as Workbench is: eight jittered samples (batch 118, 2026-10-03)

Workbench does not anti-alias an edge by supersampling it; it draws the
view again and again, each time with the projection nudged by less than
a pixel, and accumulates (`workbench_effect_antialiasing.cc`). Eight
samples by default (`U.viewport_aa`, `render_aa` for F12), the first one
unjittered so the view is sharp at once, SMAA over the first few so they
are not aliased while the rest land. Until this batch Phiber's Solid was
the first of those frames and stopped.

### The table

`BLI_jitter_init(n)` (`jitter.cc`) seeds `RandomNumberGenerator` with
`31415926 + n`, lays `n` points along a diagonal with random noise, and
relaxes them 24 times — `BLI_jitterate1` pushing each away from its
neighbours and their wrapped copies within `1/√n`, `BLI_jitterate2` the
same along each axis within `1/n` — then centres them in `−½ ..= ½`.
`TaaSamples::init_samples` moves the point nearest the centre onto it,
draws it first, and orders the rest so each is the farthest from the one
before, in `−1 ..= 1` (half a pixel either way once `window_translate_m4`
divides by the resolution). `phiray/trace/taa.rs` is that code line for
line, generator included (`0x5DEECE66D`, `0xB`, 48 bits, `get_double =
int32(x >> 17) / 2³¹`), so the table is Blender's table.

### The gather

For each sample Workbench splats the new frame over its 3 × 3
neighbourhood with `filter_blackman_harris(|offset − neighbour|, 2)`
weights (`setup_taa_weights`), in `log2(colour + 1)` so a highlight does
not alias (#112211), and resolves by `exp2(sum / weight accumulated) −
1`. In Phiber the Solid or Wireframe kernel writes its sample into the
image's scratch words (`taa_sample`, `util.wgsl`) and a `Taa` kernel
(`wgsl/taa.wgsl`) gathers them into running sums — frame 0 resets — and
packs the resolved pixel. `Camera.taa` carries the jitter and the weight
so far. The ray is moved by the jitter (`taa_jitter`) and the raster
prepass by the same amount in clip space (`raster.wgsl`), so `raster.rs`'s
test still finds the two paths drawing the same pixels.

### Tiles

The gather clamps at a tile's edge. A frame rendered in bands is now
drawn a row over into each neighbour and laid in without that row
(`rendered/still.rs`), so the bands are the whole frame pixel for pixel
— `tiles_laid_together_are_the_whole_frame` and
`a_still_rendered_in_bands_is_the_still_rendered_whole` say so.

### What it costs

Eight dispatches of the shading kernel and eight of the gather where
there was one. The first is the frame of before, shown at once; the
others land while the view rests, as Workbench's do, and any navigation
starts the count over. On the raster path (hardware) a Solid sample was
0–18 ms in the user's journal, so the eighth lands within a few frames;
the next journal's `solid` lines will say. A sliced sample runs the gather
after its last slice.

### Not ported yet

SMAA on the first frames (`workbench_effect_smaa_frag.glsl`, three
passes with the area and search textures) — the edges are aliased until
the eighth sample. The sample counts from the file
(`Scene.display.viewport_aa`, `render_aa`) — eight always. The
accumulation is of display-space colour with the grid in it; Blender
accumulates the linear colour before the view transform and composites
the overlays after.

## 26. The object outline and the curvature effect (batch 119, 2026-10-03)

Workbench's Solid view draws two things over its composite that a plain
shaded hit does not have. The **object outline**
(`workbench_effect_outline.cc`): the prepass writes every fragment's
object id (`resource_id + 1`) to a buffer, and a full-screen pass samples
it at the pixel and its four neighbours one `U.pixelsize` away,
`outline_opacity = 1 − dot(equal(center, adjacent), 0.25)`, and blends
`object_outline_color × opacity` premultiplied over the colour. The
**curvature effect** (`workbench_curvature.bsl.hh`), part of the opaque
composite when Cavity's type is Screen or Both: from the prepass's
view-space normals, `normal_diff = (up.y − down.y) + (right.x − left.x)`,
`curvature_soft_clamp`ed by `0.5 / ridge_factor²` (positive, a ridge) or
`0.7 / valley_factor²` (negative, a valley), ×2, and the colour is
multiplied by `clamp((1 − cavity)(1 + edges)(1 + curvature), 0, 4)` —
unless the ids across the pixel differ (an object's edge) or all are 0
(the background).

Phiber: the Solid kernel (`solid.wgsl`) now leaves per pixel its
scene-linear colour, the grid's premultiplied colour and coverage, the
object id (the instance plus one, 0 for a miss) and the view normal in
the scratch words the denoiser never touches in that mode (`fx_store`),
and still writes the plain sample. An `Fx` kernel (`fx.wgsl`), between
the shading kernel and the TAA gather, reads the four neighbours
(clamped at the image's edge, as a texture fetch is), applies the
curvature factor to the colour of a surface pixel, blends the outline
over it in linear space, puts it through the view transform and lays the
grid over — the sample the gather then filters. `Camera.fx` is `[outline
on, 0.5 / ridge², 0.7 / valley², pixelsize]` and `Camera.outline` the
colour, scene-linear; the tracer adds the step when either effect is
asked for and X-ray is off (the transparent pass has neither), so a
Solid frame with both off is the frame of batch 118 untouched.

The settings come from the file: `View3DShading.flag` bits 0 and 5,
`object_outline_color`, `cavity_type` and the four factors
(`phiphx::SceneSolidFx`), seeded into `view3d.shade.<rna>` as the view
opens and read by `BSolidFx` (`scene/solid_fx.rs`). Blender's startup
file has the outline on and black and Cavity off, and so does Phiber's
factory state.

Bands of a still larger than the device binds overlap by two rows now
(the effects and the gather each read a neighbour) and stay seamless.

Not yet: Cavity ▸ World — the screen-space ambient occlusion
(`cavity_compute`, the Alchemy AO: 16 Hammersley disk samples a sample
rotated by a 64 × 64 blue-noise jitter from `eevee::lut::blue_noise`,
`matcap_ssao_distance` 0.2, `matcap_ssao_attenuation` 1 from
`Scene.display`) — the table is the reason it waits; shadows
(`workbench_shadow.cc`); and a final render's own settings
(`Scene.display.shading`), so F12 draws no outline. The Shading popover
switches the outline and Cavity in the app since batch 120.

### A hand-built camera's Material Preview (batch 120)

`color_render.rs`'s image had gone dark and mirror-like on a white
sweep: Material Preview has lit from the studio image's irradiance and
prefiltered copies since §22 (batch 108), through `Camera::preview`,
which only `lit` set — the example assembles its own camera. `still`
now fills those words for a Material Preview camera that lacks them
(`world::studio_lighting`, the one owner, used by `lit` too), and the
examples' `sky.a` is 0 (it is the theme's sweep, not opacity). Female-01
through the example matches Blender's `blender_color.png` again.

## 27. Solid's Lighting, Color and Background (batch 121, 2026-10-03)

Workbench picks each object's `Material` record by the view's
`color_type` (`workbench_engine.cc`, `get_material`): the material's
viewport colour (Material, Texture when no image), `ob.color` (Object), a
hue hashed from `ob.id.name` (Random), the view's `single_color`
(Single), or the vertex colour (Attribute) — the last four with
`pack_data(0.0, 0.4, alpha)`, no metal and roughness 0.4
(`workbench_materials.cc`). The composite then lights by `shading.light`:
Studio (`get_world_lighting`), MatCap (`get_matcap_lighting`) or Flat
(`color.rgb = base_color`). The background is the theme's, the world's
`horr/horg/horb`, or `shading.background_color`
(`ED_view3d_background_color_get`).

Phiber: `Camera.shade = [light, color_type]` (the kernel keeps Studio at
0 so a zeroed camera is unchanged; the colour word is
`eV3DShadingColorType`'s) and `Camera.single`. The instance record has
four spare words, now the object's colour (28–30, linear) and the murmur
hash of its id name (31), written by `Scene::instance_of`
(`rendered/scene.rs` hands both). `solid_at` swaps the base colour,
roughness and metal for Object / Random / Single and returns the colour
unlit for Flat; `random_color` is `BLI_hash_int_01` and `hsv_to_rgb_v`
in WGSL over the hash `hash.rs` computed on the CPU — Blender's
`BLI_hash_mm2` and Jenkins' `final` mix, line for line, with the `OB`
prefix and the terminator the id name carries. The sky is set on the CPU
(`BSolidShade::background`, `rendered/view.rs`), display-encoded as the
theme's is; World without a world falls back to the theme, as Blender
does.

Not ported: matcap images (MatCap lights as Studio), the studio-light
picker and world-space lighting, Attribute (needs vertex colours in the
tracer's vertex record) and Texture (Solid uploads no image by design,
§24), the wireframe colour type, Object colour's alpha, and
`Scene.display.shading` for a Workbench F12.

## 28. Cavity ▸ World — screen-space ambient occlusion (batch 122, 2026-10-03)

Workbench's composite reads a depth and a normal buffer and, for each
opaque pixel, walks a jittered disk of neighbours (`cavity_compute`,
`workbench_cavity.bsl.hh`, after the Alchemy AO paper): the disk is
`CavityEffect::load_samples_buf`'s Hammersley points — angle from the
radical inverse of the index plus `it_add = (i / samples) · 0.499`, radius
`fmod((i + 0.5 + it_add) / samples, 1)` — one run of `ssao_samples` per
TAA sample out of `min(samples_len · ssao_samples, 512)`; each pixel turns
the disk by its blue-noise angle and slides the radius by `(bn − 0.5) /
sample_count` (`load_jitter_tx`, EEVEE's `blue_noise` table), and the disk
is `cavity_distance` wide at the pixel's depth. A neighbour's view
position minus the pixel's, against the normal: positive is a cavity,
negative an edge, both over `1 / (len (1 + len² attenuation))` and past
the `0.05 len + 0.0001` bias; the background takes the pixel's depth less
`cavity_distance`. `cavities · valley` is clamped to 1, `edges · ridge`
not, and the colour is multiplied by `(1 − cavities)(1 + edges)(1 +
curvature)` clamped to 4.

Phiber: the opaque pass stores the view depth (word 20n) beside the id and
normal; `fx.wgsl`'s `cavity` is the shader above over the stored buffers,
with `view_pos` standing in for `drw_point_screen_to_view` (the canvas
point of `solid.wgsl`'s `main`, times `−z / dist` under perspective) and
the offset in pixels from the frame's view span `1 / scale` (`1 / (scale
· aspect)` up). The blue-noise table is `blue_noise.rs`, bound to the
pass as `[cos φ, sin φ, bn₁]`; the radical inverse is `reverseBits(i) ·
2⁻³²`. `Camera.cavity` and `.ssao` carry the settings; `Scene.display`'s
three come from the file through phiphx `SceneSsao`.

Not ported: `Scene.display.shading` for a Workbench F12 (the render path
draws no effects), and Workbench's shadow and depth of field.

## 29. Shadow — the one directional light (batch 123, 2026-10-03)

Workbench's Shadow (`V3D_SHADING_SHADOW`) is a stencil shadow volume pass
(`workbench_shadow.cc`): each casting object's silhouette is extruded along
`direction_ws` — `Scene.display.light_direction` with y and z swapped and x
and z negated — and the composite darkens the covered pixels. The factor
is `get_shadow`'s: `light_factor = −N · shadow_direction_vs`, `mix =
smoothstep(shadow_shift, shadow_focus, light_factor)`, zero where the
stencil is set, then `mix · shadow_intensity + (1 − shadow_intensity)`;
`shadow_focus` here is `1 − clamp(focus, 0.0001, 0.99999)(1 − shift)`.

Phiber: `solid.wgsl`'s `shadow_factor` is that factor with the stencil
replaced by a shadow ray — from the hit, nudged off the surface against the
light's travel, `trace(org, −dir, INF, true)`: any surface means covered.
The direction reaches the kernel in world space (`Camera.shadow`) and is
taken into view space by the camera's axes for the dot with the stored view
normal. Applied to the opaque colour before the effects pass stores it, so
`(1 − cavity)(1 + edges)(1 + curvature)` multiplies over it as in Blender's
composite; X-ray takes none.

Not ported: `OB_DRAW_NO_SHADOW_CAST` (an object's Viewport Display ▸ Shadow
off), and `Scene.display.shading` for a Workbench F12.

## 30. Backface Culling and Specular Lighting (batch 124, 2026-10-03)

Two of `View3DShading.flag`'s switches. `V3D_SHADING_BACKFACE_CULLING`
sets `DRW_STATE_CULL_BACK` on Workbench's mesh passes, so a face wound away
from the viewer is never rasterized; the tracer tests the primary hit's
geometric normal against the ray and traces on past a turned face, the
sky where nothing is behind. `V3D_SHADING_SPECULAR_HIGHLIGHT` is
`WorldData.use_specular`: clear, `get_world_lighting` takes the base colour
as the whole diffuse colour and no specular at all — `workbench.wgsl`
branches the same way. Both ride in `Camera.shade`'s spare words, inverted
where Blender's default has the flag set.
