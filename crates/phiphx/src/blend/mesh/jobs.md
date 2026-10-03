# Jobs — `phiphx/blend/mesh`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 3 | 3 |

## Todo

- [ ] Loose edges for wireframe display (#22)
- [ ] Colour attributes; 5.x `custom_normal`
- [x] Edge / vertex creases (`crease.rs`: 4.x named layers, 5.x attributes, 3.x `CD_CREASE`, legacy `MEdge.crease`)

## Done

- [x] Positions, faces, corner UVs (active map), material index, smooth flags — 2.8 to 5.2
- [x] 5.x attribute storage (array and single-value)
- [x] Legacy `MVert` / `MPoly` / `MLoopUV`; vertex-group weights

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
