# Animation Space

> **Blender references:**
> - Blender Manual: [`editors/dope_sheet/`](../../blender-manual/manual/editors/dope_sheet/) - Animation Editors
> - `DNA_space_types.h` - `SpaceAction`, `SpaceNla`, `SpaceIpo`

---

## Space Spec

```rust
// store/spaces/animation/def.rs

pub static ANIMATION_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::Animation,
    label: "Animation",
    areas: ANIMATION_AREAS,
    columns: &[0.18, 0.82],  // outliner + main area
};

static ANIMATION_AREAS: &[BArea] = &[
    BArea { zone: Top,    default_host: Topbar,
        hosts: &[Topbar], fraction: 1.0 },
    BArea { zone: Left,   default_host: Outliner,
        hosts: &[Outliner, Spreadsheet], fraction: 1.0 },
    BArea { zone: Center, default_host: View3D,
        hosts: &[View3D], fraction: 0.5 },
    BArea { zone: Bottom, default_host: DopeSheet,
        hosts: &[DopeSheet, GraphEditor, Nla, Timeline],
        fraction: 0.5 },
];
```

## View Tree

```
BSpaceType::Animation (outliner + viewport + animation editor)
├─ Zone: Top
│  └─ BArea: { zone: Top, BHost: Topbar, fraction: 1.0 }
│       └─ Host: Topbar
│            └─ Region: Header → Pane: Center → Part: Menubar
│
├─ Zone: Left (18%)
│  └─ BArea: { zone: Left, default_host: Outliner, fraction: 1.0 }
│       └─ Host: Outliner
│            ├─ Region: Header → Pane: Center → Part: Menubar
│            └─ Region: Center → Pane: Center → Part: Custom (outliner tree)
│
├─ Zone: Center (50% - top half)
│  └─ BArea: { zone: Center, default_host: View3D, fraction: 0.5 }
│       └─ Host: View3D
│            ├─ Region: Header
│            │    ├─ Pane: Left (Relative)   → Part: Modebar
│            │    └─ Pane: Center (Relative) → Part: Menubar
│            └─ Region: Center
│                 ├─ Pane: Left (AbsLeft, 44px)   → Part: Toolbar
│                 └─ Pane: Center (Relative)       → Part: Custom (3D viewport)
│
└─ Zone: Bottom (50% - bottom half)
   └─ BArea: { zone: Bottom, default_host: DopeSheet, fraction: 0.5 }
        └─ Host: DopeSheet
             ├─ Region: Header
             │    └─ Pane: Center (Relative) → Part: Menubar (Key, Select, Marker)
             └─ Region: Center
                  ├─ Pane: Left (AbsLeft, 200px) → Part: Channels
                  └─ Pane: Center (Relative)      → Part: Custom (dope sheet grid)
```

## Key Differences from Layout

- **Two columns** - outliner + combined viewport/animation area
- **Bottom area** is 50% - large animation editor (DopeSheet default)
- **DopeSheet, GraphEditor, Nla, Timeline** all switchable in bottom
- **Channels** part in the animation editor left pane
- **No right column** - no Properties panel
