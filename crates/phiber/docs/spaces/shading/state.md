# Shading Space - State Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part`

---

## Space State

```rust
impl SpaceState {
    pub fn shading() -> Self {
        let mut active_hosts = HashMap::new();
        active_hosts.insert(BZoneType::Top, BHostType::Topbar);
        active_hosts.insert(BZoneType::Left, BHostType::Outliner);
        active_hosts.insert(BZoneType::Center, BHostType::View3D);
        active_hosts.insert(BZoneType::Right, BHostType::ShaderEditor);
        Self { space_type: BSpaceType::Shading, active_hosts }
    }
}
```

---

## Zone State

```rust
pub fn shading_zones() -> Vec<ZoneState> {
    vec![
        ZoneState { zone: BZoneType::Top, visible: true, active_area: 0 },
        ZoneState { zone: BZoneType::Left, visible: true, active_area: 0 },
        ZoneState { zone: BZoneType::Center, visible: true, active_area: 0 },
        ZoneState { zone: BZoneType::Right, visible: true, active_area: 0 },
    ]
}
```

---

## BArea State

```rust
pub fn shading_areas() -> Vec<AreaState> {
    vec![
        AreaState { zone: BZoneType::Top, BHost: BHostType::Topbar, focused: false },
        AreaState { zone: BZoneType::Left, BHost: BHostType::Outliner, focused: false },
        AreaState { zone: BZoneType::Center, BHost: BHostType::View3D, focused: true },
        AreaState { zone: BZoneType::Right, BHost: BHostType::ShaderEditor, focused: false },
    ]
}
```

---

## Host State

- **View3DState** - `mode: BMode::Object`, `shading: MaterialPreview`, `zoom: 1.0`
- **ShaderEditorState** - `active_node: None`, `scroll: (0.0, 0.0)`, `zoom: 1.0`
- **OutlinerState** - `search: ""`, `expanded: vec!["Scene"]`, `selected: None`
- **TopbarState** - minimal

See `layout/state.md` for full host state definitions.

---

## Region/Pane/Part State

Same as Layout - View3D uses Header (Modebar + Menubar) + Center (Toolbar + Viewport + Sidebar/Panel).
ShaderEditor uses Header (Menubar) + Center (Sidebar: node categories + Custom: node graph).
