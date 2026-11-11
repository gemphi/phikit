# device/usd

The USD device: `.usd`, `.usda`, `.usdc`, `.usdz`. `UsdDevice` implements
[`Device`](../mod.rs); `Phx` reaches it through `DeviceList`.

The object model is usdkit's (brought in from `phikit/crates/apple/src/macos/usdkit`).

| Path | What it is |
|---|---|
| `stage.rs`, `prim.rs`, `attribute.rs`, `xform.rs`, `error.rs` | `UsdStage`, `UsdPrim`, `UsdValue`, `XformResolver`, `UsdaError` |
| `read/` | the usdkit USDA parser (`UsdaParser`), `scene.rs` (stage to `SceneModel`), `peek.rs` (header only) |
| `read/mesh/` | the heuristic readers `UsdImporter` (USDA lines, USDC strings, point clouds); the fallback for what the parser has no variant for, and for `.usdc` |
| `write/` | the usdkit printer (`UsdaPrinter`), the older text writers (`UsdaWriter`, `UsdExporter`), `script.rs` (`BlenderScene::to_usda`) |
| `write/scene/` | `UsdSceneWriter`: a `SceneModel` to a USD layer (was `export::usd::UsdStage`) |
| `package/` | the `.usdz` zip container (feature `usdz-import`) |
| `test/` | unit tests; integration tests are in `crates/phiphx/tests/` |

Old paths still work: `phiphx::export::usd::{UsdStage, UsdExporter, UsdaWriter, stage, mesh}` and
`phiphx::import::usd::{UsdImporter, UsdScene}` re-export from here.

## What is not done

- `write/scene/` writes text directly; it does not build a usdkit `UsdStage` and print it. The
  printer has no relationships, connections, metadata, `timeSamples` or instancing yet, which the
  scene writer needs.
- The parser reads a `matrix4d` (added for `xformOp:transform`), but not `timeSamples`, relationships
  or references; a file that uses them falls back to `read/mesh/`.
- `UsdImporter::import_file_as_mesh` (used by phiber) still runs the heuristic readers. The device's
  `read` goes through the parser first.
