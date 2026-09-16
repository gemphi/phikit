# Compositing — Mock

Full-screen compositor node editor.

## Composition

| Zone     | Slot | Default host  | Notes                                          |
|----------|-----:|---------------|------------------------------------------------|
| `Top`    |    1 | `Topbar`      |                                                |
| `Center` |    1 | `Compositor`  | `Use Nodes` toggle drives placeholder (Req 10.3) |
| `Bottom` |    1 | `Statusbar`   |                                                |

## Blender parity references

- `bl_ui/space_node.py` `NODE_HT_header` for Compositor.
- `SpaceNodeEditor.tree_type = 'CompositorNodeTree'`.
- Placeholder `"Use Nodes disabled"` per Requirement 10.3.
