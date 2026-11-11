# device/phx/cloud

Native PHX Exporter

Direct export to PHX binary format.
Optimized for PHI workflow.

Callers enter through [`phx::Phx`](../../../phx.rs); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `chunk.rs` | The chunk format: header, mesh chunk, `PhxChunkList`. |
| `mesh.rs` | PHX mesh chunk reader. |
| `read.rs` | PHX binary reader: import a point cloud from PHX data. |
| `../test/cloud.rs`, `../test/cloud_chunk.rs` | tests |
| `write.rs` | PHX point cloud chunk and binary writer. |

Status and next steps: [`jobs.md`](jobs.md).
