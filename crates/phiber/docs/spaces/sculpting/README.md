# Sculpting Space

> **Blender references:**
> - Blender Manual: [`sculpt_sculpting/`](../../blender-manual/manual/sculpt_sculpting/) - Sculpt Mode
> - `DNA_space_types.h` - `SpaceView3D` with `View3D.shading.type`

---

## Space Spec

```rust
// store/spaces/sculpting/def.rs

pub static SCULPTING_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::Sculpting,
    label: "Sculpting",
    areas: SCULPTING_AREAS,
    columns: &[0.0, 1.0, 0.22],  // viewport + right panel
};

static SCULPTING_AREAS: &[BArea] = &[
    BArea { zone: Top,    default_host: Topbar,
        hosts: &[Topbar], fraction: 1.0 },
    BArea { zone: Center, default_host: View3D,
        hosts: &[View3D, ImageEditor], fraction: 1.0 },
    BArea { zone: Right,  default_host: Properties,
        hosts: &[Properties, AssetBrowser], fraction: 1.0 },
];
```

## View Tree

```
BSpaceType::Sculpting (viewport + right panel)
├─ Zone: Top
│  └─ BArea: { zone: Top, BHost: Topbar, fraction: 1.0 }
│       └─ Host: Topbar
│            └─ Region: Header → Pane: Center → Part: Menubar
│
├─ Zone: Center (flex-1)
│  └─ BArea: { zone: Center, default_host: View3D, fraction: 1.0 }
│       └─ Host: View3D (Sculpt mode)
│            ├─ Region: Header
│            │    ├─ Pane: Left (Relative)   → Part: Modebar (Sculpt, Paint)
│            │    └─ Pane: Center (Relative) → Part: Menubar
│            └─ Region: Center
│                 ├─ Pane: Left (AbsLeft, 44px)   → Part: Toolbar (sculpt brushes)
│                 ├─ Pane: Center (Relative)       → Part: Custom (3D viewport)
│                 └─ Pane: Right (AbsRight, 300px) → Part: Sidebar + Panel
│
└─ Zone: Right (22%)
   └─ BArea: { zone: Right, default_host: Properties, fraction: 1.0 }
        └─ Host: Properties
             ├─ Region: Header → Pane: Center → Part: Menubar
             └─ Region: Center → Pane: Center → Part: Panel (sculpt tools, brush settings)
```

## Key Differences from Layout

- **No left column** - no outliner (sculpting focuses on the model)
- **Right panel** always visible - brush settings, sculpt options
- **View3D** locked to Sculpt mode via `ModebarState`
- **Toolbar** shows sculpt brushes instead of modeling tools
- **Asset Browser** switchable in right panel (for brush assets)
