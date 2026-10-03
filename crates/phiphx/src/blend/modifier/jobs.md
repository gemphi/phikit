# Jobs — `phiphx/blend/modifier`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 4 | 6 |

## Todo

- [ ] Bevel (display); Array caps / merge / fit modes; Solidify complex mode; Displace with a texture
- [ ] Armature: envelopes, dual quaternions, B-Bone segments, multi-modifier
- [x] Subdivision: edge and vertex creases (batch 88, `crease.rs`; OpenSubdiv's uniform rules, sharpness past `quality` dropped)
- [ ] Subdivision: UV smoothing; Gregory end caps past `quality` (Blender approximates there, phiphx gives the exact limit)
- [ ] Report skipped (unsupported) modifiers per object

## Done

- [x] Declarative `KINDS` table → `BModifier` trait objects, file order, viewport-enabled only
- [x] Armature (vertex-group skinning from saved pose), Subdivision (Catmull-Clark, limit surface, weights carried), Mirror
- [x] Flat edge topology and parallel evaluation; silhouettes match Blender renders (IoU ≥ 0.999)
- [x] Array, Triangulate, Solidify (simple mode), Displace (textureless), Weld — `tests/stack.rs`, `tests/merge.rs`
- [x] Shared `geom` (area-weighted vertex normals, open boundary) instead of a copy per modifier
- [x] Batched `Skinner` hook: trailing Armature → `SkinJob` (CSR weights, object-space bones), one batch per file, `CpuSkin` reference + fallback (`tests/skinner.rs`)

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
