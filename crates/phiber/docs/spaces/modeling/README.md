# Modeling Space

> **Blender references:**
> - Blender Manual: [`interface/window_system/workspaces.rst`](../../blender-manual/manual/interface/window_system/workspaces.rst) - Modeling workspace
> - `DNA_space_types.h` - `SpaceView3D`

---

## Space Spec

```rust
// store/spaces/modeling/def.rs

pub static MODELING_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::Modeling,
    label: "Modeling",
    areas: MODELING_AREAS,
    columns: &[0.0, 1.0, 0.0],  // single column - full viewport
};

static MODELING_AREAS: &[BArea] = &[
    BArea { zone: Top,    default_host: Topbar,
        hosts: &[Topbar], fraction: 1.0 },
    BArea { zone: Center, default_host: View3D,
        hosts: BHostType::switchable(), fraction: 1.0 },
    BArea { zone: Bottom, default_host: Timeline,
        hosts: &[Timeline, DopeSheet, GraphEditor], fraction: 0.28 },
];
```

## View Tree

```
BSpaceType::Modeling (single column)
├─ Zone: Top
│  └─ BArea: { zone: Top, BHost: Topbar, fraction: 1.0 }
│       └─ Host: Topbar
│            └─ Region: Header
│                 └─ Pane: Center (Relative) → Part: Menubar
│
├─ Zone: Center (flex-1)
│  └─ BArea: { zone: Center, default_host: View3D, fraction: 1.0 }
│       └─ Host: View3D
│            ├─ Region: Header
│            │    ├─ Pane: Left (Relative)   → Part: Modebar (Object, Edit, Sculpt, ...)
│            │    └─ Pane: Center (Relative) → Part: Menubar (View, Select, Add, Object)
│            └─ Region: Center
│                 ├─ Pane: Left (AbsLeft, 44px)   → Part: Toolbar
│                 ├─ Pane: Center (Relative)       → Part: Custom (3D viewport)
│                 └─ Pane: Right (AbsRight, 300px) → Part: Sidebar + Panel
│
└─ Zone: Bottom (28%)
   └─ BArea: { zone: Bottom, default_host: Timeline, fraction: 0.28 }
        └─ Host: Timeline
             ├─ Region: Header
             │    └─ Pane: Center (Relative) → Part: Menubar
             └─ Region: Center
                  └─ Pane: Center (Relative) → Part: Custom (timeline ruler)
```

## Key Differences from Layout

- **No left/right columns** - single full-width viewport
- **Bottom area** has timeline for animation playback
- **View3D** is the primary host (modeling focused)
- **Modebar** shows modeling modes: Object, Edit, Sculpt, Vertex Paint, Weight Paint
