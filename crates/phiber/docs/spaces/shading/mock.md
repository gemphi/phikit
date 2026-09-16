# Shading — Mock

Multi-panel workspace for shader authoring: node editor at bottom, image
editor on the left, viewport in the middle, outliner + properties on the
right.

## Composition (7 area slots)

| Zone     | Slot | Default host | Permitted hosts                   |
|----------|-----:|--------------|-----------------------------------|
| `Top`    |    1 | `Topbar`     |                                   |
| `Left`   |    1 | `Image`      | `Image`, `File`, `Asset`          |
| `Center` |    1 | `View3D`     | Material Preview / Rendered       |
| `Right`  |    1 | `Outliner`   |                                   |
| `Right`  |    2 | `Buts`       |                                   |
| `Bottom` |    1 | `Shader`     | `Shader`, `Node`, `Compositor`, `GeoNode` |
| `Bottom` |    2 | `Statusbar`  |                                   |

## Blender parity references

- `bl_ui/space_node.py` Shader tree UI.
- Requirement 7.4: viewport enters Material Preview when shading is active.
- Requirement 7.5: `Shader.selected` drives the sidebar props re-render.
