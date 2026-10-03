# cloud

Point clouds: the `PointCloud` type, statistics and layout adapters,
serde, and the mesh → cloud sampling configuration.

Callers enter through [`phx::Phx`](../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `config.rs` | Configuration for mesh to point cloud conversion. |
| `methods.rs` | Point cloud methods. |
| `mod.rs` | Point clouds: the `PointCloud` type, statistics and layout adapters, serde, and the mesh → cloud sampling configuration. |
| `serde_impl.rs` | Serialization for `PointCloud`. |
| `types.rs` | Point cloud data structures. |

Status and next steps: [`jobs.md`](jobs.md).
