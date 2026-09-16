# Shading Space

> **Blender references:**
> - Blender Manual: [`editors/shader_editor/`](../../blender-manual/manual/editors/shader_editor/) - Shader Editor
> - `DNA_space_types.h` - `SpaceNode`, `SpaceView3D`

---

## Space Spec

```rust
// store/spaces/shading/def.rs

pub static SHADING_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::Shading,
    label: "Shading",
    areas: SHADING_AREAS,
    columns: &[0.18, 0.47, 0.35],  // outliner + viewport + shader editor
};

static SHADING_AREAS: &[BArea] = &[
    BArea { zone: Top,    default_host: Topbar,
        hosts: &[Topbar], fraction: 1.0 },
    BArea { zone: Left,   default_host: Outliner,
        hosts: &[Outliner, FileBrowser], fraction: 1.0 },
    BArea { zone: Center, default_host: View3D,
        hosts: &[View3D, ImageEditor], fraction: 1.0 },
    BArea { zone: Right,  default_host: ShaderEditor,
        hosts: &[ShaderEditor, Compositor], fraction: 1.0 },
];
```

## View Tree

```
BSpaceType::Shading (three columns)
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
├─ Zone: Center (47%)
│  └─ BArea: { zone: Center, default_host: View3D, fraction: 1.0 }
│       └─ Host: View3D (Material Preview shading)
│            ├─ Region: Header
│            │    ├─ Pane: Left (Relative)   → Part: Modebar
│            │    └─ Pane: Center (Relative) → Part: Menubar
│            └─ Region: Center
│                 ├─ Pane: Left (AbsLeft, 44px)   → Part: Toolbar
│                 ├─ Pane: Center (Relative)       → Part: Custom (3D viewport)
│                 └─ Pane: Right (AbsRight, 300px) → Part: Sidebar + Panel
│
└─ Zone: Right (35%)
   └─ BArea: { zone: Right, default_host: ShaderEditor, fraction: 1.0 }
        └─ Host: ShaderEditor
             ├─ Region: Header
             │    └─ Pane: Center (Relative) → Part: Menubar (Node, Add, Select)
             └─ Region: Center
                  ├─ Pane: Left (AbsLeft, 240px) → Part: Sidebar (node categories)
                  └─ Pane: Center (Relative)      → Part: Custom (node graph)
```

## Key Differences from Layout

- **Three columns** - outliner, viewport, shader editor
- **ShaderEditor** replaces Properties in the right column
- **View3D** defaults to Material Preview shading mode
- **ShaderEditor** has a sidebar for node category browsing
- **Compositor** is switchable in right column
