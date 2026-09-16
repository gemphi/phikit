# Scripting — Mock

Text editor + Python console + Info log + 3D viewport for Blender-style
Python development.

## Composition

| Zone     | Slot | Default host | Permitted hosts               |
|----------|-----:|--------------|-------------------------------|
| `Top`    |    1 | `Topbar`     |                               |
| `Left`   |    1 | `Text`       | `Text`, `Console`, `Info`     |
| `Center` |    1 | `View3D`     |                               |
| `Right`  |    1 | `Buts`       |                               |
| `Bottom` |    1 | `Console`    | `Console`, `Info`             |

## Blender parity references

- `bl_ui/space_text.py` text editor menus.
- `bl_ui/space_console.py` REPL.
- `bl_ui/space_info.py` log rows.
