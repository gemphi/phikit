# scene

Kit-agnostic runtime scene data + the single blend→scene traversal.

Pure Rust — no gpui, no OS deps — mirroring the
`apple::services::outliner` pattern. Consumers reach these types via
`crate::scene::…`.

## File split (200-line file cap)

- `mod.rs` (this file) — core types (`SceneObject`, `SceneMesh`,
`SceneNormals`, `SceneModel`) + child module declarations + re-exports.
- `types_ext.rs` — extended scene surfaces (timeline / world / render /
camera / light / action) read by non-mesh spaces.
- `error.rs` — `SceneLoadError` + its message clamp.
- `import.rs` — blend → scene conversion helpers (fan-triangulation,
per-face normal synthesis, camera / light partitioning).
- `tests_traversal.rs` — traversal tests (opening lives in `crate::phx`).

Requirements: 6.1, 6.3, 7.2

Callers enter through [`phx::Phx`](../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `collection.rs` | Collection tree + view-layer state of a loaded scene — what Blender's Outliner "View Layer" mode draws (`tree_display_view_layer.cc`). |
| `error.rs` | Load-error surface — `SceneLoadError` + its message clamp. |
| `extras.rs` | Non-mesh objects the viewport overlay draws: cameras, lights, empties. |
| `geometry.rs` | Pure geometry helpers shared by the blend and phx mappers: per-vertex UVs from per-corner UVs, and normal selection / synthesis. |
| `import.rs` | Blend → scene conversion helpers. |
| `import_extras.rs` | Camera / light / empty objects → `SceneCamera` / `SceneLight` / `SceneEmpty`, from the DNA read by `importer::blend::object_data`. |
| `material.rs` | Material slot data shared by the viewport and the Properties panel (Principled inputs, normal map). |
| `object.rs` | One read object → renderable `SceneObject` (triangulation, normals, UVs, slots). |
| `mod.rs` | Kit-agnostic runtime scene data + the single blend→scene traversal. |
| `polyfill.rs` | Face triangulation, following Blender's `mesh_calc_tessellation_for_face`: triangles as-is, quads split (0,1,2)+(0,2,3), n-gons ear-clipped in 2D… |
| `report.rs` | What Blender prints after a file read (`file_read_reports_finalize`, wm_files.cc) and shows in the status bar (`info_stats.cc`): timings, scene… |
| `tests_traversal.rs` | `SceneModel::from_blend` traversal tests. |
| `tests_types.rs` | Basic scene-type invariants (Default, Clone, PartialEq). |
| `texture.rs` | Base-colour and normal-map textures: decode, cache, attach to materials (#26). |
| `types_ext.rs` | Extended scene surfaces read by non-mesh spaces (Animation / Rendering / Shading / Compositing / Info). |
| `ui.rs` | "Load UI" (#28): the workspace and 3D view a `.blend` restores, reduced to what the viewport consumes. |

Status and next steps: [`jobs.md`](jobs.md).
