# The gap list — what is actually open, ranked

Built from the 37 `jobs.md` files the modules already keep, not from a fresh
survey: **70 open items, 133 done**. Those files are the real tracking system,
so this ranks them rather than replacing them.

**Read the 70 as an upper bound.** I spot-checked two items before ranking on
them and *both were already done*:

| tracked as open | actually |
| --- | --- |
| `blend/jobs.md` — "Bézier evaluation" | implemented: `anim/fcurve.rs:179`, `IPO_BEZ => bezier(a, b, frame)`, with a test |
| `view/jobs.md` — "Viewport interaction: orbit, pan, dolly, numpad views" | orbit and dolly across six `view3d/` files; numpad views in `op/render.rs` |
| `scene/jobs.md` — "frame → matrices → re-evaluated meshes" | the solve, the pose and the post-Armature re-run all exist; three narrow cases do not (Tier 0 below) |

Three for three. All three lines are now corrected in place. That is the same drift that had `workflow.md` describing a
two-card placeholder long after the node editor drew real graphs, and four
driver kinds listed as working while nothing called them. So: a sweep of the
70 against the code is itself a job, and until it is run, every count below is
"as tracked", not "as measured".

## Tier 0 — the keystone, re-scoped after reading the code

**Per-frame geometry.** The tracked line reads *"frame → object / bone
matrices → re-evaluated meshes (per-frame cache)"*, and **most of that is
already built** — the third stale line found in this sweep, and the one that
mattered most, because a batch spent building it would have rebuilt working
code.

What exists: `SceneMotion::{solved_at, posed_at, meshes_at, skinned_at}` and
`SceneRigs::meshes_at`, driven by phiber at `v2/scene/pose/mod.rs:67`. A
**rigged** mesh re-poses per frame and the modifiers *after* its Armature
re-run with it.

What does not, and this is the whole remaining gap:

1. **Driven modifier settings are fixed at the open frame.** The stacks were
   built once with `frame_current`'s values, so a driven Displace strength
   holds the open frame's number along the whole timeline.
2. **The pre-armature prefix is fixed at open** — right for the pose, wrong
   once a driver moves a prefix modifier's setting.
3. **A non-rigged object never re-evaluates.** `source.evaluate` is called
   only from `rig.rs:73`; an object with no Armature gets a new matrix per
   frame and never new geometry.

The blocker is ownership, not maths. `BlendSkinSource` is handed out as an
`Arc` and `tail` is `Vec<Box<dyn BModifier>>`, so `set_driven` (which needs
`&mut`) cannot reach it; and the `BlendFile` is not retained after the open
(`from_blend_owned` frees the raw buffers), so the stack cannot be re-read per
frame either. The choice is between interior mutability on the driven
settings, a `clone_box` on `BModifier` so each frame gets its own stack, or
caching one evaluated stack per distinct frame. **That decision is the batch**
— it should be made before any code.

Five tracked items still wait on it:

1. `part/jobs.md` — "Timeline channels from the file's F-Curves (**after
   `phiphx` animation**)" — says so in its own text.
2. `tests/jobs.md` — "Per-frame truth **once animation lands**".
3. `jobs.md` (crate root) — "Progressive open: return rest meshes first,
   evaluated meshes when the modifier stack finishes".
4. Driven modifier settings (batch 188): measured, landing, and **invisible**,
   because a mesh is evaluated once at the saved frame where the file's own
   values already agree with the drivers.
5. A driven Subdivision level (batch 189): the modifier is now built and the
   driver reaches it, and nothing can show it until the mesh is evaluated at a
   frame where the level differs.

This is the answer to "why does it feel like we are running behind". It is not
that the small items are slow — four landed today. It is that one unbuilt
thing is the reason several finished things do not show.

## Tier 1 — per host, as tracked

### Animation
- Legacy `bAction.curves` and layered actions (4.4+) — `blend/jobs.md`
  (Bézier, on the same line, is done; the line needs splitting)
- Object constraints and drivers "with the animation work" — `object/jobs.md`
- Timeline channels from F-Curves — blocked on Tier 0

### Graphs (Shading · ECS Wand · Compositing)
The editors draw real graphs; what is open is what feeds them:
- Image colour space and UDIM tiles — `shade/jobs.md`
- One texture per socket, Mapping types, coordinate kinds — `shade/jobs.md`
  (**verify first**: `workflow.md` records Sky, Gabor and White Noise as
  landed in batch 160, so this line may be partly stale too)

### Geometry
- Bevel; Array caps / merge / fit modes; Solidify complex mode; Displace with
  a texture — `modifier/jobs.md`
- Armature: envelopes, dual quaternions, B-Bone segments, multi-modifier
- Subdivision: UV smoothing; Gregory end caps past `quality`
- **Report skipped (unsupported) modifiers per object** — small, and it turns
  every future gap into something the app tells you instead of something
  someone measures
