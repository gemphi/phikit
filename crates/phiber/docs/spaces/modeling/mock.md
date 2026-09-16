# Modeling — Mock

Maximised-viewport workspace focused on geometry editing.

## Composition

| Zone     | Slot | Default host | Notes                                  |
|----------|-----:|--------------|----------------------------------------|
| `Top`    |    1 | `Topbar`     | Shared chrome                          |
| `Center` |    1 | `View3D`     | Enters Edit Mode by default (Req 3.3)  |
| `Bottom` |    1 | `Statusbar`  |                                        |

## Blender parity references

- `startup.blend` declares Modeling with `Edit Mode` default.
- `bl_ui/space_view3d.py` VIEW3D_MT_edit_mesh menu tree.
