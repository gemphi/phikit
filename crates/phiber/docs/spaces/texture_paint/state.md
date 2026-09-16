# Texture Paint Space - State Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part`

---

## Space State

```rust
impl SpaceState {
    pub fn texture_paint() -> Self {
        let mut active_hosts = HashMap::new();
        active_hosts.insert(BZoneType::Top, BHostType::Topbar);
        active_hosts.insert(BZoneType::Left, BHostType::ImageEditor);
        active_hosts.insert(BZoneType::Right, BHostType::View3D);
        Self { space_type: BSpaceType::TexturePaint, active_hosts }
    }
}
```

---

## Zone State

```rust
pub fn texture_paint_zones() -> Vec<ZoneState> {
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
pub fn texture_paint_areas() -> Vec<AreaState> {
    vec![
        AreaState { zone: BZoneType::Top, BHost: BHostType::Topbar, focused: false },
        AreaState { zone: BZoneType::Left, BHost: BHostType::ImageEditor, focused: false },
        AreaState { zone: BZoneType::Right, BHost: BHostType::View3D, focused: true },
    ]
}
```

---

## Host State

- **ImageEditorState** - `mode: BMode::TexturePaint`, `zoom: 1.0`, `brush: Some("Brush")`
- **View3DState** - `mode: BMode::TexturePaint`, `shading: Material`, `zoom: 1.0`
- **TopbarState** - minimal

See `layout/state.md` for full host state definitions.

---

## Region/Pane/Part State

Same as Layout - ImageEditor uses Header (Menubar) + Center (Toolbar + Custom: 2D canvas).
View3D in Texture Paint mode uses Header (Modebar + Menubar) + Center (Toolbar + Viewport).
