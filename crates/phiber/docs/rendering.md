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
one pixel's world width in orthographic. Shadows were hard here until
§44, which aims the ray over the light's shape as EEVEE does. What EEVEE
has and this does not, yet: screen-space reflections of other objects and
ambient occlusion.

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

- ~~Anti-aliasing~~ — §25, batch 118; ~~SMAA on the first frames and the
  sample counts~~ — §39, batch 139.
- ~~The object outline and the curvature effect~~ — §26, batch 119.
  Still: Cavity ▸ World (SSAO, `workbench_cavity.bsl.hh`), shadows
  (`workbench_shadow.cc`), matcaps; ~~`shading.light`, `color_type`, the popover's
  Lighting and Color~~ — §27, batch 121 (MatCap, Attribute and Texture
  still draw as Studio / Material).
- Material Preview on EEVEE's model (a raster prepass too; the prefiltered
  environment shading of batch 108 stays). ~~Soft shadows~~ — §44, batch
  144. Still: screen-space reflections, ambient occlusion.

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

~~SMAA on the first frames~~ and ~~the sample counts~~ — §39, batch 139.
The accumulation is of display-space colour with the grid in it; Blender
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

## 31. Color ▸ Attribute (batch 125, 2026-10-03)

Workbench's `V3D_SHADING_VERTEX_COLOR` shades with the mesh's active colour
attribute at `pack_data(0.0, 0.4, 1.0)` and falls back to the material
colour for a mesh without one (`SceneState::has_color`). Phiber: phiphx
reads the attribute off Blender 5's `attribute_storage` — byte or float,
point or corner domain, bytes decoded from sRGB — to a colour per loop,
then per triangle corner (`SceneCorners::colors`); the build paints the
tracer's vertices with them as sRGB bytes (`Scene::paint`), uploaded after
the instance table; `solid.wgsl` decodes and interpolates the three
corners. The modifier stack carries them with the UVs (batch 130).

## 32. Lighting ▸ MatCap (batch 126, 2026-10-03)

Workbench's MatCap samples a studio-light image by the view normal
(`get_matcap_lighting`: `matcap_uv_compute(I, N, flipped)` over the diffuse
and specular textures, `diffuse · base_color + specular · use_specular`).
Blender's matcaps are DWA-compressed multilayer EXRs the Rust `exr` crate
cannot read, so they ship decoded as 256² sRGB PNGs at half scale
(`assets/matcap/`, 1.1 MB, CC0) and are decoded back to linear once per
name (`scene/matcap.rs`). The Solid build uploads the chosen one's images
first in the atlas and `solid.wgsl` samples them with `sample_env` (rows
turned); `Camera.matcap` carries the slots and the flip. A name the set
lacks — the user's own, or Blender's old default `basic_1` — falls to the
first by name, as `BKE_studiolight_find_default` does.

## 33. Color ▸ Texture (batch 127, 2026-10-03)

`V3D_SHADING_TEXTURE_COLOR` draws the material's image as the colour
(`workbench_image_lib.glsl`), Material where a mesh has no UVs or image.
Solid's build uploads the base-colour images under Texture and nothing
else; `solid.wgsl` takes the sampled image as the base colour.

## 34. F12 with the Workbench engine (batch 128, 2026-10-04)

`workbench_render.cc` renders F12 with `Scene.display.shading` and
`render_aa` when the engine is `BLENDER_WORKBENCH`. Phiber: phiphx reads
that shading with the same `View3DShading` reader as the viewport's
(`BlendShading`, `SceneDisplayShading`), and `scene/render_engine.rs`
turns it into the still's look and the camera's words — Solid or
Wireframe, the matcap and images, every effect — at `render_aa` samples
without denoise; other engines path-trace as before.

## 35. The user's matcaps (batch 129, 2026-10-04)

