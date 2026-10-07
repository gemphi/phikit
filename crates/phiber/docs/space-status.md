# Spaces — what works, what doesn't

One row per workspace tab, one row per editor. Kept current with the batch
numbers; **last updated at batch 170**.

| | meaning |
|---|---|
| **works** | does what Blender does, covered by tests |
| **partial** | the common path works, named gaps below |
| **shell** | lays out and draws chrome, no content behind it |
| **n/a** | Blender has it, we have not started it |

---

## The workspace tabs

| Space | Status | What works | What doesn't |
|---|---|---|---|
| **Layout** | **works** | Viewport, Outliner, Properties, Timeline — opening a file, navigating, selecting, playing, rendering; the overlay is the GPU's now too — grid, axes and the selection outline (batch 166) | the outline has no anti-aliased line and no occlusion fade; X-ray still outlines on the CPU |
| **Animation** | **works** | playback of objects, NLA, rigs, constraints, IK and rig drivers; object drivers with every variable kind Blender has but Context Property, in all three target spaces (batch 171); keying (I, auto-key, Alt I); Dope Sheet pick / move / delete; markers; actions, slots, NLA tracks and strips | Graph editor *editing*, channel rows per F-curve, NLA tweak mode, drivers on materials / modifiers / shape keys, shape keys, F-curve modifiers; ~390 inert menu entries |
| **Rendering** | **works** | one full-width Image editor on the Render Result, as Blender's; F12 / Ctrl F12, slots, passes, region, save, movies, audio; Render ▸ Engine decides what a render draws with — Workbench rasterizes, EEVEE / Cycles / `PHIRAY` path-trace (batch 160) | — |
| **UV Editing** | **works** | the UV editor draws and edits — pick, Shift-add, drag (batch 164) — and **unwraps**: least-squares conformal maps with pins as the boundary condition they are, plus Cube / Cylinder / Sphere / View projections and island packing (batch 170). On a sphere band ours distorts *less* than Blender's own unwrap | no box or island select; Angle Based's ABF pre-pass is not ported (Conformal is the solver both methods end in) |
| **Shading** | **partial** | the viewport is the full one, in Material Preview; the node editor is being built now (add menu, links, sockets, frames) | the node graph is not yet the file's own tree for every node type |
| **Compositing** | **works for the per-pixel nodes** | shares the node editor's canvas; the scene's tree is read, evaluated — every per-pixel node and all nineteen Mix modes within 2×10⁻⁴ of Blender's own compositor (batch 167) — and **run over a render**, on the scene-linear pixels before the view transform, as Blender's render pipeline runs it (batch 168). The viewport shows the raw render, as Blender's does | the neighbourhood filters, the transforms, the keyers and the curve nodes are named by `BCompTree::declined` and pass their image through (`rendering.md` §66, §67) |
| **Geometry Nodes** | **partial** | shares the node editor's canvas; the Spreadsheet reads the mesh's attributes over Vertex / Face / Face Corner from its dataset region (batch 161) | no geometry-node evaluation |
| **Texture Paint** | **shell** | lays out: Image editor beside the viewport | no painting — brushes, strokes, texture slots |
| **Scripting** | **shell** | lays out: Text, Console, Info beside the viewport | no Python, no console, no text editing |

*(Modeling and Sculpting exist with specs but are deliberately not offered
as tabs.)*

---

## The editors themselves

The real unit of work: a workspace is only as done as the editors in it.

