# Modeling Space - State Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part`

---

## Space State: `state/space/mod.rs`

```rust
//! SpaceState for Modeling - tracks active host per zone.
use buikit::{BHostType, BSpaceType, BZoneType};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SpaceState {
    pub space_type: BSpaceType,
    pub active_hosts: HashMap<BZoneType, BHostType>,
}

impl SpaceState {
    pub fn modeling() -> Self {
        let mut active_hosts = HashMap::new();
        active_hosts.insert(BZoneType::Top, BHostType::Topbar);
        active_hosts.insert(BZoneType::Center, BHostType::View3D);
        active_hosts.insert(BZoneType::Bottom, BHostType::Timeline);
        Self { space_type: BSpaceType::Modeling, active_hosts }
    }

    pub fn active_host(&self, zone: BZoneType) -> Option<BHostType> {
        self.active_hosts.get(&zone).copied()
    }

    pub fn set_host(&mut self, zone: BZoneType, BHost: BHostType) {
        self.active_hosts.insert(zone, BHost);
    }
}
```

---

## Zone State: `state/zone/mod.rs`

```rust
//! ZoneState - tracks which areas are visible per zone.
use buikit::BZoneType;

#[derive(Clone, Debug)]
pub struct ZoneState {
    pub zone: BZoneType,
    pub visible: bool,
    pub active_area: usize,
}

pub fn modeling_zones() -> Vec<ZoneState> {
    vec![
        ZoneState { zone: BZoneType::Top, visible: true, active_area: 0 },
        ZoneState { zone: BZoneType::Center, visible: true, active_area: 0 },
        ZoneState { zone: BZoneType::Bottom, visible: true, active_area: 0 },
    ]
}
```

---

## BArea State: `state/area/mod.rs`

```rust
//! AreaState - tracks host and focus per area.
use buikit::{BHostType, BZoneType};

#[derive(Clone, Debug)]
pub struct AreaState {
    pub zone: BZoneType,
    pub host: BHostType,
    pub focused: bool,
}

pub fn modeling_areas() -> Vec<AreaState> {
    vec![
        AreaState { zone: BZoneType::Top, BHost: BHostType::Topbar, focused: false },
        AreaState { zone: BZoneType::Center, BHost: BHostType::View3D, focused: true },
        AreaState { zone: BZoneType::Bottom, BHost: BHostType::Timeline, focused: false },
    ]
}
```

---

## Host State: `state/host/mod.rs`

Modeling space uses these host states:
- **View3DState** - `mode: BMode::Object`, `shading: Solid`, `zoom: 1.0`, `rails: RailState::default()`
- **TimelineState** - `frame_current: 1`, `playing: false`, `frame_range: (1, 250)`
- **TopbarState** - minimal (no interactive state)

See `layout/state.md` for full host state definitions.

---

## Region/Pane/Part State

Same as Layout - see `layout/state.md` for RegionState, PaneState, and PartState definitions.
Modeling space uses the same region/pane/part structure as Layout for View3D and Timeline hosts.
