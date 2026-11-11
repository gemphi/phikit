# blend/object

Objects. DNA fields, transforms and parent chain, rotation modes, kind, camera/light/empty data, collections, view layer, world.

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `light_driver.rs` | Drivers on a light — `Light.adt`, writing the lamp's own properties, each with its own RNA hard range (they are not uniform: `energy` is unbounded, `spot_size`'s minimum is 1°). |
| `light_driver_read.rs` | Reading lights' and the world's drivers out of the file. The DNA struct for a light is `Lamp`, in 5.0.1 too. |
| `collection.rs` | `Scene.master_collection` → `Collection.children` / `gobject` walk. |
| `data.rs` | Non-mesh object data Blender's overlay engine draws: `Camera` (`DNA_camera_types.h`), `Light` (`DNA_light_types.h`) and the empty display fields… |
| `dna.rs` | Legacy full-DNA object extraction (kept for the asset pipeline). |
| `kind.rs` | `Object.type` → Blender type name (`OB_*`, `DNA_object_types.h`, 5.2). |
| `layer.rs` | View layer extraction — what Blender's Outliner "View Layer" mode shows. |
| `meta.rs` | Object metadata extraction: modifiers and instance info. |
| `mod.rs` | Objects. |
| `read.rs` | Object block extraction (assembling `BlendSceneObject`s). |
| `rotation.rs` | Rotation / matrix math — ports of `eulO_to_mat3`, `quat_to_mat3`, `axis_angle_to_mat3` (`blenlib/intern/math_rotation.cc`) and… |
| `rotation_tests.rs` | Tests for the `BKE_object_to_mat4` port. |
| `transform.rs` | Object transform extraction — a port of Blender's `BKE_object_to_mat4` (`blenkernel/intern/object.cc`) and `eulO_to_mat3` / `quat_to_mat3` /… |
| `world.rs` | World-matrix resolution and view-layer flag application — the parts of `BKE_object_where_is_calc` / `BKE_layer_collection_sync` Phiber needs. |

Status and next steps: [`jobs.md`](jobs.md).
