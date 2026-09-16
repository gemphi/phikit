# Compositing Space

> **Blender references:**
> - Blender Manual: [`editors/compositor/`](../../blender-manual/manual/editors/compositor/) - Compositor Editor
> - `DNA_space_types.h` - `SpaceNode` (compositor tree)

---

## Space Spec

```rust
// store/spaces/compositing/def.rs

pub static COMPOSITING_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::Compositing,
    label: "Compositing",
    areas: COMPOSITING_AREAS,
    columns: &[0.0, 1.0, 0.0],  // single column - full node editor
};

static COMPOSITING_AREAS: &[BArea] = &[
    BArea { zone: Top,    default_host: Topbar,
        hosts: &[Topbar], fraction: 1.0 },
    BArea { zone: Center, default_host: Compositor,
        hosts: &[Compositor], fraction: 1.0 },
];
```

## View Tree

```
BSpaceType::Compositing (single column - full node editor)
├─ Zone: Top
│  └─ BArea: { zone: Top, BHost: Topbar, fraction: 1.0 }
│       └─ Host: Topbar
│            └─ Region: Header → Pane: Center → Part: Menubar
│
└─ Zone: Center (flex-1)
   └─ BArea: { zone: Center, default_host: Compositor, fraction: 1.0 }
        └─ Host: Compositor
             ├─ Region: Header
             │    └─ Pane: Center (Relative) → Part: Menubar (Node, Add, Select, View)
             └─ Region: Center
                  ├─ Pane: Left (AbsLeft, 240px) → Part: Sidebar (node categories)
                  ├─ Pane: Center (Relative)      → Part: Custom (compositor node graph)
                  └─ Pane: Right (AbsRight, 300px) → Part: Panel (node properties)
                                                      → Part: Preview (backdrop viewer)
```

## Key Differences from Layout

- **Single column** - full-width compositing node editor
- **Compositor** is the only host
- **Sidebar** shows node category browser
- **Panel** shows selected node properties
- **Preview** part shows backdrop viewer for compositing output
- **No outliner, no viewport, no timeline**
