# Phiber against Blender's manual, chapter by chapter

Blender's own table of contents as the scoreboard: every chapter of
`manual/render/` and `manual/animation/` in the vendored manual
(`crates/phiber/docs/blender-manual/`), with what Phiber has against it.

**How to read the marks.** ✅ works and is covered by a test · ◐ the common
path works, the gap is named · ▢ laid out, nothing behind it · ✗ not started.

**What is verified here.** Rows marked **(measured 18 Oct)** were checked
against pose 26 on the GTX 1660 Ti in this session. Everything else is taken
from the project's own status docs, which `gap-list.md` records as unaudited
— three of three spot-checks elsewhere were stale, so treat an unmeasured
row as a claim, not a fact.

---

## `manual/render/` — Rendering

| Blender chapter | Phiber | Notes |
|---|---|---|
| `introduction` | ✅ | F12 / Ctrl F12, the render window, cancel |
| `render_engines` | ◐ | Phiray (the tracer) and Workbench (the rasterizer). EEVEE and Cycles are *names a file carries*, both rendered by Phiray — and the Engine row now says Phiray rather than pretending **(measured 18 Oct)** |
| `cameras` | ✅ | the file's active camera, focal length, sensor fit, orthographic, DoF with the camera's own settings; framing matched Blender at **IoU 0.9962** on pose 26 **(measured 18 Oct)** |
| `lights/light_object` | ✅ | sun, point, spot, area; shape, radius, Size Y, a sun's Angle, a spot's Blend and its cone on a scaled object |
| `lights/light_linking` | ✗ | |
| `lights/world` | ◐ | colour, strength, environment image with importance sampling, Sky baked and within 1.6 % of Cycles. **A missing environment image lights the scene magenta, exactly as Blender does** — which is what pose 26 shows **(measured 18 Oct)** |
| `materials/*` | ◐ | Principled BSDF whole; Mix/Add Shader as two lobes; the closure nodes folded to Principled parameters |
| `shader_nodes/*` | ◐ | every texture node (Noise, Voronoi, Checker, Brick, Gradient, Magic, Wave, White Noise, Gabor, Sky), Mapping whole, all 41 Math operations, Bump, Color Ramp. Volumes fold to nothing |
| `color_management/*` | ◐ | Standard, Filmic, AgX with exposure and gamma; **Look is not read** |
| `freestyle/*` | ✗ | |
| `layers/view_layer` | ◐ | one view layer; the Outliner's eye and camera restrictions, which are **two different questions** and are now treated as such in a render **(measured 18 Oct)** |
| `layers/passes` | ◐ | Combined, Depth, Normal, Diffuse Color; the rest are listed and not rendered |
| `layers/filter`, `layers/override` | ◐ | Material Override works (`PHIBER_OVERRIDE`); the filters do not |
| `output/properties/format` | ✅ | resolution, %, render region and crop — read, applied, saved |
| `output/properties/output` | ✅ | path and format; **`-o` now names frames Blender's way** — the last component is a prefix, `####` padding honoured **(measured 18 Oct)** |
| `output/properties/metadata` | ✗ | no stamp |
| `output/animation` | ◐ | Ctrl F12 over a range; FFmpeg Video through the system `ffmpeg`, else MJPEG AVI. No Overwrite / Placeholders / frame step |
| `output/audio/*` | ◐ | Render Audio… as FLAC or WAV; no audio in a movie |
| `cycles/render_settings/sampling` | ◐ | Max Samples drives the render and **opens at the 2048 floor** so the panel and the render agree. Adaptive sampling, noise threshold and time limit are read and saved but **not applied** **(measured 18 Oct)** |
| `cycles/render_settings/light_paths` | ◐ | Diffuse bounces applied; the other bounces and clamping read only |
| `cycles/render_settings/film` | ✅ | Film ▸ Transparent through the tracer and into an RGBA save |
| `cycles/render_settings/motion_blur` | ✗ | panel only |
| `cycles/render_settings/volumes` | ✗ | |
| `cycles/render_settings/performance` | ◐ | the tracer bands by the device's limit on its own; no thread or tile controls |
| `cycles/gpu_rendering` | ✅ | the adapter is chosen in Preferences ▸ System and shown in the Render tab |
| `cycles/baking`, `cycles/osl/*` | ✗ | |
| `eevee/*` | ✗ | EEVEE is not implemented; a file naming it is traced |

---

## `manual/animation/` — Animation & Rigging

| Blender chapter | Phiber | Notes |
|---|---|---|
| `introduction` | ✅ | playback, the transport, the frame range |
| `animation_editors` | ◐ | Timeline ✅, Dope Sheet ◐, Graph ◐, NLA ◐, Drivers ◐. **The wheel zoom and Dope Sheet key-drag had regressed and were restored and tested earlier today** |
| `keyframes/introduction` | ✅ | |
| `keyframes/editing` | ◐ | Dope Sheet: pick, box-select, move (G), delete, duplicate, snap, mirror — saved and read back by Blender. **The Graph editor draws but does not edit** |
| `keyframes/keying_sets` | ✗ | K falls through to Insert Keyframe; no keying-set UI |
| `armatures/*` | ✅ | every bone through its inheritance, Blender's legacy IK ported step for step; THYMUS's 328 bones match Blender at six frames |
| `armatures/posing/bone_constraints/*` | ◐ | Child Of, Track To, Damped/Locked Track, the Limits, the Copies, Stretch To, Transformation, Armature, Action, Pivot, Maintain Volume, Floor, Follow Path, Clamp To. **Spline IK and Shrinkwrap are not evaluated** |
| `lattice` | ◐ | read and applied in the stack |
| `constraints/*` | ◐ | as above, for objects |
| `actions` | ✅ | layered (4.4+) and legacy, slots, Push Down, Stash, the Action editor |
| `drivers/*` | ◐ | every variable kind but Context Property, all three target spaces, to 9.5×10⁻⁷ of Blender's own depsgraph. **Drivers whose target is a material, modifier or shape key are listed but do not reach the scene** |
| `markers` | ✅ | add, select, box-select, move, rename, delete, jump, lock; saved |
| `shape_keys/*` | ✗ | not animated, no Shape Key editor |
| `motion_paths` | ✗ | |
| `in_betweens` | ✗ | |

---

## Where Blender is genuinely ahead

Said plainly, because a scoreboard that only counts wins is no use:

1. **Speed on a heavy scene.** Pose 26 traced at about **4 s a sample** at
   1440 × 1440 over 7.3 M faces and 615 instances; a lighter scene in the
   same app runs at 0.1 s a sample. Cycles would not be 40× slower on the
   same geometry. The suspect is two-level BVH traversal, and the honest
   next step is a profile against `intern/cycles/kernel/bvh/` rather than a
   claim.
2. **Find Missing Files.** The menu entry exists; pose 26 needs the function.
3. **Adaptive sampling.** Every pixel takes Max Samples, so the 2048 floor
   costs full price on pixels Cycles would have stopped early.
4. **Motion blur, volumes, caustics, EEVEE, Freestyle, baking.** Not built.
5. **Shape keys and the Graph editor's editing** on the animation side.

## Where Phiber is ahead

1. **Unwrapping.** On a sphere band ours distorts *less* than Blender's own
   unwrap — 4.29° worst and 1.363° mean against 4.68° and 1.377°.
2. **The command line states its own policy.** `phiber --help` prints the
   two ways it differs from Blender rather than leaving them to be found.
3. **A render that cannot run says why.** `Tracer::why_not` names the limit
   it hit; Blender's "the GPU declined" equivalents are often silent.
