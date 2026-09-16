# Geometry Nodes — Mock

Procedural geometry authoring: viewport on top, node graph on the bottom,
spreadsheet on the side.

## Composition

| Zone     | Slot | Default host   | Permitted hosts                        |
|----------|-----:|----------------|----------------------------------------|
| `Top`    |    1 | `Topbar`       |                                        |
| `Center` |    1 | `View3D`       |                                        |
| `Right`  |    1 | `Spreadsheet`  | `Spreadsheet`, `Buts`, `Outliner`      |
| `Bottom` |    1 | `GeoNode`      | `GeoNode`, `Node`, `Shader`, `Compositor` |
| `Bottom` |    2 | `Statusbar`    |                                        |

## Blender parity references

- `SpaceNodeEditor.tree_type = 'GeometryNodeTree'`.
- Requirement 11.5: adding a Geometry Nodes modifier auto-selects that tree.