| Editor | Status | Where it stands |
|---|---|---|
| **3D Viewport** | **works** | all four shading modes render on the GPU through phiray over one BVH — Wireframe, Solid (Workbench ported), Material Preview, Rendered. Overlays, gizmos, camera with DoF, soft shadows, cavity, matcap, textures, X-ray, render region. The overlay is the GPU's: the floor grid and axes accumulate per sample in every kernel, and the **selection outline** is Blender's own id-edge pass over the finished image (batch 166). The CPU projects nothing for a frame the tracer delivered — only the lamp, camera and empty wireframes, the origin dots and the 3D cursor are still drawn over it, a few dozen line segments. Open: no anti-aliased line (Blender's Smooth Wires off), no occlusion fade, and X-ray keeps the CPU's convex hull |
| **Properties** | **works** | Render, Output, View Layer, Scene, World, Object, Modifier, Material tabs; values read from and saved back to the file |
| **Outliner** | **works** | hierarchy, collections, filters, visibility and render switches, X-ray |
| **Timeline** | **works** | playback, transport, frame range, markers, keying settings |
| **Dope Sheet** | **partial** | pick, move and delete keys and save them; channel rows per F-curve not done |
| **Image editor** | **works** | Render Result and its slots, New / Open / Reload / Copy / Invert / Flip / Rotate / Save As, render region |
| **File browser** | **works** | Open, Save As, Revert, the recent list, filters |
| **Graph editor** | **partial** | curves draw with their handles; no editing |
| **NLA editor** | **partial** | tracks and strips draw and play; no tweak mode, no strip dragging |
| **Drivers editor** | **partial** | lists every driven object's curves; rig drivers and **object transform drivers** are evaluated — within 2×10⁻⁴ of Blender's own matrices (batch 165) | drivers on materials, modifiers, shape keys and custom properties are listed but not run; a driver chained off another reads the keyed value under it |
| **Node editor** | **in progress** | the other agent is building it now — add menu, links, sockets, frames, picking |
| **UV editor** | **partial** | draws, and picks and moves UV corners in Edit Mode; read-only in Object Mode as Blender's is (batch 164) | no unwrap, pin, box or island select |
| **Spreadsheet** | **partial** | the dataset region lists the mesh's domains with their element counts; the table reads the chosen one — `position` and `UVMap` on Vertex, `material_index` on Face, the colour attribute on Face Corner (batch 161) | no Edge domain (the reader triangulates and keeps no edge list), no row filters or pinning, meshes only |
| **Sequencer** | **shell** | lays out only |
| **Text / Console / Info** | **shell** | lay out only; the Info editor does carry the report log |
| **Movie Clip** | **n/a** | not started |

---

## Reading the file — separate from the spaces, and ahead of them

| | Status |
|---|---|
| Mesh, UVs, material indices, smooth flags | **works** — matches Blender exactly across 1,276 meshes in 8 files |
| The modifier stack | **works** — silhouettes match Blender's own renders at IoU ≥ 0.999 |
| Rigs, constraints, IK, actions, NLA, drivers | **works** — pose for pose against Blender at every frame of a 140-frame animation |
| Lights, world, cameras, render settings | **works** — and now measured against Cycles' own numbers (`render-parity.md` §2.6) |
| Unwrapping a mesh | **works** (batch 170) — least-squares conformal maps measured against Blender's own: on a developable cylinder 0.001° of angle distortion against Blender's 0.000°, on a sphere band **4.29° worst and 1.363° mean against Blender's 4.68° and 1.377°**. A spherical cap with a degenerate pole is worse at the pole and better on average, and is recorded as such. Pins are the solver's fixed points, which is what they are for |
| The base primitives (Plane, Cube, Circle, UV Sphere, Ico Sphere, Cylinder, Cone, Torus, Grid, Monkey, Metaball, **Text**, Gem) | **works** (batches 168–169) — all thirteen are in `BShape` and the Add menu's order, each with a slug that reads back and a mesh the test insists has triangles in it; **Text** is a real font curve — DejaVu Sans outlines baked, filled by ear clipping with the counters bridged out as holes, and extruded (`rendering.md` §69). the gem was not an icosahedron at all until now (eighteen of its twenty triangles joined non-adjacent vertices, most wound inward); Ico Sphere, Grid and Metaball are new, the Metaball polygonised from Blender's own `MB_BALL` falloff by marching tetrahedra. Every one is checked for adjacency, winding and manifoldness rather than for a triangle count |
| The scene's compositor tree | **works** for every per-pixel node (batches 167–168) — run over a render on the scene-linear pixels before the view transform, as Blender's pipeline runs it; the viewport shows the raw render, as Blender's does. Read from both DNA spellings (`compositing_node_group` in 5.x, `nodetree` + `use_nodes` before it), and every **per-pixel** node evaluated within 2×10⁻⁴ of Blender's own compositor over sixteen awkward pixels: Brightness/Contrast, Exposure, Gamma, Invert, Hue Saturation Value, Posterize, RGB to BW, Set Alpha, Alpha Convert, Alpha Over (three modes), Mix (all nineteen `MA_RAMP_*` modes), constants, and the inputs and outputs. Declined and *named* rather than skipped: the neighbourhood filters, the transforms, the keyers, the curve nodes and anything reading a second datablock |
| Materials: Principled, the shader nodes, procedural textures | **works** — Mix/Add Shader, the closure nodes, Noise, Voronoi, Checker, Brick, Gradient, Magic, Wave, White Noise, Gabor, Mapping whole, a Math node between a texture and its socket (all 41 operations), and the **Sky** texture baked to the world's environment and measured within **1.6% of Cycles** above the horizon (batches 160, 163). Open: multiple scattering is read as single scattering (the inter-scattering term is not ported), and the two analytic models — Preetham, Hosek/Wilkie — are declined rather than approximated |

---

## Closing the partials — the queue, in order

Every `partial` and `shell` above, with what is actually left in it and
roughly what it costs. Ordered by value per unit of work, not by how it
reads in the table. Kept in step with the rows.

| # | The gap | Where | Size | Status |
|---|---|---|---|---|
| 1 | **White Noise, Gabor** texture nodes | phiphx + phiray | small | ✅ **batch 160** |
| 2 | **Math node** between a texture and its socket — all 41 operations; it was a *loss*, not a missing feature: a texture behind one vanished from the material | phiphx + phiray | small | ✅ **batch 160** |
| 3 | **Render engine enum** — it was worse than `PHIRAY` doing nothing: the row was a dropdown *no* render read, which asked the file instead | phiber | small | ✅ **batch 160** |
| 4 | **Spreadsheet attributes** — and the left region was the generic Toolbar, where Blender has the dataset region; the Spreadsheet has no tools | phiber | medium | ✅ **batch 161** |
| 5 | **Dope Sheet channel rows per F-curve** — keys pick, move and delete today, but only on one row per object | phiber | medium | **blocked**: the other agent owns `canvas/time/` |
| 6 | **Graph editor editing** — curves and handles draw; no select, move or handle drag | phiber | medium | **blocked**: the other agent owns `canvas/time/` |
| 7 | **UV editing** — select, move, **unwrap** (LSCM with pins) and the projections | phiphx + phiber | large | ✅ **batches 164, 170**; box and island select still queued |
| 8 | **NLA tweak mode and strip dragging** | phiber | medium | queued |
| 9 | **Drivers beyond rigs** — every driver *variable* Blender has but Context Property, all three target spaces, and custom-property drivers ordered before the channel drivers (`docs/driver-parity.md`) | phiphx | medium | ◐ **batches 165, 171**; material, modifier and shape-key *targets* still listed only |
| 10 | **Sky texture** — baked to an equirect image the environment path already carries, so no kernel change; within 1.6% of Cycles above the horizon | phiphx | large | ✅ **batch 163** |
| 11 | **Overlay pass on the GPU** — the grid was *already* on the GPU; the selection outline was the whole cost, and is Blender's id-edge pass now | phiber + phiray | large | ◐ **batch 166** (outline; no AA line, no occlusion fade, X-ray on the CPU) |
| 12 | **Compositor evaluation** — nodes draw, nothing runs | phiphx + phiber | large | ✅ **batches 167–168** for every per-pixel node; the neighbourhood filters, transforms, keyers and curve nodes are declined and named |
| 13 | **Geometry-node evaluation** | phiphx + phiber | very large | queued |
| 12b | **The composite into a render** — done as a tracer entry point that re-displays a composited linear image, so there is one implementation of the view transform rather than two (§67) | phiray + phiber | medium | ✅ **batch 168** |
| 12c | **The compositor's neighbourhood nodes** — Blur first, which also means a band's composite can no longer be its own image | phiphx + phiber | large | queued |
| 14 | **Texture Paint** — brushes, strokes, texture slots | phiber | very large | queued |
| 15 | **Scripting** — Python, console, text editing | phiber | very large | not planned |

The node editor is the other agent's in-flight work and is not in this
queue; neither is anything under `canvas/time/`.

---

## How this is kept

Updated at the end of every batch that changes a space's standing, together
with `ui-parity-status.md` (the detail) and `workflow.md` (the step-by-step).
This file is the summary those two are too long to be.
