# Sculpting Space - Store Structure

> **Blender references:**
> - `DNA_screen_types.h` - `bScreen`, `ScrArea`, `ARegion`
> - Blender Manual: `workspaces.rst` lines 89-90

---

## Store File Layout

```
crates/phiber/src/store/
  spaces/sculpting/
    mod.rs                - SculptingSpace spec: zones, BAreas, grid fractions
    def.rs                - zone→area→host assignments, pane/part defs
```

---

## Space Level: `store/spaces/sculpting/mod.rs`

```rust
//! Sculpting space spec - viewport + right properties panel.
//! Ref: Blender Manual - workspaces.rst lines 89-90
//! "Sculpting: Has the 3D Viewport in Sculpt Mode and Properties."

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;
use crate::store::areas::BArea;

pub static SCULPTING_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::Sculpting,
    label: "Sculpting",
    zones: &[Top, Center, Right],
    areas: &SCULPTING_AREAS,
    columns: &[0.78, 0.22],  // viewport + right panel
};

static SCULPTING_AREAS: &[BArea] = &[
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
        hosts: &[Properties, AssetBrowser],
        fraction: 1.0,
    },
];
```

---

## Space Level: `store/spaces/sculpting/def.rs`

```rust
//! Sculpting space definitions.

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

pub static SCULPTING_COLUMNS: &[f32] = &[0.78, 0.22];

pub static SCULPTING_DEFAULT_HOSTS: &[(BZoneType, BHostType)] = &[
    (BZoneType::Top,    BHostType::Topbar),
    (BZoneType::Center, BHostType::View3D),
    (BZoneType::Right,  BHostType::Properties),
];

pub static SCULPTING_SWITCHABLE: &[(BZoneType, &[BHostType])] = &[
    (BZoneType::Center, SWITCHABLE_CENTER),
    (BZoneType::Right,  SWITCHABLE_RIGHT),
];

static SWITCHABLE_CENTER: &[BHostType] = &[
    BHostType::View3D, BHostType::ImageEditor,
];

static SWITCHABLE_RIGHT: &[BHostType] = &[
    BHostType::Properties, BHostType::AssetBrowser,
];
```

---

## Host Specs

Sculpting space uses View3D (Sculpt mode), Topbar, and Properties.
See `layout/store.md` for full host spec definitions.

- **View3D** (Sculpt mode) - Header (Modebar: Sculpt/Paint + Menubar) + Center (Toolbar: sculpt brushes + Viewport + Sidebar/Panel)
- **Topbar** - Header (Menubar only)
- **Properties** - Header (Menubar) + Center (Panel: sculpt tools, brush settings)
