# Texture Paint — Mock

Image editor beside a 3D viewport in Texture Paint mode.

## Composition

| Zone     | Slot | Default host | Permitted hosts    |
|----------|-----:|--------------|--------------------|
| `Top`    |    1 | `Topbar`     |                    |
| `Left`   |    1 | `Image`      | `Image`, `Uv`      |
| `Center` |    1 | `View3D`     | Texture Paint mode |
| `Right`  |    1 | `Buts`       | `Buts` only        |
| `Bottom` |    1 | `Statusbar`  |                    |

## Blender parity references

- `bl_ui/space_view3d_toolbar.py` texture paint brush palette.
- Placeholder `"No Image"` per Requirement 6.5.
