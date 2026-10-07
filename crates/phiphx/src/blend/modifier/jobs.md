# Jobs — `phiphx/blend/modifier`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 5 | 7 |

## Todo

- [ ] **Write driven modifier settings into the stack** (batch 174 read them; nothing consumes them yet). Two parts, and the second is the real one: `modifier_jobs` / `evaluate_modifiers` must take the frame's driven values and hand each reader its overrides, and a mesh is currently evaluated **once at open** rather than per frame, so a driven modifier would still not animate without a re-evaluation path. The semantics are settled and measured (`driver.rs`, `tests/mod_driver_truth.rs`), so this is mechanical — but it lands in `mod.rs`, where another agent is active, so coordinate before starting
- [ ] Bevel (display); Array caps / merge / fit modes; Solidify complex mode; Displace with a texture
- [ ] Armature: envelopes, dual quaternions, B-Bone segments, multi-modifier
- [x] Subdivision: edge and vertex creases (batch 88, `crease.rs`; OpenSubdiv's uniform rules, sharpness past `quality` dropped)
- [ ] Subdivision: UV smoothing; Gregory end caps past `quality` (Blender approximates there, phiphx gives the exact limit)
- [ ] Report skipped (unsupported) modifiers per object

## Done

- [x] Modifier drivers read (`driver.rs`, `driver_read.rs`): `modifiers["Name"].property` off the **object's** `AnimData`, with each modifier's type resolved from its own stack because the path does not carry it and the same property name differs by type (`strength` on Displace and Lattice, `merge_threshold` on Weld and Mirror). Float settings clamp to their RNA range; integer settings take a driver's float by **truncation toward zero** — a C `(int)` cast, measured against Blender as 0.7 → 0, 2.5 → 2, −0.7 → 0, −2.5 → −2, where a floor gives −1 and −3 — then clamp, and the minima are not all zero (`Subsurf.quality` 1, `Triangulate.min_vertices` 4). 85 values over 5 frames exact on the integers and within 6×10⁻⁸ on the floats (`tests/mod_driver_truth.rs`, MODDRIVERS fixture). **Not yet written into the stack** — see Todo
- [x] Declarative `KINDS` table → `BModifier` trait objects, file order, viewport-enabled only
- [x] Armature (vertex-group skinning from saved pose), Subdivision (Catmull-Clark, limit surface, weights carried), Mirror
- [x] Flat edge topology and parallel evaluation; silhouettes match Blender renders (IoU ≥ 0.999)
- [x] Array, Triangulate, Solidify (simple mode), Displace (textureless), Weld — `tests/stack.rs`, `tests/merge.rs`
- [x] Shared `geom` (area-weighted vertex normals, open boundary) instead of a copy per modifier
- [x] Batched `Skinner` hook: trailing Armature → `SkinJob` (CSR weights, object-space bones), one batch per file, `CpuSkin` reference + fallback (`tests/skinner.rs`)

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
