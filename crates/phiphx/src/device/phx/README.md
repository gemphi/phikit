# device/phx

The PHX device: the engine's own scene file. `PhxDevice` (in `mod.rs`)
implements `Device` for it: `detect` (`PHX_SCENE\x01` or `PHX\0`), `peek`,
`read` into a `SceneModel` (one object per mesh), `write` from a `SceneModel`.

Callers enter through [`phx::Phx`](../../phx.rs); this folder is one of its parts.

## Files
| File | What it is |
|---|---|
| `mod.rs` | `PhxDevice`. |
| `open.rs` | A PHX file as a `SceneModel` (`model`) and back (`to_phx`); `peek`. |
| `read.rs` | `PhxImporter`: a PHX scene as an `AssetImportResult` for the asset pipeline. |
| `sidecar.rs` | The `.phx` sidecar written beside a `.blend` (`BlendScene::write_sidecar`) or a USD file (`write_for_usd`). |
| `project.rs` | `ProjectScaffold`: a `.phx` project folder from any supported asset. |
| `cloud/` | The PHX point cloud: chunk format (`chunk.rs`, `PhxChunkList`), writer, reader. |
| `scene/` | The PHX scene: magic, JSON and geometry; `PhxScene` and its parts. |
| `test/` | Unit tests; modules reach them by `#[path]`. |

Status and next steps: [`jobs.md`](jobs.md).