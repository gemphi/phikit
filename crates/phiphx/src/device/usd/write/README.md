# export/usd
Native USD Exporter
Direct export to USDA/USDC format without external dependencies.
Optimized for PHI PHX workflow.
Callers enter through [`phx::Phx`](../../../phx.rs); this folder is one of its parts.

**Scenes** go through one writer: [`UsdStage`](scene/) turns a `SceneModel` into a
USD 1.0 text layer (`SceneModel::to_usda`, and `BlenderScene::to_usda` by way of it).
Z-up, `metersPerUnit = 1` (Blender's convention), `defaultPrim = "World"`.
Meshes, point clouds alone use the older writers below.

| Scene part | USD |
|---|---|
| parent chain | nested `Xform`, each with its local `xformOp:transform` |
| collections | `Scope`, `active = false` when excluded |
| mesh | `Mesh` of triangles, face-varying `normals`, `primvars:st`, `displayColor`, `subdivisionScheme = "none"` |
| material slots | `rel material:binding`, a `GeomSubset` per slot when faces use several |
| shared meshes | `/World/Prototype/<n>`, users are `instanceable` references |
| camera | `Camera`: focal length, apertures from sensor fit, clipping, ortho, shift, focus distance / fStop |
| lights | SphereLight (point, spot + ShapingAPI), DistantLight, RectLight, DiskLight; units in `stage/light.rs` |
| world | `DomeLight` (colour x strength, or the HDRI path) |
| materials | `UsdPreviewSurface`, images via `UsdUVTexture` + `UsdPrimvarReader_float2` |

Names are escaped to `[A-Za-z0-9_]` (`UsdName`), made unique per parent, and the Blender
spelling is kept as `displayName`.

## Files
| File | What it is |
|---|---|
| `stage.rs` | `UsdStage`: the scene writer (layer header, walk order) |
| `stage/camera.rs` | `SceneCamera` to `UsdGeomCamera` |
| `stage/check.rs` | `UsdCheck`: structural self check of a written layer |
| `stage/head.rs` | name, transform and visibility every placed prim opens with |
| `stage/light.rs` | `SceneLight` to UsdLux, and the watts law |
| `stage/material.rs` | `SceneMaterial` to `UsdPreviewSurface` |
| `stage/mesh.rs` | `SceneMesh` to `UsdGeomMesh` with bindings |
| `stage/name.rs` | Blender name to USD identifier |
| `stage/node.rs` | meshes, cameras, lights and empties as one parented list |
| `stage/out.rs` | indented line buffer with block depth |
| `stage/proto.rs` | shared meshes as prototypes |
| `stage/tests.rs` | hand-built scenes; fixtures are in `tests/usd_stage_truth.rs` |
| `stage/text.rs` | numbers, strings, arrays, matrices as USDA text |
| `stage/texture.rs` | image nodes of a material |
| `stage/tree.rs` | collections and objects as the Xform tree |
| `stage/world.rs` | `SceneWorld` to a `DomeLight` |
| `mesh.rs` | Mesh/scene -> USDA exporter (the older single-mesh path) |
| `point_cloud.rs` | Point cloud -> USDA exporters |
| `tests.rs` | tests |
| `writer.rs` | USDA text writer (Y-up, for `mesh.rs` and `point_cloud.rs`) |
Status and next steps: [`jobs.md`](jobs.md).