# blend/modifier

The viewport modifier stack (Blender `mesh_calc_modifiers`), limited to
modifiers that change how an existing mesh looks.

Declarative: [`KINDS`] maps a DNA `ModifierData.type` to a reader that
turns the saved settings into a [`BModifier`]. The stack runs in file
order on an [`BlendEvalMesh`]; unknown or disabled modifiers are skipped
(and listed in the read report by the caller).

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `driver.rs` | Drivers on a modifier's settings: the per-type setting table, each property's RNA range, and the integer rule — a driver's float is taken by truncation toward zero, not a round and not a floor. |
| `driver_read.rs` | Reading them off the object's `AnimData` (`modifiers["Name"].property`) and resolving each modifier's type from its own stack, because the path does not carry it. |
| `armature.rs` | Armature deform (#16, first modifier) — Blender `armature_deform.cc`. |
| `mirror.rs` | Mirror modifier (`MirrorModifierData`, Blender `mesh_mirror.cc`): for each enabled axis (X, Y, Z in that order) append a reflected copy — about… |
| `mod.rs` | The viewport modifier stack (Blender `mesh_calc_modifiers`), limited to modifiers that change how an existing mesh looks. |
| `nodes.rs` | Geometry Nodes modifier (#57, `MOD_nodes.cc`) for the bundled "Smooth by Angle" group: the Set Shade Smooth chain from Group Input to Group Output, read with the shader tree reader. |
| `nodes_field.rs` | Its field evaluator: Edge Angle, Is Edge / Shade Smooth, Boolean Math, Compare, per edge or face with Blender's domain adaptation (`mesh_attributes.cc`). |
| `nodes_inputs.rs` | The modifier's input values (pre-5.1 `settings.properties`, 5.1+ `system_properties`), the interface defaults, and the mesh's `sharp_edge` attribute. |
| `nodes_legacy.rs` | The "Auto Smooth" modifier Blender INVENTS at load for a pre-4.2 file whose mesh carries the legacy auto-smooth flag (`BKE_main_mesh_legacy_convert_auto_smooth`) — how the user's files get this shading. |
| `nodes_topo.rs` | The edge domain rebuilt from an evaluated mesh's faces and loose edges, with each edge's faces (`create_edge_map`). |
| `nodes_ops.rs` | The Boolean Math and Compare operation tables, and the angle between two face normals (`angle_normalized_v3v3`). |
| `nodes_corners.rs` | Corner normals that split on sharp edges, faces and non-manifold edges — Blender's `normals_calc_corners`, which the evaluated mesh is drawn with. |
| `nodes_fans.rs` | The fan walk itself: which edges a fan may cross and the angle-weighted normal every corner of one takes. |
| `subsurf.rs` | Subdivision Surface (`SubsurfModifierData`), evaluated like Blender's OpenSubdiv path for the viewport `levels`: Catmull-Clark (or Simple)… |
| `tests.rs` | Modifier evaluator unit tests (pure mesh math). |
| `topology.rs` | Flat mesh adjacency for the subdivision evaluator: edges from a sorted corner list (no hashing), per-corner edge ids and CSR vertex → edge /… |

Status and next steps: [`jobs.md`](jobs.md).
