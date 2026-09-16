# Shading Space - Store Structure

> **Blender references:**
> - `DNA_screen_types.h` - `bScreen`, `ScrArea`, `ARegion`
> - Blender Manual: `workspaces.rst` lines 98-99

---

## Store File Layout

```
crates/phiber/src/store/
  spaces/shading/
    mod.rs                - ShadingSpace spec: zones, BAreas, grid fractions
    def.rs                - zone→area→host assignments, pane/part defs
```

---

## Space Level: `store/spaces/shading/mod.rs`

```rust
//! Shading space spec - three columns: outliner + viewport + shader editor.
//! Ref: Blender Manual - workspaces.rst lines 98-99
//! "Shading: Has the Outliner, 3D Viewport in Material Preview, and Shader Editor."

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;
use crate::store::areas::BArea;

pub static SHADING_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::Shading,
    label: "Shading",
    zones: &[Top, Left, Center, Right],
    areas: &SHADING_AREAS,
    columns: &[0.18, 0.47, 0.35],
};

static SHADING_AREAS: &[BArea] = &[
    BArea {
        zone: Top,
        default_host: Topbar,
        hosts: &[Topbar],
        fraction: 1.0,
    },
    BArea {
        zone: Left,
        default_host: Outliner,
        hosts: &[Outliner, FileBrowser],
        fraction: 1.0,
    },
    BArea {
        zone: Center,
        default_host: View3D,
        hosts: &[View3D, ImageEditor],
        fraction: 1.0,
    },
    BArea {
        zone: Right,
        default_host: ShaderEditor,
        hosts: &[ShaderEditor, Compositor],
        fraction: 1.0,
    },
];
```

---

## Space Level: `store/spaces/shading/def.rs`

```rust
//! Shading space definitions.

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

pub static SHADING_COLUMNS: &[f32] = &[0.18, 0.47, 0.35];

pub static SHADING_DEFAULT_HOSTS: &[(BZoneType, BHostType)] = &[
    (BZoneType::Top,    BHostType::Topbar),
    (BZoneType::Left,   BHostType::Outliner),
    (BZoneType::Center, BHostType::View3D),
    (BZoneType::Right,  BHostType::ShaderEditor),
];

pub static SHADING_SWITCHABLE: &[(BZoneType, &[BHostType])] = &[
    (BZoneType::Left,   SWITCHABLE_LEFT),
    (BZoneType::Center, SWITCHABLE_CENTER),
    (BZoneType::Right,  SWITCHABLE_RIGHT),
];

static SWITCHABLE_LEFT: &[BHostType] = &[BHostType::Outliner, BHostType::FileBrowser];
static SWITCHABLE_CENTER: &[BHostType] = &[BHostType::View3D, BHostType::ImageEditor];
static SWITCHABLE_RIGHT: &[BHostType] = &[BHostType::ShaderEditor, BHostType::Compositor];
```

---

## Host Specs

Shading space uses Outliner, View3D (Material Preview), ShaderEditor, and Topbar.
See `layout/store.md` for full host spec definitions.

- **View3D** (Material Preview) - Header (Modebar + Menubar) + Center (Toolbar + Viewport + Sidebar/Panel)
- **ShaderEditor** - Header (Menubar: Node/Add/Select) + Center (Sidebar: node categories + Custom: node graph)
- **Outliner** - Header (Menubar) + Center (Custom: outliner tree)
- **Topbar** - Header (Menubar only)
