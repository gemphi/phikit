# Scripting Space - Store Structure

> **Blender references:**
> - `DNA_screen_types.h` - `bScreen`, `ScrArea`, `ARegion`
> - Blender Manual: `workspaces.rst` lines 113-114

---

## Store File Layout

```
crates/phiber/src/store/
  spaces/scripting/
    mod.rs                - ScriptingSpace spec: zones, BAreas, grid fractions
    def.rs                - zone→area→host assignments, pane/part defs
```

---

## Space Level: `store/spaces/scripting/mod.rs`

```rust
//! Scripting space spec - three columns: outliner + text editor + info/console.
//! Ref: Blender Manual - workspaces.rst lines 113-114
//! "Scripting: Has the Outliner, Text Editor, and Info editor."

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;
use crate::store::areas::BArea;

pub static SCRIPTING_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::Scripting,
    label: "Scripting",
    zones: &[Top, Left, Center, Right],
    areas: &SCRIPTING_AREAS,
    columns: &[0.18, 0.47, 0.35],
};

static SCRIPTING_AREAS: &[BArea] = &[
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
        default_host: TextEditor,
        hosts: &[TextEditor],
        fraction: 1.0,
    },
    BArea {
        zone: Right,
        default_host: Info,
        hosts: &[Info, PythonConsole],
        fraction: 1.0,
    },
];
```

---

## Space Level: `store/spaces/scripting/def.rs`

```rust
//! Scripting space definitions.

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

pub static SCRIPTING_COLUMNS: &[f32] = &[0.18, 0.47, 0.35];

pub static SCRIPTING_DEFAULT_HOSTS: &[(BZoneType, BHostType)] = &[
    (BZoneType::Top,    BHostType::Topbar),
    (BZoneType::Left,   BHostType::Outliner),
    (BZoneType::Center, BHostType::TextEditor),
    (BZoneType::Right,  BHostType::Info),
];

pub static SCRIPTING_SWITCHABLE: &[(BZoneType, &[BHostType])] = &[
    (BZoneType::Left,  SWITCHABLE_LEFT),
    (BZoneType::Right, SWITCHABLE_RIGHT),
];

static SWITCHABLE_LEFT: &[BHostType] = &[BHostType::Outliner, BHostType::FileBrowser];
static SWITCHABLE_RIGHT: &[BHostType] = &[BHostType::Info, BHostType::PythonConsole];
```

---

## Host Specs

Scripting space uses Outliner, TextEditor, Info, and Topbar.
See `layout/store.md` for full host spec definitions.

- **TextEditor** - Header (Menubar: File/Edit/Format/View) + Center (Custom: text editor with syntax highlighting)
- **Info** - Header (Menubar) + Center (Custom: info log / console output)
- **Outliner** - Header (Menubar) + Center (Custom: outliner tree)
- **Topbar** - Header (Menubar only)
