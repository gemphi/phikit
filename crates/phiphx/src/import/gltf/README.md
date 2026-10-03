# import/gltf

`.gltf` / `.glb` format importer

Converts glTF 2.0 scenes to point clouds and meshes.

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `full.rs` | Full glTF import that loads buffers and converts to a point cloud. |
| `importer.rs` | `GltfImporter` and the `Importer` trait implementation. |
| `mesh.rs` | Mesh and point-cloud extraction from glTF documents. |
| `scene.rs` | glTF scene wrapper with primary/active-subject selection. |
| `tests.rs` | Tests for the glTF importer. |
| `transform.rs` | Transform helpers: node transforms, TRS conversion, point transformation. |

Status and next steps: [`jobs.md`](jobs.md).
