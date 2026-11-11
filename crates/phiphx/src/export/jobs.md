# Jobs — `phiphx/export`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 3 | 2 |

## Todo

- [ ] Tests for LAS / PCD / XYZ round trips
- [x] `export/blend` moved to `device/blend/write/script/` (it writes PHX / glTF / USD from a `BlenderScene`, not `.blend`); `export/blend.rs` is a `pub use` shim
- [ ] `device/blend/write/script/chunk.rs` is 274 code lines: under 300, split if it grows

## Done

- [x] `PointCloudExporter`; PLY, PCD, LAS, XYZ, JSON, binary, glTF, USD, PHX writers
- [x] Loose `export_*` functions folded into their types (`PhxScene`, `PhxChunkList`, `BlenderScene`, `PointCloud::from_phx`); duplicate glTF/USD exporters removed

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
