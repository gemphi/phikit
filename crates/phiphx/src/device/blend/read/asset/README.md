# blend/asset

Asset pipeline. `AssetImportResult` from a blend, `.phx` sidecar export.

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `import.rs` | Asset-pipeline entry (`AssetImporter` → `BlendImporter::import`): reads the file, writes the `.phx` sidecar, and flattens the scene into… |
| `mod.rs` | Asset pipeline. |
| `phx.rs` | Emit a portable `.phx` scene alongside every imported `.blend`. |

Status and next steps: [`jobs.md`](jobs.md).
