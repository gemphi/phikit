# Animation Space - State Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part`

---

## Space State

```rust
impl SpaceState {
    pub fn animation() -> Self {
        let mut active_hosts = HashMap::new();
        active_hosts.insert(BZoneType::Top, BHostType::Topbar);
        active_hosts.insert(BZoneType::Left, BHostType::Outliner);
        active_hosts.insert(BZoneType::Center, BHostType::View3D);
        active_hosts.insert(BZoneType::Bottom, BHostType::DopeSheet);
        Self { space_type: BSpaceType::Animation, active_hosts }
    }
}
```

---

## Zone State

```rust
pub fn animation_zones() -> Vec<ZoneState> {
    vec![
        ZoneState { zone: BZoneType::Top, visible: true, active_area: 0 },
        ZoneState { zone: BZoneType::Left, visible: true, active_area: 0 },
        ZoneState { zone: BZoneType::Center, visible: true, active_area: 0 },
        ZoneState { zone: BZoneType::Bottom, visible: true, active_area: 0 },
    ]
}
```

---

## BArea State

```rust
pub fn animation_areas() -> Vec<AreaState> {
    vec![
        AreaState { zone: BZoneType::Top, BHost: BHostType::Topbar, focused: false },
        AreaState { zone: BZoneType::Left, BHost: BHostType::Outliner, focused: false },
        AreaState { zone: BZoneType::Center, BHost: BHostType::View3D, focused: false },
        AreaState { zone: BZoneType::Bottom, BHost: BHostType::DopeSheet, focused: true },
    ]
}
```

---

## Host State

- **View3DState** - `mode: BMode::Object`, `shading: Solid`, `zoom: 1.0`
- **DopeSheetState** - `frame_current: 1`, `playing: false`, `frame_range: (1, 250)`, `selected_channels: vec![]`
- **OutlinerState** - `search: ""`, `expanded: vec!["Scene"]`, `selected: None`
- **TopbarState** - minimal

See `layout/state.md` for full host state definitions.

---

## Region/Pane/Part State

Same as Layout - DopeSheet uses Header (Menubar) + Center (Channels: left pane + Custom: dope sheet grid).
View3D and Outliner use the same structure as Layout.
