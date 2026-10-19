# Jobs — `phiphx/scene`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 1 | 14 |

## Todo

- [ ] **Per-frame geometry — most of it is already built; this is what is left** (re-scoped in the batch-191 sweep, which found this line far broader than the gap). `SceneMotion::{solved_at, posed_at, meshes_at, skinned_at}` and `SceneRigs::meshes_at` exist and phiber drives them (`v2/scene/pose/mod.rs:67`), so a **rigged** mesh re-poses per frame and the modifiers *after* its Armature re-run with it (`BlendSkinSource::evaluate`). Three things do not:
  1. **Driven modifier settings are fixed at the open frame.** The stacks in `base` and `tail` were built once, with `frame_current`'s driven values (batches 188–190). A Displace whose strength is driven therefore holds the open frame's strength all the way along the timeline.
  2. **The pre-armature prefix `base` is fixed at open.** Correct for the pose — nothing before the Armature depends on it — and wrong as soon as a driver changes a prefix modifier's setting.
  3. **A non-rigged object never re-evaluates.** `source.evaluate` is called only from `rig.rs:73`, so an object with no Armature gets a new matrix per frame and never new geometry.
  **(1) is done (batch 193).** `SceneRigs` carries the modifier driver sets, `SceneRigs::driven_at(motion, frame)` works out that frame's values by object address, and `meshes_at` hands each skin its own to `evaluate_driven`, which clones the tail before the settings land on it (batch 192, Blender's `copy_data`). Verified against `truth/mod_driver.txt` at all five frames — 75 values, and the witness is a Subdivision level walking 0 → 2 → 0 → 11 → 0 (`tests/driven_frames.rs`). An empty set costs one lookup and no clone.

  **Still open: (2) the prefix, and (3) non-rigged objects.** (3) is the bigger one and it is where the remaining work is: `source.evaluate*` is reached only from `rig.rs`, so an object with no Armature still has no per-frame path at all — `MODDRIVERS` is exactly such a file, which is why its driven values are now correct per frame and no mesh moves yet. Blender does not special-case rigs: the depsgraph gives any object with a driven input a relation to its geometry component and re-evaluates it. Matching that needs the non-rigged stacks retained past the open, the way `BlendSkinSource` retains the rigged ones.

## Done

- [x] Missing textures are reported as an answer, not a wall (`texture_miss.rs`): pose 26 asks for 304 images and 289 are on another machine, which filled the Info editor with 289 near-identical Blender lines and no way to act. The tally now leads with the count and what to do, ranks the folders by how many images each owes, and keeps six of Blender's own lines as evidence — a recorded divergence, since Blender prints all 289 (`texture_miss::tests`)
- [x] Find Missing Files (`texture_find.rs`): an image whose path is another machine's (`E:\…`, `//../../../…`) is looked for by file name under the `.blend`'s folder tree and its `textures/`, as `BKE_bpath_missing_files_find` does on command — automatically here, recorded as a divergence; `image_present` and `SceneTexture::candidates_in` read the same list so "present" and "found" agree (`texture_find::tests`)
- [x] A missing world HDRI is recorded, not reproduced (`BlendWorld::missing`): Blender lights the scene magenta; the world now carries the file name, a white tint, and the read report's warning, and phiber lights with Blender's studio `forest` instead (`tests/world_truth.rs`, pose 26's `autumn_field_puresky_4k.exr`)
- [x] Split `tests_traversal.rs` (275 lines) into `tests_traversal` (geometry, 123), `tests_material` (97) and a shared `tests_fixture` (61)
- [x] `BFaces` (`blend/mesh/list.rs`): every face's corners in one array with a start per face, instead of a `Vec` per face — HEXA-COMBINED opens with 580 MB less (2009 MB peak) and 2.5 s faster
- [x] `SceneModel::from_blend_owned`: each object's raw buffers are freed as it converts, so the raw file arrays and the runtime scene are not both held
- [x] `SceneMesh::corners`: shading normal per triangle corner (split normals, else smooth vertex normals on smooth faces; zero = flat) (`tests/corners.rs`)
- [x] `SceneCorners` (`corners.rs`): a 4-byte normal id per corner into a table of unique normals (was 12 bytes) — HEXA-COMBINED keeps ~500 MB less
- [x] `SceneMaps` / `SceneMap`: images linked to Roughness, Metallic and Emission Color, decoded like the base-colour image; matches Blender (`tests/maps_truth.rs`, `SHADE-MAPS.blend`)
- [x] Memory figure in the read report: `memory_bytes` on `SceneReadReport`, `mem_line()` formatter (KiB/MiB/GiB binary base), `scene_heap_bytes` in `phx.rs` sums mesh buffers
- [x] `SceneObject::{mesh, uvs, material_indices}` are `Arc`s: cloning the model (phiber does it every frame) no longer copies geometry
- [x] `SceneModel::from_blend` / `from_phx`; parallel triangulation + normals
- [x] Materials with viewport and base colour, decoded textures (packed, `//`-relative)
- [x] Saved UI (`SceneUi`: workspace, view, lens, shading), read report, collections, selection

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
