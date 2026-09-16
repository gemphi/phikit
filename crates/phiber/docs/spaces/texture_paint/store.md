# Texture Paint Space - Store Structure

> **Blender references:**
> - `DNA_screen_types.h` - `bScreen`, `ScrArea`, `ARegion`
> - Blender Manual: `workspaces.rst` lines 95-96

---

## Store File Layout

```
crates/phiber/src/store/
  spaces/texture_paint/
    mod.rs                - TexturePaintSpace spec: zones, BAreas, grid fractions
    def.rs                - zone→area→host assignments, pane/part defs
```

---

## Space Level: `store/spaces/texture_paint/mod.rs`

```rust
//! Texture Paint space spec - two equal columns: texture canvas + 3D viewport.
//! Ref: Blender Manual - workspaces.rst lines 95-96
//! "Texture Paint: Has the Image Editor and the 3D Viewport in Texture Paint mode."

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;
use crate::store::areas::BArea;

pub static TEXTURE_PAINT_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::TexturePaint,
    label: "Texture Paint",
    zones: &[Top, Left, Right],
    areas: &TEXTURE_PAINT_AREAS,
    columns: &[0.5, 0.5],
};

static TEXTURE_PAINT_AREAS: &[BArea] = &[
    BArea {
        zone: Top,
        default_host: Topbar,
        hosts: &[Topbar],
        fraction: 1.0,
    },
    BArea {
        zone: Left,
        default_host: ImageEditor,
        hosts: &[ImageEditor],
        fraction: 1.0,
    },
    BArea {
        zone: Right,
        default_host: View3D,
        hosts: &[View3D],
        fraction: 1.0,
    },
];
```

---

## Space Level: `store/spaces/texture_paint/def.rs`

```rust
//! Texture Paint space definitions.

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

pub static TEXTURE_PAINT_COLUMNS: &[f32] = &[0.5, 0.5];

pub static TEXTURE_PAINT_DEFAULT_HOSTS: &[(BZoneType, BHostType)] = &[
    (BZoneType::Top,   BHostType::Topbar),
    (BZoneType::Left,  BHostType::ImageEditor),
    (BZoneType::Right, BHostType::View3D),
];

// No switchable hosts - both areas are fixed
```

---

## Host Specs

Texture Paint space uses ImageEditor (texture canvas), View3D (Texture Paint mode), and Topbar.
See `layout/store.md` for full host spec definitions.

- **ImageEditor** - Header (Menubar: Image/Paint) + Center (Toolbar: paint brushes + Custom: 2D texture canvas)
- **View3D** (Texture Paint mode) - Header (Modebar: Paint + Menubar) + Center (Toolbar: paint tools + Viewport)
- **Topbar** - Header (Menubar only)
