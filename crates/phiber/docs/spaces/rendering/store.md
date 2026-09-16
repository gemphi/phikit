# Rendering Space - Store Structure

> **Blender references:**
> - `DNA_screen_types.h` - `bScreen`, `ScrArea`, `ARegion`
> - Blender Manual: `workspaces.rst` lines 104-105

---

## Store File Layout

```
crates/phiber/src/store/
  spaces/rendering/
    mod.rs                - RenderingSpace spec: zones, BAreas, grid fractions
    def.rs                - zone→area→host assignments, pane/part defs
```

---

## Space Level: `store/spaces/rendering/mod.rs`

```rust
//! Rendering space spec - viewport + render properties.
//! Ref: Blender Manual - workspaces.rst lines 104-105
//! "Rendering: Has the 3D Viewport in Rendered shading and Properties."

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;
use crate::store::areas::BArea;

pub static RENDERING_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::Rendering,
    label: "Rendering",
    zones: &[Top, Center, Right],
    areas: &RENDERING_AREAS,
    columns: &[0.65, 0.35],
};

static RENDERING_AREAS: &[BArea] = &[
    BArea {
        zone: Top,
        default_host: Topbar,
        hosts: &[Topbar],
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
        default_host: Properties,
        hosts: &[Properties],
        fraction: 1.0,
    },
];
```

---

## Space Level: `store/spaces/rendering/def.rs`

```rust
//! Rendering space definitions.

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

pub static RENDERING_COLUMNS: &[f32] = &[0.65, 0.35];

pub static RENDERING_DEFAULT_HOSTS: &[(BZoneType, BHostType)] = &[
    (BZoneType::Top,    BHostType::Topbar),
    (BZoneType::Center, BHostType::View3D),
    (BZoneType::Right,  BHostType::Properties),
];

pub static RENDERING_SWITCHABLE: &[(BZoneType, &[BHostType])] = &[
    (BZoneType::Center, SWITCHABLE_CENTER),
];

static SWITCHABLE_CENTER: &[BHostType] = &[BHostType::View3D, BHostType::ImageEditor];
```

---

## Host Specs

Rendering space uses View3D (Rendered shading), Properties (Render tab), and Topbar.
See `layout/store.md` for full host spec definitions.

- **View3D** (Rendered shading) - Header (Modebar + Menubar) + Center (Toolbar + Viewport + Sidebar/Panel)
- **Properties** (Render tab) - Header (Menubar: Render/Output/Scene) + Center (Panel: render settings)
- **Topbar** - Header (Menubar only)
