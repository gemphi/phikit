# Texture Paint Space

> **Blender references:**
> - Blender Manual: [`sculpt_paint/texture_paint/`](../../blender-manual/manual/sculpt_paint/texture_paint/) - Texture Paint
> - `DNA_space_types.h` - `SpaceImage`, `SpaceView3D`

---

## Space Spec

```rust
// store/spaces/texture_paint/def.rs

pub static TEXTURE_PAINT_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::TexturePaint,
    label: "Texture Paint",
    areas: TEXTURE_PAINT_AREAS,
    columns: &[0.5, 0.5],  // two equal columns
};

static TEXTURE_PAINT_AREAS: &[BArea] = &[
    BArea { zone: Top,    default_host: Topbar,
        hosts: &[Topbar], fraction: 1.0 },
    BArea { zone: Left,   default_host: ImageEditor,
        hosts: &[ImageEditor], fraction: 1.0 },
    BArea { zone: Right,  default_host: View3D,
        hosts: &[View3D], fraction: 1.0 },
];
```

## View Tree

```
BSpaceType::TexturePaint (two equal columns)
├─ Zone: Top
│  └─ BArea: { zone: Top, BHost: Topbar, fraction: 1.0 }
│       └─ Host: Topbar
│            └─ Region: Header → Pane: Center → Part: Menubar
│
├─ Zone: Left (50%)
│  └─ BArea: { zone: Left, default_host: ImageEditor, fraction: 1.0 }
│       └─ Host: ImageEditor (texture canvas)
│            ├─ Region: Header
│            │    └─ Pane: Center (Relative) → Part: Menubar (Image, Paint)
│            └─ Region: Center
│                 ├─ Pane: Left (AbsLeft, 44px) → Part: Toolbar (paint brushes)
│                 └─ Pane: Center (Relative)     → Part: Custom (2D texture canvas)
│
└─ Zone: Right (50%)
   └─ BArea: { zone: Right, default_host: View3D, fraction: 1.0 }
        └─ Host: View3D (texture paint mode)
             ├─ Region: Header
             │    ├─ Pane: Left (Relative)   → Part: Modebar (Paint)
             │    └─ Pane: Center (Relative) → Part: Menubar
             └─ Region: Center
                  ├─ Pane: Left (AbsLeft, 44px) → Part: Toolbar (paint tools)
                  └─ Pane: Center (Relative)     → Part: Custom (3D viewport)
```

## Key Differences from Layout

- **Two equal columns** - texture canvas on left, 3D viewport on right
- **ImageEditor** shows the 2D texture being painted
- **View3D** in Texture Paint mode
- **Toolbar** shows paint brushes and tools in both hosts
- **No bottom area** - no timeline
