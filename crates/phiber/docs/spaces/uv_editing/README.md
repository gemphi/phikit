# UV Editing Space

> **Blender references:**
> - Blender Manual: [`editors/uv/`](../../blender-manual/manual/editors/uv/) - UV Editor
> - `DNA_space_types.h` - `SpaceImage`

---

## Space Spec

```rust
// store/spaces/uv_editing/def.rs

pub static UV_EDITING_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::UvEditing,
    label: "UV Editing",
    areas: UV_EDITING_AREAS,
    columns: &[0.5, 0.5],  // two equal columns
};

static UV_EDITING_AREAS: &[BArea] = &[
    BArea { zone: Top,    default_host: Topbar,
        hosts: &[Topbar], fraction: 1.0 },
    BArea { zone: Left,   default_host: ImageEditor,
        hosts: &[ImageEditor, UvEditor], fraction: 1.0 },
    BArea { zone: Right,  default_host: View3D,
        hosts: &[View3D], fraction: 1.0 },
];
```

## View Tree

```
BSpaceType::UvEditing (two equal columns)
├─ Zone: Top
│  └─ BArea: { zone: Top, BHost: Topbar, fraction: 1.0 }
│       └─ Host: Topbar
│            └─ Region: Header → Pane: Center → Part: Menubar
│
├─ Zone: Left (50%)
│  └─ BArea: { zone: Left, default_host: ImageEditor, fraction: 1.0 }
│       └─ Host: ImageEditor (UV editor)
│            ├─ Region: Header
│            │    ├─ Pane: Left (Relative)   → Part: Modebar (UV, Vertex)
│            │    └─ Pane: Center (Relative) → Part: Menubar (Select, Image, UV)
│            └─ Region: Center
│                 ├─ Pane: Left (AbsLeft, 44px) → Part: Toolbar (UV tools)
│                 └─ Pane: Center (Relative)     → Part: Custom (UV grid view)
│
└─ Zone: Right (50%)
   └─ BArea: { zone: Right, default_host: View3D, fraction: 1.0 }
        └─ Host: View3D
             ├─ Region: Header
             │    └─ Pane: Center (Relative) → Part: Menubar
             └─ Region: Center
                  ├─ Pane: Left (AbsLeft, 44px) → Part: Toolbar
                  └─ Pane: Center (Relative)     → Part: Custom (3D viewport)
```

## Key Differences from Layout

- **Two equal columns** - UV editor on left, 3D viewport on right
- **ImageEditor** is the primary host (UV editing mode)
- **Modebar** shows UV-specific modes
- **No bottom area** - no timeline needed
- **No sidebar** in UV editor - all tools in toolbar