Blender lists a user's own matcaps from `BLENDER_USER_DATAFILES/studiolights/
matcap/` beside the shipped ones. Phiber scans that folder once (every
Blender version under the config root, newest first), lists the images after
the shipped set and decodes one when named — LDR as sRGB, EXR / HDR as
linear, fitted to 256², no specular pass; a missing or unreadable one falls
to the default as `BKE_studiolight_find_default` would.

## 36. An object's Shadow switches (batch 131, 2026-10-04)

Blender keeps two per-object shadow toggles: Viewport Display ▸ Shadow
(`Object.dtx & OB_DRAW_NO_SHADOW_CAST`), which Workbench reads for Solid
and Wireframe, and Visibility ▸ Ray Visibility ▸ Shadow
(`Object.visibility_flag & OB_HIDE_SHADOW`), which Cycles and EEVEE read.
phiphx reads both into `SceneObject::shadow` (`BlendShadowCast`); the
viewport build asks `casts_in(shading.uses_viewport_color())` and tells
the tracer `Scene::hide_shadow(inst)`. The kernel carries it as bit 1 of
the instance's flag word: `trace_blas` skips that instance for shadow
rays only, so the object is still drawn, lit and occluding for the camera.

## 37. The Shadow checkboxes (batch 132, 2026-10-04)

The two switches of §36 are now in the Properties editor's Object tab —
Viewport Display ▸ Shadow (`show_shadows`) and Visibility ▸ Ray Visibility
▸ Shadow (`visible_shadow`), on by default — seeded from the active
object, kept per object in `ctx.scene.shadow_cast` (`scene/shadow.rs`),
laid over the viewport's object list and written to the file on save
(`dtx` and the visibility field). The build's choice of switch per shading
mode is unchanged.

## 38. Depth of Field (batch 133, 2026-10-04)

Workbench's `workbench_effect_dof.cc`, through the camera looked through
with the camera's own Depth of Field: `dof.wgsl` ports its five shaders —
the CoC-aware half-resolution prepare and two downsamples, the 49-sample
bokeh gather shaped by the blades, the 3 × 3 median and the resolve
blending the blur in by the circle of confusion — as stages of the effects
kernel, every sample before the anti-aliasing gather, the noise offset
walking with the sample. `Camera::dof` carries `DofPass::init`'s words
(`BSolidFx::dof_words` from `BCameraLens`, the camera's lens, sensor,
f-stop, focus and clip end; the frame's share of the region scales the
sensor), zero unless Solid's `use_dof`, a camera view and the camera's
`CAM_DOF_ENABLED` all hold — in the viewport and for a Workbench F12.

## 39. SMAA over the first samples, and the sample count (batch 139, 2026-10-04)

Workbench runs SMAA over its anti-aliasing accumulation while the
jittered samples are still landing (`workbench_effect_antialiasing.cc`:
`smaa_mix_factor_ = 1 − sample / 4`, so the first frame is all SMAA, the
fourth onward the plain resolve), which is why a Solid view in Blender
never shows a staircase, not even for a frame. The three passes are
Jimenez's SMAA 1x (`gpu_shader_smaa_lib.glsl`, `SMAA_PRESET_HIGH`:
threshold 0.1, 16 search steps, 8 diagonal, corner rounding 25) with the
luma weights all 1: **edge detection** over the accumulation's luma with
local contrast adaptation, **blending weights** from the searches along
the edge line — the orthogonal ones two texels a step with the search
texture adding the last step, the diagonal ones — and the area texture's
coverages, corner patterns rounding the ends; and **neighbourhood
blending**, which mixes a pixel with the neighbour the weights point at,
then the stage-2 resolve: the blend mixed with the plain accumulation by
`mix_factor`, over the weight accumulated, out of log space.

Phiber: `smaa_lib.wgsl` and `smaa.wgsl` are the library line for line as
compute, a `Smaa` kernel after the gather with the stage in `Camera::
slice.z` (`Tracer::tail`); the colour it reads is the gather's running
sums (words n..4n, Blender's `taa_accumulation_tx_`), the edges and the
weights live in the words the frame is done with (17n..23n, the opaque
pass's ids and normals and the Depth of Field's half image), and the area
(160 × 560 RG8) and search (64 × 16 R8) tables are one storage buffer
from `BLI_smaa_textures.h`'s bytes (`smaa.rs`, `smaa/area.bin`,
`smaa/search.bin`; MIT). Texture fetches are bilinear with the edge
clamped, as the library asks and as Blender bound them until the draw
manager rewrite (`GPU_SAMPLER_FILTER`; today's `bind_texture` takes the
default nearest sampler for the edges and weights, which the library's
bilinear tricks were not written for). Texcoords are `(x + ½) / w` with
row 0 at the top — a vertical mirror of Blender's, which SMAA does not
mind. From the fourth sample the stages return at once.

**The count.** `SceneState::samples_len`: a Solid or Wireframe viewport
draws `U.viewport_aa` samples — the user preference, 8 by default, not
the scene's — `scene->display.viewport_aa` only when a Rendered or
Material view runs on the Workbench engine, and `render_aa` for a render;
0 (`SCE_DISPLAY_AA_OFF`) is no anti-aliasing at all, 1 (`_FXAA`) the one
sample with SMAA left on it, else 5, 8, 11, 16 or 32 with a jitter table
each (`TaaSamples::x5 … x32`). Phiber: `taa::offset(k, n)` has the five
tables (`taa::COUNTS`, `taa::count` falling back to the five-sample one
as `setup_view`'s `switch` does); `Tracer::set_aa_samples` is the count
apart from the path tracer's target — until now a Workbench F12's
`render_aa` reached `set_samples` and was ignored, eight always — and a
new count starts the samples over (`reset_taa`). Preferences ▸ Viewport
▸ Quality ▸ Viewport Anti-Aliasing (`part/pref/quality.rs`,
`scene/viewport_aa.rs`) is the preference, Blender's seven items over
`ctx.pref.viewport_aa`, session state like the render device; the
viewport posts a new view when it changes (`BRayShot::aa`), the SSAO disk
runs over that many samples, and a Workbench render takes `render_aa`
with 0 and 1 meaning what they do in Blender. A still the single pass
leaves SMAA on is drawn in bands 40 rows over into each other and its
region 40 pixels wider (`Tracer::seam_rows`: as far as SMAA reads), where
two sufficed for the gather.

Tests: phiray `tests/smaa.rs` (the first frame of a slanted edge has
pixels between the two colours where a staircase has none — 0 with SMAA
off, 48+ on — and the settled frame is the gather's; off, the single
pass, five, a count without a table, a change of count), `trace/smaa.rs`
(the tables are Blender's bytes), `taa.rs` (every count's table);
phiber `tests/viewport_anti_aliasing.rs` (the Quality panel, the default,
a choice reposts the view), `scene/viewport_aa.rs`, `pref/quality.rs`,
`rendered/still_tests.rs` (bands under the single pass are the whole
frame), `rendered/region.rs`.

Not yet: the preference saved to disk (no preference is); a Rendered or
Material view on the Workbench engine taking `Scene.display.viewport_aa`;
Blender's one-sample-with-SMAA while navigating (`is_navigating`:
Phiber's small first frames of a drag are each sample 0, so SMAA is on
them, which is the same picture); the linear-light accumulation.

## 40. The surface is what the Material Output is linked to (batch 140, 2026-10-04)

Until now a material was its tree's **first** Principled BSDF — whatever
the Material Output was linked to. In the production files that was wrong
more often than it looks: `LAND`'s `HB04_Glass_Shader` links a Glossy
BSDF to the output with a Principled left over beside it; `FLOWER-CITY`
has a dozen `mat_glass.*` with an idle Glass node next to the linked
Principled (fine by luck), a `Material` that is Mix Shader(Fac = Light
Path ▸ Is Shadow Ray; Glass, Transparent) — the fake-caustics setup — and
two `MOSS-GRASS-TEXTURE` materials that mix a Diffuse and a Principled by
a Voronoi through a colour ramp; and the texture and normal-map searches
started from that first Principled too, so a Mix of two Principleds read
the wrong one's image.

Blender: `ntreeShaderOutputNode` — the output with `NODE_DO_OUTPUT`, else
the first — and then Cycles' SVM and EEVEE's codegen evaluate the closure
tree itself. Phiber (`phiphx/blend/shade/surface.rs`) walks it the same
way and folds it to the one `BlendBsdf` the tracer takes: a **Mix
Shader** lerps the two shaders' parameters by its factor (folded through
`eval.rs`, so a ramp or a value feeds it; a texture folds to its mean, as
colours do), an **Add Shader** averages the lobes and sums the emission,
a reroute passes through, and each closure node lands as the Principled
parameters that draw most like it — **Glass / Refraction** as full
transmission at their colour, roughness and IOR; **Transparent** and
**Holdout** as alpha 0; **Emission** as its radiance over black; **Diffuse
/ Translucent / Toon** as a rough dielectric; **Glossy / Anisotropic /
Metallic** as a metal at its roughness and anisotropy; **Sheen / Velvet**
as the sheen lobe; **Subsurface Scattering** as the subsurface lobe.
**Light Path** folds for a camera ray (Is Camera Ray 1, the other
Is-rays 0, the depths 0), so the fake-caustics `Material` folds to the
glass the camera is shown — water at IOR 1.33 — where it used to fold
half transparent. A group, a volume or another node the fold does not
know leaves the surface `None`, and the material falls back to its
Principled as before. The Principled the surface *reaches* (through Mix /
Add, the first input first) is now the one `principled`, the base-image
search and the normal-map search use.

Exact where the shaders share a lobe (two Principleds, a Principled with
a Transparent, Glass with Glossy), an approximation otherwise: a Mix of a
Diffuse and a Glossy is one half-metal, not two lobes — the fold is what
anything wanting one set of numbers reads (Solid's colour, the panels);
the tracer takes the two lobes themselves since batch 141 (§41).

Tests: `phiphx/tests/surface_truth.rs` (THYMUS: the linked Principled
keeps its own numbers, the idle Glass node does not win; LAND: the Glossy
is a metal at IOR 1.5; FLOWER-CITY: the Light Path mix is the glass, the
mirror its Principled, the moss its lerp with the texture found through
the mix — the production files when present), `surface.rs` (mixed, added),
`bsdf_truth.rs` now skips a material whose surface is not the Principled
its Blender script read. Known: with `LAND` present, `bsdf_truth` differs
on four materials (`Material.099`, `.101`, `.111`, `Terracota_Shader.001`)
whose Base Color comes from colour ramps fed by a Mapping node — the
script's fold gives up there and reports the socket default 0.8, Phiber's
folds the Mapping; both stand in for a texture. Pre-existing (the same on
the batch 139 reader), noted here rather than hidden.

## 41. A Mix Shader's two closures are two lobes (batch 141, 2026-10-05)

Cycles samples a Mix Shader by weight: `svm_node_mix_closure` scales the
two closures' weights by the factor, and the BSDF sampler then picks one
closure per bounce with probability proportional to its weight — a coin
per hit, unbiased, which is why a Diffuse mixed with a Glossy converges
to both and not to a half-metal. EEVEE evaluates both closures and blends
their results. Phiber now does each in its kernel.

**The record.** A material (`Mat`, 32 words now, was 30) names a second
material and the weight it takes — `second` (`NO_MAT` for one lobe) and
`mix` — and a face's `Tri` carries the same (`Tri::with_second`), so a
mixed surface is two entries of the material table and the face points
at the first. The emitter table weighs a face's power by both lobes
(`Mat::power_in`). The tracer's tight-binding test moved from 380 to 396
bytes with the wider record (`tests/trace.rs`).

**The kernels.** `layout.wgsl` splits `tri_at` into `tri_with(f, m)`,
the face with any material word; `mat_second` and `mat_mix` read the
lobe. The path tracer (`shade.wgsl`) draws `rnd < mix` at each hit and
shades the face with the other material when it falls — the denoiser's
guides, the texture, the alpha, the glass all come from the lobe picked.
Material Preview (`preview.wgsl`) shades the surface once per lobe
(`preview_lobe`) and lerps the two `Layer`s by the weight — radiance,
alpha and throughput — the continued ray the heavier lobe's; a mixed
surface costs two shadings there, an unmixed one nothing more.
Workbench (Solid / Wireframe) shades the viewport colour and reads no
lobe (`BSurface::tri` leaves the mix out under `uses_viewport_color`).

**From the file.** `shade/surface.rs`'s `surface_mix`: when the output's
surface (through reroutes) is a Mix Shader whose two inputs both fold,
[`BlendMix`] keeps each side folded, the factor (through `eval.rs`, so a
Light Path or a ramp decides it; a texture folds to its mean), and which
side holds the Principled the surface reaches — the lobe the base image,
the maps and the normal map belong to (`BSurface::tri`, which gives that
lobe the textures and the other its colour alone). `BlendMaterial::mix`
→ `SceneMaterial::mix`; `bsdf` stays the two folded together. The
per-face build left `scene.rs` for `rendered/surface.rs` (`BSurface`) at
the line cap.

Tests: phiray `tests/mix.rs` (a red diffuse and a white mirror as one
material at 0.5: the path tracer's 256-sample frame within noise of the
mean of the two shaded alone, in linear light; Preview's frame that mean
exactly; weights 0 and 1 each lobe alone), `tests/trace.rs`; phiber
`rendered/surface.rs` (a Mix Shader is two materials and a weight, the
first the face's, none under Workbench); phiphx `tests/surface_truth.rs`
(THYMUS's glass one lobe; FLOWER-CITY's water two with the camera's
weight 0 — the tracer then shades the glass alone — and its moss two with
the Principled's side named and the ramp's factor between).

Not yet: a per-pixel factor (a Voronoi or an image alpha feeding the Fac
folds to one number — the common cutout, Mix(Transparent, Principled) by
an image's alpha, still works through the Principled's own texture alpha
once the fold lands on it); a third lobe (a Mix of Mixes folds the inner
one); Add Shader as two lobes (folded, the emission summed).

## 42. Noise, Voronoi and Bump per pixel (batch 142, 2026-10-05)

Cycles evaluates a Noise or Voronoi Texture at every shading point —
`svm_node_tex_noise` / `svm_node_tex_voronoi` over the coordinate the
node's Vector is linked to — and a Bump node from the height's change
across the pixel's footprint (`svm_node_set_bump`). Phiber folded a
texture to its mean: a Noise on the base colour drew as one grey, a
Voronoi on a Mix Shader's factor as one weight, a Bump as nothing. The
kernels now carry the textures and evaluate them per hit.

**The port** (`procedural.wgsl`, from `intern/cycles/kernel/svm/noise.h`,
`fractal_noise.h`, `noisetex.h`, `voronoi.h`, `displace.h` and
`util/hash.h`): Jenkins' lookup3 (`hash_uint2/3`) for the Perlin lattice,
`hash_pcg3d` for the Voronoi cells; Perlin with Blender's fade and
gradient table, `snoise_3d` at its 0.9820 scale repeating every 100 000;
fBm over Detail, Roughness and Lacunarity, normalized to 0 ..= 1, the
Color output three offset fBms; Distortion as three more; Voronoi F1
Euclidean with Randomness, fractal over Detail, the Distance normalized
by the largest it can be when the node's Normalize is on, the Color the
cell's hash. The Bump is `svm_node_set_bump` with the node's Strength,
Distance and Blender's Filter Width (0.1): the height at the hit and at
two footprint-length steps along the surface, the gradient turned into a
normal, blended to the shading normal by the strength.

**The slots.** A material (`Mat`, 64 words now, was 32) carries two
procedural slots of 16 words (`Proc`): the colour slot at `m + 32` — a
Noise or Voronoi on the base colour (`proc_base`), on the mix weight
(`mix_weight`), or both when the same texture feeds both — and the bump
slot at `m + 48` (`proc_normal`). A slot names its kind (Fac / Color,
Distance / Color, `PROC_NORMALIZE` for a normalized Voronoi), its
coordinates (Generated, Object or UV), the Mapping node's Location and
Scale, the texture's Scale, Detail, Roughness, Lacunarity and
Distortion / Randomness, and a colour ramp as a 256 × 1 atlas image
(`ramp`) sampled at the texture's scalar — the ramp's colour as the
output, its luminance as the float. Generated coordinates are the hit in
the object's texture space, the instance record (40 words now, was 32)
carrying the bounding box's corner and size (`Instance::texspace`,
`BKE_mesh_texspace_calc`: an axis the box is flat on takes a half-extent
of one about its centre, so a plane's coordinate across it is a half and
not the float noise of the hit divided by nothing — which is what drew
the first version as a speckle). Object coordinates are the hit through
the instance's inverse (`object_point`); UV the face's.

**The kernels.** The path tracer (`shade.wgsl`) computes the hit's
object point and UV before the lobe pick, so the mix weight is the
slot's value there; the normal through the bump slot after the normal
map; the base colour through the colour slot before the image multiplies
it. Material Preview (`preview.wgsl`) does the same per lobe and reads
the weight per pixel for its lerp. Workbench reads none of it.

**From the file** (`shade/procedural.rs`, `BlendProcedural`): Principled
Base Color ← [Color Ramp ←] Noise / Voronoi; the surface Mix Shader's
Fac the same way, through reroutes; Principled Normal ← Bump(Strength,
Distance) ▸ Height ← [ramp ←] texture (a Normal Map behind the Bump is
still read, `links.rs`). The texture's Vector: unlinked is Generated; a
Point Mapping gives its Location and Scale and is looked through to a
Texture Coordinate's Generated, Object or UV output (a UV Map node is
UV). The ramp is baked by `BKE_colorband_evaluate`'s port (linear, ease,
constant) at 256 positions to linear RGBA bytes. Sockets are matched by
identifier or name, since Blender 4.x labels the Mix Shader's and Color
Ramp's `Fac` "Factor" — which had also left the Mix Shader's folded
factor at its default in a 4.x file. `BlendMaterial::procedural` →
`SceneMaterial::procedural`; `rendered/surface.rs` fills the slots: the
base's and the Bump's on the lobe holding the Principled, the factor's
on the first lobe (the one the tracer reads the weight from), the ramp
uploaded once per build. The colour fold (`eval.rs`) still gives the
material one base colour — the ramp at the texture's mean — for Solid
and the material list.

Tests: phiray `tests/procedural.rs` (a plain wall flat; a Noise varies
it, grey for Fac, coloured for Color; Voronoi cells; a red–blue ramp
adds no green beyond the lighting's and leans both ways; a Noise on the
weight of a red / white mix varies where a constant is flat, the mean
about the same; a Bump varies a flat wall — in both modes), `src/trace/
instance.rs` (a flat axis's texture space), `tests/trace.rs` (the
records' sizes); phiphx `tests/procedural_truth.rs` over the new
`PROCEDURAL.blend` fixture (`fixtures/make_procedural.py`, Blender
5.0.1: a ramped Noise over a Mapping of Object coordinates with a
Voronoi Bump; a Noise on a Mix Shader's Fac through a reroute; a
Voronoi's Color over UVs); phiber `rendered/surface_tests.rs` (the slots
on the right lobes, the ramp's atlas id, none under Workbench),
`tests/material_procedural.rs` (the fixture through the scene build).

Not yet: the other noise types (Musgrave's, Ridged, Hetero Terrain) and
Voronoi features (F2, Smooth F1, Distance to Edge, N-Sphere) and metrics;
Wave, Magic, Brick, Checker, Gradient, Sky; Mapping's Rotation and its
Texture / Vector / Normal types; Normal, Camera, Window and Reflection
coordinates; a Mix node between the texture and the socket (folds to the
mean); a second colour slot (two different textures on one lobe's base
and factor leave the factor at its constant); 4D (W) textures.

## 43. The rest of the texture nodes (batch 143, 2026-10-05)

Batch 142 evaluated a Noise (fBm) or a Voronoi (F1, Euclidean) per pixel;
every other texture node still folded to a grey of 0.5. The kernels now
carry Cycles' SVM for all of them, split by family:

- `noise.wgsl` — the hashes and Perlin of batch 142, and all five Noise
  types (`fractal_noise.h`): fBm, Multifractal, Hybrid Multifractal,
  Ridged Multifractal and Hetero Terrain with Offset and Gain,
  `noise_select` as `noisetex.h` has it, Normalize honoured (fBm only),
  Detail clamped to 0 ..= 15 and Roughness to ≥ 0 as the node does.
- `voronoi.wgsl` — `voronoi.h`'s 3D case: F1, F2, Smooth F1 (the 5³
  neighbourhood with Cycles' correction terms), Distance to Edge (its
  own fractalization of lerps) and N-Sphere Radius, over Euclidean,
  Manhattan, Chebychev and Minkowski (with its Exponent);
  `max_distance` per metric (× 2 for F2) for Normalize.
- `pattern.wgsl` — Checker (`checker.h`, with its precision nudge), Brick
  (`brick.h`: Blender's integer noise for the tint, offset and squash
  every n-th row, the mortar smoothed), Gradient (all seven types,
  saturated), Magic (the ten distortion rounds, written as one switch
  per round) and Wave (bands or rings along X / Y / Z / diagonal or
  spherical, sine / saw / triangle, the fBm distortion).

**The slot** (`Proc`, now `trace/proc.rs`) grew from 16 to 40 words to
hold what the nodes add: the type (noise type, Voronoi feature, gradient
type, wave type), a direction (Voronoi metric, bands direction, brick
offset frequency), rings (rings direction, squash frequency, magic
depth), the profile, Offset / Gain, Smoothness / Exponent, Bias, Brick
Width and Row Height, and three colours (Color1, Color2, Mortar);
`procedural.wgsl`'s head lists every word. A material is 112 words (32
and two slots); the kind's flag `PROC_NORMALIZE` moved to bit 8 and now
covers Noise too. `Proc::checker`, `brick`, `gradient`, `magic`, `wave`
start each node at Blender's defaults; `colored()` reads the other output.

**From the file** (`shade/texnode.rs`): the node's sockets folded through
`eval.rs` with `node_shader_tex_*.cc`'s defaults, and its storage —
`NodeTexNoise.type` / `normalize` (fBm and normalized for a file older
than Blender 4.1 / 4.0, which had neither), `NodeTexVoronoi.feature` /
`distance` / `normalize`, `NodeTexGradient.gradient_type`,
`NodeTexMagic.depth`, `NodeTexWave` (type, both directions, profile),
`NodeTexBrick` (offset, squash and their frequencies). The output read
decides the kind (Fac / Color; Voronoi's Distance / Color / Radius;
Position and W are not evaluated and fold as before).

A note on Brick: it is a pattern in X and Y. On a wall standing in the
XZ plane, Generated Y is one value across the whole wall — Blender draws
that wall as one row's slice, often all mortar, and so does this; the
test draws its bricks on a floor.

Tests: phiray `tests/textures.rs` (a red / blue Checker is those two
colours; Brick's mortar a minority of dark pixels; a Linear Gradient never
falls left to right; Magic coloured; Wave varies; each Noise type varies
and differs from fBm; F2 lies farther than F1; every feature and metric,
and N-Sphere Radius, varies), `trace/proc.rs` (40 words, the kind's
flag); phiphx `tests/texture_nodes_truth.rs` over eight new planes in
`PROCEDURAL.blend` (`make_procedural.py`): Checker, Brick, Gradient,
Magic, Wave, a Ridged Noise, Distance to Edge, Smooth F1 Minkowski — every
setting the script set read back; phiber `rendered/surface_tests.rs` (a
Brick's settings at the words the kernel reads).

Not yet: Sky, Gabor, White Noise, IES, Image's procedural cousins
(Environment is the world's); 4D (W) and 1D / 2D dimensions (read as
3D); Mapping's Rotation and its Texture / Vector / Normal types; a Mix
node between a texture and its socket; the colour fold (Solid, the
material list) still takes a texture's mean as 0.5 grey, so a red / blue
Checker's Solid colour is grey, not purple.

## 44. Material Preview's shadows are as soft as the light (batch 144, 2026-10-05)

Batch 108's Material Preview took one shadow ray per lamp, straight at its
centre, so every shadow had a razor edge — the one thing that gave the mode
away beside Blender's, where the default point light's 0.1 m radius already
blurs a contact shadow over a few pixels.

EEVEE does not aim at the centre. `shadow_ray_generate_punctual` picks a
point on the light's shape and traces to that: a disk of the Radius turned
to face the surface (a point light being a sphere, the disk takes the
radius that spans the same angle, `light_sphere_disk_radius`), or the
rectangle itself for an Area light. `shadow_ray_generate_directional`
picks a direction inside the Sun's Angle (`sample_uniform_cone`). One such
ray per frame, averaged over the viewport's sixteen samples.

`shadow.wgsl` ports those three shapes with EEVEE's own sampling
(`eevee_sampling_lib.bsl.hh`: `sample_disk`, `sample_uniform_cone`,
`sample_circle`). What it cannot port is the accumulation: a Preview frame
is one dispatch and the same frame every time (§22), so there is nothing
to average over. It takes `SHADOW_TAPS` (8) rays at once instead, over the
Hammersley set EEVEE samples its own lookup tables with (`hammersley_2d`,
`van_der_corput_radical_inverse`), and averages their visibility. A light
whose shape is a point — Shadow Soft Size at 0, which EEVEE also draws
hard — still takes the one ray at the centre, so nothing pays for softness
it hasn't asked for. The three-point rig's key and rim, which have no
light record of their own, take the same taps over the 0.06 rad cone the
path tracer gives them (`rig_visible`).

An Area light's taps spread over a square of its edge: `Light::size` is
the edge length for an area light and a radius for the rest
(`scene/extras.rs`). Blender's Size Y and the light's own shape are not
read here, so every area light is sampled as a square one — §46 reads
both and spreads the taps over the real shape.

The cost is eight shadow rays where there was one, per lamp, per pixel —
but only where the lamp has a shape. Measured on this container's software
GPU the whole preview suite went from 83 s to 312 s; on hardware §22's
13 ms sample is the figure to re-measure. A tap count that followed the
penumbra's size on screen would be the next refinement if that proves too
dear.

Tests: phiray `tests/preview.rs` — `a_lamp_casts_a_shadow_as_soft_as_its_radius`
replaces `a_lamp_casts_a_hard_shadow`: a small card's shadow on a big one,
the lamp just off the axis so the shadow's outer edge clears the card's
silhouette, counting the pixels along that edge which are neither the
umbra nor the open light. A point-sized lamp has none — it steps from one
to the other — radius 0.3 has two and radius 0.6 has four; and a Sun's
Angle softens the same way, at the angle rather than a radius.

Finding the geometry took three tries, which is worth recording: with the
lamp where batch 108 had it, the blocker hid its own penumbra — the camera
and the lamp were on the same side, the visible frame covers only world
x ∈ [−0.72, 0.72] at the far card, and the soft edge fell outside it. The
old test still passed, because the step it measured was the boundary
between the umbra and the blocker's own lit face, not the shadow's edge at
all.

Still not EEVEE's, in Material Preview: screen-space reflections of other
objects (the environment alone is reflected) and ambient occlusion —
though both are off in EEVEE by default, since Raytracing
(`SCE_EEVEE_SSR_ENABLED`) and Fast GI (`SCE_EEVEE_FAST_GI_ENABLED`) are
not in `SceneEEVEE.flag`'s default, so the two match until the file turns
Raytracing on. The sun's Angle, fixed here at Blender's default, is read
from the file as of §45.

## 45. A light's shape comes from the file (batch 145, 2026-10-05)

Batch 144 made every lamp's shadow as soft as the lamp's shape, which
turned a field nobody had looked at into something visible: the shape
itself. A sun's was not read at all. `rendered/world.rs` wrote
`(0, 0.0093, 0.0)` — Blender's default 0.526° in radians, hard-coded — so
a file whose Sun has a 10° Angle drew the crisp shadow of a 0.5° one, and
no setting in the file could change it.

The reason it was missed is that a sun's shape is not a length. Blender
keeps a point's and a spot's softness in `Light.radius` and an area
light's in `Light.area_size`, both metres, but a sun's in
`Light.sun_angle`, radians — the angular diameter of the disc in the sky,
0.526° being the real sun's. `blend/object/data.rs` read `radius` and
`area_size` and never `sun_angle`, and `scene/import_extras.rs` gave a
sun `radius`, which for a sun is a number Blender's UI does not even
show. So a sun carried 0.1 m as though it were an angle, and phiber threw
that away for its own constant.

Now `BlendLightData::sun_angle` is read (`SUN_ANGLE`, the 0.526° default,
when a file predates the field), `SceneLight::size` is documented as the
one shape field it is — a radius, an edge, or a sun's angular diameter —
and `world.rs` passes it through. The tracer already halved it for the
cone it samples (`shadow.wgsl`, `cone_dir`), which matches Cycles taking
`angle/2` as the half-angle, so nothing downstream changed.

Tests: phiphx `tests/light_truth.rs` over a new `LIGHTS.blend`
(`fixtures/make_lights.py`): a Sun at 10°, a Point at radius 0.4, an Area
at size 2.5, a Spot at radius 0.2 and a 60° cone — each a value the old
default would not give, so a read value cannot be mistaken for a
fallback; phiber `tests/light_shapes_reach_the_tracer.rs`, the same four
through `world.rs`'s `light` into the tracer's records.

One thing the fixture turned up: an area light's stored `Light.energy` is
not the Watts its UI shows. Blender keeps 4/π of it — the script's 500 W
reads back as 636.6 — so passing the stored number straight through, as
the reader does, is right. The test asserts the scaled value and says why.

An area light's Size Y and its shape are read as of §46. Not read still:
the `.phx` round trip carries no light shape at all — `PhxLight` has
only the spot angle and area size, and its import populates no lights.

## 46. An area light's shape (batch 146, 2026-10-05)

§45 gave every lamp the shape the file keeps for it, but an area light's
shape is two numbers and a form, and only the first number was read. The
taps spread over a square of `Light::size` in an arbitrary direction in
the light's plane (§44 said as much), so a strip light — the softbox a
studio scene is lit with — blurred its shadow equally both ways, where
Blender blurs it along the strip and barely across it.

Blender keeps an area light's form in `Light.area_shape`: Square (0),
Rectangle (1), Disk (4) or Ellipse (5), and its edges in `area_size` and
`area_sizey`. EEVEE reads the second edge only for the two irregular
shapes — `is_irregular = ELEM(area_shape, LA_AREA_RECT, LA_AREA_ELLIPSE)`
— so a Square and a Disk take one size both ways however `area_sizey`
happens to be set, and it then scales both by the object's
(`eevee_light.cc`: `l_area.size *= scale.xy() / 2`).

All of that is now carried. The light record grew a row: `right` (the
light's own X axis, normalized), `size_y` (its edge across that) and
`shape`; 80 bytes a light, so the camera's light table went from 768 to
1280 bytes. `shadow.wgsl` spreads the taps over that shape in that plane
— straight across for a Rectangle, through `sample_disk` for a Disk or an
Ellipse, which is what `shadow_ray_generate_punctual` does for
`LIGHT_RECT` against the rounded shapes — with `right` across `size` and
`cross(dir, right)` across `size_y`.

The axes and the scale come out of the same place: the object's matrix.
`m.x_axis` normalized is `right`, and its length is the object's scale in
X, so the edges are `size * |m.x_axis|` and `size_y * |m.y_axis|` — one
read, both facts.

The conversion from a file's light to the record moved out of `world.rs`
into `rendered/lamp.rs` as a trait on `SceneLight` (`BLamp`: `kind`,
`shape`, `record`), the same shape as §43's `BProcSlot` for a texture
node, so the behaviour sits on the type rather than in a function beside
it.

Tests: phiphx `tests/light_truth.rs` — the fixture gained a Rectangle
3 × 0.5 on an object scaled 2 in X, a Disk at 1.5 whose Size Y is left at
the default and must still read 1.5 both ways, and an Ellipse 2 × 0.75;
phiber `tests/light_shapes_reach_the_tracer.rs` — the shapes, the
normalized axis and the scaled edge (the Rectangle's X edge reads 6.0),
and that the kinds without a plane leave the area fields at zero; phiray
`tests/preview.rs` — `an_area_lights_penumbra_follows_its_own_shape`: the
scan across the shadow's edge is horizontal, so a strip 1.2 along X
softens it and the same strip turned into the plane's other axis leaves it
a step. That last one is the test that would have caught §44's square.

Not read still: an area light's `spotblend` equivalent has no meaning
here, but the `.phx` round trip carries no light shape at all — `PhxLight`
has only the spot angle and area size, and its import populates no lights.

## 47. A spot's Blend (batch 147, 2026-10-05)

The same mistake as §45's sun, found by looking rather than reasoning. Both
kernels faded a spot's cone with

```wgsl
e *= smoothstep(li.cone, mix(li.cone, 1.0, 0.15), dot(-l, li.dir));
```

and 0.15 is `Light.spotblend`'s **default** (`DNA_light_types.h`). The
field was never read, so every spot in every file drew with Blender's
default blend: a spot set to a hard edge still had a soft one, and a spot
set to fade across its whole cone faded across a seventh of it.

The shape of the expression was right. EEVEE's
`light_spot_attenuation` is `smoothstep(0, 1, cos · spot_mul + spot_bias)`
with `spot_mul = 1 / spot_blend` and `spot_bias = −spot_size · spot_mul`
(`eevee_light.cc`), which is `smoothstep(spot_size, spot_size +
spot_blend, cos)` — a smoothstep in cosine from the cone's edge inwards,
exactly what was there — and `spot_blend = (1 − spot_size) × spotblend`,
which is the `mix(cone, 1, …)` with the file's number in place of the
constant.

So `Light` carries `blend`, that width in cosine, computed where EEVEE
computes it — on the CPU, in `rendered/lamp.rs`, "to avoid differences
between platforms" as EEVEE's comment puts it about the neighbouring
cosine — and both kernels read `smoothstep(li.cone, li.cone + li.blend,
cos)`. It took one of §46's pad words, so the record did not grow.

Tests: phiphx `tests/light_truth.rs` — the fixture's spot now carries
Blend 0.6 and a second spot (`Sharp`) carries 0, neither of them the
default; phiber `tests/light_shapes_reach_the_tracer.rs` — the width in
cosine against `(1 − cone) × 0.6`, and that a lamp with no cone carries
none; phiray `tests/preview.rs` —
`a_spots_blend_fades_the_edge_of_its_cone`: a 6° spot's pool of light on a
card, counting the pixels across its edge that are neither the pool nor
the dark — Blend 0 gives at most two, and the count grows with Blend.

The cone has to be narrow for that test because the frame sees about a
twentieth of the card at that distance (§44's note): a 25° spot lights the
whole frame and has no visible edge at all, which is how the first attempt
failed.

Not yet at the time: a spot on an object with a non-uniform scale is an
*ellipse* in EEVEE (`spot_size_inv = scale.z / scale.xy`, used in
`light_spot_attenuation`'s `ellipse` term); here the cone stayed round.
Ported in batch 150 (§50), which also shows that the smoothstep above is
that term's `[1, 1]` case.

## 48. Clippy's dead code, and what it turned out to be (batch 148, 2026-10-05)

A cleanup batch, and worth recording because the obvious reading of the
warnings was wrong.

`cargo clippy -p phiphx` named seven items as never used: `basis_at`,
`EASE_IN_OUT`, `BChannel`, `pose_channels`, `bases_at`, `poses_at`,
`Topology::is_boundary` and `BCreaseLevel::creases`. Deleting them looked
like the fix. Five of them are *used by the Blender-truth tests* —
`anim/tests.rs` calls `bases_at`, `poses_at`, `pose_channels` and
`basis_at` to check our pose maths against the matrices Blender itself
printed, and `easing.rs`'s own tests use `EASE_IN_OUT`. Clippy reports
them because it lints the plain library build too, where `#[cfg(test)]`
code is absent; `--all-targets` does not change that.

