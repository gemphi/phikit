# Phiber — where it stands, and what a world-class version still needs

*Written 2026-09-26, against the tree as synced that day. Every number below
was measured in this session, not recalled; where a claim could not be
measured it says so.*

---

## 1. The one-paragraph verdict

Phiber reads a `.blend` faithfully enough that the geometry, materials,
cameras, world, output settings, render passes and now key columns match
Blender to the precision the truth tests check, and its path tracer — after
the linear-accumulation and GGX fixes of 2026-09-26 (`rendering.md` §18) —
lights a white-override render of the test figure to within 1% of Cycles and
its metal to within 3%. The Rendering space works end to end: F12 renders on a worker
with progress and cancel, the result lands in the Image Editor, and Image →
Save As writes PNG-8/16, EXR half/float and multi-layer EXR with Blender's
own layer names. The code has one shape at every level and a guard that
keeps it. What is **not** there yet: procedural material networks are not
evaluated, five of the twenty-seven editors draw a placeholder in their
centre (the Timeline, the Dope Sheet and the Spreadsheet no longer do), and
twenty-one
interface files plus ten library files are still over the 240-line cap. None of these are mysteries — each has a measurement and a
next step below.

---

## 2. The 240-line rule

**The rule:** no source file over 240 lines. Not a style preference — past a
screenful a file has stopped being one thing, and the split is nearly always
obvious.

**Enforced since 2026-09-23** by `tests/level_structure.rs::no_source_file_outgrows_a_read`
over `src/v2`. The test fails if any file *not on the list below* crosses the
cap, **and** fails if a listed file drops under it and is not removed from the
list — so the backlog can only shrink.

### `phiber/src/v2` — 21 of 445 files over (was 25 before the sets were split)

| file | lines | what the split is |
| --- | --- | --- |
| `theme/semantic.rs` | 451 | one 375-line `impl BResolvedUiStyle`; group by subject: surfaces, hosts, tabs, gutters |
| `area/size/mod.rs` | 357 | resize maths + persistence; split store from geometry |
| `scene/source.rs` | 343 | loaded-model access + loading pipeline; two subjects |
| `host/mod.rs` | 341 | `BHostType` enum + `BHostSpec` + `BHostItem`/`BHostList` views; move the views out |
| `part/pref/mod.rs` | 323 | Preferences part; sections into members |
| `part/file/set.rs` | 312 | file-browser set; split by subject as outliner was |
| `part/outliner/mod.rs` | 310 | item + list views; one file each |
| `scene/render_job.rs` | 304 | camera maths + job machinery; two subjects, already commented as such |
| `view/prim/card.rs` | 301 | |
| `part/file/listing.rs` | 290 | |
| `view/draw/pane.rs` | 281 | |
| `view/draw/canvas/view3d/rendered/mod.rs` | 281 | `BRayView`, `BRayDeclined`, `still`, `still_stepwise`; the stills belong in their own member |
| `view/draw/tab/mod.rs` | 280 | |
| `app/mod.rs` | 278 | |
| `region/mod.rs` | 268 | |
| `view/draw/flex/token.rs` | 261 | |
| `view/draw/card.rs` | 260 | |
| `scene/render_save.rs` | 257 | shallow / deep-PNG / EXR / multilayer writers; one member each |
| `view/draw/region/mod.rs` | 255 | |
| `view/mod.rs` | 250 | |
| `part/mod.rs` | 246 | |

Done this session: `host/set.rs` 1030 → 102, `space/set.rs` 661 → 41,
`part/menubar/set.rs` 528 → 43 (+ `item/`, `topbar/`, `editor/` members),
`part/outliner/set.rs` 342 → 10 (+ `mode/`, `toggle/`, `row_type/`, `filter/`).

### `phiray` and `phiphx` — 10 files over, **not yet guarded**

