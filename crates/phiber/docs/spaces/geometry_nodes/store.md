# Geometry Nodes Space - Store Structure

> **Blender references:**
> - `DNA_screen_types.h` - `bScreen`, `ScrArea`, `ARegion`
> - Blender Manual: `workspaces.rst` lines 110-111

---

## Store File Layout

```
crates/phiber/src/store/
  spaces/geometry_nodes/
    mod.rs                - GeometryNodesSpace spec: zones, BAreas, grid fractions
    def.rs                - zone→area→host assignments, pane/part defs
```

---

## Space Level: `store/spaces/geometry_nodes/mod.rs`

```rust
//! Geometry Nodes space spec - viewport + node editor.
//! Ref: Blender Manual - workspaces.rst lines 110-111
//! "Geometry Nodes: Has the 3D Viewport and Geometry Node Editor."

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;
use crate::store::areas::BArea;

pub static GEOMETRY_NODES_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::GeometryNodes,
    label: "Geometry Nodes",
    zones: &[Top, Center, Right],
    areas: &GEOMETRY_NODES_AREAS,
    columns: &[0.55, 0.45],
};

static GEOMETRY_NODES_AREAS: &[BArea] = &[
    BArea {
        zone: Top,
        default_host: Topbar,
        hosts: &[Topbar],
        fraction: 1.0,
    },
    BArea {
        zone: Center,
        default_host: View3D,
        hosts: &[View3D],
        fraction: 1.0,
    },
    BArea {
        zone: Right,
        default_host: GeoNode,
        hosts: &[GeoNode, ShaderEditor],
        fraction: 1.0,
    },
];
```

---

## Space Level: `store/spaces/geometry_nodes/def.rs`

```rust
//! Geometry Nodes space definitions.

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

pub static GEOMETRY_NODES_COLUMNS: &[f32] = &[0.55, 0.45];

pub static GEOMETRY_NODES_DEFAULT_HOSTS: &[(BZoneType, BHostType)] = &[
    (BZoneType::Top,    BHostType::Topbar),
    (BZoneType::Center, BHostType::View3D),
    (BZoneType::Right,  BHostType::GeoNode),
];

pub static GEOMETRY_NODES_SWITCHABLE: &[(BZoneType, &[BHostType])] = &[
    (BZoneType::Right, SWITCHABLE_RIGHT),
];

static SWITCHABLE_RIGHT: &[BHostType] = &[BHostType::GeoNode, BHostType::ShaderEditor];
```

---

## Host Specs

Geometry Nodes space uses View3D, GeoNode, and Topbar.
See `layout/store.md` for full host spec definitions.

- **View3D** - Header (Modebar + Menubar) + Center (Toolbar + Viewport + Sidebar/Panel)
- **GeoNode** - Header (Menubar: Node/Add/Select/View) + Center (Sidebar: node categories + Custom: geometry node graph)
- **Topbar** - Header (Menubar only)
