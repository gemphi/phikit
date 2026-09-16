# Compositing Space - State Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part`

---

## Space State

```rust
impl SpaceState {
    pub fn compositing() -> Self {
        let mut active_hosts = HashMap::new();
        active_hosts.insert(BZoneType::Top, BHostType::Topbar);
        active_hosts.insert(BZoneType::Center, BHostType::Compositor);
        Self { space_type: BSpaceType::Compositing, active_hosts }
    }
}
```

---

## Zone State

```rust
pub fn compositing_zones() -> Vec<ZoneState> {
    vec![
        ZoneState { zone: BZoneType::Top, visible: true, active_area: 0 },
        ZoneState { zone: BZoneType::Center, visible: true, active_area: 0 },
    ]
}
```

---

## BArea State

```rust
pub fn compositing_areas() -> Vec<AreaState> {
    vec![
        AreaState { zone: BZoneType::Top, BHost: BHostType::Topbar, focused: false },
        AreaState { zone: BZoneType::Center, BHost: BHostType::Compositor, focused: true },
    ]
}
```

---

## Host State

- **CompositorState** - `active_node: None`, `scroll: (0.0, 0.0)`, `zoom: 1.0`, `backdrop_visible: false`
- **TopbarState** - minimal

See `layout/state.md` for full host state definitions.

---

## Region/Pane/Part State

Same as Layout - Compositor uses Header (Menubar) + Center (Sidebar: node categories + Custom: node graph + Panel: node properties + Preview: backdrop viewer).
