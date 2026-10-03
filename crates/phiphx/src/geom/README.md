# geom

Geometry primitives shared by every reader and exporter: vertices,
bounds, triangle meshes with surface sampling, the sampling BVH and the
primary-subject selector.

Callers enter through [`phx::Phx`](../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `bounds.rs` | Axis-aligned bounding box. |
| `bvh.rs` | BVH spatial acceleration for point cloud sampling. |
| `mesh.rs` | Triangle mesh representation and sampling methods. |
| `mod.rs` | Geometry primitives shared by every reader and exporter: vertices, bounds, triangle meshes with surface sampling, the sampling BVH and the… |
| `selector.rs` | Primary subject selector and source abstraction traits. |
| `vertex.rs` | Vertex types and their serialization. |

Status and next steps: [`jobs.md`](jobs.md).