- Loose edges for wireframe (#22) — `mesh/jobs.md`; this is the 192-vertex
  parity gap from batch 183, already measured against Blender to six decimals
- Colour attributes; 5.x `custom_normal`
- Parent types `PARBONE` / `PARSKEL` / `PARVERT`; collection instances and
  dupli — `object/jobs.md`

### Rendering / viewport
- GPU viewport: renderer 2 (appkit wgpu, Material Preview) and renderer 3
  (WebGPU path tracer) — `view/jobs.md`
- Camera view zoom / passepartout — `scene/jobs.md`
- Memory figure in the read report — `scene/jobs.md`

### Layout / chrome
- Saved area grid and editor types from the file (#28) — tracked in **two**
  places, `space/jobs.md` and `blend/scene/jobs.md`
- Area sizes restored from the file's screen — `area/jobs.md`
- Outliner expand state; Properties tab

## Tier 2 — cross-cutting, and cheaper than it looks

Three themes account for **14 of the 70**, and each is one decision applied
many times rather than fourteen problems:

| theme | count | where |
| --- | --- | --- |
| "Declarative audit" — every part built from its `set.rs` table | 5 | host, pane, region, space, part |
| One look: spatial elevation scale, one menu look, one zone look | 4 | theme, view ×2, zone |
| Files over the 240-line cap | 5 | `modifier/mod.rs` 429, `spatial_object.rs` 417, `chunks.rs` 274, `format/material.rs` 259, `geom/mesh.rs` 248, `tests_traversal.rs` 217 |

## Tier 3 — not on the critical path

USD/glTF through `Phx::open`, the USDC binary writer, WebAssembly, memory-mapped
loading for files over 1 GB, LAS/PCD/XYZ round-trip tests, deleting the
duplicate reader in `crates/apple/src/phiphx`.

## Definition of done, per host

The reason the four big areas stall is that "done" has been a feeling. Each
line below is a **number that moves**, so progress is countable without anyone
re-reading the code — which is the failure mode this whole document exists to
fix.

| host | done when | counts today |
| --- | --- | --- |
| **Animation** | a per-frame truth test compares N frames of Blender's evaluated object matrices **and** meshes against ours, worst error stated | truth tests exist per *feature* (drivers, NLA, rigs) but none per *frame*; `tests/jobs.md` tracks it as open |
| **Graphs** | every node type the fixtures carry is drawn, and socket shapes read 9 of 9 from `display_shape` | shapes: **2 of 9** (`paint/dot.rs`); node types drawn: not counted anywhere |
| **Geometry** | `open_parity.rs`'s `SHORT` table is empty, and every unsupported modifier is reported per object rather than dropped | `SHORT` holds **2 rows** (192 verts of loose-edge subdivision, 48 of curve tessellation); skipped modifiers: not reported |
| **Rendering** | each engine either renders as itself or says it does not, and a Cycles comparison states its tolerance per fixture | `rendering.md` carries measured comparisons; the engine choice is read, saved and **ignored** — one tracer renders them all |

Three of those four already have the measuring instrument built and only the
count is missing. The fourth — Animation — needs the instrument, and that is
`tests/jobs.md`'s per-frame truth line.

**The cheapest lever is "report skipped modifiers per object".** It is one
open item in `modifier/jobs.md`, and it turns the Geometry row from something
someone measures by hand into something every file open prints. The same trick
would serve Graphs: count the node types a file carries that the editor cannot
draw, and say so.

## Which documents to trust

Verified against the code during this session. Everything not listed was not
checked, and given the hit rate should be assumed stale until it is.

| document | state |
| --- | --- |
| `driver-parity.md` | **current** — rewritten through batches 185–190 as each change landed |
| `blender-head-to-head.md` | **current** — timings re-measured A/B in batch 184 |
| `workflow.md` | **corrected** — three driver rows claimed "works" when nothing called the pass; the node-editor placeholder claim was already stale |
| `blend/jobs.md`, `view/jobs.md`, `scene/jobs.md` | **corrected** — one stale line each, found by spot-check |
| `animation-rendering-status.md`, `space-status.md`, `ui-parity-status.md`, `panel-parity.md` | **unchecked** — the four status docs. Three of three spot-checks elsewhere were stale, so these are the next thing to audit, not to cite |

The rule that would stop the drift: a document that claims a feature works
names the test that proves it. Every corrected line above failed that test —
each claimed a capability with no caller, no count, and no test behind it.

## What I would do next, in order

1. **Decide the ownership question in Tier 0**, then build only the three
   narrow cases — not the whole line, most of which exists.
2. **Sweep the 70 against the code.** Two of two spot-checks were stale; the
   list is the project's memory and it is lying in at least two places.
3. **Report skipped modifiers per object.** Small, and it converts gap-finding
   from a manual measurement into something every file open tells you.
4. **Loose-edge subdivision.** Already measured against Blender; it is a
   closed problem waiting to be typed.

## On the screenshots

The annotated how-to-use screenshots are gated on "when all 5 hosts are done".
Nothing in Tier 0 or Tier 1 blocks *taking* them — the viewport navigates, the
editors draw, renders work. What they would currently document is an app whose
animation does not re-evaluate geometry per frame, so any screenshot of the
timeline would be showing a mesh that does not move with it. That is the one
thing worth fixing before the guide, rather than all seventy.
