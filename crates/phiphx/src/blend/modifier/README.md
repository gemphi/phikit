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
| `subsurf.rs` | Subdivision Surface (`SubsurfModifierData`), evaluated like Blender's OpenSubdiv path for the viewport `levels`: Catmull-Clark (or Simple)… |
| `tests.rs` | Modifier evaluator unit tests (pure mesh math). |
| `topology.rs` | Flat mesh adjacency for the subdivision evaluator: edges from a sorted corner list (no hashing), per-corner edge ids and CSR vertex → edge /… |

Status and next steps: [`jobs.md`](jobs.md).
