# Jobs — `phiphx/export`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 6 | 3 |

## Todo

- [ ] Tests for LAS / PCD / XYZ round trips
- [ ] Rename `export/blend` (it writes PHX / glTF / USD from a `BlenderScene`, not `.blend`)
- [ ] Split `export/blend/chunks.rs` (274 lines)

- [ ] `BlenderScene::to_gltf` still writes the first mesh as a point cloud; fold it into `SceneTarget::Gltf` as `to_usda` was
- [ ] glTF target: one combined world-space mesh, no materials; a node per object with its material is the next step. GLB is not a target yet
- [ ] PLY target writes points (with normals), not faces

## Done

- [x] `PointCloudExporter`; PLY, PCD, LAS, XYZ, JSON, binary, glTF, USD, PHX writers
- [x] Loose `export_*` functions folded into their types (`PhxScene`, `PhxChunks`, `BlenderScene`, `PointCloud::from_phx`); duplicate glTF/USD exporters removed

- [x] One convert path for every format (`target.rs`, `src/convert.rs`, `examples/convert.rs`): `Phx::convert(in, out)` opens anything `Phx::open` reads into a `SceneModel` and writes it by `SceneTarget` (USDA, PHX, glTF, PLY), picked from the extensions; USD to USD is copied as it is. `BlenderScene::to_usda` now goes through the same `SceneModel::to_usda`. The server runs `convert <upload> <out>.usda` on every uploaded file; where Blender is installed it may use Blender's own USD exporter instead — phiphx is the converter that runs everywhere (no Blender, no Python). Tests: `tests/usd_scene.rs`

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
