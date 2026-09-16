# Sculpting — Mock

Sculpt-mode focused workspace with brush properties on the right.

## Composition

| Zone     | Slot | Default host | Notes                          |
|----------|-----:|--------------|--------------------------------|
| `Top`    |    1 | `Topbar`     |                                |
| `Center` |    1 | `View3D`     | Sculpt Mode default (Req 4.2)  |
| `Right`  |    1 | `Buts`       | `Buts`, `Outliner`             |
| `Bottom` |    1 | `Statusbar`  |                                |

## Blender parity references

- `bl_ui/space_view3d_toolbar.py` sculpt brush strip.
- Sidebar `Tool` tab surfaces by default per Requirement 4.4.
