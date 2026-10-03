# import

Point cloud importer.

Handles importing 3D formats (USD, glTF, OBJ) and converting to point clouds.

Callers enter through [`phx::Phx`](../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `asset.rs` | Unified asset importer — the **single entry point** for all format imports. |
| `format.rs` | Known 3D formats and basic format detection / parsing. |
| `gltf/` | `.gltf` / `.glb` format importer Converts glTF 2.0 scenes to point clouds and meshes. |
| `gltf.rs` | `.gltf` / `.glb` format importer |
| `phx.rs` | PHX format importer. |
| `ply.rs` | PLY format importer. |
| `project.rs` | Project Importer service — encapsulates asset parsing and Reality Composer Pro project scaffolding. |
| `usd/` | `.usd` / `.usda` / `.usdc` / `.usdz` format importer Converts Universal Scene Description files to point clouds and meshes. Handles USDA (ASCII),… |
| `usd.rs` | `.usd` / `.usda` / `.usdc` / `.usdz` format importer |

See `flow.md` for the import → `.phx` → GPU pipeline.

Status and next steps: [`jobs.md`](jobs.md).
