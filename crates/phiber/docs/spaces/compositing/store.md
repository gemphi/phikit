# Compositing Space - Store Structure

> **Blender references:**
> - `DNA_screen_types.h` - `bScreen`, `ScrArea`, `ARegion`
> - Blender Manual: `workspaces.rst` lines 107-108

---

## Store File Layout

```
crates/phiber/src/store/
  spaces/compositing/
    mod.rs                - CompositingSpace spec: zones, BAreas, grid fractions
    def.rs                - zone→area→host assignments, pane/part defs
```

---

## Space Level: `store/spaces/compositing/mod.rs`

```rust
//! Compositing space spec - single column full-width node editor.
//! Ref: Blender Manual - workspaces.rst lines 107-108
//! "Compositing: Has the Compositor."

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;
use crate::store::areas::BArea;

pub static COMPOSITING_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::Compositing,
    label: "Compositing",
    zones: &[Top, Center],
    areas: &COMPOSITING_AREAS,
    columns: &[1.0],
};

static COMPOSITING_AREAS: &[BArea] = &[
    BArea {
        zone: Top,
        default_host: Topbar,
        hosts: &[Topbar],
        fraction: 1.0,
    },
    BArea {
        zone: Center,
        default_host: Compositor,
        hosts: &[Compositor],
        fraction: 1.0,
    },
];
```

---

## Space Level: `store/spaces/compositing/def.rs`

```rust
//! Compositing space definitions.

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

pub static COMPOSITING_COLUMNS: &[f32] = &[1.0];

pub static COMPOSITING_DEFAULT_HOSTS: &[(BZoneType, BHostType)] = &[
    (BZoneType::Top,    BHostType::Topbar),
    (BZoneType::Center, BHostType::Compositor),
];

// No switchable hosts - single host
```

---

## Host Specs

Compositing space uses Compositor and Topbar.
See `layout/store.md` for full host spec definitions.

- **Compositor** - Header (Menubar: Node/Add/Select/View) + Center (Sidebar: node categories + Custom: compositor node graph + Panel: node properties + Preview: backdrop viewer)
- **Topbar** - Header (Menubar only)
