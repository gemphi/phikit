# scene

`SceneModel`: the kit-agnostic runtime scene every view draws. Pure Rust, no
gpui, no OS deps. Consumers reach these types via `crate::scene::...`.

Blender-specific reading (`SceneModel::from_blend`, the progressive open, the
rigs, motion and collection tables) is in
[`device/blend/read/scene/model`](../device/blend/read/scene/model/README.md);
its public types are re-exported here, so `phiphx::scene::SceneRigList` and
`phiphx::scene::motion::SceneMotion` keep their paths.

Callers enter through [`phx::Phx`](../phx.rs); this folder is one of its parts.

## Files
| File | What it is |
|---|---|
| `mod.rs` | `SceneObject`, `SceneMesh`, `SceneNormalList`, `SceneModel`, child modules and re-exports. |
| `surface.rs` | Extended scene surfaces read by non-mesh spaces (timeline, world, render, action, NLA, drivers). |
| `extra.rs` | Cameras, lights and empties the overlay draws. |
| `corner.rs` | `SceneCornerList`: shading normals per triangle corner. |
| `error.rs` | `SceneLoadError` and its message clamp. |
| `geometry.rs` | Per-vertex UVs from per-corner UVs; normal selection and synthesis. |
| `material.rs` | Material slot data shared by the viewport and the Properties panel. |
| `polyfill.rs` | Face triangulation, following Blender's `mesh_calc_tessellation_for_face`. |
| `report.rs` | What Blender prints after a file read, and the status bar's figures. |
| `heap.rs` | What a loaded scene costs in memory. |
| `texture.rs`, `texture_find.rs`, `texture_miss.rs` | Base-colour and normal-map textures; Find Missing Files; the tally of what is missing. |
| `ui.rs` | "Load UI": the workspace and 3D view a `.blend` restores. |
| `uv.rs` | A mesh's active UV map as the UV editor draws it. |
| `test/` | Unit tests; modules reach them by `#[path]`. |

Status and next steps: [`jobs.md`](jobs.md).