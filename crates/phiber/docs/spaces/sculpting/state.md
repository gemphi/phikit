# Sculpting Space - State Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part`

---

## Space State: `state/space/mod.rs`

```rust
use buikit::{BHostType, BSpaceType, BZoneType};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SpaceState {
    pub space_type: BSpaceType,
    pub active_hosts: HashMap<BZoneType, BHostType>,
}

impl SpaceState {
    pub fn sculpting() -> Self {
        let mut active_hosts = HashMap::new();
        active_hosts.insert(BZoneType::Top, BHostType::Topbar);
        active_hosts.insert(BZoneType::Center, BHostType::View3D);
        active_hosts.insert(BZoneType::Right, BHostType::Properties);
        Self { space_type: BSpaceType::Sculpting, active_hosts }
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

## Zone State

```rust
use buikit::BZoneType;

pub fn sculpting_zones() -> Vec<ZoneState> {
    vec![
        ZoneState { zone: BZoneType::Top, visible: true, active_area: 0 },
        ZoneState { zone: BZoneType::Center, visible: true, active_area: 0 },
        ZoneState { zone: BZoneType::Right, visible: true, active_area: 0 },
    ]
}
```

---

## BArea State

```rust
use buikit::{BHostType, BZoneType};

pub fn sculpting_areas() -> Vec<AreaState> {
    vec![
        AreaState { zone: BZoneType::Top, BHost: BHostType::Topbar, focused: false },
        AreaState { zone: BZoneType::Center, BHost: BHostType::View3D, focused: true },
        AreaState { zone: BZoneType::Right, BHost: BHostType::Properties, focused: false },
    ]
}
```

---

## Host State

Sculpting space uses these host states:
- **View3DState** - `mode: BMode::Sculpt`, `shading: Solid`, `zoom: 1.0`, `rails: RailState::default()`
- **PropertiesState** - `active_tab: Render`, `scroll: 0.0` (sculpt tools/brush settings)
- **TopbarState** - minimal

See `layout/state.md` for full host state definitions.

---

## Region/Pane/Part State

Same as Layout - see `layout/state.md`. View3D in Sculpt mode uses the same
region/pane/part structure. Properties uses Header + Center (Panel part).
