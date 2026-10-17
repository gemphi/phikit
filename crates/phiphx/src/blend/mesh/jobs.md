# Jobs — `phiphx/blend/mesh`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 2 | 4 |

## Todo

- [x] **Loose edges for wireframe display (#22)** (batch 194). `mesh_loose_edges()` in `crease.rs` detects edges not used by any face; `BlendEvalMesh.loose_edges` carries them through the modifier stack — mirror (remap via `map[]`), array (offset duplication), solidify (inner shell copy), weld (remap via `moved[]`, drop degenerate), subsurf (`refine()`); triangulate passes through naturally. `BlendSceneObject.loose_edges` exposes them to the scene layer
- [ ] Colour attributes; 5.x `custom_normal`
- [x] Edge / vertex creases (`crease.rs`: 4.x named layers, 5.x attributes, 3.x `CD_CREASE`, legacy `MEdge.crease`)

## Done

- [x] Positions, faces, corner UVs (active map), material index, smooth flags — 2.8 to 5.2
- [x] 5.x attribute storage (array and single-value)
- [x] Legacy `MVert` / `MPoly` / `MLoopUV`; vertex-group weights

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
