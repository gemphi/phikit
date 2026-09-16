# Rendering Space - State Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part`

---

## Space State

```rust
impl SpaceState {
    pub fn rendering() -> Self {
        let mut active_hosts = HashMap::new();
        active_hosts.insert(BZoneType::Top, BHostType::Topbar);
        active_hosts.insert(BZoneType::Center, BHostType::View3D);
        active_hosts.insert(BZoneType::Right, BHostType::Properties);
        Self { space_type: BSpaceType::Rendering, active_hosts }
    }
}
```

---

## Zone State

```rust
pub fn rendering_zones() -> Vec<ZoneState> {
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
pub fn rendering_areas() -> Vec<AreaState> {
    vec![
        AreaState { zone: BZoneType::Top, BHost: BHostType::Topbar, focused: false },
        AreaState { zone: BZoneType::Center, BHost: BHostType::View3D, focused: true },
        AreaState { zone: BZoneType::Right, BHost: BHostType::Properties, focused: false },
    ]
}
```

---

## Host State

- **View3DState** - `mode: BMode::Object`, `shading: Rendered`, `zoom: 1.0`
- **PropertiesState** - `active_tab: Render`, `scroll: 0.0`
- **TopbarState** - minimal

See `layout/state.md` for full host state definitions.

---

## Region/Pane/Part State

Same as Layout - View3D in Rendered shading uses the same region/pane/part structure.
Properties uses Header (Menubar) + Center (Panel: render settings).
