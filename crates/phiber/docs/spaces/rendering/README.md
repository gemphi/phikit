# Rendering Space

> **Blender references:**
> - Blender Manual: [`render/`](../../blender-manual/manual/render/) - Rendering
> - `DNA_space_types.h` - `SpaceView3D`, `SpaceButs`

---

## Space Spec

```rust
// store/spaces/rendering/def.rs

pub static RENDERING_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::Rendering,
    label: "Rendering",
    areas: RENDERING_AREAS,
    columns: &[0.0, 0.65, 0.35],  // viewport + render properties
};

static RENDERING_AREAS: &[BArea] = &[
    BArea { zone: Top,    default_host: Topbar,
        hosts: &[Topbar], fraction: 1.0 },
    BArea { zone: Center, default_host: View3D,
        hosts: &[View3D, ImageEditor], fraction: 1.0 },
    BArea { zone: Right,  default_host: Properties,
        hosts: &[Properties], fraction: 1.0 },
];
```

## View Tree

```
BSpaceType::Rendering (viewport + render properties)
├─ Zone: Top
│  └─ BArea: { zone: Top, BHost: Topbar, fraction: 1.0 }
│       └─ Host: Topbar
│            └─ Region: Header → Pane: Center → Part: Menubar
│
├─ Zone: Center (65%)
│  └─ BArea: { zone: Center, default_host: View3D, fraction: 1.0 }
│       └─ Host: View3D (Rendered shading)
│            ├─ Region: Header
│            │    ├─ Pane: Left (Relative)   → Part: Modebar
│            │    └─ Pane: Center (Relative) → Part: Menubar
│            └─ Region: Center
│                 ├─ Pane: Left (AbsLeft, 44px)   → Part: Toolbar
│                 ├─ Pane: Center (Relative)       → Part: Custom (3D viewport, rendered)
│                 └─ Pane: Right (AbsRight, 300px) → Part: Sidebar + Panel
│
└─ Zone: Right (35%)
   └─ BArea: { zone: Right, default_host: Properties, fraction: 1.0 }
        └─ Host: Properties (Render tab active)
             ├─ Region: Header → Pane: Center → Part: Menubar (Render, Output, ...)
             └─ Region: Center → Pane: Center → Part: Panel (render settings)
```

## Key Differences from Layout

- **Two columns** - viewport + render properties
- **View3D** defaults to Rendered shading mode
- **Properties** locked to Render tab
- **No left column** - no outliner
- **No bottom area** - no timeline
- **ImageEditor** switchable for render result preview
