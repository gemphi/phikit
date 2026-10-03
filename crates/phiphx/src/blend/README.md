# blend

The native `.blend` reader. Parses Blender's SDNA container and extracts
the scene Blender would show, without Blender installed. Entered through
`crate::phx::Phx`; `BlendImporter` serves the asset pipeline.

Callers enter through [`phx::Phx`](../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `asset/` | Asset pipeline. `AssetImportResult` from a blend, `.phx` sidecar export. |
| `file/` | The `.blend` container. Header, block headers, block table and pointer lookup (`scope`), DNA and struct layout, quick peek, names. |
| `mesh/` | Mesh geometry. Positions, faces, UVs, normals, face attributes, CustomData (≤ 4.x) and attribute storage (5.x), vertex groups. |
| `mod.rs` | The native `.blend` reader. |
| `modifier/` | The viewport modifier stack (Blender `mesh_calc_modifiers`), limited to modifiers that change how an existing mesh looks. Declarative: [`KINDS`]… |
| `object/` | Objects. DNA fields, transforms and parent chain, rotation modes, kind, camera/light/empty data, collections, view layer, world. |
| `scene/` | Scene assembly. `extract_scene`, scene properties, result types, saved UI (workspace, 3D view). |
| `write/` | Writing the file back: `to_bytes` / `write_to`, field patching by DNA, the view layer's blocks, the marker list, `BlendEdits`. |
| `shade/` | Materials. Slots, viewport colour and Principled inputs, shader node tree, images (keys and packed bytes). |
| `tests/` | Tests for this folder. |

## Ground truth

`tests/blender_truth.rs` compares per-mesh digests with `tests/truth/mesh_digest.txt` (made by Blender 5.0.1); `crates/phiber/tests/viewport_vs_blender.rs` compares whole saved views with Blender renders in `tests/truth/views/`.

Status and next steps: [`jobs.md`](jobs.md).