| file | lines |
| --- | --- |
| `phiphx/src/blend/modifier/bbone.rs` | 673 |
| `phiphx/src/spatial/spatial_object.rs` | 496 |
| `phiray/src/trace/kernel.rs` | 443 |
| `phiphx/src/export/blend/chunks.rs` | 396 |
| `phiphx/src/format/material.rs` | 384 |
| `phiray/src/trace/mod.rs` | 332 |
| `phiphx/src/geom/mesh.rs` | 314 |
| `phiphx/src/scene/texture.rs` | 265 |
| `phiphx/src/blend/scene/output.rs` | 258 |
| `phiphx/src/scene/tests_traversal.rs` | 246 |

The guard only watches `phiber/src/v2`. Extending it to the other two crates
is a one-line change plus these ten names on the list; it is the first thing
to do so the rule means the same thing everywhere.

---

## 3. Structure

**In place and guarded** (`tests/level_structure.rs`, four tests):

- Every one of the eight levels — `space zone area host region pane part modal`
  — is a folder with `mod.rs` (structure) and `set.rs` (content).
- Every member of a level is a folder of its own. 27 hosts, 11 workspaces,
  35 parts, 12 modals, each `<name>/mod.rs` + `<name>/set.rs`.
- Nowhere in `v2` does `x.rs` sit beside `x/`. Seventeen such pairs were
  collapsed.
- Lookup is an exhaustive `match` on the type (`host.spec()`, `host.slug()`,
  `space.spec()`), not a table indexed by discriminant. Adding a variant
  stops the build until its folder exists — the old positional table compiled
  fine and silently handed the new host its neighbour's regions.
- Free functions that duplicated a method (`host_slug(h)` → `set::slug(h)`)
  are gone; one concept, one name, on the type.

**Not yet uniform:**

- Inside `view/draw/canvas/view3d/rendered/` the members are loose files
  (`key.rs`, `scene.rs`, `settle.rs`, `studio.rs`, `vert.rs`, `worker.rs`,
  `world.rs`). Same under `theme/`, `view/draw/`, `scene/`, `state/`. The
  "member is a folder" guard covers only the eight levels. Extending it to
  the whole of `v2` is the next structural step; it will surface ~150 loose
  files to fold.
- `def.rs` — v1's third file per level (constants: labels, icons) — was not
  reintroduced. Labels and icons live in `<member>/mod.rs`. If the three-file
  form is wanted, it is mechanical.
- `region` and `pane` have no members: their specs are declared inline inside
  each host's `set.rs`. That is arguably right (a region has no identity
  apart from its host) but it is the one place the shape differs.

---

## 4. Rendering quality — measured

Reference: Blender 5.0.1 Cycles, 32 samples, Material Preview lighting
(`forest.exr`, scene lamps and world off, flat grey background), on
`THYMUS-BOLDROID-WAVE-003.blend`, 720 × 1080 orthographic front.

| measure (Material Preview, 32 samples, Filmic) | Blender | Phiber (start of day) | Phiber (now) |
| --- | --- | --- | --- |
| figure mean, **white material override**, Standard view | 226.9 | 198 | **233.0** |
| figure mean, real materials | 56.5 | 87.8 | **61.4** |
| figure p50 / p95 | 43 / 139 | 68 / 218 | **53 / 130** |
| shin metal patch | 32.5 | — | **33.7** |
| chest patch | 73.0 | 102.7 | 85.8 |
| head patch | 92.5 | — | 78.7 |

The whole-figure number is a measure again (`rendering.md` §19): the rubber
lining is now black on both sides — its Mix nodes are `Undefined[...]` in the
file and Blender renders the socket default, which phiber now does too — the
measuring example no longer leaves the file's lamps on in Material mode, and
the specular lobe is no longer counted twice against the sky.

**What moved it:** Material Preview now lights with Blender's own
`forest.exr` (bundled, CC0, converted to RGBE at full range — the first
conversion went through the view transform and came back clamped at 1.0,
which is why the metal was flat; the guard now asserts max radiance > 500).
The studio HDRI is *hidden* (lights the model, background stays grey), as
Blender's `studiolight_background_alpha = 0` does. And the fallback
three-point rig no longer sits on top of an HDRI world: it was adding ~20%
to every lamp-less scene in Rendered mode too.

