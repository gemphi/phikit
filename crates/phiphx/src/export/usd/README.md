# export/usd

Native USD Exporter

Direct export to USDA/USDC format without external dependencies: single
meshes, point clouds, the shapes of `data/shape.tsv`, and whole scenes.
Every layer is what RealityKit / Reality Composer Pro and `phiusd` open:
`defaultPrim`, `metersPerUnit = 1`, `upAxis = "Y"`, meshes with
`faceVarying` normals, a constant `displayColor`, and a `UsdPreviewSurface`
material bound to each (RealityKit ignores `displayColor`).

A scene goes out through `SceneTarget` (`export/target.rs`), the write
side's one `match`; USD is one arm. Nothing here knows what the scene was
read from.

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `mesh.rs` | One `Mesh` → USDA (`UsdExporter::export_mesh_usda`) |
| `name.rs` | `UsdNameSet` (names made USD identifiers, unique among siblings) and `usd_word` (the writer's own prim names, `data/usd.tsv`) |
| `point_cloud.rs` | Point cloud → USDA exporters |
| `scene.rs` | `SceneModel::to_usda` → `UsdaScene`: every shown object an Xform at its world transform (Z up turned to Y up) holding its mesh; materials once each under the materials scope; per-face slots as `GeomSubset`s; `SceneModel::shows` |
| `shape.rs` | `Shape::to_usda`: a shape's Xform (the default prim), a Mesh per part, a material per part |
| `stage.rs` | `UsdaWriter::mesh` (`UsdMesh`), `UsdaWriter::look` (a `PreviewSurfaceMaterial` as a `UsdPreviewSurface`), `UsdaWriter::transform` |
| `tests.rs` | tests |
| `writer.rs` | USDA text writer (`UsdaWriter::stage` opens a layer on its default prim) |

Status and next steps: [`jobs.md`](jobs.md).
