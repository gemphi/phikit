# v2/scene

Default scene (Cube, Sphere, Gem) + the Scene_Source that both the
Viewport and the Outliner iterate. Pure data + pure math — no gpui, no
theme widgets — so it can be unit-tested in isolation.

View orientation, zoom, pitch, pan, and every scene-related state key
live in `keys.rs`. Projection + viewport-fit math lives in `project.rs`.
Tests are split across three focused files so all files stay under the
200-line cap.

## Files

| File | What it is |
|---|---|
| `camera.rs` | The viewport camera (#44) — Blender's `RegionView3D` orbit model. |
| `color.rs` | Per-triangle colour of a scene object — material slot resolution and the Solid (viewport colour) vs Material Preview (base colour) choice. |
| `extras/` | Viewport overlay "extras" — the wire shapes Blender's overlay engine draws for cameras, lights and empties (`overlay_extra.cc`,… |
| `keys.rs` | View-state keys + accessors. |
| `keys_tests.rs` | Focused tests for the import-progress staging vocabulary. |
| `mod.rs` | Default scene (Cube, Sphere, Gem) + the Scene_Source that both the Viewport and the Outliner iterate. |
| `project.rs` | 2D + 3D projection functions and the viewport-fit AABB helpers. |
| `shading.rs` | Viewport shading mode — Blender's `View3DShading.type` (`V3D_SHADING_WIREFRAME / SOLID / MATERIAL / RENDERED`, `DNA_view3d_enums.h`). |
| `source.rs` | Scoped Scene_Source context — threads the cached loaded [`SceneModel`] through a single render pass to BOTH the Viewport and the Outliner without… |
| `tests_project.rs` | Projection + viewport-fit tests. |
| `tests_scene.rs` | `current_scene` + `BObject3D` tests. |
| `tests_state.rs` | State-key + view-accessor tests. |

Status and next steps: [`jobs.md`](jobs.md).
