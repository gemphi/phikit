# Geometry Nodes Space - State Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part`

---

## Space State

```rust
impl SpaceState {
    pub fn geometry_nodes() -> Self {
        let mut active_hosts = HashMap::new();
        active_hosts.insert(BZoneType::Top, BHostType::Topbar);
        active_hosts.insert(BZoneType::Center, BHostType::View3D);
        active_hosts.insert(BZoneType::Right, BHostType::GeoNode);
        Self { space_type: BSpaceType::GeometryNodes, active_hosts }
    }
}
```

---

## Zone State

```rust
pub fn geometry_nodes_zones() -> Vec<ZoneState> {
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
pub fn geometry_nodes_areas() -> Vec<AreaState> {
    vec![
        AreaState { zone: BZoneType::Top, BHost: BHostType::Topbar, focused: false },
        AreaState { zone: BZoneType::Center, BHost: BHostType::View3D, focused: false },
        AreaState { zone: BZoneType::Right, BHost: BHostType::GeoNode, focused: true },
    ]
}
```

---

## Host State

- **View3DState** - `mode: BMode::Object`, `shading: Solid`, `zoom: 1.0` (shows geometry node modifier result)
- **GeoNodeState** - `active_node: None`, `scroll: (0.0, 0.0)`, `zoom: 1.0`
- **TopbarState** - minimal

See `layout/state.md` for full host state definitions.

---

## Region/Pane/Part State

Same as Layout - GeoNode uses Header (Menubar) + Center (Sidebar: node categories + Custom: geometry node graph).
View3D uses the same structure as Layout.
