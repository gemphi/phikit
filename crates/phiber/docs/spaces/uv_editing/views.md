# UV Editing Space - View Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part → BView`

---

## Full View Tree

```
BSpaceGrid (UV Editing)
│
├─ Zone: Top
│  └─ BArea: { zone: Top, BHost: Topbar, fraction: 1.0 }
│       └─ Host: Topbar
│            └─ Header region (hflex, 32px)
│                 └─ Center pane (Relative) → Menubar part → part/menubar.rs
│
├─ Zone: Left
│  └─ BArea: { zone: Left, default_host: ImageEditor, fraction: 1.0 }
│       └─ Host: ImageEditor (UV mode; switchable to UvEditor)
│            ├─ Header region (hflex, 32px)
│            │    ├─ Pane: Left (Relative)   → Part: Modebar → part/modebar.rs (UV/Vertex tabs)
│            │    └─ Center pane (Relative) → Part: Menubar → part/menubar.rs (Image/UV/Select menus)
│            └─ Center region (hflex, flex-1 - 2 panes)
│                 ├─ Left pane (AbsLeft, 44px)   → Toolbar part → part/toolbar.rs (UV tools)
│                 └─ Center pane (Relative)        → Custom part → part/custom.rs → effect.custom_view()
│                                                      (UV grid view with texture overlay)
│
└─ Zone: Right
   └─ BArea: { zone: Right, default_host: View3D, fraction: 1.0 }
        └─ Host: View3D
             ├─ Header region (hflex, 32px)
             │    ├─ Pane: Left (Relative)   → Part: Modebar → part/modebar.rs
             │    └─ Center pane (Relative) → Part: Menubar → part/menubar.rs
             └─ Center region (hflex, flex-1 - 3 panes)
                  ├─ Left pane (AbsLeft, 44px)   → Toolbar part → part/toolbar.rs
                  ├─ Center pane (Relative)        → Custom part → part/custom.rs → effect.custom_view()
                  └─ Right pane (AbsRight, 300px) → Sidebar part → part/sidebar.rs
                                                       → Panel part → part/panel.rs
```

---

## Render Walk

```rust
// BSpaceGrid::render() for UV Editing
let spec = store::spaces::space_spec(BSpaceType::UvEditing);
// spec.zones = [Top, Left, Right]
// spec.areas = [Top→Topbar, Left→ImageEditor, Right→View3D]

for zone in spec.zones {
    for area in spec.areas.iter().filter(|a| a.zone == zone) {
        let host = state.active_host(zone).unwrap_or(area.default_host);
        let view = BHostView::new(host, theme);
    }
}
```

---

## Part Reuse

All parts shared with Layout space. ImageEditor in UV mode uses Toolbar (UV tools)
and Custom (UV grid view) - the same `part/toolbar.rs` and `part/custom.rs` views,
just with different store spec data.
