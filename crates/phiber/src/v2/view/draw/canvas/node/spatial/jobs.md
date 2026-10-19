# Jobs — `canvas/node/spatial`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 7 | 9 |

The spatial node view — the same graph as solids over a floor. **A view, not an
editor** (decided 2026-10-14); Flat is the parity path and nothing here is
ported from Blender, which has no 3D node editor. The wish list this is ticked
against is the project's `spatial-node-view.md`. This file has to stay true:
the node editor's own docs once described a two-card placeholder long after the
real graph was being drawn, and a wish list read as a feature list is the
failure this guards.

Example: pose 26 of CW-POSES (`CHATTY-POSES-RENDER-SCENE-03_Duo pose 26_1.blend`,
Blender 3.6, 337 materials, 3 073 nodes, 3 062 links; richest graph
`Material.129` at 49 nodes / 63 links — 17 Mix, 17 Image Texture, 3 Mapping,
3 Texture Coordinate, 2 Principled, 2 Color Ramp) and Epic Female 01
(`Female-Epic-01-Dress-posing.blend`, 236 materials, 2 288 nodes; richest
`Material.017` at 70 nodes / 105 links). `cargo run --release -p phiber
--example spatial_graph -- FILE --top 12` prints what the view would show.

## Todo

- [ ] **Links: the other routing.** The tube is Blender's curve lifted (`tube.rs`); orthogonal with rounded corners is the alternative the proposal left open — judge with pose 26's `Material.129` on screen, where 17 textures fan into 17 mixes
- [ ] **Long-link folding** — a wire across forty nodes leaves and arrives without being drawn through the middle; both ends keep their true positions
- [ ] **The three-plane stack** — preview / body / sockets as parallel planes rather than one slab
- [ ] **The preview plane showing the node's output** — flagged expensive: needs something that evaluates one node
- [ ] **Easing on the mode transition**, so Flat → Spatial reads as the graph lifting off the page; the camera already lands on the same thing (`camera.rs`), which is the hard half
- [ ] **A depth buffer, if the painter's sort ever shows.** One solid can overdraw a tube passing behind it at a shallow pitch (`paint.rs`); a per-pixel depth test is the fix
- [ ] **`Undefined[...]` nodes.** Pose 26 carries 1 and Epic Female 01 carries 110 nodes whose idname is `Undefined[ShaderNodeMix]` — a type the saving Blender did not know. `class_of` reads the inner name and colours it a converter; Blender draws "Undefined" in red (`node_draw_basis`, `NodeTypeUndefined`). A `phiphx` reader concern, listed here because this view is where it was found

## Done

- [x] **All nine socket shapes** (`paint/shape.rs`, batch 197): `BlendSocket::display_shape` is read, `BSocketRow` carries it, and the painter draws the shape the file says — circle, square, diamond, their three dotted variants, line, volume grid, list — ported from `gpu_shader_2D_node_socket_frag.glsl`, radii and all. **This improves the flat editor today**, which is why it was the prerequisite: a Geometry Nodes tree is unreadable with one shape, because the diamond is what says a socket takes a field. The socket solids on the connection plane are unblocked (`shape::tests`)
- [x] **Titles on the top face** (`title.rs`): a billboard over each solid's projected top, sized by the header's projected height within `TITLE_PX`, dropped below `TITLE_FLOOR` (Flat's `LABEL_FLOOR`), faded on a muted node; painted last so a name is never behind a nearer solid
- [x] **The mode toggle, per space, remembered per space, camera kept** (`mode.rs`, `NodeKey::Mode`): a Flat / Spatial radio at every node editor's header right end, before its overlay popover; absent is Flat (`mode::tests`)
- [x] **The shared layout pass** (`layout.rs`): every node's rect and every link's two ends in view units, read off the SAME functions Flat's `plan` uses — `tests::seen_from_above_every_tube_end_is_the_sockets_flat_pixel`, `…every_solid_is_its_card`
- [x] **A camera over the graph** (`camera.rs`): orbit (left drag), pan (middle drag) and dolly (wheel); the pivot and distance are the flat view's centre and width at Blender's 50 mm, so pitch 90 IS the flat framing and either switch lands on the same thing; only `yaw, pitch` is stored (`camera::tests::straight_down_is_the_flat_framing`)
- [x] **Node solids** (`set.rs`, `solid.rs`): the class radius 2 / 6 / 14 / 26, the hex prism for the field class, lit by a fixed light (`ink.rs`), back faces culled (`solid::tests`)
- [x] **Height = class** (`set.rs`): sources low, operators higher, shaders higher still, the output tallest — the graph rises toward where it is going (`tests::the_graph_rises_toward_its_output`)
- [x] **Frames as floor plates** (`layout.rs`): a frame's bounds, as Flat computes them, as a thin plate under its children
- [x] **Links as tubes, lit solids, depth shading, a ground grid, selection as a rim** (`tube.rs`, `paint.rs`): Blender's noodle curve on the floor and a lift over the rims; the flat canvas's grid shade on the floor; the active node's rim drawn wider than a selected one's (`tube::tests`)

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
