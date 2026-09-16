# Geometry Nodes Space

> **Blender references:**
> - Blender Manual: [`modeling/geometry_nodes/`](../../blender-manual/manual/modeling/geometry_nodes/) - Geometry Nodes
> - `DNA_space_types.h` - `SpaceNode` (geometry node tree)

---

## Space Spec

```rust
// store/spaces/geometry_nodes/def.rs

pub static GEOMETRY_NODES_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::GeometryNodes,
    label: "Geometry Nodes",
    areas: GEOMETRY_NODES_AREAS,
    columns: &[0.0, 0.55, 0.45],  // viewport + node editor
};

static GEOMETRY_NODES_AREAS: &[BArea] = &[
    BArea { zone: Top,    default_host: Topbar,
        hosts: &[Topbar], fraction: 1.0 },
    BArea { zone: Center, default_host: View3D,
        hosts: &[View3D], fraction: 1.0 },
    BArea { zone: Right,  default_host: GeoNode,
        hosts: &[GeoNode, ShaderEditor], fraction: 1.0 },
];
```

## View Tree

```
BSpaceType::GeometryNodes (viewport + node editor)
├─ Zone: Top
│  └─ BArea: { zone: Top, BHost: Topbar, fraction: 1.0 }
│       └─ Host: Topbar
│            └─ Region: Header → Pane: Center → Part: Menubar
│
├─ Zone: Center (55%)
│  └─ BArea: { zone: Center, default_host: View3D, fraction: 1.0 }
│       └─ Host: View3D
│            ├─ Region: Header
│            │    ├─ Pane: Left (Relative)   → Part: Modebar
│            │    └─ Pane: Center (Relative) → Part: Menubar
│            └─ Region: Center
│                 ├─ Pane: Left (AbsLeft, 44px)   → Part: Toolbar
│                 ├─ Pane: Center (Relative)       → Part: Custom (3D viewport)
│                 └─ Pane: Right (AbsRight, 300px) → Part: Sidebar + Panel
│
└─ Zone: Right (45%)
   └─ BArea: { zone: Right, default_host: GeoNode, fraction: 1.0 }
        └─ Host: GeoNode
             ├─ Region: Header
             │    └─ Pane: Center (Relative) → Part: Menubar (Node, Add, Select, View)
             └─ Region: Center
                  ├─ Pane: Left (AbsLeft, 240px) → Part: Sidebar (node categories)
                  └─ Pane: Center (Relative)      → Part: Custom (geometry node graph)
```

## Key Differences from Layout

- **Two columns** - viewport + geometry node editor
- **GeoNode** is the primary host in right column
- **View3D** shows the geometry node modifier result
- **ShaderEditor** switchable in right column
- **No left column** - no outliner
- **No bottom area** - no timeline
