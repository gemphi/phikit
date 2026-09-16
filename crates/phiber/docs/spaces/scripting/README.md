# Scripting Space

> **Blender references:**
> - Blender Manual: [`editors/text_editor/`](../../blender-manual/manual/editors/text_editor/) - Text Editor
> - `DNA_space_types.h` - `SpaceText`, `SpaceConsole`, `SpaceInfo`

---

## Space Spec

```rust
// store/spaces/scripting/def.rs

pub static SCRIPTING_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::Scripting,
    label: "Scripting",
    areas: SCRIPTING_AREAS,
    columns: &[0.18, 0.47, 0.35],  // outliner + text editor + info/console
};

static SCRIPTING_AREAS: &[BArea] = &[
    BArea { zone: Top,    default_host: Topbar,
        hosts: &[Topbar], fraction: 1.0 },
    BArea { zone: Left,   default_host: Outliner,
        hosts: &[Outliner, FileBrowser], fraction: 1.0 },
    BArea { zone: Center, default_host: TextEditor,
        hosts: &[TextEditor], fraction: 1.0 },
    BArea { zone: Right,  default_host: Info,
        hosts: &[Info, PythonConsole], fraction: 1.0 },
];
```

## View Tree

```
BSpaceType::Scripting (three columns)
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
│  └─ BArea: { zone: Center, default_host: TextEditor, fraction: 1.0 }
│       └─ Host: TextEditor
│            ├─ Region: Header
│            │    └─ Pane: Center (Relative) → Part: Menubar (File, Edit, Format, View)
│            └─ Region: Center
│                 └─ Pane: Center (Relative) → Part: Custom (text editor with syntax highlighting)
│
└─ Zone: Right (35%)
   └─ BArea: { zone: Right, default_host: Info, fraction: 1.0 }
        └─ Host: Info
             ├─ Region: Header → Pane: Center → Part: Menubar
             └─ Region: Center
                  └─ Pane: Center (Relative) → Part: Custom (info log / console output)
```

## Key Differences from Layout

- **Three columns** - outliner, text editor, info/console
- **TextEditor** is the primary host (center)
- **Info** host shows operator log and warnings
- **PythonConsole** switchable in right column
- **No viewport** - no View3D
- **No bottom area** - no timeline
- **No toolbar** in text editor - uses menubar for actions
