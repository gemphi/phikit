# Jobs — `phiphx/geom`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 2 | 2 |

## Todo

- [ ] Split `geom/mesh.rs` (248 lines)
- [ ] Shapes: UVs per part (RealityKit needs them for textured looks), a numbered d20 (faces numbered like the dice), more shapes as rows (rounded cylinder, wedge, arrow)

## Done

- [x] Vertex, bounds, mesh with area-weighted sampling, BVH, subject selector
- [x] Shapes from `data/shape.tsv` (`shape.rs`, `shape/`): cube, dice (rounded edges, 21 pip discs from `data/pip.tsv`), sphere, cylinder, cone, torus, plane, capsule, pyramid, tetrahedron, octahedron, icosahedron; parts with per-corner normals; written to `crates/appkit/app/phiogy/asset/shape/` by `examples/shape_usd.rs`. Tests: `geom::shape::tests::*`

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
