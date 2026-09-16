# Modeling Space - Store Structure

> **Blender references:**
> - `DNA_screen_types.h` - `bScreen`, `ScrArea`, `ARegion`
> - Blender Manual: `workspaces.rst` lines 86-87

---

## Store File Layout

```
crates/phiber/src/store/
  spaces/modeling/
    mod.rs                - ModelingSpace spec: zones, BAreas, grid fractions
    def.rs                - zone→area→host assignments, pane/part defs
```

---

## Space Level: `store/spaces/modeling/mod.rs`

```rust
//! Modeling space spec - single-column viewport with timeline.
//! Ref: Blender Manual - workspaces.rst lines 86-87
//! "Modeling: Has the 3D Viewport and Timeline."

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;
use crate::store::areas::BArea;

pub static MODELING_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::Modeling,
    label: "Modeling",
    zones: &[Top, Center, Bottom],
    areas: &MODELING_AREAS,
    columns: &[1.0],  // single column
};

static MODELING_AREAS: &[BArea] = &[
    BArea {
        zone: Top,
        default_host: Topbar,
        hosts: &[Topbar],
        fraction: 1.0,
    },
    BArea {
        zone: Center,
        default_host: View3D,
        hosts: BHostType::switchable(),
        fraction: 1.0,
    },
    BArea {
        zone: Bottom,
        default_host: Timeline,
        hosts: &[Timeline, DopeSheet, GraphEditor],
        fraction: 0.28,
    },
];
```

---

## Space Level: `store/spaces/modeling/def.rs`

```rust
//! Modeling space definitions - grid fractions and area host assignments.

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

pub static MODELING_COLUMNS: &[f32] = &[1.0];
pub static MODELING_CENTER_ROWS: &[f32] = &[0.72, 0.28];

pub static MODELING_DEFAULT_HOSTS: &[(BZoneType, BHostType)] = &[
    (BZoneType::Top,    BHostType::Topbar),
    (BZoneType::Center, BHostType::View3D),
    (BZoneType::Bottom, BHostType::Timeline),
];

pub static MODELING_SWITCHABLE: &[(BZoneType, &[BHostType])] = &[
    (BZoneType::Center, SWITCHABLE_CENTER),
    (BZoneType::Bottom, SWITCHABLE_BOTTOM),
];

static SWITCHABLE_CENTER: &[BHostType] = &[
    BHostType::View3D, BHostType::ImageEditor, BHostType::UvEditor,
    BHostType::NodeEditor, BHostType::ShaderEditor, BHostType::Compositor,
    BHostType::GeoNodes, BHostType::TextEditor, BHostType::PythonConsole,
    BHostType::ClipEditor, BHostType::Spreadsheet,
];

static SWITCHABLE_BOTTOM: &[BHostType] = &[
    BHostType::Timeline, BHostType::DopeSheet,
    BHostType::GraphEditor,
];
```

---

## Host Specs

Modeling space uses the same host specs as Layout - View3D, Topbar, and Timeline.
See `layout/store.md` for the full host spec definitions (regions, panes, parts).

- **View3D** - Header (Modebar + Menubar) + Center (Toolbar + Viewport + Sidebar/Panel)
- **Topbar** - Header (Menubar only)
- **Timeline** - Header (Menubar) + Center (Custom timeline ruler)
