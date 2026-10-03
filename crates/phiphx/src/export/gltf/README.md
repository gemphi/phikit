# export/gltf

Native glTF 2.0 Exporter

Direct export to glTF format without external dependencies.
Optimized for PHI PHX workflow.

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `base64.rs` | Minimal base64 encode/decode helpers used for embedding buffer data. |
| `buffers.rs` | glTF buffer/accessor/bufferView assembly. |
| `mesh.rs` | Triangle mesh glTF export. |
| `point_cloud.rs` | Point cloud glTF/GLB export. |
| `tests.rs` | tests |

Status and next steps: [`jobs.md`](jobs.md).
