# v2/space

Space — workspace tab. Per-space content in sibling modules.

## Files

| File | What it is |
|---|---|
| `animation.rs` | Animation workspace — View3D center, Dope Sheet bottom, Outliner+Buts right. |
| `compositing.rs` | Compositing workspace — Compositor fills the whole area. |
| `geometry_nodes.rs` | Geometry Nodes workspace — View3D (top) + GeoNode/Spreadsheet (bottom). |
| `item.rs` | `BSpaceItem` — one space. |
| `layout.rs` | Layout space content — area declarations. |
| `list.rs` | `BSpaceList` — the app root. |
| `mod.rs` | Space — workspace tab. |
| `modeling.rs` | Modeling workspace — View3D fills the whole area. |
| `rendering.rs` | Rendering workspace — View3D (left) + Image Editor (right). |
| `scripting.rs` | Scripting workspace — Text editor, Console, Info, View3D, Outliner+Buts. |
| `sculpting.rs` | Sculpting workspace — View3D + Properties column. |
| `set.rs` | Space set — one `BSpaceSpec` per `BSpaceType`. |
| `shading.rs` | Shading workspace — Blender ref: startup.blend default `Shading` workspace. |
| `spec.rs` | `BSpaceSpec` — one space with its area set inline. |
| `texture_paint.rs` | Texture Paint workspace — Image (left) + View3D (right). |
| `type.rs` | `BSpaceType` — workspace tab tag. |
| `uv_editing.rs` | UV Editing workspace — Uv editor (left) + View3D (right). |

Status and next steps: [`jobs.md`](jobs.md).
