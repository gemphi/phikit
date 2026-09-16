# UV Editing Space - State Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part`

---

## Space State

```rust
impl SpaceState {
    pub fn uv_editing() -> Self {
        let mut active_hosts = HashMap::new();
        active_hosts.insert(BZoneType::Top, BHostType::Topbar);
        active_hosts.insert(BZoneType::Left, BHostType::ImageEditor);
        active_hosts.insert(BZoneType::Right, BHostType::View3D);
        Self { space_type: BSpaceType::UvEditing, active_hosts }
    }
}
```

---

## Zone State

```rust
pub fn uv_editing_zones() -> Vec<ZoneState> {
    vec![
        ZoneState { zone: BZoneType::Top, visible: true, active_area: 0 },
        ZoneState { zone: BZoneType::Left, visible: true, active_area: 0 },
        ZoneState { zone: BZoneType::Right, visible: true, active_area: 0 },
    ]
}
```

---

## BArea State

```rust
pub fn uv_editing_areas() -> Vec<AreaState> {
    vec![
        AreaState { zone: BZoneType::Top, BHost: BHostType::Topbar, focused: false },
        AreaState { zone: BZoneType::Left, BHost: BHostType::ImageEditor, focused: true },
        AreaState { zone: BZoneType::Right, BHost: BHostType::View3D, focused: false },
    ]
}
```

---

## Host State

- **ImageEditorState** - `mode: BMode::UvEdit`, `zoom: 1.0`, `uv_sync_select: false`
- **View3DState** - `mode: BMode::UvEdit`, `shading: Solid`, `zoom: 1.0`
- **TopbarState** - minimal

See `layout/state.md` for full host state definitions.

---

## Region/Pane/Part State

Same as Layout - ImageEditor uses Header (Modebar + Menubar) + Center (Toolbar + Custom).
View3D uses the same structure as Layout.