**What is still wrong, in priority order:**

1. ~~**Contrast.**~~ **Fixed** (`rendering.md` §18). The cause was not the
   lobe alone: phiray transformed every *sample* through the view transform
   and averaged the results, where Cycles averages linear radiance and
   transforms once — a concave, clamped curve averaged that way is always
   darker. Linear accumulation plus Cycles' GGX (`ggx_sample_vndf`, Smith Λ)
   took the override render from 198 to 225.4 against 226.9. Still open
   here: the Material Preview background is a sky gradient where Blender's
   is flat.
2. **Procedural materials are folded, not evaluated.** The colour graph is
   constant-folded (Mix, ColorRamp, Hue/Sat, Bright/Contrast, Gamma, Math,
   Clamp, Invert; textures at their mean) — a flat colour per material, not
   the texture. `mat_rubber_lining` turned out not to be a fold problem at
   all: its Mix nodes are stored as `Undefined[ShaderNodeMix]` (saved by a
   Blender without the node), so *every* Blender renders the socket default,
   black, and phiber now matches (`rendering.md` §19). Real evaluation of
   Noise / Musgrave / Gradient / ColorRamp networks in WGSL, or baking them
   to a texture at load, remains the work item.
3. **Thigh speckles** — ray precision where garment and body nearly coincide.
   Cycles offsets along the geometric normal by a distance-scaled epsilon
   (`ray_offset`); ours is `eps + t × 1e-5`. Measure: over-dark pixel count.
4. **135 body vertices > 0.5 mm** after B-Bone deformation (shoulders, head).
   Unfinished from the armature work.
5. **Multi-bounce GI default.** The Render panel's `diffuse_bounces` (RNA
   default 4, Blender's) reaches `Camera::bounces` and the kernel takes it
   whenever it is set, so the depth *is* 4 in practice; the "1 without a
   table" rule only applies when the panel says 0. What differs from Cycles
   is the tail: a path cut off inside the scene under a plain-colour world
   is closed with `0.8 ×` the world seen from the last surface, where Cycles
   simply stops. Re-measure the cavity numbers (0/82/1417 pre-date env NEE)
   before deciding whether that tail stays.
6. **Light-path passes** (Diffuse/Glossy/Transmission × Direct/Indirect),
   Mist, Position, UV, Vector, indices, Cryptomatte. Asked for, read, and
   deliberately not written rather than written black.
7. **Not started:** film transparency, motion blur, panoramic cameras,
   volumes, adaptive sampling, clamping, texture mipmaps/anisotropic
   filtering, the `PHIRAY` engine entry selecting the renderer, OIDN-grade
   denoising (ours is à-trous — good, not Intel-good), Glass BSDF as its own
   closure, hair.

**Speed:** 32 samples at 720 × 1080 on lavapipe (software Vulkan) in the
sandbox: 30 s. Blender Cycles CPU on the same box: 34 s. Not a fair
comparison in either direction — no GPU here — and *not* a number to quote.
Real timing needs your machine.

---

## 5. The interface — host by host

Every host paints from one shade ladder (`ui-parity-status.md`, *Palette*):
the same slot is the same shade in every editor, climbing inward as Blender's
greys do — header 4, body 5, card 6, control 7, hover 8, active 9, canvas 10.

Thirteen editors draw real content in their centre. Two draw the `fallback`
placeholder (Sequencer, Clip). The rest are chrome-only by nature, or empty
as Blender's own are without data.

