# geom

Geometry primitives shared by every reader and exporter: vertices,
bounds, triangle meshes with surface sampling, the sampling BVH, the
primary-subject selector, and the ordinary shapes every asset library
starts with (`shape`).

Callers enter through [`phx::Phx`](../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `bounds.rs` | Axis-aligned bounding box. |
| `bvh.rs` | BVH spatial acceleration for point cloud sampling. |
| `mesh.rs` | Triangle mesh representation and sampling methods. |
| `mod.rs` | Geometry primitives shared by every reader and exporter: vertices, bounds, triangle meshes with surface sampling, the sampling BVH and the… |
| `selector.rs` | Primary subject selector and source abstraction traits. |
| `shape.rs` | `Shape` — one row of `data/shape.tsv` (cube, dice, sphere, cylinder, cone, torus, plane, capsule, pyramid, tetrahedron, octahedron, icosahedron); `Shape::part()` builds its parts. `ShapeForm` is how a row is made; names, sizes and colours are only in the table. Written as USDA by `export/usd/shape.rs` (`Shape::to_usda`). |
| `shape/part.rs` | `ShapePart` (name, colour, finish, polygons over points, a normal per corner) and `ShapeMesh`, the builder that welds points by position. |
| `shape/lathe.rs` | Profiles swept round the up axis: sphere, cylinder, cone, capsule, torus (and the dice's pip discs). |
| `shape/solid.rs` | Flat-faced shapes: the box with rounded edges, the plane, and convex solids as the hull of their corners (pyramid, regular solids). |
| `shape/pip.rs` | The dice's pips from `data/pip.tsv`: which face shows which value (opposite faces sum to 7) and where its pips sit. |
| `shape/tests.rs` | Table names and parts, closed solids, outward normals, 21 pips, USDA read-back, and the Phiogy shape files equal to what the generator writes. |

Regenerate the shape files Phiogy reads:
`cargo run -p phiphx --example shape_usd -- crates/appkit/app/phiogy/asset/shape`
| `vertex.rs` | Vertex types and their serialization. |

Status and next steps: [`jobs.md`](jobs.md).
