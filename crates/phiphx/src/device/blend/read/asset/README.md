# blend/asset

Asset pipeline. `AssetImportResult` from a blend. The `.phx` sidecar it writes is in `device/phx/sidecar.rs`.

Callers enter through [`phx::Phx`](../../../../phx.rs); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `import.rs` | Asset-pipeline entry (`AssetImporter` → `BlendImporter::import`): reads the file, writes the `.phx` sidecar, and flattens the scene into… |
| `mod.rs` | Asset pipeline. |

Status and next steps: [`jobs.md`](jobs.md).