| host | centre | header / panels | notes |
| --- | --- | --- | --- |
| 3D Viewport | **real** — Wire / Solid / Material / Rendered, gizmo, grid | real | the flagship |
| Image Editor | **real** — Render Result at `sima->zoom` (wheel, pan, Frame All), passes and display channels, Blender's render info bar | `IMAGE_HT_header`: View / Image menus, New / Open → Render Result, slot / view layer / pass / channels | `rendering.md` §21; UV editor shares the canvas and adds Select / UV |
| UV Editor | **real** (Image canvas) | | |
| Shader / Node / Compositor / Geometry Nodes | **real** — node canvas | | node *display*, not evaluation |
| Outliner | **real** — tree, modes, toggles, filters, row icons | real | |
| Properties | **real** — 16 tabs of sections; Render, Output, View Layer seeded from the file | real | most other tabs are declared but not seeded or acted on |
| File Browser / Asset Browser | **real** listing, sidebar, execute bar, purposes | real | |
| Top Bar / Status Bar / Activity | real | | |
| Timeline | **real** — scrub strip, grid, range, key columns, playhead, scrubbing | real — `playback_controls` transport, frame / start / end | `timeline.md`; keys read from the file's actions |
| Dope Sheet | **real** — the Timeline window with its channel region: Summary and object rows, backdrops, key columns | real | `timeline.md` §2b; object channels stay folded |
| Graph Editor | **real** — value grid and scale, F-curves in Blender's colours and fades, key vertices, handles in `TH_HANDLE_*` with their rings, value cursor, strip, playhead, scrubbing | real | `timeline.md` §2c; opens on `graph_create`'s view; scrollers and the channel list's curves not yet |
| NLA Editor | **real** — object / action line / tracks last first at `NLATRACK_HEIGHT`, the action's band and key columns, strips with holds, repeats, mute, selection, outlines, names | real | `timeline.md` §2d; phiphx reads `AnimData.nla_tracks`, guarded against bpy |
| Drivers Editor | **real** — `ED_drivers_editor_init`'s `−2 .. 2` view, base-ten grids, every shown object's drivers sampled as `draw_fcurve_curve` samples them (Generator polynomials included), the value and input cursor lines, channel rows named as `getname_anim_fcurve` names them with colour swatches | real | `timeline.md` §2e; phiphx reads `AnimData.drivers` with Blender 5.0's bone-hide versioning — 101 for the rig, as bpy; sidebar Drivers tab and driver variables not yet |
| Sequencer / Clip | placeholder | real | |
| Info | **real** — the report log (file reads, stats, warnings) | real | |
| Spreadsheet | **real** — the active object's evaluated vertices, index + `position` x/y/z, three decimals | real | Vertex domain only; Edge / Face / Face Corner domains and other attributes not listed |
| Text / Console | empty — as Blender's are without a text block; no Python here | real | |
| Preferences | sections exist | | |

Properties: `render`, `output` and `view_layer` (pass toggles) round-trip
file ↔ panel ↔ renderer. `object` (Transform: location, rotation as XYZ
Euler degrees, scale, dimensions) and `material` (Surface: base colour,
metallic, roughness) now show the **active object's** values and follow it
when it changes (`part/props/active`, a rule on `ctx.object`, guarded by
`properties_follow_the_active_object.rs`); they are read-outs of the folded
scene, not yet written back to the renderer. `scene`, `world`, `data`,
`texture`, `modifier`, `particles`, `physics`, `constraint`, `collection`,
`tool` are declared with sections but not seeded — decoration, in the sense
§11 of `rendering.md` used the word.

Workspaces: all eleven declared with their areas; the tab strip offers nine
(Modeling and Sculpting hidden). Switching works. Whether each workspace's
*layout* matches Blender's default screen was checked for Layout; the others
are unverified.

---

## 6. Reading parity — what `phiphx` gets right

Truth-tested against Blender 5.0.1 (each has a `truth/*.py` and a `*.txt`):
mesh digests, vertex positions after modifiers (armature incl. B-Bones,
within 0.5 mm except the 135 noted), Principled BSDF sockets incl. SSS /
sheen / coat / anisotropy, normal maps, shade maps, packed images, world
Background node + Mapping rotation, ID properties (`scene["cycles"]`),
camera params incl. DoF and focus object, Output panel (path, format, depth,
colour mode, codec — with the depth-flag and separator-normalisation traps
documented), render pass flags and EXR channel names, view transforms, frame
range, resolution.

