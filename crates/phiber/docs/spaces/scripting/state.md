# Scripting Space - State Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part`

---

## Space State

```rust
impl SpaceState {
    pub fn scripting() -> Self {
        let mut active_hosts = HashMap::new();
        active_hosts.insert(BZoneType::Top, BHostType::Topbar);
        active_hosts.insert(BZoneType::Left, BHostType::Outliner);
        active_hosts.insert(BZoneType::Center, BHostType::TextEditor);
        active_hosts.insert(BZoneType::Right, BHostType::Info);
        Self { space_type: BSpaceType::Scripting, active_hosts }
    }
}
```

---

## Zone State

```rust
pub fn scripting_zones() -> Vec<ZoneState> {
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
pub fn scripting_areas() -> Vec<AreaState> {
    vec![
        AreaState { zone: BZoneType::Top, BHost: BHostType::Topbar, focused: false },
        AreaState { zone: BZoneType::Left, BHost: BHostType::Outliner, focused: false },
        AreaState { zone: BZoneType::Center, BHost: BHostType::TextEditor, focused: true },
        AreaState { zone: BZoneType::Right, BHost: BHostType::Info, focused: false },
    ]
}
```

---

## Host State

- **TextEditorState** - `active_file: None`, `cursor: (0, 0)`, `scroll: 0.0`
- **InfoState** - `log_entries: vec![]`, `scroll: 0.0`, `filter: InfoFilter::All`
- **OutlinerState** - `search: ""`, `expanded: vec!["Scene"]`, `selected: None`
- **TopbarState** - minimal

See `layout/state.md` for full host state definitions.

---

## Region/Pane/Part State

Same as Layout - TextEditor uses Header (Menubar) + Center (Custom: text editor).
Info uses Header (Menubar) + Center (Custom: info log).
Outliner uses Header (Menubar) + Center (Custom: outliner tree).
