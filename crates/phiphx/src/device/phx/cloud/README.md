# export/phx

Native PHX Exporter

Direct export to PHX binary format.
Optimized for PHI workflow.

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `mesh.rs` | PHX mesh chunk reader. |
| `read.rs` | PHX binary reader: import a point cloud from PHX data. |
| `tests.rs` | tests |
| `write.rs` | PHX point cloud chunk and binary writer. |

Status and next steps: [`jobs.md`](jobs.md).
