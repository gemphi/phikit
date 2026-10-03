# export/usd

Native USD Exporter

Direct export to USDA/USDC format without external dependencies.
Optimized for PHI PHX workflow.

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `mesh.rs` | Mesh/scene → USDA exporter |
| `point_cloud.rs` | Point cloud → USDA exporters |
| `tests.rs` | tests |
| `writer.rs` | USDA text writer |

Status and next steps: [`jobs.md`](jobs.md).