**Not read, or read but unused:** shader node networks beyond the Principled
fold (see §4.2), Glass/Glossy/Emission as separate closures, light node
trees, volumes, hair/curves, shape keys, drivers, constraints (read but not
evaluated), the animation system beyond F-curves on transforms, Cryptomatte,
Overwrite / Placeholders / File Extensions / stamping in the Output panel,
motion blur shutter.

---

## 7. Method — what has kept this honest

- Every claim about Blender is checked against the 5.2 source in `REFS/` or
  against a file Blender actually wrote (`truth/`). Three times this week the
  source or the file contradicted the first assumption (`depth` is a flag not
  a count; Blender normalises path separators on read; Mist's channel is `Z`).
- Every guard is mutation-tested before it counts. Five survivors were found
  and dealt with: a duplicated `has_image` check, a focus target sitting on the
  camera axis, a plane-vs-sphere focus distinction below pixel resolution
  (recorded as untested rather than claimed), a `blades` width that cannot
  matter, and a rig-off rule the test was not actually measuring.
- Twice this week the record was wrong and got corrected in the doc rather
  than the measurement: the render speckles were not B-Bones; the
  `color_render` orthographic scale was inverted the *other* way from what the
  earlier note claimed (figure 2.3× too large, not too small).

---

## 8. What "world class" needs — the plan

Scoped to the project's stated goal: **read like Blender, render like
Blender, faster.** Authoring (editing meshes, keyframing, node editing) is
not in that goal and is not costed here.

### Now (days)
1. Extend the 240 guard to `phiray` and `phiphx`; split the 31 files. Start
   with the ten that are also *content* (`theme/semantic.rs`,
   `part/file/set.rs`, `render_save.rs`, `render_job.rs`, `kernel.rs`).
2. Extend the member-is-a-folder guard to all of `v2`; fold the loose files.
3. ~~GGX glossy lobe + drop the ad-hoc damping~~ — done, with linear
   accumulation (`rendering.md` §18). ~~Flat Material Preview background~~ —
   done: the viewport background is one colour in every mode, as Blender's
   `TH_BACKGROUND_SINGLE_COLOR` default.
4. Re-measure the cavity darkening with env NEE and decide whether the
   truncated-path tail under plain-colour worlds stays (item 5 of §4).

### Next (weeks)
5. Node-network evaluation for the common procedural set (Noise, Musgrave,
   Voronoi, Gradient, ColorRamp, Mix, MixRGB, Math, Bright/Contrast, Bump,
   Mapping, TexCoord). Bake to texture at load; evaluate live later.
6. Ray-offset precision (the speckles) and the last 135 vertices.
7. Light-path passes in the integrator; Mist / Position / UV / indices.
8. Film transparency + shadow catcher; alpha becomes meaningful.
9. Wire the remaining Properties tabs the way Render/Output were:
   `seed_*_props` from the file, `*_set` back out. Object, World, Material
   first — they change what renders.

### Then (months)
10. Finish the animation editors: the Graph editor's channel curves and
    scrollers, the Drivers sidebar and driver variables — the Timeline, Dope
    Sheet, Graph, NLA and Drivers editors are done (`timeline.md`) on one
    shared canvas. Sequencer and Clip need reading first.
11. Motion blur, panoramic cameras, volumes, adaptive sampling, clamping.
12. Denoiser to OIDN quality, or bind OIDN.
13. Real GPU timing on your machine against Cycles on the same GPU, at the
    same sample count, and a published table. The "faster" half of the goal
    has never been measured where it counts.

### What would make it *the* reference rather than *a* port
- A truth suite that runs Blender headless in CI against every fixture and
  fails on drift, so parity is a number on every commit rather than a claim.
- The 240 rule, the structure guards and mutation testing kept as they are —
  they are what has caught the wrong turns so far.
