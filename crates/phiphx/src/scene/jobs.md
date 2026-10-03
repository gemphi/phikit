# Jobs — `phiphx/scene`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 3 | 3 |

## Todo

- [ ] Animation state: frame → object / bone matrices → re-evaluated meshes (per-frame cache)
- [ ] Memory figure in the read report (status bar Memory segment)
- [ ] Split `tests_traversal.rs` (217 lines) into two files

## Done

- [x] `BFaces` (`blend/mesh/list.rs`): every face's corners in one array with a start per face, instead of a `Vec` per face — HEXA-COMBINED opens with 580 MB less (2009 MB peak) and 2.5 s faster
- [x] `SceneModel::from_blend_owned`: each object's raw buffers are freed as it converts, so the raw file arrays and the runtime scene are not both held
- [x] `SceneMesh::corners`: shading normal per triangle corner (split normals, else smooth vertex normals on smooth faces; zero = flat) (`tests/corners.rs`)
- [x] `SceneCorners` (`corners.rs`): a 4-byte normal id per corner into a table of unique normals (was 12 bytes) — HEXA-COMBINED keeps ~500 MB less
- [x] `SceneMaps` / `SceneMap`: images linked to Roughness, Metallic and Emission Color, decoded like the base-colour image; matches Blender (`tests/maps_truth.rs`, `SHADE-MAPS.blend`)
- [x] `SceneObject::{mesh, uvs, material_indices}` are `Arc`s: cloning the model (phiber does it every frame) no longer copies geometry
- [x] `SceneModel::from_blend` / `from_phx`; parallel triangulation + normals
- [x] Materials with viewport and base colour, decoded textures (packed, `//`-relative)
- [x] Saved UI (`SceneUi`: workspace, view, lens, shading), read report, collections, selection

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
