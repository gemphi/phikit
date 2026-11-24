# geom

Geometry primitives shared by every reader and exporter: vertices,
bounds, triangle meshes with surface sampling, the sampling BVH, the
primary-subject selector, the ordinary shapes every asset library
starts with (`shape`), and things built of parts as a collection's
tables say (`thing`).

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
| `shape/lathe.rs` | Profiles swept round the up axis: sphere, cylinder, cone, capsule, torus, dome (and the dice's pip discs). |
| `shape/solid.rs` | Flat-faced shapes: the box with rounded edges (one size each way, `rounded_block`), the plane, and convex solids as the hull of their corners (pyramid, regular solids). |
| `shape/pip.rs` | The dice's pips from `data/pip.tsv`: which face shows which value (opposite faces sum to 7) and where its pips sit. |
| `shape/tests.rs` | Table names and parts, closed solids, outward normals, 21 pips, USDA read-back, and the Phiogy shape files equal to what the generator writes. |

Regenerate the shape files Phiogy reads:
`cargo run -p phiphx --example shape_usd -- crates/appkit/app/phiogy/asset/shape`
| `thing.rs` | Things built of parts from a collection's tables (`thing.tsv`, `look.tsv`, `source.tsv` in one folder; steps per form in `data/thing.tsv`): each row a box, ball, pill, can, cone, ring, dome, a letter of a font, an outline of an svg drawing, or another thing, placed, turned, rounded, worn in a look, repeated. A thing inside keeps its layer; swapped looks pass down. Written as USDA by `export/usd/thing.rs`: a thing that wears its own looks is a reference to its own file. |
| `thing/read.rs` | The tables read: `ThingSet`, `Thing`, `ThingRow`, `ThingForm`, `ThingLook`. |
| `thing/build.rs` | `ThingSet::build`: `ThingTree` of `ThingNode` meshes and groups; repeats, turns, swaps; when a group may refer to its file. |
| `thing/form.rs` | A row's own mesh, its form at its size. |
| `thing/outline.rs` | Letters and svg drawings raised to solids through `blend::text`'s display list (the path a `.blend` text takes), fitted to the size. |
| `thing/svg.rs` | An svg file's paths as rings (moves, lines, arcs). |
| `thing/tests.rs`, `thing/svg_tests.rs` | Once City's tables (Phiogy's `data/once`): looks known, every thing built, sixteen couches and trees, letters fitted with their counters open, the bird from Phiogy's mark, references and swaps, repeats, and Phiogy's files equal to what the example writes; the svg reader. |

Regenerate Once City's files:
`cargo run -p phiphx --example thing_usd -- crates/appkit/app/phiogy/data/once crates/appkit/app/phiogy/asset/once`
| `vertex.rs` | Vertex types and their serialization. |

Status and next steps: [`jobs.md`](jobs.md).
