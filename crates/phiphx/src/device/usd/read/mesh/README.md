# import/usd

`.usd` / `.usda` / `.usdc` / `.usdz` format importer

Converts Universal Scene Description files to point clouds and meshes.
Handles USDA (ASCII), USDC (binary crate), and USDZ (zip container).

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `importer.rs` | `UsdImporter` definition and public entry points. |
| `parse.rs` | Low-level USDA array parsing helpers. |
| `scene.rs` | USD scene wrapper with primary/active-subject selection. |
| `tests.rs` | USD importer smoke tests. |
| `usda.rs` | USDA (ASCII) → point cloud parser. |
| `usda_build.rs` | Mesh/cloud finalisation shared by the USDA parsers. |
| `usda_mesh.rs` | USDA (ASCII) → full mesh parser (preserves triangles and transforms). |
| `usda_parse.rs` | Shared per-line state-machine helpers used by the USDA parsers. |
| `usdc.rs` | USDC (binary crate) heuristic extraction. |
| `usdz.rs` | USDZ (zip container) handling. |

Status and next steps: [`jobs.md`](jobs.md).