So the fix is to say what they are rather than to remove them: `basis_at`
and `EASE_IN_OUT` are `#[cfg(test)]`, and `anim/pose.rs` is a
`#[cfg(test)] pub mod` in full — the whole module is a pose oracle
(`BChannel`, `bases_at`, `poses_at` recompute a rig the way
`BKE_pchan_to_mat4` does, for the truth test to compare against), and the
app poses through `rig/` and `follow.rs`. A release build no longer
carries it, and the warnings are gone with nothing lost.

The two that really were dead are gone: `Topology::is_boundary` returned
`edge_nfaces[e] != 2`, which `BSharpness::of` computes inline anyway, and
`BCreaseLevel::creases` rebuilt a vector nothing read.

**What the warnings were hiding.** `redundant_clone` is `deny` for the
whole workspace (`Cargo.toml`), and earlier passes over phiphx had been
grepped for "never used" alone, so four denials went unseen and
`cargo clippy -p phiphx` was failing: a `clean.clone()` whose branch
returns, a `dflt.path.clone()` at the end of its value's life, a
`had.clone()` and a `solid_shade.clone()` in a test. All four are plain
moves. phiphx and phiray are now clean under
`cargo clippy --all-targets`.

**Still failing: phiber.** 42 more `redundant_clone` denials, nearly all
`move || id.clone()` in `view/draw/` (`modal/`, `pane.rs`, `tab/`,
`zone/`, `space/`, `track/`, `tree/`, `region/`, `split.rs`, `sort.rs`,
`toggle.rs`, `card.rs`, `ctrl/number.rs`), and three uses of a disallowed
method — `std::process::Command::output` in `scene/video/mod.rs` (twice)
and `scene/mixdown.rs`, which the workspace forbids because it blocks the
calling thread for an unknown time and wants `smol::process::Command`
instead. Those live in files the other agent is editing, and a
forty-file mechanical sweep across them would collide with work in
flight, so they are left for a batch of their own.

## 49. The encode ran on the UI thread (batch 149, 2026-10-05)

Batch 148 left phiber failing `cargo clippy`, and the three
`disallowed_methods` denials in it turned out to name a real freeze rather
than a style point.

The workspace forbids `std::process::Command::output`
(`disallowed_methods = "deny"` in `Cargo.toml`, the list itself in
`clippy.toml`: *"can block the current thread for an unknown duration"*,
replacement `smol::process::Command::output`) because it blocks the
calling thread until the child exits. Three calls used it:
`scene/video/mod.rs`'s `BMovieJob::ffmpeg_present` and `finish`, and
`scene/mixdown.rs`'s `BMixdown::write`. Following their callers up:

- `finish` is called from `app/op/render.rs`'s animation loop, inside
  `cx.spawn(async move |this, cx| …)`. In gpui `cx.spawn` is the
  **foreground** executor — the one that paints — so the `ffmpeg` encode
  of the whole frame range ran on the UI thread. On the wave file (140
  frames) that is seconds to minutes with the window frozen and not
  repainting.
- `mixdown_to` (Render ▸ Render Audio…, `app/op/render_view.rs`) did not
  even have that much: it called `write` straight from the handler, so
  the encode ran inside the click.

So the freeze is real and the fix is to move the work, not to change how
the program is spawned. The two callers put it on the pool:

```rust
// render.rs — the animation loop, already inside cx.spawn
let made = cx.background_spawn(async move { job.finish() }).await;

// render_view.rs — Render Audio…, which had no spawn at all
cx.spawn(async move |this, cx| {
    let made = cx.background_spawn(async move { mix.write(&file) }).await;
    let _ = this.update(cx, |app, cx| { /* info::push over `made` */ cx.notify(); });
})
.detach();
```

`cx.background_spawn` comes from gpui's `AppContext` trait, so both files
import it. `mixdown_to` now takes a `&mut Context<Self>`, which threaded
through `app/op/file.rs`'s `handle_file_commit` and its five callers. The
reports it pushes (`Saved: '…'`, the WAV fallback warning, the error) now
land from `this.update` when the encode returns, which is where Blender
puts them too: `WM_jobs` reports when the job ends, not when it starts.

**The lint's own replacement does not work here, and that is the finding.**
The first attempt did what `clippy.toml` suggests: `async fn` over
`smol::process::Command`, whose `output()` is a future. It compiled, and
it broke two tests:

```
Detected activity on thread Some("async-process") ThreadId(10), but test
scheduler is running on Some("ctrl_f12_with_ffmpeg_video_writes_one_movie")
ThreadId(2). Your test is not deterministic.
```

`smol::process` runs a reactor on a thread of its own (`async-io`'s
driver), and when the child exits that thread wakes the task — which
means it calls into `phicore`'s `TestScheduler`, whose
`assert_correct_thread` fails any wake-up that does not come from the
test's thread (`test_scheduler.rs:120`). A `smol` future is deterministic
nowhere near the gpui harness.

So the three functions stay blocking `std::process::Command` and say so,
each with `#[allow(clippy::disallowed_methods)]` and a doc line naming
`cx.background_spawn` as where they belong. That is the pattern the rest
of the tree already uses for a program it has to run (`util/src/test.rs`,
`util/src/command/darwin.rs`): the lint is about *which thread* blocks,
and a pool thread blocking is what a pool is for. `phiber` needs no
`smol` dependency.

One test had to change with the behaviour rather than around it:
`render_menu_actions.rs` read the mixdown file on the line after the
commit, which only worked while the encode happened inside the click. It
drives the executor first now (`cx.run_until_parked()`) — the write lands
when the pool runs, which is the point of the batch.

**The clone sweep.** The other 49 denials were `redundant_clone`, also
`deny` workspace-wide: 42 that plain `cargo clippy -p phiber` names and 7
more that only `--all-targets --keep-going` reaches, because clippy stops
at the first failing target and `cargo test` never builds examples at
all. Nearly all are `move || id.clone()` in a closure that is the last
use of `id` — 39 files under `view/draw/` (`modal/`, `pane.rs`, `tab/`,
`zone/`, `space/`, `track/`, `tree/`, `region/`, `split.rs`, `sort.rs`,
`toggle.rs`, `card.rs`, `ctrl/number.rs`, `canvas/`), `app/`, `part/` and
`scene/`, plus 14 test files. All plain moves.

Two of them were not:

- `examples/color_render.rs` had been broken since batch 146, which moved
  the free `light()` into `BLamp::record` — invisible because `cargo
  test` does not build examples. It is `.map(BLamp::record)` now.
- `tests/space_shapes.rs:340` was a false positive: the clone feeds a
  `prop_assert_eq!`, which consumes both sides, so removing it stopped
  the file compiling. Restructured to call the pure `expected_shape`
  helper twice instead, which is what the assertion wanted anyway.

One of the 49 sits in `canvas/time/mod.rs`, which the other agent is
rewriting (it has grown a `scrub_height` and a new `paint_channels`
signature since batch 148). That file is **not** in this batch's archive:
the one-token change — `BTimeGesture { view: view.clone(), … }` to the
field shorthand `{ view, … }` — was applied in place on the machine's own
current copy with `sed`, so their work in flight is untouched and
`cargo clippy` is still clean there.

Worth recording about the tooling: `redundant_clone`'s span points at the
`.`, not at `clone`, so a position-exact sweep script has to accept both
columns; and a sweep is not finished until `--all-targets --keep-going`
is clean, or clippy's "first failing target" behaviour makes the count
look like it oscillates.

`cargo clippy --all-targets` is now clean for all three crates: phiphx
214 passing and 6 asset-dependent, phiray 97 passing, phiber 1274 passing
and 13 failing — all thirteen the other agent's work in progress.

## 50. A scaled spot is an ellipse (batch 150, 2026-10-05)

The last of the light-shape gaps, and batch 147's own "not yet": a spot
whose object is not uniformly scaled lights an *ellipse* in EEVEE, not a
circle. Both kernels drew it round.

`eevee_light.cc` keeps the scale as a pair, inverted:

```c
const float spot_size = cosf(la->spotsize * 0.5f);
const float spot_blend = (1.0f - spot_size) * la->spotblend;
l_spot.spot_size_inv = scale.z / max(scale.xy(), float2(1e-8f));
l_spot.spot_mul = 1.0f / max(1e-8f, spot_blend);
l_spot.spot_bias = -spot_size * l_spot.spot_mul;
```

and `eevee_light_lib.bsl.hh` spends it in the light's own frame:

```glsl
float3 lL = light_world_to_local_direction(light, L);
float ellipse = inversesqrt(1.0f + length_squared(lL.xy * spot.spot_size_inv / lL.z));
float spotmask = smoothstep(0.0f, 1.0f, ellipse * spot.spot_mul + spot.spot_bias);
return (lL.z > 0.0f) ? spotmask : 0.0f;
```

`L` is `light.position() - P`, the vector from the surface to the light,
and the local frame's +Z is the object's +Z — the way the lamp looks
*from*. So `lL.z` is the cosine term the kernels already had, and the
whole of the new behaviour is in `ellipse`.

**Why this is a generalization and not a replacement.** At
`spot_size_inv = [1, 1]`,

    ellipse = 1 / sqrt(1 + |lL.xy|² / lL.z²) = lL.z / |lL| = cos θ

and `ellipse * spot_mul + spot_bias` = `(cos θ − cos_size) / spot_blend`,
which is exactly batch 147's `smoothstep(cone, cone + blend, dot(-l, dir))`.
So the ported form has to agree with the old one on every unscaled
spot — which is what `spot_scale == [1, 1]` in the record means, and what
the old test for Blend still checks.

