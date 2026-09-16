# UV Editing — Mock

Side-by-side UV editor and 3D viewport for UV unwrapping.

## Composition

| Zone     | Slot | Default host | Permitted hosts    |
|----------|-----:|--------------|--------------------|
| `Top`    |    1 | `Topbar`     |                    |
| `Left`   |    1 | `Uv`         | `Uv`, `Image`      |
| `Center` |    1 | `View3D`     | Edit Mode, UV Sync |
| `Bottom` |    1 | `Statusbar`  |                    |

## Blender parity references

- `bl_ui/space_image.py` UV editor menus.
- `SpaceImageEditor.mode = 'UV'` sets the left-panel behaviour.
- `"No active UV Map"` placeholder per Requirement 5.5.
