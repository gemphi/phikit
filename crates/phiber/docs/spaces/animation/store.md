# Animation Space - Store Structure

> **Blender references:**
> - `DNA_screen_types.h` - `bScreen`, `ScrArea`, `ARegion`
> - Blender Manual: `workspaces.rst` lines 101-102

---

## Store File Layout

```
crates/phiber/src/store/
  spaces/animation/
    mod.rs                - AnimationSpace spec: zones, BAreas, grid fractions
    def.rs                - zone→area→host assignments, pane/part defs
```

---

## Space Level: `store/spaces/animation/mod.rs`

```rust
//! Animation space spec - outliner + viewport (top) + animation editor (bottom).
//! Ref: Blender Manual - workspaces.rst lines 101-102
//! "Animation: Has the Outliner, 3D Viewport, and Dope Sheet."

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;
use crate::store::areas::BArea;

pub static ANIMATION_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::Animation,
    label: "Animation",
    zones: &[Top, Left, Center, Bottom],
    areas: &ANIMATION_AREAS,
    columns: &[0.18, 0.82],
};

static ANIMATION_AREAS: &[BArea] = &[
    BArea {
        zone: Top,
        default_host: Topbar,
        hosts: &[Topbar],
        fraction: 1.0,
    },
    BArea {
        zone: Left,
        default_host: Outliner,
        hosts: &[Outliner, Spreadsheet],
        fraction: 1.0,
    },
    BArea {
        zone: Center,
        default_host: View3D,
        hosts: &[View3D],
        fraction: 0.5,
    },
    BArea {
        zone: Bottom,
        default_host: DopeSheet,
        hosts: &[DopeSheet, GraphEditor, Nla, Timeline],
        fraction: 0.5,
    },
];
```

---

## Space Level: `store/spaces/animation/def.rs`

```rust
//! Animation space definitions.

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

pub static ANIMATION_COLUMNS: &[f32] = &[0.18, 0.82];
pub static ANIMATION_CENTER_ROWS: &[f32] = &[0.5, 0.5];

pub static ANIMATION_DEFAULT_HOSTS: &[(BZoneType, BHostType)] = &[
    (BZoneType::Top,    BHostType::Topbar),
    (BZoneType::Left,   BHostType::Outliner),
    (BZoneType::Center, BHostType::View3D),
    (BZoneType::Bottom, BHostType::DopeSheet),
];

pub static ANIMATION_SWITCHABLE: &[(BZoneType, &[BHostType])] = &[
    (BZoneType::Left,   SWITCHABLE_LEFT),
    (BZoneType::Bottom, SWITCHABLE_BOTTOM),
];

static SWITCHABLE_LEFT: &[BHostType] = &[BHostType::Outliner, BHostType::Spreadsheet];
static SWITCHABLE_BOTTOM: &[BHostType] = &[
    BHostType::DopeSheet, BHostType::GraphEditor,
    BHostType::Nla, BHostType::Timeline,
];
```

---

## Host Specs

Animation space uses Outliner, View3D, DopeSheet, and Topbar.
See `layout/store.md` for full host spec definitions.

- **View3D** - Header (Modebar + Menubar) + Center (Toolbar + Viewport + Sidebar/Panel)
- **DopeSheet** - Header (Menubar: Key/Select/Marker) + Center (Channels: left pane + Custom: dope sheet grid)
- **Outliner** - Header (Menubar) + Center (Custom: outliner tree)
- **Topbar** - Header (Menubar only)