**What moved.** `Light` grew `spot_scale: [f32; 2]` in the last two pad
words, so the record is still 80 bytes (§46's budget holds). `light.wgsl`
gained `spot_mask(li, l)` — it lives there because `light.wgsl` is the one
file both the path tracer's and Material Preview's kernels include — and
both call sites now call it instead of smoothstepping a dot product:

```wgsl
fn spot_mask(li: Light, l: vec3<f32>) -> f32 {
    let d = -l;
    let z = -dot(l, li.dir);
    if (z <= 0.0) { return 0.0; }
    let xy = vec2<f32>(dot(d, li.right), dot(d, cross(li.dir, li.right))) * li.spot_scale;
    let ellipse = inverseSqrt(1.0 + dot(xy, xy) / (z * z));
    return smoothstep(li.cone, li.cone + li.blend, ellipse);
}
```

The frame is the light's own, so a spot now needs `right` — the field
batch 146 added for area lights. `BLamp::record` fills it for a spot too
(`BLamp::spot_scale` is the new trait method that computes
`scale.z / scale.xy` from `matrix_world`); `up` is `cross(dir, right)`,
as `shadow.wgsl` already derives it for an area light. `xy` enters
squared, so neither axis's sign matters — only which scale pairs with
which axis, and that is fixed by `right`.

phiphx needed nothing: the object's scale was already reaching the reader
in `SceneLight::matrix_world`. `tests/light_truth.rs` now says so
(`a_spots_object_scale_reaches_the_reader`), because a silent change there
would make this port wrong with no test to notice.

**The reach is exactly `tan(half angle) / spot_scale`.** The mask passes
where `ellipse > cone`, i.e. where `(s · tan θ)² < tan²(half angle)` — so
the pool's radius along an axis is the round radius over that axis's
scale, with no approximation. That is what phiray's
`a_scaled_spots_pool_of_light_is_an_ellipse` asserts rather than merely
"wider than tall": a 6° spot four units away gives a 26-pixel pool at
`[1, 1]` and a 34-pixel one at 0.75, which is 26 / 0.75 to within a pixel.
Measured on the frame, not predicted — 0.5 was the first value tried and
it filled the frame, which would have made the test pass for the wrong
reason.

The fixture grew a light for it: `Squashed`, a 50° spot on an object
scaled 1.5 in Y (`fixtures/make_lights.py`, Blender 5.0.1). Every other
spot in `LIGHTS.blend` is unscaled, and the test says that too, so the
round case stays pinned.

**Found while reading, not fixed here.** `SceneModel::from_phx`
(`scene/import_phx.rs`, 162 lines) and `PhxScene::add_light` have no
callers at all. The `.phx` sidecar fast path they belong to was retired
when the pointer remap became a hash lookup — `phx.rs` says so in
as many words — and the asset pipeline reaches a sidecar through
`PhxImporter::import` instead. Clippy cannot see it, because both are
`pub` in a library. So the queue's old "the `.phx` round trip carries no
light shape" item is not a parity gap: nothing writes a light into a
`.phx` and nothing reads a scene back out of one. It is dead code to
remove — done in batch 151 (§51), `import_phx.rs` whole.

## 51. The Mapping node, whole (batch 151, 2026-10-05)

Since batch 142 a texture's coordinate has gone through a Mapping node,
but only two of its four settings: Location and Scale. Rotation was read
as nothing, and the node's **Type** — Point, Texture, Vector, Normal —
was not read at all, so every Mapping drew as a Point one. A material
whose texture is turned 45° drew it straight.

Cycles does all four in one function (`svm_mapping`, `mapping_util.h`;
EEVEE's `gpu_shader_material_mapping.glsl` is the same four lines):

```c
const Transform rotationTransform = euler_to_transform(rotation);
switch (type) {
  case NODE_MAPPING_TYPE_POINT:
    return transform_direction(&rotationTransform, (vector * scale)) + location;
  case NODE_MAPPING_TYPE_TEXTURE:
    return safe_divide(transform_direction_transposed(&rotationTransform, (vector - location)), scale);
  case NODE_MAPPING_TYPE_VECTOR:
    return transform_direction(&rotationTransform, (vector * scale));
  case NODE_MAPPING_TYPE_NORMAL:
    return safe_normalize(transform_direction(&rotationTransform, safe_divide(vector, scale)));
}
```

Texture is Point's inverse — move, unrotate, unscale — which is what a
*texture's* own transform means as against a point's. Vector drops the
Location because a direction has no position. Normal scales the other way
and renormalizes.

**The Euler matrix, and the one thing to get right.** Cycles'
`euler_to_transform` builds an XYZ Euler as a `Transform`, whose `t.x`,
`t.y`, `t.z` are its **rows**. WGSL's `mat3x3` constructor takes
**columns**. So `euler_xyz` in `procedural.wgsl` is that function
transposed into column order, and then `rot * v` is
`transform_direction` and `transpose(rot) * v` is
`transform_direction_transposed`, exactly as `svm_mapping` spends them.
Getting this backwards would rotate the right amount the wrong way, which
a 45° test would still catch but a symmetric one would not.

`safe_divide` and `safe_normalize` came over too (`safe_div`,
`normalize_or_zero`): zero where the divisor is, the vector itself where
it has no length. The old code's `max(size, 1e-6)` was a different thing
— a clamp, not a guard — and only ever applied to the bounding box.

**Where it sits.** The `Proc` slot had exactly four spare words (`_pad` at
27 and `_pad3` at 37–39), which is what Rotation (three) and the Type
(one) need, so the slot is still 40 words and §42's budget holds.
phiphx's reader grew a `BlendMapping` — the coordinate plus the node's
four settings, with the identity as its `Default` — and `BlendProc::mapped`
takes one, so `coordinates()` returns a value rather than a widening
tuple. The Type is `bNode.custom1`, read as a `u16` the way `MixRGB`'s
blend mode and `Math`'s operation already are.

**A coincidence worth recording.** The first kernel test compared a Point
mapping at Location `[0.3, 0.2]` against a Texture one and found the two
frames identical, which looked like the Type not being read. It was
arithmetic: Point moves the coordinate by `+location` and Texture by
`−location`, so they are `2 × location` apart — and at Scale 40 that is
24 and 16 whole checker cells, which a periodic pattern cannot show. At
`[0.01, 0.005]` it is a fraction of a cell and the frames differ over
hundreds of channels. A pattern's own period is a trap for any test that
measures a shift.

The fixture grew a plane for it: `Turned`, a Checker through a
Texture-type Mapping at Location `[0.1, 0, 0]`, Rotation 45° about Z and
Scale `[2, 2, 1]` (`fixtures/make_procedural.py`). The planes with no
Mapping, and `Ramped`'s Point one, are asserted to read the identity and
Point, so the old behaviour stays pinned.

**Dead code removed.** §50 found that `SceneModel::from_phx` and
`PhxScene::add_light` have no callers; both are gone, and
`scene/import_phx.rs` with them. The `.phx` sidecar fast path they served
was retired when the pointer remap became a hash lookup, and the asset
pipeline reaches a sidecar through `PhxImporter::import` instead.
`PhxLight` itself stays — it is part of the serialized format, not an
unused function. Clippy could not see any of this, because `pub` items in
a library are never dead to it; it took reading the callers.

Also gone: a test-only `use` in `anim/channels.rs` that batch 148's
`#[cfg(test)]` work left unused in the library build (the call now names
its path in full). The one warning left in the three crates is an unused
`BKey as _` in `tests/the_time_axis_zooms_under_the_wheel.rs`, which is
the other agent's file in flight.

## 52. The same animation in both, frame by frame (batch 152, 2026-10-05)

Every comparison so far has been a **still** (§2.4, §2.5 of
`render-parity.md`): one front view of one pose. A still cannot see whether
the rig poses at the right frame, whether the camera's own animation is
read, or whether the picture is steady over time — so this batch renders a
*range* in both and measures the three things a still cannot.

The file is `THYMUS-BOLDROID-WAVE-003.blend`: 273 objects, a 328-bone rig,
`rigAction` over frames 1–140 at 24 fps, camera `Camera.001`, Cycles at
1440 × 1440, Filmic. Frames 1, 21, 41 … 121 at a quarter resolution and 16
samples, both sides.

### The recipe

```powershell
$f = "crates\phiphx\tests\fixtures\THYMUS-BOLDROID-WAVE-003.blend"
# Blender: one PNG per frame through the scene's own camera
blender -b -P crates\phiphx\tests\truth\blender_animation.py -- $f out\blender 1 140 20 cycles 16 4
# Phiber: the same frames, the same camera, the same samples
$env:PHIBER_SCALE=4
cargo run --release -p phiber --example animation_render -- $f out\phiber 1 140 20 16
# Side by side, frame by frame
cargo run --release -p phiber --example animation_compare -- out\blender out\phiber contact.png
```

Both write `fNNNN.png` and a `times.txt` of `frame posed_ms render_ms
samples`, so the two can be compared on speed as well as on pixels.

### What `animation_compare` measures that a still cannot

- **Motion.** The mean absolute change from one frame to the next, in each
  sequence. A rig that does not pose, or poses at the wrong frame, matches
  well on frame 1 and has near-zero motion after it — which a per-frame
  difference alone reports as a small constant error rather than as the
  whole animation missing.
- **Steadiness.** A path tracer's noise changes every frame even when the
  subject does not, so motion where Blender's is near zero is this render's
  own flicker.
- **Posing time**, separately from rendering time, on both sides.

### What matched, first run

Pose for pose and frame for frame. The silhouette, the limb positions, the
hand's gesture and the camera's framing are the same in all seven frames
(the contact sheet). That is the rig, the constraints, the drivers, the
modifier stack and the camera's own animation all agreeing at seven points
across 120 frames — the thing this test existed to check.

**Posing is three times faster here**: 29 ms a frame against Blender's
83 ms, and that is a fair comparison because both run on the CPU.

Rendering is **not** a fair comparison in this container and should not be
quoted as one: there is no GPU, so phiray runs on lavapipe (a software
Vulkan device) while Cycles runs its own mature CPU kernel on the same two
cores. 20.7 s a frame against 4.7 s says what software rasterization costs,
not what the tracer costs.

### What did not match, and why — two real bugs

**1. A missing environment image lit the whole scene white.** The first run
came out at a mean difference of **35.2 / 255**, with Phiber's background
pure white where Blender's was black, and Phiber's figure *olive* where
Blender's was *magenta*.

The olive was the giveaway. The materials' base colours in this file really
are olive (`mat_panel_…` is `[0.254, 0.216, 0.078]`), and Blender's torso
pixel was `(76, 5, 37)` against Phiber's `(78, 72, 38)` — red and blue
within two levels, green 67 apart. Olive × magenta light is
`(0.254, 0, 0.078)`, which is exactly Blender's pixel. So Blender was
lighting the scene with a **magenta dome** and Phiber with a white one.

The world's Background Color is linked to an Environment Texture whose
file is not shipped with the `.blend`. `shade/world.rs` set
`world.color = [1.0; 3]` whenever an Environment Texture node existed —
present or not — so a missing HDRI became a full-white dome. Blender folds
a missing image to magenta (`IMB`'s "invalid" colour), which `eval.rs`
already did for a material's `ShaderNodeTexImage`; the world was the one
place that did not. It does now, and `world.texture` is `None` for a file
Blender would not find, so the renderer lights from `color` instead of
hunting for it.

That one change took the mean difference from **35.2 to 13.0 / 255**.

Worth knowing for any test of it: whether an image is present is answered
against the `.blend`'s own folder, and that folder is recorded by
`Phx::open` alone (`blend::set_image_source` writes a global). A test that
reads a file with `BlendFile::parse_bytes` leaves it unset, and
`image_present` then answers *true* for everything — which is why
`tests/world_truth.rs` opens through `Phx`, and why it passed against the
old behaviour until it did.

**2. Film ▸ Transparent never reached a render driven from an example.**
Phiber's background was then magenta where Blender's was transparent. The
flag was read correctly (`Scene.r.alphamode`, `film_transparent: true` in
the model) and the kernels honour it (`shade.wgsl`'s `p.film.x`) — but
`render.film_transparent` is seeded by **`seed_output_props`**, and
`camera_render.rs`, `color_render.rs` and the new `animation_render.rs`
all called only `seed_render_props`. `render_set` then fell back to its
default `false`.

The app is fine — `app/import.rs` calls both seeds, so F12 in the window
has always honoured it. What this means is narrower and worth recording:
**every comparison in §2.4 and §2.5 was measured with a background Blender
had left out.** All three examples seed both now, and the alpha comes back
0 on the background and 255 on the figure.

### Where it stands

| | Blender (Cycles, 16 spp) | Phiber |
|---|---|---|
| pose / camera / framing | — | the same at all seven frames |
| mean \|difference\| on the figure | — | **15.6 / 255** |
| mean figure brightness | 43.1 | 33.2 |
| motion per frame | 5.05 | 3.48 |
| posing | 83 ms | **29 ms** |
| rendering (software GPU here) | 4.7 s | 20.7 s |

What is left to close is **level and specular**: Phiber reads about a
quarter darker on the figure, and Blender's highlights on the shoulders and
chest are brighter and tighter. The motion ratio of 0.69 is the same story
— much of Blender's frame-to-frame change at 16 samples is its own
Monte-Carlo noise, which Phiber's denoiser removes. Those are the next
measurements to take, with the denoiser off and a white material override,
so the light is measured on its own.

## 53. Where the missing light was (batch 153, 2026-10-05)

§52 left one number open: Phiber's figure read about a quarter darker than
Blender's, and "level and specular" was as precise as it got. A whole-scene
difference says nothing about *which* term is short, so this batch splits
the light in two and measures each.

### The isolation

`truth/blender_animation.py` and `examples/animation_render.rs` take three
switches of the same names, so the same scene can be rendered three ways on
both sides:

| | |
|---|---|
| `PHIBER_NO_WORLD=1` | the world contributes nothing — the lamps alone |
| `PHIBER_NO_LIGHTS=1` | every lamp is dropped — the world alone |
| `PHIBER_VIEW=Standard\|Filmic\|AgX` | overrides the file's view transform |

THYMUS frame 41, a quarter resolution, 16 samples, Cycles against the path
tracer, measured over the figure (alpha, and not the transparent black):

| | Blender | Phiber | Phiber / Blender |
|---|---|---|---|
| both | 38.0 | 30.1 | 0.79 |
| lamps only | 35.5 | 21.4 | **0.60** |
| world only | 18.0 | 23.0 | **1.28** |

So the single "a quarter darker" was two errors in opposite directions,
partly cancelling: **the four lamps give 40 % less light than Blender's**,
and **the world dome gives 28 % more**. Neither is visible in a figure that
mixes them, which is why five batches of still comparisons never named it.

(The lamp row is what the rest of this section is about. After the fix
below it reads 1.10, and the whole figure 1.18 — the table at the end has
the numbers as they stand.)

### The floor, found on the way

The first run of the isolation produced an impossible number: Phiber's
*world only* came out at 38.8, **brighter than the same scene with the
lamps on as well** (30.1). Light is additive; removing a term cannot add
one.

`rendered/world.rs`'s `lit()` had this:

```rust
w.map(|c| c.max(if usable.is_empty() { 0.25 } else { 0.0 }))
```

— a floor of 0.25 a channel on the world's radiance, applied only when the
scene has no usable lamp, so that "studio renders still read". Two things
are wrong with it. It lifts any scene whose lamps are all dark, which
Blender does not do. And because it clamps **per channel** it re-tints a
coloured world: this file's dome is magenta `[1, 0, 1]`, and the floor
turned it into `[1, 0.25, 1]` — pink. The world-only mean RGB was
`(67.8, 29.7, 43.7)` against Blender's `(69.2, 0.1, 45.3)`: red and blue
within 2 %, green thirty times too high.

The floor is gone for a file that has a world. Material Preview still
reads, because it is lit by the studio HDRI through `studio_lighting`,
which is where "still reads" belongs; and a view with no world at all keeps
its fallback, because the canvas colour is then the only thing there is to
light with. With it gone, world-only falls to 23.0 and its green to 7.7 —
the honest number above.

That is the last of the hand-set constants §3.8 of `render-parity.md` was
tracking in the shading path.

### What the two ratios are not

Worth ruling out before the next batch chases them:

- **Not the view transform.** Both sides render the file's own Filmic, and
  the per-channel ratios under lamps-only are 0.64 / 0.60 / 0.56 — close to
  uniform. A tone curve mismatch would not scale the channels together.
- **Not the light count.** The file has four lamps and the kernel takes up
  to four per sample before it starts picking at random, so all four are in
  every sample.
- **Not the missing HDRI.** That was §52's bug and it is fixed; the world
  term is now 28 % high rather than 2× high and magenta rather than white.

The file's lamps are two 1500 W spots (25°, Blend 0.15, Radius 0.25), one
939 W point (Radius 0.1) and an 11.8 W square area light — so the shortfall
is mostly in the spot and point terms, where the kernel spends
`color × energy / (4π²d²)`. Whether that constant is right cannot be
settled on a 273-object scene with four lamps and a coloured dome.

### The lamp-power truth test, and the π it found

A controlled measurement settles in one run what a 273-object scene cannot:
`phiphx/tests/truth/blender_lamp_power.py` builds a scene rather than
loading one — a 20 m plane at albedo 0.8, a black world, an orthographic
camera straight above, one lamp 2 m up — renders it in Cycles to **OpenEXR
at the Standard view transform**, and prints the centre pixel's
*scene-linear radiance*. No tone curve, no film, nothing to argue about.

```
point  1000 W at 2 m   5.046
spot   1000 W, 60°     5.048     (a spot's power is a point's, masked)
sun       5 W/m²       1.271     (irradiance already; distance does not enter)
area    100 W, 1 m²    1.862
```

The point number is the textbook one. Irradiance under a lamp of power `P`
at distance `d` is `P / (4π d²)`, and a Lambertian surface of albedo `a`
turns that into radiance `a E / π`:

    0.8 × 1000 / (4π² × 4) = 5.066   against Cycles' 5.046

`phiray/tests/lamp_power.rs` sets up the same scene and reads
`Tracer::radiance`. It came out at **1.802** — a factor of **2.80** short.

The cause is one π, counted twice. `light.wgsl` had

```wgsl
var e = li.color / (4.0 * PI * PI * d2);
```

and `brdf_w` divides the diffuse albedo by π itself, as a BSDF should. So
the kernel was dividing by π in the light *and* in the surface: `e` must be
**irradiance**, `P / (4π d²)`, and it was radiance-after-Lambert instead.
`preview.wgsl` carried the same expression. Both are `4.0 * PI * d2` now,
and the point lamp lands at 5.661 against Cycles' 5.046.

That residual 12 % is two modelling differences, not a constant: the tracer
carries a GGX lobe at `f0 = 0.04` over its Lambert, and the two count an
indirect bounce off a 20 m plane slightly differently. The test's tolerance
is 15 % and says so — it exists to catch a factor, and a factor of π is
214 %.

**What this was costing.** Every lamp in every render was π times too dim,
in both the path tracer and Material Preview, since the kernels were
written. It never showed as "the lights are wrong" because the world term
and the studio rig were not affected, so a lit scene came out somewhat flat
rather than obviously dark — §52's "a quarter darker" was this, diluted by
a world dome that was itself too bright.

### Where the three terms stand now

The same isolation, re-run on the fixed tracer:

| | Blender | Phiber | Phiber / Blender |
|---|---|---|---|
| both | 38.0 | 44.8 | 1.18 |
| lamps only | 35.5 | 39.2 | **1.10** (was 0.60) |
| world only | 18.0 | 23.0 | **1.28** (unchanged — the π was in the lamp term) |

The lamp term went from 40 % short to 10 % over, and what is left of it is
in the same places the controlled test leaves 12 %: a GGX lobe the
reference does not carry, an area light the kernel approximates with
`max(dot(-l, dir), 0) × 2`, and shadow rays that do not agree pixel for
pixel.

**The world is now the largest single error**, at 28 % high, and it is the
next thing to measure the same way — a known dome over a known albedo,
against Cycles' own number, the way `blender_lamp_power.py` does for lamps.

A note on reading these: `both` is not `lamps + world`, because a figure's
pixels are not a linear sum of two renders once shadowing and indirect
bounces are in play. The three rows are three measurements, not an
equation.

## 54. The world, measured the same way (batch 154, 2026-10-05)

§53 left the world term at **1.28×** Blender's and said to measure it the
way the lamps were measured. The world turns out to be the easiest thing in
the renderer to check, because its law has no constant in it at all:

> A uniform dome of radiance `L` puts irradiance `πL` on a surface — the
> hemisphere — and a Lambertian albedo `a` turns that back into radiance
> `aE/π = aL`.

No distance, no geometry, no `4π`. `blender_lamp_power.py` now renders the
world cases on the same plane, and Cycles agrees to four figures:

```
world  radiance 1,   albedo 0.8   ->  0.800055
world  radiance 0.5, albedo 0.8   ->  0.400027
world  magenta,      albedo 0.8   ->  (0.789885, 0.000004, 0.796692)
```

phiray's answer was not a scale factor but an **affine** one:

```
L = 0     ->  0.0680
L = 0.25  ->  0.2744
L = 0.5   ->  0.4808
L = 1     ->  0.8936
L = 2     ->  1.7191        i.e.  0.8255 L + 0.068
```

A surface lit by a *black* world was sending back 0.068. Two separate
causes, and the intercept is the one that matters.

### The rig fired on top of the world

```wgsl
if (p.count == 0u && p.envmap != 0u) { return c; }
if (p.count == 0u) { /* three-point studio rig: key 2.0, fill 0.45, rim 0.9 */ }
```

The path tracer falls back to a three-point rig for a scene with no light,
and the guard above it tested for an environment **image**. A world that is
a flat *colour* — which is every file whose Background node has no texture,
and every scene in the viewport's default world — has `envmap == 0`, so the
rig fired *on top of* it. The constant 0.068 is the rig, arriving whatever
the world does.

The same mistake had already been found once for images: the comment above
that line says adding the rig on top of a world "made every lamp-less HDRI
scene about a fifth too bright", and the fix then was to exclude the image
case alone. The guard is now what it should have been — a world is a light,
image or no image:

```wgsl
let world_lit = p.envmap != 0u || dot(p.env.rgb, p.env.rgb) > 0.0;
if (p.count == 0u && world_lit) { return c; }
```

The rig remains for a scene with no light of *any* kind, which is the only
case it was ever for.

### And the flat world was not flat

`env_at` returned, for a world with no image,

```wgsl
p.env.rgb * mix(0.35, 1.15, clamp(d.z * 0.5 + 0.5, 0.0, 1.0))
```

— a dome brighter above the horizon than below. That looks like a sky, but
a Background node with no texture is **uniform in every direction**, which
is what Blender lights with. Cosine-weighted over the upper hemisphere the
gradient averages 1.017, so an upward-facing surface got 2 % too much and a
downward-facing one a good deal too little. It returns `p.env.rgb` now.

(The *background* the camera sees is a separate term — `sky_at`, the
theme's canvas colour and its sweep — and is untouched. This is the
lighting.)

### Where it lands

```
L = 0.25  ->  0.2031
L = 0.5   ->  0.4062
L = 1     ->  0.8123
L = 2     ->  1.6246        i.e.  0.8123 L, no intercept
```

Exactly linear, and 1.5 % over Blender's 0.800 — the residual being one
indirect bounce the reference caps at 1 and the tracer does not.
`lamp_power.rs` asserts the level at 5 % and the linearity at 2 %, which is
tighter than the lamps' 15 % because this law has no constant to allow for.

### The three terms, end to end

| | Blender | Phiber, batch 52 | batch 53 (π) | batch 54 (world) |
|---|---|---|---|---|
| lamps only | 35.5 | 21.4 (0.60) | 39.2 (1.10) | 39.2 (1.10) |
| world only | 18.0 | 23.0 (1.28) | 23.0 (1.28) | measured at 1.015 on the controlled scene |
| whole figure | 38.0 | 30.1 (0.79) | 44.8 (1.18) | — |

Three batches ago the figure was a quarter dark for reasons nobody could
name. What is left is the lamp term's 10 %, and that is the GGX lobe, the
area light's `max(dot(-l, dir), 0) × 2` stand-in, and shadow rays — all
things the controlled test can now be extended to measure one at a time.

## 55. The area light was half a light (batch 155, 2026-10-05)

§54 left the lamp term 10 % over and named three things in it: a GGX lobe
the reference does not carry, shadow rays, and the area light's stand-in
factor. The last of those turns out not to be a 10 % effect at all.

The kernel had, for `kind == 3`:

```wgsl
e *= max(dot(-l, li.dir), 0.0) * 2.0;
```

on top of `e = color / (4π d²)`, so an area light delivered
`P·cos / (2π d²)`.

EEVEE says what it should be. `eevee_light.cc`'s `shape_radiance_get`
converts a lamp's radiant flux to the radiance its surface emits:

```c
case LIGHT_RECT:
case LIGHT_ELLIPSE: {
  float area = this->area().size.x * this->area().size.y * 4.0f;   /* size is the half-extent */
  if (this->type == LIGHT_ELLIPSE) { area *= M_PI / 4.0f; }
  return float(M_1_PI) / area;                                      /* L = P / (π A) */
}
```

So the emitted radiance is `P / (π A)` over the **full** rectangle, and
what lands on a surface straight below is `L · A · cos / d²` —
`P·cos / (π d²)`. **The area cancels**, which is exactly what the comment
above that switch means by "make illumination power constant": grow a
Blender area light and it does not get brighter, it spreads the same watts
over more surface. So the kernel is right to ignore `size` here, and wrong
only in its constant: with `e` already carrying `1/(4π d²)`, the factor is
**4**, not 2.

Measured on the controlled scene — a 100 W square metre 2 m above an albedo
of 0.8:

| | |
|---|---|
| Cycles | 1.862 |
| phiray, factor 2 | 1.035 — **44 % out** |
| phiray, factor 4 | ~2.07 — 11 % over |
| the far-field limit, `0.8 · P / (π² d²)` | 2.026 |

The remaining 11 % is not a constant: it is the source's *finite size*. A
1 m light 2 m away subtends enough solid angle that the point
approximation over-counts, and Cycles integrates the real rectangle. EEVEE
carries a fitted correction for the same reason (`point_radiance_get`'s
`mrp_scaling`, "the fit was found by reducing the average error compared to
cycles"). Porting that is a separate, bounded job; the factor of two was
the bug.

**What it was costing.** Every area light in every render was half as
bright as Blender's, in both kernels, and an area light is the most common
lamp in an interior or a product shot. It did not show on THYMUS because
that file's area light is 11.8 W against two 1500 W spots — which is the
case for a controlled test rather than a scene: a 44 % error on one term
can hide entirely behind the arithmetic of a real file.

`lamp_power.rs` now covers a point, a spot, a sun, an area light and a
uniform world — five absolute numbers, all against Cycles' own.

## 56. A flat world never bounced (batch 156, 2026-10-05)

The scoreboard's first open item was bounce depth: several residuals looked
like one-bounce differences, and the shadows under the figure's arms read
lighter here than in Blender. Measuring it needed a scene with a shadow in
it, which the lamp-power plane has not got.

`blender_lamp_power.py` grew one. Two walls a metre high and a metre apart
make a trench along Y; the camera looks straight down into it and the
centre pixel is the floor between them, which can see only the strip of sky
the trench leaves open. Cycles, by diffuse-bounce count:

```
open plane          0.800
trench, 1 bounce    0.462
trench, 4 bounces   0.584      <- Blender's default
trench, 12 bounces  0.605
```

(The first attempt at this put a *lid* over the plane, which the
orthographic camera above simply photographed instead: 0.800, unshadowed.
A trench is the shape that puts the shadow where the camera can see it.)

phiray gave **0.504 at one bounce and 0.504 at four** — the same number,
because `Camera::bounces` was doing nothing at all.

### Why

```wgsl
fn indirect(pos, n, seed) {
    let lit = p.envmap != 0u;
    if (p.glow == 0u && !lit) {
        let dir = cosine_dir(n, seed);
        return env_at(dir) * mix(0.25, 1.0, visible(pos, dir, p.ao));
    }
    …the real path…
```

A scene with no glowing meshes and **no environment image** never reached
the bounce path: it got one occlusion ray with a floor of 0.25. A world
that is a flat *colour* has `envmap == 0`, so that is every file whose
Background node has no texture — the same guard, in the same shape, as
batch 154's studio rig and batch 153's ambient floor. Three places tested
for an image where they meant *a world*.

The predicate is the one the other two now use:

```wgsl
let lit = p.envmap != 0u || dot(p.env.rgb, p.env.rgb) > 0.0;
```

### And then the tail

Routing a flat world into the path made `bounces` mean something — and made
one bounce *worse*: 0.661 against Cycles' 0.462. The path closes with

```wgsl
if (!escaped && !sampled) { acc += thr * env_at(nrm) * 0.8; }
```

— a truncated path gets the world added as if nothing were in the way, so
that it does not go black. That is a reasonable stand-in for an
environment *image* that has no sampling table, where a cosine ray rarely
finds the bright part by chance. For a **uniform** world it is pure
over-count: a cosine ray finds a uniform dome with exactly the probability
the surface is open to it, so the path is already unbiased. The tail now
applies to an image only.

### Where it lands

| | Cycles | phiray before | after |
|---|---|---|---|
| open plane | 0.800 | 0.812 | 0.812 |
| trench, 1 bounce | 0.462 | 0.504 | 0.397 |
| trench, **4 bounces** | **0.584** | 0.504 (ignored) | **0.584** |

Exact at four, which is the number that matters because four is what
Blender ships as Max Bounces → Diffuse. One is left loose in the test on
purpose: Cycles counts a bounce from the camera hit and this path counts
from the surface, so the two differ by a step at the shallowest depth and
agree once there is room to converge. What the test does assert at one
bounce is the *shape* — a trench floor much darker than open ground, and
climbing with depth, as Cycles' own 0.462 → 0.584 → 0.605 does.

Two more hand-set constants are gone from the uniform-world case: the 0.25
occlusion floor and the 0.8 tail. And Light Paths → Max Bounces → Diffuse
now does something for an ordinary file, which it did not before.

## 57. An area light is not a point (batch 157, 2026-10-05)

§55 fixed the area light's constant and left it 11 % over Cycles, with the
residual named as the source's finite size and the fix named as EEVEE's
`mrp_scaling` fit. The fit turns out to be the wrong thing to port, and the
right thing is shorter.

**Why not the fit.** `point_radiance_get`'s `mrp_scaling` is under a comment
that says what it is for — *"Volume light is evaluated as point lights"*.
It is EEVEE's correction for lighting a *volume* with an area light, not a
surface; EEVEE shades surfaces with a real linearly-transformed-cosine
integration of the rectangle. Applied to our surface term it gives

```
tmp = (π/2) / (π/2 + √A) = 0.6111
mrp = tmp + (1 − tmp)/π   = 0.7349        ->  radiance 1.489
```

against Cycles' 1.862 — 20 % *under*, worse than the 11 % over it was meant
to fix. Porting a fit without its context is how that happens.

**What the geometry says.** A disk of area `A` and radiance `L`, seen
head-on from distance `d`, puts

```
E = L · π r² / (r² + d²)        with  r² = A/π
```

on a surface — the standard form factor. With `L = P / (πA)` that comes to

```
E = P / (π (d² + A/π))
```

so the whole of the correction is **`d²` becoming `d² + A/π`**, and the
far-field form (§55) is its `d² ≫ A` limit. No fit, no constant.

In the kernel, where `e` already carries `1/(4π d²)`:

```wgsl
let shape = li.shape == AREA_DISK || li.shape == AREA_ELLIPSE;
let a = max(li.size * li.size_y, 0.0) * select(1.0, 0.25 * PI, shape);
e *= max(dot(-l, li.dir), 0.0) * 4.0 * d2 / (d2 + a / PI);
```

A disk or an ellipse takes `πab/4` for its area, as `shape_radiance_get`
does. The constants moved from `shadow.wgsl` to `camera.wgsl` on the way:
`shadow.wgsl` is included only by Material Preview's kernel, and the path
tracer needs them now too.

**Measured.**

| | Cycles | far field (§55) | with the form factor |
|---|---|---|---|
| 100 W, 1 m², at 2 m | 1.862 | 2.07 (11 % over) | **1.909 (2.6 % over)** |
| the same at 8 m | — | 0.1267 | 0.1301 (2.7 % over) |

The second row is the check that matters: it is a *shape* correction, not a
scale. Four times further away the source is a point again and the two
forms agree, which a fitted factor would not do.

The area light has gone 0.56× → 1.11× → **1.026×** over three batches. What
is left is the same ~2 % every term carries — the GGX lobe at `f0 = 0.04`
the reference's Principled puts almost none of into this geometry — and
that is item 3 on the scoreboard, not an area-light problem.

## 58. The specular lobe, and a flaw in the measuring stick (batch 158, 2026-10-05)

Item 3 on the scoreboard was specular: Blender's highlights on the
shoulders and chest read tighter and brighter than ours in every frame of
the contact sheet. A highlight has two properties and they fail
differently — how much light it sends back, and how wide it is — so both
want measuring.

`blender_lamp_power.py` grew a specular case: the same plane with **Base
Color black**, so the diffuse term is zero and only the microfacet lobe
answers, lit by the same 1000 W point lamp, with lamp and camera both
straight overhead — which puts the half-vector on the normal, the lobe's
peak, where its normalization shows most. Roughness swept:

```
roughness 0.1   142.403
roughness 0.3     7.450
roughness 0.6     0.4936
roughness 1.0     0.06865
```

**The peak.** phiray against those: ×1.007, ×1.110, ×1.093, ×1.021 — right
at the extremes and about a tenth high in the middle. Not a constant, and
small next to everything this campaign has found.

**The width, and the trap.** Measured across the middle row at roughness
0.3, Blender's lobe is 8 pixels wide at half maximum and phiray's was
**14** — 75 % broader, which is exactly what the contact sheet shows.

It was not the kernel. Comparing the kernel's own output against the GGX
formula it implements showed the measured lobe wider than its own
prediction at *every* roughness, and increasingly so as the lobe narrowed —
which is the signature of a blur, not of a wrong roughness. The blur was
in the test: `overhead()` built its camera with `scale: 1.0`, and the
kernel's orthographic extent is `1 / (aspect × scale)`, so the frame
spanned **one metre** where the reference's `ortho_scale = 2.0` spans two.
Every profile came out twice as wide in pixels.

With the frame matched:

| roughness 0.3, middle row | 4 px out | 3 | 2 | centre | half-max |
|---|---|---|---|---|---|
| Cycles | 0.550 | 0.725 | 0.892 | 1.000 | 8 px |
| phiray | 0.517 | 0.700 | 0.877 | 1.000 | **8 px** |
| the GGX formula | 0.538 | 0.711 | 0.878 | 0.985 | — |

All three agree within about 3 %. **The lobe's shape is Cycles' shape**,
and the broad highlights in the contact sheet are the figure's overall
+19 % brightness, not a smeared specular.

Worth being precise about what the harness flaw did and did not touch:
every number in §53–57 is a *flat field* read at the centre of the frame,
and the centre of a flat field is the centre whatever the frame spans — so
the lamp, world, area and trench measurements are all unaffected. What it
ruined was any measurement of *shape*, which is why the first one came out
75 % wrong. A measuring stick needs checking against the thing it measures
before its readings are trusted, and the check here was the kernel against
its own formula.

Both are tests now: the peak across the roughness sweep, and the lobe's
profile and half-max width against Cycles'. `lamp_power.rs` holds seven
absolute comparisons — a point, a spot, a sun, an area light near and far,
a uniform world, indirect light in shadow, and the specular lobe.

## 59. The HDRI path, and two clean negatives (batch 159, 2026-10-06)

Items 4 and 5 on the scoreboard. Item 4 was phrased as a fixture problem —
THYMUS' environment image is not shipped, so both renderers fall back to
magenta, and every number measured on that file is a measurement of a
fallback. The deeper question underneath it is whether the **environment
image path** is right at all: on our side it is entirely different code
from a flat world. An image is sampled through a CDF for next-event
estimation, carries a prefiltered chain, is weighted against the cosine ray
by MIS, and gets the tail that §56 left in place for exactly this case. Any
of those can hold a normalization error a flat colour would never show.

Two measurements, in order of strength.

### A uniform image must equal a uniform colour

The strongest invariant available, because it needs no reference at all: a
dome of radiance 1 is a dome of radiance 1 however it is stored.

```
flat colour  0.8121
image        0.8043          (Cycles 0.8001)
```

0.96 % apart, and both within 1.5 % of Cycles. The image path's
normalization is sound.

### A banded sky, against Cycles

A uniform image cannot catch a *sampling* error, because every direction
gives the same answer. So `blender_lamp_power.py` now writes a real
equirectangular EXR — radiance 4 above 50° elevation, 0.1 elsewhere — loads
it as the world's Environment Texture, and renders the plane through it.

```
Cycles   1.2910
phiray   1.3417        ×1.039
```

4 % over, which is the band every other term sits in. The test builds the
same sky from the same rule rather than reading the EXR, so the comparison
cannot drift on an image loader; the file (`truth/sky.exr`) is there for
Blender's side and for anyone who wants to look at it.

**The orientation is the part worth writing down.** phiray's rows run
**zenith first** — `env_at` maps `v = 0.5 − atan2(z, |xy|)/π`, so `v = 0`
is straight up — where Blender's image buffer starts at the nadir. The two
skies are each other's rows reversed. Get that backwards and a sky lights
from below, which on a flat plane looks merely *dim* rather than wrong, and
would have been read as a sampling error.

### What this means for the scoreboard

Both are **negative results**, and they are worth as much as the four bugs
before them: the environment-image path is right, so the remaining gap on
THYMUS is not hiding there. Item 4 is what it looked like at the start — a
missing fixture, not a defect. Five of the seven terms in `lamp_power.rs`
now have no known error above 4 %.

### Item 5, in passing

`animation_render` takes `PHIRAY_DENOISE=0`. The denoiser smooths
frame-to-frame noise, so `animation_compare`'s motion column reads low with
it on; a measurement of how much an animation moves wants it off. The
switch is the same one `color_render` has had.

## §60 — The last two texture nodes, and a material override to measure light with (batch 160)

Two separate pieces of work, both of them things the status table called
**partial**.

### The texture nodes: White Noise and Gabor

`space-status.md`'s reading table has carried one `partial` for materials:
*"Sky / Gabor / White Noise and a Math node between a texture and its
socket not yet"*. Two of those four are now in, ported from Cycles.

**White Noise** (`svm/white_noise.h`) is the cheap one and was overdue: it
is Jenkins' lookup3 over the coordinate, with no interpolation between
cells at all, and `noise.wgsl` already had `rot`, `jenkins_final` and
`hash_uint2/3` because Perlin needs them. What was missing was the
four-argument mixer and the float-shaped wrappers, which are now
`hash.wgsl` — `hash_uint`, `hash_uint4`, and the nine
`hash_floatN_to_floatM` forms the two new nodes spend.

The node's dimension setting is the whole of its behaviour and it lives in
`bNode.custom1`, not a storage struct — the same place a Mapping node keeps
its Type, so `texnode.rs` grew a `custom1` reader and `procedural.rs`'s
`mapping_type` now calls it instead of keeping its own copy. **1D hashes W
alone**, which means the entire surface comes out one flat value; no other
texture node in the set can do that, so it is what the test turns on.

**Gabor** (`svm/gabor.h`) is 146 lines of WGSL and the most mathematical
node in the set: sparse Gabor convolution (Lagae 2009) with Tavernier's
stratified sampling and normalization (2019), evaluated as Tricard's
complex phasor (2019) so that **Value, Phase and Intensity all fall out of
one sum**. Eight impulses a cell over a 3 × 3 (2D) or 3 × 3 × 3 (3D)
neighbourhood — 72 or 216 windowed Gaussians a pixel.

Two places Cycles divides by a zero and reads NaN are guarded here, and
said so in the comments: an Orientation straight up the Z axis has no
azimuth, and a file that zeroed the Orientation vector would normalize to
NaN.

Three words were already free in the slot for them. A Gabor keeps its
Frequency in `detail`, its Anisotropy in `roughness`, its Orientation 2D in
`lacunarity` and its Orientation 3D in `color1`; a White Noise keeps its
dimensions in `variant` and its W in `offset`. No record grew.

**A trap worth recording.** A Gabor has **two input sockets both named
`Orientation`** — Blender tells them apart by identifier (`Orientation 2D`,
`Orientation 3D`) and hides whichever the type does not use. `socket_values`
keys by *name*, so reading the 3D one that way returns the 2D float's
bytes; the first run read `[0.333, 0.333, 0.333]` where the fixture had set
`(0, 1, 0)`. `texnode.rs` now has a `vector` reader that goes through
`socket_named`, which tries the identifier first.

The fixture is regenerated: `make_procedural.py` has two more planes,
`Hashed` (White Noise ▸ Value, 4D, W 1.5) and `Gabored` (Gabor ▸ Value, 3D,
Scale 12, Frequency 4, Anisotropy 0.25, Orientation (0, 1, 0)), and
`PROCEDURAL.blend` was rebuilt against Blender 5.0.1. `bpy` is a pip wheel,
so the fixture can be rebuilt in the test container without a Blender
install — worth knowing for every fixture after this one.

Still open on that row: **Sky** (the Nishita model, which needs a
precomputed table, and is the larger of the four) and a **Math node between
a texture and its socket**.

### The material override

§2.6 of `render-parity.md` ended with a measurement to make next: *"with the
denoiser off and a white material override, so the light is measured on its
own rather than through the materials."* Batch 159 did the denoiser. This is
the override.

Blender has it already — **View Layer ▸ Override ▸ Material**
(`ViewLayer.mat_override`), which EEVEE reads at `eevee_material.cc:446`
before a single node is evaluated:

```cpp
blender_mat = (material_override) ? material_override : <the slot's>
```

`rendered/stand.rs` is that, as `BMatStand`: one grey Lambertian surface —
albedo as asked, fully rough, dielectric — standing in for every material.
`BSurface::tri` consults it first and returns the stand-in, so the slot is
not read at all; a mix's second lobe is never even added to the scene. An
override that kept the slot's textures would not be an override.

Specular is deliberately *not* switched off: Blender's Principled keeps its
0.5 IOR level under an override too, and a measurement that removed it
would not be measuring what Blender renders.

Both sides take the same switch, as the others do:

```powershell
$env:PHIBER_OVERRIDE=1    # phiber: BMatStand::grey(1.0)
PHIBER_OVERRIDE=1         # blender_animation.py: view_layer.material_override
```

What it is *for*: a level difference between two renderers is either the
light arriving at a surface or what the surface does with it, and those two
cannot be told apart while every face carries its own albedo, roughness,
texture and mix weight. THYMUS reads 15.2 / 255 apart with its own
materials. With this on, the same comparison measures the light alone — and
whatever is left after that is the materials.

### The Math node, and four more words in the slot

The third of the four gaps, and the one that was not a missing *feature* but
a **loss**: `procedural.rs`'s walk from a material socket back to its
texture went through reroutes and one Color Ramp, and broke on anything
else. A Math node in that position therefore left `texture()` looking at a
Math node, which it has no reader for — so the whole procedural was dropped
and the material came out flat. A Noise behind a `Multiply` was not *wrong*;
it was **gone**.

`svm_math` is ported whole — all 41 of `NodeMathOperation` — as
`math.wgsl`, with its guards from `util/math_base.h`: `safe_divide`,
`safe_powf` over `compatible_powf` (a negative base with a fractional
exponent has no real answer; `x^0` is 1 including `0^0`; a negative base
with a whole exponent keeps its sign), `safe_logf`, the two modulos,
`wrapf`, `pingpongf`, `smoothminf`, `compatible_signf`, `compatible_atan2`.
Each returns zero where the operation has no answer, which is what Cycles
does rather than carry a NaN through the shade.

The slot had no room for it, so **the procedural slot grew from 40 words to
44** — the operation, its second and third Value sockets, and Clamp — and a
material from 112 to 120. Four words were *not* stolen from a node type
that does not use them: `bias`, `width`, `height` and the two spare colours
are all live for Brick and Checker, and a Brick behind a Math node would
have collided. The growth moved `bump_proc` from `m + 72` to `m + 76` and
reset the data table's floor in `trace.rs` from 1068 bytes to 1132 — that
test counts bytes exactly, on purpose, and is the reason the layout cannot
drift silently.

In the kernel the Math node runs **after** the texture and after any ramp,
on the scalar, and then paints the colour grey from it — which is what
Blender does when a Math node feeds a colour socket. Clamp is applied last.

**Another two-sockets-one-name trap**, the same shape as the Gabor's: a Math
node's three inputs are **all named `Value`**. Only the identifiers
`Value_001` and `Value_002` tell the operands apart, which is why
`texnode.rs` reads them through `socket_named`.

Reading it is pinned by `Mathed` in the fixture (a Noise ▸ Fac through
Multiply 2.5 with Clamp on); the kernel by
`a_math_node_after_a_texture_maps_its_value`, which reads each operation off
a **Linear Gradient** rather than a noise — the Gradient's Fac is a known
ramp from 0 to 1 across the frame, so `× 0` must flatten it, `> 0.5` must
cut it into exactly two halves with one step between, and Clamp on `× 4`
must flatten the bright end that the unclamped version keeps climbing.
Predictable, not merely different.

That leaves **Sky** alone on the materials row.

### The engine row, which nothing read

Third partial of the batch, from the Rendering workspace's row rather than
the materials one: *"the engine enum (`PHIRAY`) is selectable with nothing
behind it."*

The honest description of the old behaviour is worse than "PHIRAY does
nothing": **Properties ▸ Render ▸ Engine was a dropdown no render ever
read.** `BStillLook::for_engine` asked `model.render.engine` — the engine
the *file was saved with*. So a `.blend` saved under Workbench rendered
Workbench, and changing the row to anything, Workbench included, changed
nothing. `PHIRAY` was the visible symptom because no `.blend` can name it,
so it could only ever be chosen there, where nothing was listening.

`BRenderSet` carries it now, as `BRenderEngine` — the panel's value first,
the file's only until the panel holds one, which is exactly how every other
row in that struct already read. `for_engine` branches on
`set.engine.traces()` instead of a string compare against the file:
Workbench rasterizes `Scene.display.shading`, every other engine —
`PHIRAY` among them — path-traces. An engine the build does not know (an
add-on's) falls back to EEVEE rather than refusing to render, as Blender
does with a missing engine.

What `PHIRAY` *means*, stated plainly: the same tracer EEVEE and Cycles
render through here, named as itself rather than borrowing Cycles' name.
F12 now reports which engine drew the frame, as Blender's status line does.

Pinned by `tests/render_engine_row.rs`.


## §61 — The Spreadsheet's domains (batch 161)

Item 4 of the queue: *"the Spreadsheet lays out; no attribute data."* That
was half true. It did show a table — one row per **vertex** with
`position x/y/z` — and nothing else: no other domain, no other attribute,
and the left region held the **generic Toolbar**, which is wrong on its own
terms because Blender's Spreadsheet has no tools. That region is the
**dataset region** (`spreadsheet_dataset_draw.cc`): the object's geometry as
a tree, one row per domain, the chosen one active.

So the batch is a new part, `BPartType::Dataset`, and the table reading from
whichever domain it names.

**The domains and what each carries.** Blender lists the attributes the
geometry *has*, hiding any whose name begins with a dot
(`allow_procedural_attribute_access`) — which is why `.corner_vert` and
`.select_vert` are not columns, and why a mesh with no corner attribute
shows a Face Corner table of indices and nothing else **in Blender too**.
Here, what the reader carries decides the rest:

| domain | elements | columns |
|---|---|---|
| Vertex | `vertices.len()` | `position x/y/z`, and `UVMap u/v` when the mesh has a UV layer |
| Face | `indices.len() / 3` | `material_index`, when the faces were assigned per face |
| Face Corner | `indices.len()` | `Color r/g/b`, when the corners carry a colour attribute |

**Edge is deliberately not offered.** The reader triangulates, so
`SceneMesh` carries `indices` and no edge list — and the edges of a
triangulated mesh are *not* the file's edges, because a quad's diagonal is
not one. A count derived from the triangles would be **wrong rather than
missing**, which is worse. `.edge_verts` is read already, for creases
(`phiphx`'s `mesh/crease.rs`), but it does not reach the scene; that is the
one change the Edge domain needs. Asking for `EDGE` falls back to the
Vertex domain a fresh Spreadsheet opens on.

**One difference worth stating rather than hiding:** `SceneObject::uvs` is
**per vertex** (`len == vertices.len()`), where Blender's UV layer is per
corner. So `UVMap` sits on the Vertex domain here and on Face Corner in
Blender. It is a reader simplification, not an editor bug, and the column
comment says so.

A blank cell is not a zero: an index past the end of its array prints
nothing, because a blank says *no value here* and `0.000` says *zero*.

The split follows Blender's own files — `column.rs` for which columns a
domain has (`spreadsheet_column.cc`), `row.rs` for one element's values
(`spreadsheet_layout.cc`), `dataset.rs` for the region
(`spreadsheet_dataset_draw.cc`), `set.rs` for the domain itself.

Still open on that row: the Filter panel's row filters, pinning, and
Blender's other geometry components (curves, point clouds, instances,
Grease Pencil) — the Spreadsheet here reads meshes.

### A merge, and why the failure count was a lie

While landing batch 161 the mirror of the machine was checked file by file
for the first time in a while, and **103 source files had drifted** — all of
them the other agent's: the node editor's whole `add/`, `edit/`, `drag/`,
`pick/` and `wire/` trees, `canvas/time/` restructured (`gesture.rs` became
`gesture/mod.rs` + `gesture/drag.rs`, `row/` gained its own tests), plus
`state/key/op.rs`, `part/pref/`, `part/props/pref/control/`, and
`phiphx`'s `shade/graph.rs` and `shade/socket.rs`. Thirty-nine of the
103 were files that did not exist here at all.

Two consequences worth naming:

**Every "13 failing, all theirs" count in the last few batches was measured
against a tree three weeks behind theirs.** Their fixes were not in it — the
three `level_structure` files they had already brought under 240 lines still
read as over, and the layout golden read 4092 lines stale because their
Animation gizmo rows had moved and this tree had the old rows. None of that
was a real failure; it was a stale working copy reporting on work that had
already been done.

**The drift check missed `Cargo.toml`.** The mirror compared `*.rs` only, so
the merge compiled against a manifest without the `log` dependency the other
agent had added for `app/journal.rs` — seven `cannot find \`log\` in the
crate root` errors from a file that is correct. The check covers the
manifests and `clippy.toml` now.

The lesson, written down because it will come up again: **a shared tree with
two agents in it needs the mirror verified at the start of a batch, not
only at the end.** Verifying after extraction proves the batch landed; it
says nothing about what arrived from the other side in between.

### What the merge actually found, and one file of ours in it

With the trees in step, the phiber suite reads **10 failing, and all ten are
the other agent's** — their key-budget test over their own
`state/key/op_tests.rs`, two renderer-style budgets, the activity bar's
treatment, the Outliner's guide lines, two NLA editor tests, the timeline
wheel zoom and their layout golden. The three `level_structure` files and
several others that had been counted against us for batches were **already
fixed on their side**; this tree had simply never seen the fix.

Two things in the shared tree are mid-refactor and are **not** to be
"fixed" from here, which `STALE.md` now says in as many words:

- `crates/phiber/tests/chrome_wiring.rs` **does not compile**:
  `part/activity/option.rs` turned `options_for` into a method on the group
  type and the test still calls it as a free function.
- Two `redundant_clone` denies in `canvas/node/frame/tests.rs`, so
  `cargo clippy -p phiber --all-targets` fails on the shared tree. Removing
  the clone the lint points at does not compile — the value is used after
  it — so it is theirs to resolve. A batch gates its own clippy on its own
  targets while that stands.

**One failure was ours**, and worth recording as the cost of a
two-line addition: `rendered/mod.rs` reached **243 lines** when `stand.rs`
was declared and exported in it, over the 240-line read limit. The fix is
not a squeeze — the thread-local render state (`BRayState`, `STATE`,
`POSTED`, `MAX_SIDE`, `views_posted`, `shown_image`) moved into
`rendered/state.rs`, which is where it belonged next to `wake.rs`,
`clock.rs` and `settle.rs`, each of which already keeps its own. `mod.rs`
is 204 lines and declares modules rather than holding state.


## §62 — The Sky Texture, and the last thing on the materials row (batch 163)

`space-status.md`'s reading table has carried one `partial` on materials
since the table existed. Batch 160 closed three of its four gaps; **Sky was
the one left**, and it is closed here — for the two models that are an
atmosphere.

### The shape of the thing

Blender has **four** sky models, and the first thing to get right is which
is which, because `NodeSkyType` numbers them in the order you would not
guess: **Preetham 0, Hosek/Wilkie 1, single scattering 2, multiple
scattering 3** — the two analytic models first. Reading `0..=1` as "the
atmosphere ones" is a one-character mistake that silently reads every sky as
the wrong model, and it is the mistake this batch made first.

Preetham and Hosek/Wilkie are **not** this atmosphere. They are analytic
models over a precomputed coefficient dataset — `sky_hosek_data.h` is 3821
lines of it — and a world naming either is **declined**: it keeps the
Background colour it folded to rather than being handed the wrong sky.

The other two are Nishita's, and the important fact about them is that
**Blender does not evaluate them per ray either**.
`node_shader_tex_sky.cc` precomputes a 512 × 256 table on the CPU and the
kernel samples it, through a non-linear latitude mapping that concentrates
rows near the horizon:

```c
y = copysign(sqrt(fabs(dir_elevation) * M_2_PI_F), dir_elevation) * 0.5 + 0.5
```

So a sky is an image here too — which means the whole path that already
carries one carries it: the CDF for next-event estimation, the prefiltered
chain, the MIS weight against the cosine ray. **§59 measured that path
against Cycles and found it sound**, which is why this batch could be a
reader and a bake with **no kernel change at all**.

### What was ported

`sky_single_scattering.cpp`, whole, as `phiphx::sky`:

- `spectrum.rs` — 21 wavelengths from 380 to 780 nm; the sun's irradiance
  at the top of the atmosphere, the Rayleigh and ozone coefficients at each,
  the CIE XYZ colour matching functions, and the 8-node Gauss–Laguerre
  quadrature Blender uses for optical depth (its own comment gives the
  sympy that produced the nodes).
- `atmosphere.rs` — the densities (Rayleigh and Mie exponential, ozone a
  layer around 25 km), the two phase functions, the earth and atmosphere
  intersections, `ray_optical_depth` by that quadrature rather than ray
  marching, and `single_scattering`: 32 segments along the ray, each one's
  in-scattering from the sun attenuated by the depth there and back.
  Blender's one shortcut is kept — the density of each scattering type has
  the same spectrum everywhere, so only a scalar per type travels along the
  ray instead of a full spectrum.
- `mod.rs` — the bake, **straight into an equirectangular image** in
  `env_at`'s own layout rather than into Blender's non-linear table. The
  cost is more samples near the poles and fewer near the horizon for the
  same pixel count; the gain is that nothing downstream has to know the sky
  is a sky. The sky is symmetric about the sun's azimuth, so half the
  columns are integrated and the other half mirrored, as Blender does.

### Measured against Cycles

`truth/blender_sky.py` renders the same sky with an **equirectangular
camera**, so the frame *is* the environment image Cycles samples — no
geometry, no lighting, nothing between the model and the measurement.

**Above the horizon, every row is within 1.6% of Cycles.** The profile, row
by row from the horizon to the zenith:

```
elev    ours    Cycles   ratio
 2.8°   8.901   8.842    1.007
 8.4°   7.124   7.146    0.997
14.1°   5.497   5.522    0.995
25.3°   3.081   3.123    0.987
47.8°   1.357   1.372    0.989
70.3°   0.922   0.926    0.997
87.2°   0.831   0.818    1.016
```

That is the atmosphere model confirmed — it is where the sky is physics.

**Below the horizon they do not agree, and the numbers are recorded rather
than tuned away.** The model has no ground in it: a ray pointing down goes
through the Earth, so Blender fills those rows by fading its horizon row
out, `(1 - 2.5 sin θ)³` over the first 23.6° of depression. This does the
same from the same formula, and comes out **×1.35 just under the horizon
rising to ×1.69 at 20° down**. The formula and the mapping both check out on
paper — Blender's non-linear table rows cancel *exactly* against the
kernel's `sqrt` lookup, so the stored fade at a given elevation really is
`(1 - 2.5 sin θ)³` — so something in Cycles' own sky path attenuates that
band further than the GPU path this was read from. It is a few degrees of
fake ground rather than sky; what is pinned is its shape: present, falling
with depression, black past the cut.

### The sliders, by the sign of what they do

Each one is a consequence of a table being right, so each is asserted by
its direction, measured first rather than assumed:

| | zenith | horizon | red-to-blue at the horizon |
|---|---|---|---|
| default | 0.831 | 8.901 | 1.90 |
| **Air ×2** | 1.270 | 6.397 | 6.56 |
| **Dust ×4** | 0.851 | 6.201 | 2.66 |
| **Ozone ×4** | 0.647 | 6.572 | **1.19** |

More air brightens the zenith and *dims* the horizon while reddening it
hard — on a long path extinction wins and blue is what is lost. The one
worth keeping is ozone: it dims everything and turns the sky **bluer**,
because the Chappuis band absorbs in the middle of the spectrum. That is
what makes a twilight sky blue, and it is the sign that catches a mistyped
`OZONE_COEFF` — get the band's shape wrong and the sky reddens instead.

### The sun disc

The disc is not in the sky image: it is one small, very bright solid angle,
and a 64-wide equirect frame cannot hold it without blowing out a pixel. It
is a radiance of its own (`SKY_single_scattering_precompute_sun`), and what
is checked is that it is far brighter than any sky, **reddens as the sun
sets** (blue scatters out of the long path first), is black below the
horizon or with the disc off, and takes Sun Intensity as a plain multiplier.

### Two traps this batch walked into

1. **`aerosol_density` is `dust_density` in the file.** Blender's source
   calls the field `aerosol_density`; the DNA name in a 5.0 file is
   `dust_density` — the struct was renamed and the DNA name kept. Reading
   only the source's name lost the slider *silently*: every other field read,
   so a sky with heavy dust came out as a sky with none. Both names are
   tried now and the file decides.
2. **A fixture script wrote the `.blend` over itself.** `make_sky.py`
   derived its output path with `__file__.replace("make_sky_fixture.py",
   "SKY.blend")` after the file had been renamed, so the replace matched
   nothing, the path stayed `make_sky.py`, and Blender saved a `.blend`
   where the script had been. Derive a fixture's output from its own name
   and the rename cannot do that.

### What is still open

**Multiple scattering is read as single scattering** — the same atmosphere
with the inter-scattering term left out, so a sky asking for it comes out
darker and less blue in its shadowed parts than Blender's. That is the
honest thing to say until `sky_multiple_scattering.cpp` (401 lines) is
ported too. And the two analytic models remain declined rather than
approximated.


## §63 — Editing UVs, and where a drag's anchor has to live (batch 164)

Item 7: *"the UV editor draws; no UV editing — select, move, unwrap, pin."*
This is **select and move**. Unwrap and pin are named at the end as still
open, because they are a different kind of work.

### The shape

Blender keeps UV selection on the mesh's loops and a move writes the loop's
`uv` outright. Here both are **session state** over a loaded model that
stays what the file said — `ctx.uv.selected` and `ctx.uv.moved`, the same
shape `scene::edit` uses for objects added and renamed, so Undo is a state
step and a reload a clean slate with no copy of the mesh to keep in step.

A corner is `(map, corner)`: which of the shown maps and which entry of its
`uvs`. `scene/uv_edit.rs` holds the two lists and the four operations over
them — `picked` (a plain pick replaces, Shift adds, Shift on a selected
corner drops it, as `uv_select_pick`'s toggle does), `nudged` (every
selected corner moved a further step, **folded into** the offset already
there so a second drag continues the first), `holds` and `uv_of`.

`canvas/image/uv_pick.rs` is the gesture: the nearest corner within 12 px,
measured against the UVs **as they now stand** so a moved corner is picked
where it is drawn rather than where the file put it. A click where nothing
is within the radius picks *nothing* — it does not fall back to the least
far corner on the square, because then a stray click anywhere would arm a
drag on something.

The painter walks corners by **index** now rather than by value, because a
face has to be drawn with its moved corners where they were moved to; a
selected corner is the accent and two pixels bigger, as Blender draws
`TH_VERTEX_SELECT` larger than an unselected vertex.

### The bug worth the whole section

The gesture first kept the drag's anchor in an `Rc<Cell<Option<Point>>>` on
itself, copying `border.rs`, which keeps the render region's box that way.
Clicking worked. **Dragging did nothing at all**, and the reason is worth
writing down:

`BUvGesture::of` runs every frame. The press writes the selection; that
write re-renders; the re-render builds a *new* gesture with a *new* cell;
and the mouse-move handler now attached is the new one, whose cell is
`None`. The anchor was thrown away between the press and the first move.

`border.rs` gets away with a cell because **it writes nothing until the
drag ends** — its press only sets the anchor. A gesture that writes on the
press cannot use one. So `UvKey::Drag` is a state key, read fresh each
frame, and the move folds the step into the offsets and advances the anchor.

The general rule, since it will come up again: **a cell on a per-frame
element survives only as long as nothing is written.** The moment a handler
writes state, everything it was holding is gone, and the next handler in
the same gesture is a different closure over a different cell.

### Two files over the line

Declaring one module pushed `canvas/image/mod.rs` to 243 lines, the same way
`stand.rs` pushed `rendered/mod.rs` there in batch 162. Again the fix is
structural rather than a squeeze: the Image and UV editors' geometry —
`view_all`, `zoom_for`, `uv_square`, `rect` — moved into
`canvas/image/place.rs`, where it is one concern instead of four methods
wedged between the view's state and its window. `mod.rs` is 214 lines.

Twice in five batches is a pattern: **a `mod.rs` in this tree sits a line or
two under the limit, so adding a module to one means moving something out of
it.** Worth expecting rather than rediscovering.

### Still open on that row

**Unwrap** is an algorithm (angle-based flattening, `uvedit_unwrap_ops.cc`
over `slim` or the ABF solver) and belongs in a batch of its own. **Pin**
needs a flag per corner that survives into the file, which is a writer
change rather than an editor one. Box select, island select and the sticky
modes are each a day's work on top of the picking that is now there.


## §64 — Object drivers, evaluated (batch 165)

Item 9: *"Drivers editor — partial, rig drivers only."* The reader has
listed every driven object's driver F-curves for the editor since batch 143;
what it had never done is **run** the ones that are not a rig's. A driver on
an object's own `location`, `rotation_euler` or `scale` was read, drawn as a
channel in the Drivers editor, and then ignored — the object stood where its
keys put it.

### Where it hooks

`BlendMotionList::solve` built each object's channels as
`self.list.iter().map(|m| m.raw_at(frame))` — the actions and the NLA.
**Blender's order is the animation system, then the drivers over it**: the
depsgraph evaluates a property's animation and its driver after, so a driver
wins over a key on the same channel. So the pass goes exactly there, and
`raw_at` is now a list-level method — the channels with the drivers run —
with `solve` calling it.

A driver's variables read a **snapshot** of what the actions left, taken
before any driver writes. That is deliberate: the order the objects are
driven in then cannot change the answer.

### What it is

`blend/anim/driver/` beside `blend/anim/drivers.rs` (which reads the list
the editor shows). The evaluation is the rig's, reused rather than
rewritten: `rig::driver`'s `DRIVER_TYPE_*`, its expression parser, and the
same two steps — `evaluate_driver` over the variables, then the driver's own
**F-curve over that value** rather than over the frame. Average, Sum, Min,
Max and a simple Python expression all behave as the rig's do, because they
are the same code.

A variable is read when it is a **Single Property** whose path is one of the
object's transform channels; the target is resolved by its block address
against the objects in the scene. Everything else — a custom property, a
material or modifier setting, a Transform Channel or Rotational Difference
variable — reads **0**, as a file whose scripts do not run does. A driver
nobody can evaluate is better than a driver evaluated from a guess.

### Measured against Blender

`fixtures/OBJDRIVERS.blend` (`make_obj_drivers.py`) is a cube keyed moving
in X from 0 to 10 over frames 1–21, and three objects driven from it:

| object | channel | driver |
|---|---|---|
| **Wheel** | `rotation_euler[2]` | Average of one variable, through a Generator of `0.5 + 2x` |
| **Scaled** | `scale[0]` | the Python expression `1 + x / 10` |
| **Summed** | `location[2]` | the Sum of the driver's X and Y |
| **Still** | — | nothing, so the list carries an undriven object too |

`truth/blender_object_drivers.py` takes each object's matrix from the
**evaluated** depsgraph copy at five frames — which is where a driver's
result lands — and decomposes it. Every channel of every object at every
frame agrees to better than **2 × 10⁻⁴**.

### Two things the fixture taught

**`driver_add` leaves two keyframes on the curve.** The F-curve a new driver
gets is a 1:1 ramp, and a Generator modifier sits *over* those keys rather
than instead of them (`evaluate_fcurve_ex` runs the modifiers after the
keys). An assertion that a driver curve has no keys is simply wrong, and the
useful assertion is the value the two come to together: the Wheel's angle is
`0.5 + 2x`, not the ramp's `x`.

**A matrix decomposition wraps an angle and hides the mapping.** Reading the
Wheel's rotation back out of its world matrix gives −0.783 at frame 6, not
5.5 — the same angle, wrapped into (−π, π] by the quaternion. It matched
Blender because Blender's `decompose()` wraps identically, but it says
nothing about whether the Generator ran. Reading the **channel** through
`raw_at` says 5.5, and that is the assertion worth having.

### What is still open

A driver chained off another driver's output reads the keyed value
underneath it: this pass is single and unordered where Blender's depsgraph
orders driver against driver. And drivers on **materials**, **modifiers**,
**shape keys** and **custom properties** are still read-and-listed only —
each needs the thing it drives to be evaluated per frame, which is a
different piece of work in each case.

## §65 — The selection outline on the GPU (batch 166)

Queue item 11 read "the overlay pass on the GPU — grid, outlines, gizmos
and extras still draw on the CPU over the GPU image". Reading the code
first, as the batch is supposed to, the item turned out to overstate the
problem in one place and understate it in another.

**The grid is already on the GPU.** `grid_over` lives in `util.wgsl` and is
called by `shade.wgsl`, `preview.wgsl`, `solid.wgsl` and `wire.wgsl` — every
kernel — and accumulated per sample as a premultiplied overlay in words
20n..24n, composited onto the tone-mapped mean in `emit`. `BGuide::
paint_floor` on the CPU only runs as a stand-in for a frame the tracer has
not delivered. The same holds for `paint_over`: the light, camera and empty
wireframes, the origin dots and the 3D cursor are a few dozen line
segments, and painting them costs nothing worth measuring.

**The selection outline was the whole of the cost.** `view3d/mod.rs` said
so itself:

> With the GPU drawing, only the selection's outlines are projected here.

`cache::meshes(false, …)` projected, for every selected object, either every
vertex or — for a large mesh — its precomputed extreme points
(`mesh/extreme.rs`), took the convex hull of them (`mesh/hull.rs`) and
stroked that over the image, on **every frame**, memoised against a
fingerprint that the selection and the camera are both part of. So every
orbit frame with something selected re-projected and re-hulled it. And a
convex hull is not Blender's outline: Blender outlines the object's
silhouette, concavities and holes included, so a selected torus got a ring
drawn round it rather than round it and its hole.

### Blender's two passes

`overlay_outline.hh` runs a prepass and a screen pass.

The prepass (`overlay_outline_prepass_vert.glsl`) rasterizes the **selected
objects alone** into a 16-bit id buffer. Each pixel holds a colour class in
the top two bits and the object below it:

```c
static inline uint outline_id_pack(uint outline_id, uint object_id)
{
  /* Replace top 2 bits (of the 16bit output) by outline_id.
   * This leaves 16K different IDs to create outlines between objects. */
  return (outline_id << 14u) | ((object_id << 18u) >> 18u);
}
```

The class is 0 for an object being transformed, 3 for the active object and
1 for the rest of the selection (`outline_colorid_get`), and the object part
is `resource_id + 1`, so **0 is the background** and needs no flag of its
own.

Which objects get into the prepass at all is one line in
`overlay_instance.cc`:

```c
if (object_is_selected(ob_ref) && !in_edit_paint_mode) {
  outline.object_sync(manager, ob_ref, resources, state);
}
```

Selected objects, active or not — so an **active object that is not
selected takes no outline**. The first version of `outline_class` here read
the active name first and outlined it whatever the selection said, which
meant turning Outline Selected off still left a line round the active
object. Its own test caught it, and reading that line settled what the
right answer was rather than leaving it to taste.

The screen pass (`overlay_outline_detect_frag.glsl`) reads the pixel's id
and its four neighbours, a pixel out and — with `do_thick_outlines` — two:

```c
bool has_edge(uint id, float2 uv, uint ref, uint &ref_col, float2 &depth_uv)
{
  if (ref_col == 0u) {
    /* Make outline bleed on the background. */
    ref_col = id;
    depth_uv = uv;
  }
  return (id != ref);
}
```

Two details in nine lines. A pixel holding **nothing** takes its colour
from the first non-zero neighbour, which is how the line lands *outside* the
object rather than eating a pixel of it. And a pixel whose four neighbours
all differ, or none differ, is discarded (`edge_case == ALL || edge_case ==
NONE`) — so the inside of an object and a one-pixel gap both draw nothing.

### What phiray does

The prepass is already paid for: each kernel knows which instance its
primary ray met. `util.wgsl` keeps it, one word a pixel, in the image
buffer's new words 25n..26n (`PX_WORDS` 25 → 26):

```wgsl
fn outline_store(i: u32, id: u32) {
    px[25u * p.n + i] = id;
}
```

`solid.wgsl`, `wire.wgsl`, `shade.wgsl` and `preview.wgsl` call it with
`(face(h.tri).w >> 16u) + 1u` at the primary hit and `0u` where the ray met
nothing — the same convention `fx_store` already used for Workbench's object
id.

**The class is deliberately *not* on the instance.** That was the first
design, and it is wrong: the instance table lives in `data`, and changing it
means `Tracer::reinstance`, which sets `last = None` and `frame = 0` — a
click that selects an object would have thrown away every sample traced so
far. Blender's selection outline costs the render nothing, and so must this
one. So the class is a small array of its own, one word an instance,
uploaded by a setter that touches neither the camera nor the samples:

```rust
pub fn outline(&mut self, gpu: &Gpu, classes: &[Outline]) -> bool {
    let mut words = vec![0u32; self.classes.len()];
    for (word, class) in words.iter_mut().zip(classes) {
        *word = class.word();
    }
    if words == self.classes {
        return true;
    }
    self.classes = words;
    self.reshow = true;
    self.class.write_front(gpu, &self.classes)
}
```

`Outline` (`trace/outline.rs`) is the class as a type — `None`,
`Transform`, `Select`, `Active` — carrying Blender's `color_id` (0, 1, 3),
the table word (`color_id + 1`, so 0 means "no outline" and a zeroed table
outlines nothing) and `packed`, the id the prepass would have left.

`outline.wgsl` is then the detect pass over the two together, as one more
dispatch in `Tracer::tail` — **after** the gather and SMAA, because
Blender's overlay engine runs after the shading and its anti-aliasing. It
composites into the display pixel (`px[i]`) alone, so `Tracer::display` and
a saved render carry none of it, which is also what Blender's own render
does.

### `reshow`: the one more sample a settled image owes

`sample_in` returns early when the camera has not moved and the image has
refined:

```rust
if same && self.frame + 1 >= self.samples() {
    return Some((self.frame, true));
}
```

A new selection does not change the camera, so a settled viewport would
never have drawn the new line. The fix is one flag: `outline` sets
`reshow`, the early return tests it, and the flag is cleared as the sample
goes out. The sample is not thrown away — `frame` goes up, not back to
zero — so selecting an object in a refined Rendered view costs one extra
sample and keeps the other thirty-one.

### What is not ported, and said plainly

* **The anti-aliased line.** Blender's detect pass can emit a line's two
  endpoints (`pack_line_data`) for its own resolve pass to anti-alias. There
  is no such pass here, so this is Blender's `do_anti_aliasing == false`
  branch: a one-pixel, hard-edged line, which is exactly what Blender draws
  with Preferences ▸ Viewport ▸ Overlay Smooth Wires off.
* **The occlusion fade.** `alpha_occlu` draws a selected object's outline at
  35% where another object hides it, which needs the selected objects'
  depth *independently of the scene's* — a prepass that traces them alone.
  phiray has one visibility answer a pixel, so a selected object entirely
  behind another takes no outline here. The `Camera::outline_fx` word for it
  is carried and unread, so wiring a prepass later changes no interface.
* **X-ray.** `xray` walks every surface along the ray and keeps no one
  instance, so there is no id to outline. The CPU hull is still the painter
  there, and `cache::BCpuDraw` now names that state rather than leaving it
  to a `bool`:

```rust
pub(super) enum BCpuDraw {
    /// Everything: there is no GPU image to draw over (`cpu_can_draw`).
    Whole,
    /// The selected objects' hulls alone — the tracer drew the surfaces but
    /// keeps no one instance a pixel to outline, which is X-ray.
    Hull,
    /// Nothing: the tracer drew the surfaces and the outline over them.
    Nothing,
}
```

So in Solid, Material Preview, Rendered and Wireframe the CPU now projects
**nothing at all** for a frame the tracer delivered, where it used to
project and hull every selected object every frame.

### Two traps

**`class` is a reserved keyword in WGSL.** A local named `class` in
`outline_color` failed at `create_shader_module` with `name 'class' is a
reserved keyword` — not at compile time, and not in the Rust tests that
never create a device. The list is longer than it looks (WGSL reserves most
of C++ and Rust's keywords against future use); a shader that only ever
runs behind a GPU check can carry one of them for a long time unnoticed.

**A `view.take()` in the render thread stopped refinement dead.**
`BRayShot` had to stop being `Copy` to carry the class list, and the
destructure `let Some((key, shot)) = view` became `view.take()` by reflex.
It compiles, it draws the first sample of every view correctly, and it
never draws a second: the loop refines by coming back to the same `view`
on the next turn. Two tests caught it — `render_thread_refines_latest_view`
and `a_moved_object_draws_as_a_scene_built_there` — and `view.clone()` is
the answer. `Copy` is load-bearing in a loop that reads the same value
every turn, and nothing says so at the call site.

### Tests

`crates/phiray/tests/outline.rs`: two walls, each its own instance, in
Solid.

* nothing selected draws no line of any of the three colours — the pass is
  left out of the frame altogether (`outlined()`);
* the right wall marked `Active` draws a magenta ring whose box is as wide
  and as tall as the wall, in the right half of the image, and the wall's
  **middle pixel is unchanged** from the unoutlined render: the line is at
  the edge and nowhere else;
* then, with no new scene and no new camera, the selection moves to the
  left wall as `Select`: the magenta ring is gone and a green one is in the
  left half. That is `reshow` working on a settled image;
* each of the three classes draws in its own theme colour, and a class
  named for an instance the scene does not have is ignored rather than read
  past the table.

`view3d/cache.rs`'s `the_cpu_projects_only_what_the_gpu_leaves_it`: the
three `BCpuDraw` states project nothing, the selected object's hull alone,
and every object's surfaces — and key apart in the memo, so toggling X-ray
cannot hand back the wrong projection.

## §66 — The compositor, evaluated (batch 167)

Queue item 12 read "compositor evaluation — nodes draw, nothing runs".
phiber has drawn the compositor's nodes since batch 142 and run none of
them, so a file whose look came out of the compositor — a grade, an
exposure trim, an alpha-over — rendered here as the ungraded render. That
does not look like a missing feature; it looks like a lighting bug, which
is the worst kind of gap to leave.

### Where the tree is, and the first trap

Blender's compositor is one node tree per scene over the render's pixels,
and the tree is a `bNodeTree` exactly like a material's — so
`shade::graph`'s `node_graph_at` reads its nodes, sockets and links
unchanged. What is *not* like a material's is where the pointer to it
lives, and that moved in 5.0:

```c
/* DNA_scene_types.h:2829 */
DNA_DEPRECATED char use_nodes = 0;
DNA_DEPRECATED struct bNodeTree *nodetree = nullptr;
struct bNodeTree *compositing_node_group = nullptr;
```

Three fields side by side, two of them deprecated and still written. 5.x
points the scene at a node **group**; everything before it embedded the
tree and used `use_nodes` as the switch. A reader that takes only the new
spelling finds no compositor in any file made before 5.0, and one that
takes only the old finds none in a file made after. Both are tried against
the file's own DNA — the same shape as `bNode.location` versus
`locx_legacy` (§?, batch 142).

And **there is no Composite node in 5.x**. The truth script found that the
hard way:

```
RuntimeError: Error: Node type CompositorNodeComposite undefined
```

A 5.x tree ends at the group's own output, `NodeGroupOutput`, whose input
socket is *identified* `Socket_1` and merely *named* "Image". So the
output node is found by looking for a group output as well as a Composite
and a Viewer, and what feeds it is found by walking its sockets rather than
by asking for one called "Image".

### The second trap: a blend mode that is not in `custom1`

A compositor Mix used to be `CompositorNodeMixRGB`, with the blend mode in
`bNode.custom1` and Clamp Result in `custom2`. In 5.x it is
`ShaderNodeMix`, shared with the shader trees, and the mode is in the
node's `storage`:

```c
/* DNA_node_types.h:3816 */
struct NodeShaderMix {
  eNodeSocketDatatype data_type;
  NodeShaderMixMode factor_mode;
  int8_t clamp_factor;
  int8_t clamp_result;
  int8_t blend_type;
};
```

Reading `custom1` gives 0 — plain Mix — for every `ShaderNodeMix` in every
file, so a Screen or a Multiply composites as a straight lerp. The symptom
is a wrong *number*, not a missing node, which is exactly the kind of thing
a drawing-only reader never notices. `BlendNode` now carries
`bNode.*storage` as an address (the struct it points at depends on the node
type, so the reader that wants it is the one that knows which struct to
read), and `comp::read`'s `settings_of` digs the mode out while it still
has the file and the DNA.

Its sockets moved too: `ShaderNodeMix` carries an A/B pair per data type —
`A_Float`, `A_Vector`, `A_Color`, `A_Rotation` — and marks the ones its
`data_type` does not use `SOCK_UNAVAIL`. So `node.rs`'s socket lookups take
a *list* of spellings (`sock::IMAGE` is `["Image", "Color", "A_Color"]`),
match on identifier first and name second, and skip unavailable sockets.

### What runs

Everything whose output at a pixel depends on its inputs at that same
pixel. That is the line the whole design sits on: a per-pixel tree is a
straight-line program, and `eval.rs` resolves the graph's names to indices
**once** and then runs a few dozen floats per pixel. Resolving by name per
pixel — the first version — turned a 1080p frame into minutes.

Ported, each from its own `node_build_multi_function` in
`source/blender/nodes/composite/nodes/`: Brightness/Contrast, Exposure,
Gamma, Invert Color, Hue Saturation Value, Posterize, RGB to BW, Set Alpha,
Alpha Convert, Alpha Over (plain, disjoint and conjoint), Mix in all
nineteen `MA_RAMP_*` modes, RGB and Value constants, Render Layers, Image,
Composite, Viewer, Group Output and Reroute. A muted node is a wire, as
`NODE_MUTED` means.

`ramp_blend` (`blenkernel/intern/material.cc:1891`) is now ported **whole**
rather than in part, which fixed two modes the material path had had wrong
since it was written:

* **Linear Light** was `x + fac (2y − 1)`, which is only Blender's
  `col <= 0.5` branch. Above a half Blender adds `2 (col − 0.5)`, so every
  highlight through a Linear Light mix was short.
* **Exclusion** was missing its `max(…, 0)`.

and added the seven that were silently falling through to plain Mix: Color
Dodge, Color Burn, Hue, Saturation, Value, Color and Soft Light. A
material's Mix keeps the first colour's alpha and the compositor's does
too — `shade::blend::mix` is the one implementation, with `ramp_blend`
under it.

### What does not run, named rather than skipped

`BCompTree::declined` lists, by `idname`, every node that reaches the
output and is not run, so a panel can say so. Those are the neighbourhood
filters (Blur, Glare, Defocus, Denoise, Dilate/Erode, Kuwahara,
Despeckle), the transforms (Translate, Scale, Rotate, Transform, Crop,
Flip, Lens Distortion), the keyers, the curve and balance nodes that want a
`CurveMapping`, and anything reading a second datablock (Mask, Movie Clip,
Cryptomatte). A declined node passes its image through rather than
vanishing, so a tree with a Blur in the middle still grades correctly and
simply is not blurred — and says which node it left out.

### Measured against Blender's own compositor

`truth/blender_composite.py` renders a 4 × 4 image of sixteen deliberately
awkward pixels — black, white, over-white (2.0), a negative channel,
saturated primaries, and alphas of 0, 0.25, 0.5, 0.75 and 1 — through one
node at a time, in Blender, through `bpy.ops.render.render`, and records the
output. Thirty-two cases. `tests/composite_truth.rs` runs the same
operations through `comp::pixel` and compares.

The input every case is compared against is the **passthrough** case, not
the input written: Blender writes its EXR in half precision whatever
`color_depth` says, so the pixels the nodes actually saw are a few parts in
10⁵ off the ones asked for. Taking the passthrough as the input keeps the
file format out of the measurement, and the tolerance is then 2 × 10⁻⁴ for
everything.

Every node and all nineteen mix modes agree. Three of them disagreed
first, and all three were *my* reading of a default rather than the
arithmetic:

* **Mix's alpha.** `ramp_blend`'s `MA_RAMP_BLEND` arm mixes `r_col[3]`
  along with the colour, so the first version blended the alpha. Blender's
  own render of a Mix over a pixel of alpha 0.5 comes back at **0.5**, not
  at the blend of 0.5 and 1: the node takes the first input's alpha through
  every mode. So the compositor's Mix is the material's Mix after all, and
  the two share one implementation rather than two.
* **Invert's alpha.** A new Invert node sets `custom1 = CMP_CHAN_RGB` and
  leaves `CMP_CHAN_A` clear, so the alpha is untouched until asked for.
  Defaulting both channels on gave an alpha of 0.25 against Blender's 1.0.
* **Alpha Over's Straight Alpha.** Off is the default, and off means the
  foreground is taken as **already premultiplied** and goes over as it is.
  Premultiplying by default put a foreground over black at 0.6 of itself —
  0.12 where Blender gives 0.2.

Each of those is a one-line difference that the arithmetic cannot tell you
about. They are the whole argument for rendering the reference rather than
reading the source and calling it done.

`tests/composite_tree.rs` then measures the other half: `COMPOSITE.blend`
carries one chained tree — Image → Brightness/Contrast → Hue Saturation
Value → Mix (Screen) → the group's output — and the truth file carries
Blender's render of exactly that tree over exactly those pixels. The reader
reads the tree out of the file, plans it and runs it, and comes to Blender's
pixels within the same 2 × 10⁻⁴. That exercises the DNA spellings, the node
kinds, the socket identifiers, the storage settings and the evaluation
order end to end, not just the maths.

### What is left: the seam into phiber

The evaluator is done and measured; it is **not wired into the viewport or
a render yet**, and the reason is worth stating rather than hiding.

Blender composites the render's **scene-linear** pixels and applies the view
transform afterwards. phiber's linear image is `Tracer::radiance`, and the
view transform lives only in `view.wgsl` — there is no CPU port of
`view_transform_of`, so compositing the linear buffer leaves nothing able
to turn the result back into the displayed image. Compositing the
*display-space* pixels instead would be cheap and wrong: a grade applied
after AgX is not the grade Blender applied before it, and it would be wrong
in a way that looks plausible.

So the next step is one of two things, and the second is the better design:

1. port `view_transform_of` (Standard, Filmic, AgX) to Rust, composite
   `radiance()` and re-transform; or
2. give the tracer an entry point that takes a composited linear image back
   and re-runs its own display step over it — one small kernel, no second
   implementation of the transform, and the same code path a saved PNG and
   the viewport both already go through.

Until then, `BCompTree` is read and reportable: a panel can say the tree
runs, how many nodes it runs, and which it declines.

## §67 — The composite into a render (batch 168)

§66 left the compositor evaluated and unwired, and said why: Blender
composites the render's **scene-linear** pixels and applies the view
transform afterwards, and phiber's view transform lives only in
`view.wgsl`. Two ways out were named. The second is the one taken.

### The way back in

Rather than port `view_transform_of` (Standard, Filmic, AgX) to Rust and
have two implementations of it to keep in step, phiray gained a kernel that
is **the tail of `emit` and nothing else**:

```wgsl
let k = f32(p.frame + 1u);
let l = three(p.n + 3u * i);
let gs = three(20u * p.n + 3u * i);
let cov = bitcast<f32>(px[23u * p.n + i]);
let film = bitcast<f32>(px[24u * p.n + i]);
let shown = view_transform_of(l / k, p.view) * (1.0 - cov / k) + gs / k;
```

and `Tracer::show(gpu, linear)` puts a host's changed radiance back in the
sums and runs it. After that `pixels()`, `display()` and `radiance()` all
come back agreeing with what was handed in — one transform, one grid
composite, one place.

**The trap, and the test that caught it in one run.** `self.last` keeps the
camera as it was *compared* — and the frame is deliberately not part of what
makes two views the same, so `last.frame` is 0. Taking the camera straight
from it gave `k = 1` in the kernel against a buffer holding 32 samples' worth
of sums: an image **32 times too bright**. `show` sets `frame: self.frame`
by hand, and `showing_the_image_its_own_radiance_changes_nothing` — show the
image its own radiance and nothing may move, under each of the three view
transforms — found it immediately. A round-trip test is worth more than
three assertions about the arithmetic here, because the arithmetic was never
the risk; the bookkeeping was.

The second test is the argument for the whole design:

```rust
let ratio = after[lit[0]][0] / before[lit[0]][0].max(1e-6);
assert!(ratio > 0.5, "{view:?}: the transform is concave, so the drop is less than half: {ratio}");
```

Halving the *light* is not halving the *display* — not under Standard's
gamma encoding and much less under Filmic or AgX. So a grade applied to the
displayed pixels is not the grade Blender applied before the transform, and
would have been wrong in a way that looks entirely plausible on screen.

### Where it runs, and where it deliberately does not

Blender runs the compositor as part of the **render pipeline**: F12 renders,
the result goes through the scene's tree, and the Image editor shows the
composite's output. The Rendered *viewport* shows the raw render. So
`still.rs`'s `composite` runs after the samples settle and after the
denoiser, for `still` and `still_stepwise` alike, and the viewport's worker
is left alone. That is parity, not a shortcut.

A frame larger than one binding renders in horizontal bands, and each band
is composited as its own image. That is sound *only* because every node
that runs is per-pixel — a node reading a neighbourhood would see a seam at
each band edge, and those are exactly the nodes `BCompTree::declined`
refuses to run. The comment at the seam says so, because the next person to
add a Blur node to the evaluator needs to know that the band loop is
counting on it.

### The reporting

`BComposite::composite` answers a `BCompositeRun { ran, declined }`, and the
journal gets one line:

```
Compositor: 4 nodes over the render; 1 not run (CompositorNodeBlur)
```

A declined node passes its image through rather than vanishing, so a tree
with a Blur in the middle still grades correctly, simply is not blurred, and
*says* which node it left out. Silence there would be the worst of the three
options.

### The chain, read from the file

`SceneModel.compositor` carries the tree from load (`BlendSceneProps.
compositor`, filled in `scene/read.rs` where the DNA is to hand), so
`composite::tree()` is a clone and a `runs()` filter rather than a reparse.

### Tests

`phiray/tests/show.rs` — the round trip under each view transform, and the
concavity argument above.

`phiber/tests/composite_over_a_render.rs` — the glue, with the tree built by
hand so it says exactly what it is: a Render Layers → Exposure(−1) →
Composite tree halves every lit pixel's radiance and changes the displayed
bytes; the same tree with `on: false` does neither; and the same tree with a
Blur spliced in before the Composite names the Blur, keeps the grade before
it, and passes the image on.

Together with §66's two phiphx tests that is the whole path measured:
the arithmetic against Blender's own compositor, the tree against Blender's
render of that tree, the display step against itself, and the glue against
what it is supposed to do.

## §68 — The base primitives, and a gem that was never a solid (batch 168)

Rendering the primitives side by side — a rainbow row in a rough dielectric
and a second row in polished silver, which is where a bad normal shows at
once — turned up three things at once.

### The gem was not an icosahedron

`build_icosahedron_mesh` had the twelve right vertices, exactly twenty
triangles, and every vertex on the unit sphere. It passed its test. And
**eighteen of its twenty triangles were wrong**: most joined three vertices
that are not mutually adjacent — chords across the solid rather than its
faces — and all but three were wound inward. On screen that is not a gem
with a shading bug; it is a pile of interpenetrating shards.

The faces are derived now rather than typed: a triple is a face precisely
when its three edges are all the solid's own edge length, and it is wound
so its normal points away from the centre. The test was rewritten to check
*that*, because counting triangles is what let the old one through:

```rust
assert!((dist(p, q) - edge).abs() < 1e-4, "{p}-{q} is an edge of the solid, not a chord across it");
…
assert!(nrm.0 * mid.0 + nrm.1 * mid.1 + nrm.2 * mid.2 > 0.0, "the face {a},{b},{c} is wound outward");
…
assert_eq!(seen.len(), 30, "an icosahedron has thirty edges");
assert!(seen.values().all(|&n| n == 2), "every edge is shared by exactly two faces");
```

A count is not a shape. Three assertions about *adjacency, winding and
manifoldness* are, and they are cheap enough to put on every primitive.

### A material per triangle fills the material table

The first sheet drew black cards floating where the gem and the metaball
should have been. The meshes were sound — vertices in range, no zero
normals, sane bounds. The example was asking for one material **per
triangle**, and the metaball alone is 13,668 of them; every face past the
end of the table read a material that is not there, which draws as a black
card. One material per primitive, and the cards are gone. Worth knowing
before building a scene from a mesh: `Scene::material` is a slot, not a
value, and the slots are finite.

### Black squares that moved between runs

Two of the black cards moved from frame to frame, which no mesh or material
bug can do. At 1600 × 760 on a software adapter the path trace was losing
whole workgroups — a GPU timeout, dropping tiles. At 1200 × 640 they are
gone. A rendering artifact that *moves* is a dispatch problem, not a data
one, and that one bit of evidence separates the two in a second.

### What was added

* **Ico Sphere** — the icosahedron subdivided twice onto the sphere, with
  every edge midpoint made once and shared. An unwelded subdivision has
  four times the vertices and splits the shading at every seam; the test
  checks Euler's count, 162 vertices for 320 faces, which only holds when
  they are shared.
* **Grid** — the flat `N × N` quad grid Plane is one quad of.
* **Metaball** (`spatial/implicit.rs`) — Blender's `Add ▸ Metaball ▸ Ball`
  is not a mesh in the file either: it is blobs and a threshold, tessellated
  at display time (`BKE_mball_polygonize`). The field is Blender's own
  `MB_BALL` falloff, `Σ rad² / dist²`, and the surface is polygonised with
  **marching tetrahedra** rather than marching cubes — six tetrahedra to a
  cell, sixteen sign patterns against 256, no ambiguous-face holes, and a
  table short enough to read. Normals come from the field's own gradient, so
  the surface shades as the smooth thing it is, and each triangle is wound
  against that gradient because the tetrahedron's edge order does not say
  which way is out.
* The **UV Sphere** went from 12 × 16 to Blender's own 32 × 16. Smooth
  normals cannot hide a coarse silhouette, and at 12 stacks the edge showed
  its stairs.

## §69 — Text, and the rest of the Add menu (batch 169)

### A Text object is a font curve

Blender's `Add ▸ Text` is not a mesh in the file: it is a **font curve**.
The glyphs' outlines *are* the curve, and `Curve.ext1` — Geometry ▸ Extrude
— gives it depth. So a Text object needs three things, and only the first
is about fonts:

1. the glyph outlines;
2. each glyph's contours filled, with the inner ones cut out — the counter
   of an `o`, both of a `B`;
3. a front, a back and a wall between them.

Reading a TTF at run time would mean a font parser in the crate. The
outlines are **baked** instead (`tooling/bake_font.py` → `spatial/glyph.rs`):
DejaVu Sans, ASCII 32–126, curves flattened at 0.012 em, in em units with
the baseline at y = 0. 95 glyphs, 9,178 points. It is the same geometry
with none of the parser, and the geometry is what the mesh wants.

### Which contour is a hole is nesting, not size

The filling is ear clipping with the holes **bridged** into the outer
contour first: the rightmost point of a hole is joined to the outer edge to
its right by a pair of coincident edges, which turns a
contour-with-holes into one simple polygon ear clipping can take.

The first version picked the largest contour as the outline and treated
the rest as holes. That is right for an `o` and wrong for a `%`, which is
**three separate pieces** — a slash and two rings — so two of its three
outlines were being cut out of the third and the glyph came out with a
*negative* area. The rule is parity: a contour inside an odd number of
others is a hole, and an outline takes the holes that sit inside it. The
test that found it is the one that sweeps the whole font:

```rust
for c in 33u8..127 {
    …
    assert!(total > 0.0, "{ch:?} has a positive area: {total}");
}
```

Ninety-five glyphs is a cheap sweep and it is the only thing that would
have caught `%` in particular rather than whichever letter happened to be
in the render.

One more thing the walls taught: the wall is built over the glyph's **own
contours**, not over the bridged polygon. A bridge is a pair of coincident
edges; walling it puts a blade through the middle of the letter.

### The rest of the Add menu

`BShape` had three variants — Cube, Sphere, Gem — from when the viewport
had three primitives. It has thirteen now, in Blender's own order: Plane,
Cube, Circle, UV Sphere, Ico Sphere, Cylinder, Cone, Torus, Grid, Monkey,
Metaball, Text, and the Gem kept because files and tests from before name
it. Each has a slug that reads back (`BShape::of`), a label, and — the
assertion that matters — a mesh with triangles in it:

```rust
let (v, i, n) = SpatialObject3D::buffers(shape.mesh_type());
assert!(!v.is_empty() && i.len() >= 3 && n.len() == v.len(), "{:?} has a mesh", shape);
```

A menu entry that adds nothing is worse than a missing menu entry, and
that one line is what keeps the two in step.

The palette the CPU painter and the tracer colour primitives from still
carries three shades, so the ten new shapes take the one their silhouette
is nearest rather than each needing a theme colour of its own.

## §70 — Unwrapping, and what a pin is for (batch 170)

Queue item 7's remainder: the UV editor could pick and move corners since
batch 164, but unwrap was "an algorithm of its own" and pin "a writer
change". Both are here.

### The algorithm

`phiphx::uv::lscm` is least-squares conformal maps — Lévy et al.'s
formulation, which is Blender's Unwrap ▸ Conformal and the second half of
its Angle Based. Flatten each triangle into its own plane and ask of the
map `u = U + iV` that its complex derivative be the same in both
directions. Per triangle that is one complex equation,

```text
W₁ u₁ + W₂ u₂ + W₃ u₃ = 0,   Wⱼ = (x_{j+2} − x_{j+1}) + i (y_{j+2} − y_{j+1})
```

and over the mesh a tall real system in `2V` unknowns, solved in the least
squares sense by conjugate gradients on the normal equations — with `AᵀA`
never formed, so each iteration is one pass over the triangles each way and
the memory is the mesh.

### A pin is not decoration

The system **has no unique answer on its own**. A conformal map composed
with any similarity is still conformal, so its null space is four
dimensional and at least two vertices must be fixed. Those are the pins.
That is the whole reason Blender has them, and it is why "pin" could not
be done as a drawing change: a pinned UV is a *boundary condition*.

`BUvEdit` carries `pinned` beside `selected` and `moved`, `pinning(clear)`
is P and Alt P, and `unwrapped(...)` hands the solver each pinned corner at
the UV it is **showing** — the file's, moved — so a pin holds its place
through an unwrap. With fewer than two pins the solver picks the island's
two furthest-apart vertices itself and lays them along U at their own
distance, which is what Blender does and what makes an unwrapped island
come out at roughly its own scale rather than wherever the null space
landed.

The packer respects it too: an island holding a pin is left exactly where
the solver put it. A packer that then slid it elsewhere would undo the pin,
which is the sort of thing that looks like the solver ignoring pins.

### Measured against Blender, on three meshes chosen for what they prove

`truth/blender_unwrap.py` runs Blender's own `uv.unwrap(method='CONFORMAL')`
and records the **distortion** — for every triangle corner, the angle the
mesh makes there against the angle its UVs make. Comparing UVs number for
number would say nothing, because of exactly the similarity freedom above.

| mesh | ours worst | ours mean | Blender worst | Blender mean |
|---|---|---|---|---|
| **Tube** (open cylinder, *developable*) | 0.001° | 0.000° | 0.000° | 0.000° |
| **Band** (sphere between two latitudes) | **4.29°** | **1.363°** | 4.68° | 1.377° |
| **Dome** (spherical cap, with a pole) | 83.6° | **2.362°** | 34.1° | 2.642° |

The Tube is developable: it unrolls with no distortion at all and a solver
that cannot find zero there has a real error — the figure is not a matter
of taste. The Band is not developable and has no degenerate triangle in it,
and there ours is better than Blender's on **both** figures; that is the
case that says the solver is sound.

The Dome is the interesting one. Its top row is a pole — thirteen
coincident vertices and a ring of degenerate quads — and both solvers leave
their worst error there, ours more of it. The *mean* over the whole cap is
better than Blender's, so the difference is the pole and not the solver.
That is recorded in the test rather than tuned around, and the Band exists
precisely so the pole cannot hide a real regression.

### Two things the packer taught

An island wider than the square never fits however the shelves are
arranged. The first shelf packer only checked whether a box fitted *on the
current row*, so an over-wide island wrapped to a fresh row and was laid
past the edge — a whole unwrapped cylinder outside 0..1. The check belongs
on the box, not on the row, and then the scale loop shrinks until it fits.

And one scale for every island, not one each: scaling islands
independently changes their relative texel density, which is the thing a UV
layout is for.

### Projections

`uv.cube_project`, `cylinder_project`, `sphere_project` and
`project_from_view` are in `uv::project`. Cube projection picks its plane
**per face** from the face's own normal — that is what makes it a cube
projection rather than one flat one — and the test is that each of a cube's
six faces comes back as the same unit square, because each is seen head on.
