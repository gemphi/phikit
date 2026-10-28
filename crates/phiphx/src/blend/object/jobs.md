# Jobs — `phiphx/blend/object`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 3 | 5 |

## Todo

- [ ] Parent types `PARBONE` (solved per frame in `anim/follow.rs`, not at open) and `PARSKEL`
- [ ] Collection instances and dupli (`transflag`)
- [ ] Object constraints and drivers (with the animation work)

## Done

- [x] **Vertex parenting** (`vertex_parent.rs`, `world.rs`; `BKE_object_get_parent_matrix`'s `PARVERT1` / `PARVERT3` with `give_parvert`, `ob_parvert3` and `tri_to_quat`): a child on one vertex takes that vertex's world place, one on three the frame `tri_to_quat` builds on the triangle — its normal to Z, the `v1 → v2` edge to X — standing at the centroid, with an index past the mesh giving the parent's origin. The vertices come from the parent's **evaluated** mesh, so `resolve_world_matrices` solves the saved mesh first and `reparent_to_vertices` re-solves the vertex-parented children (and whatever hangs under them) once the modifiers have run — after the frame's follow solve in `scene/read.rs`, and on the progressive path as each evaluated mesh lands (`scene/progressive_links.rs`, `SceneSwap::moved`), which is why a light, a camera or an empty can move too. 50 of the user's 230 files use `VERTEX_3`. `tests/vertex_parent_truth.rs`: a child of an Armature-posed parent within **1e-6** of Blender eager and progressive, every child's rotation within **1e-6**, and a child of a subdivided parent within **5e-3** — bounded by `modifier/subsurf.rs`'s vertex order, not by this port (`modifier/jobs.md`). Real: 32 of the 33 `VERTEX_3` empties of Mannequins-Female-Models-Rigged-Clothing-Final within **1e-3**, one at 2.9e-3, every parent under Subdivision + Armature. BC-15's three vertex-parented empties cannot be compared — their collections are excluded from the view layer, so Blender leaves their matrices unevaluated at the origin while phiphx solves them

- [x] Local and world matrices (`BKE_object_to_mat4`, parent chain), all rotation modes
- [x] Camera / light (`Lamp` ≤ 3.x) / empty display data
- [x] Collections, view layer flags (selected, hidden, excluded), world colour
- [x] `ObjectKind` names (`OB_*`)

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
