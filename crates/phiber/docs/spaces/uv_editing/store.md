# UV Editing Space - Store Structure

> **Blender references:**
> - `DNA_screen_types.h` - `bScreen`, `ScrArea`, `ARegion`
> - Blender Manual: `workspaces.rst` lines 92-93

---

## Store File Layout

```
crates/phiber/src/store/
  spaces/uv_editing/
    mod.rs                - UvEditingSpace spec: zones, BAreas, grid fractions
    def.rs                - zone→area→host assignments, pane/part defs
```

---

## Space Level: `store/spaces/uv_editing/mod.rs`

```rust
//! UV Editing space spec - two equal columns: UV editor + 3D viewport.
//! Ref: Blender Manual - workspaces.rst lines 92-93
//! "UV Editing: Has the Image Editor in UV mode and the 3D Viewport."

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;
use crate::store::areas::BArea;

pub static UV_EDITING_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::UvEditing,
    label: "UV Editing",
    zones: &[Top, Left, Right],
    areas: &UV_EDITING_AREAS,
    columns: &[0.5, 0.5],  // two equal columns
};

static UV_EDITING_AREAS: &[BArea] = &[
    BArea {
        zone: Top,
        default_host: Topbar,
        hosts: &[Topbar],
        fraction: 1.0,
    },
    BArea {
        zone: Left,
        default_host: ImageEditor,
        hosts: &[ImageEditor, UvEditor],
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

## Space Level: `store/spaces/uv_editing/def.rs`

```rust
//! UV Editing space definitions.

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

pub static UV_EDITING_COLUMNS: &[f32] = &[0.5, 0.5];

pub static UV_EDITING_DEFAULT_HOSTS: &[(BZoneType, BHostType)] = &[
    (BZoneType::Top,   BHostType::Topbar),
    (BZoneType::Left,  BHostType::ImageEditor),
    (BZoneType::Right, BHostType::View3D),
];

pub static UV_EDITING_SWITCHABLE: &[(BZoneType, &[BHostType])] = &[
    (BZoneType::Left, SWITCHABLE_LEFT),
];

static SWITCHABLE_LEFT: &[BHostType] = &[
    BHostType::ImageEditor, BHostType::UvEditor,
];
```

---

## Host Specs

UV Editing space uses ImageEditor (UV mode), View3D, and Topbar.
See `layout/store.md` for full host spec definitions.

- **ImageEditor** (UV mode) - Header (Modebar: UV/Vertex + Menubar) + Center (Toolbar: UV tools + Custom: UV grid view)
- **View3D** - Header (Menubar) + Center (Toolbar + Viewport + Sidebar/Panel)
- **Topbar** - Header (Menubar only)
