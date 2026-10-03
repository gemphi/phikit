# Jobs — `phiphx/export`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 3 | 2 |

## Todo

- [ ] Tests for LAS / PCD / XYZ round trips
- [ ] Rename `export/blend` (it writes PHX / glTF / USD from a `BlenderScene`, not `.blend`)
- [ ] Split `export/blend/chunks.rs` (274 lines)

## Done

- [x] `PointCloudExporter`; PLY, PCD, LAS, XYZ, JSON, binary, glTF, USD, PHX writers
- [x] Loose `export_*` functions folded into their types (`PhxScene`, `PhxChunks`, `BlenderScene`, `PointCloud::from_phx`); duplicate glTF/USD exporters removed

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
