# Jobs — `phiphx/export/blend`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 1 | 1 |

## Todo

- [ ] See `export` jobs (rename, split `chunks.rs`)

## Done

- [x] `BlenderScene::to_phx` / `to_gltf` / `to_usda`, `PhxChunks`, type parsing via `From<&str>`
- [x] `to_usda` folded into the one scene path: `BlenderScene` → `SceneModel` → `SceneModel::to_usda` (was the first mesh as a point cloud). Test: `export::blend::tests::to_usda_goes_through_the_scene_path`

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
