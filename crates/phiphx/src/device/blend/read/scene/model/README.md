# blend/read/scene/model

A `SceneModel` from Blender data. These files used to sit in `scene/`; they
import the Blend device, so they live in it. `phiphx::scene::*` still names
their public types.

Callers enter through [`phx::Phx`](../../../../../phx.rs); this folder is one of its parts.

## Files
| File | What it is |
|---|---|
| `import.rs` | `SceneModel::from_blend` / `from_blend_owned`: a read `BlendScene` into a scene. |
| `import_extra.rs` | Camera, light and empty objects into `SceneCamera`, `SceneLight`, `SceneEmpty`. |
| `object.rs` | One `BlendSceneObject` into a `SceneObject` (triangulation, normals, UVs, material slots). |
| `share.rs` | Instances share their member's mesh by `Arc`. |
| `progressive.rs` | The progressive open: `DeferredModifierList`, `SceneSwap`. |
| `progressive_link.rs` | Vertex-parented children while the modifiers run. |
| `build.rs` | A scene built from code (`add_object`, `adopt_rig`, `add_skinned`). |
| `rig.rs` | Meshes their rigs deform, at any frame (`SceneRigList`, `SceneSkin`, `SceneDriven`). |
| `motion.rs` | Every object's transform over time (`SceneMotion`). |
| `collection.rs` | Collection tree and view-layer state (`SceneCollection`, `SceneSelection`). |
| `relation.rs` | The viewport's relationship lines (`SceneRelation`). |

Tests: `device/blend/test/read/scene/model/`.

Status and next steps: [`jobs.md`](jobs.md).