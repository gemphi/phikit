# BArea Switcher Modal - Full Documentation

> **Type:** `BModalType::AreaSwitcher`
> **Width:** 520.0
> **Source:** `state/modals/area_switcher.rs` (no store spec yet)

---

## Overview

The BArea Switcher modal lets the user change the host type in the currently
focused area. Equivalent to Blender's editor type selector dropdown (the
icon button at the top-left of each area).

---

## View Tree

### Current (stub)

```
b_modal("Switch BArea", 520.0)
  └─ Label("Select Editor Type")
```

### Planned

```
b_modal("Switch BArea", 520.0)
  └─ Column
       ├─ Label("Select Editor Type")
       └─ Grid (4 columns)
            └─ for each host in available_hosts:
                 Button
                   ├─ Icon(host.icon())
                   ├─ Label(host.label())
                   └─ (highlight if host == current_host)
```

---

## Store Spec (planned)

No store spec file exists yet. When implemented:

```rust
// store/modals/area_switcher.rs (planned)

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

/// BArea switcher spec - which hosts can be switched to for a given zone.
pub struct AreaSwitcherSpec {
    pub zone: BZoneType,
    pub current_host: BHostType,
    pub available_hosts: &'static [BHostType],
}

impl AreaSwitcherSpec {
    pub fn for_zone(zone: BZoneType, current_host: BHostType) -> Self {
        Self {
            zone,
            current_host,
            available_hosts: BHostType::switchable(),
        }
    }
}
```

---

## State (planned)

```rust
use buikit::{BHostType, BZoneType};

#[derive(Clone, Debug)]
pub struct AreaSwitcherState {
    pub zone: BZoneType,             // which zone's area we're switching
    pub current_host: BHostType,     // currently active host
    pub available_hosts: Vec<BHostType>,  // filtered by switchable list
}

impl AreaSwitcherState {
    pub fn new(zone: BZoneType, current_host: BHostType, available: &[BHostType]) -> Self {
        Self {
            zone,
            current_host,
            available_hosts: available.to_vec(),
        }
    }

    pub fn set_host(&mut self, BHost: BHostType) {
        self.current_host = host;
    }
}
```

---

## Handler

### Current (stub)

```rust
impl BModalHandler for AreaSwitcherModal {
    fn idx(&self) -> BModalType { BModalType::AreaSwitcher }

    fn body(&self, _cx: &App) -> Vec<BView> {
        vec![BView::label("Select Editor Type")]
    }
}
```

### Planned

```rust
impl BModalHandler for AreaSwitcherModal {
    fn idx(&self) -> BModalType { BModalType::AreaSwitcher }

    fn body(&self, cx: &App) -> Vec<BView> {
        let state = cx.global::<ModalState>();
        let ModalState::AreaSwitcher(switcher) = state else {
            return vec![BView::label("No area selected")];
        };

        let mut grid: Vec<BView> = Vec::new();
        for &host in &switcher.available_hosts {
            let is_active = host == switcher.current_host;
            grid.push(BView::Button {
                label: host.label().to_string(),
                icon: Some(host.icon().to_string()),
                active: is_active,
                on_click: Box::new(move |cx| {
                    // Dispatch: set host for this zone
                    cx.dispatch(SetHost { zone: switcher.zone, BHost });
                }),
            });
        }

        vec![
            BView::label("Select Editor Type"),
            BView::Grid {
                columns: 4,
                children: grid,
            },
        ]
    }
}
```

---

## Blender Comparison

Blender's editor type selector is a dropdown menu at the top-left of each area,
not a modal. It shows an icon for the current editor type and a dropdown list
of all available editor types.

Phiber's BArea Switcher is a modal overlay - more visible and keyboard-navigable.
To match Blender's quick-switch pattern, we could also add:
- **Inline dropdown** - click the editor type icon in the header region
- **Pie menu** - quick radial selector (Blender's pie menu pattern)
- **Keyboard shortcut** - `Shift-Q` for quick editor switch (Blender convention)

---

## Zed Comparison

Zed doesn't have an area switcher - it uses a tab-based system where each tab
is a different editor type. Switching is done by clicking tabs or using
`Ctrl-Tab` / `Ctrl-Shift-Tab`.

Phiber's area switching is more like Blender's - each area can host different
editor types, and the user switches the host type within the area.
