# Shading Space - View Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part → BView`

---

## Full View Tree

```
BSpaceGrid (Shading)
│
├─ Zone: Top
│  └─ BArea: { zone: Top, BHost: Topbar, fraction: 1.0 }
│       └─ Host: Topbar
│            └─ Header region (hflex, 32px)
│                 └─ Center pane (Relative) → Menubar part → part/menubar.rs
│
├─ Zone: Left
│  └─ BArea: { zone: Left, default_host: Outliner, fraction: 1.0 }
│       └─ Host: Outliner (switchable to FileBrowser)
│            ├─ Header region (hflex, 32px)
│            │    └─ Center pane (Relative) → Menubar part → part/menubar.rs
│            └─ Center region (hflex, flex-1)
│                 └─ Center pane (Relative) → Custom part → part/custom.rs → effect.custom_view()
│
├─ Zone: Center
│  └─ BArea: { zone: Center, default_host: View3D, fraction: 1.0 }
│       └─ Host: View3D (Material Preview; switchable to ImageEditor)
│            ├─ Header region (hflex, 32px)
│            │    ├─ Pane: Left (Relative)   → Part: Modebar → part/modebar.rs
│            │    └─ Center pane (Relative) → Part: Menubar → part/menubar.rs
│            └─ Center region (hflex, flex-1 - 3 panes)
│                 ├─ Left pane (AbsLeft, 44px)   → Toolbar part → part/toolbar.rs
│                 ├─ Center pane (Relative)        → Custom part → part/custom.rs → effect.custom_view()
│                 └─ Right pane (AbsRight, 300px) → Sidebar part → part/sidebar.rs
│                                                      → Panel part → part/panel.rs
│
└─ Zone: Right
   └─ BArea: { zone: Right, default_host: ShaderEditor, fraction: 1.0 }
        └─ Host: ShaderEditor (switchable to Compositor)
             ├─ Header region (hflex, 32px)
             │    └─ Center pane (Relative) → Menubar part → part/menubar.rs (Node/Add/Select menus)
             └─ Center region (hflex, flex-1 - 2 panes)
                  ├─ Left pane (AbsLeft, 200px)  → Sidebar part → part/sidebar.rs (node categories)
                  └─ Center pane (Relative)        → Custom part → part/custom.rs → effect.custom_view()
                                                       (shader node graph)
```

---

## Render Walk

```rust
// BSpaceGrid::render() for Shading
let spec = store::spaces::space_spec(BSpaceType::Shading);
// spec.zones = [Top, Left, Center, Right]
// spec.areas = [Top→Topbar, Left→Outliner, Center→View3D, Right→ShaderEditor]

for zone in spec.zones {
    for area in spec.areas.iter().filter(|a| a.zone == zone) {
        let host = state.active_host(zone).unwrap_or(area.default_host);
        let view = BHostView::new(host, theme);
    }
}
```

---

## Part Reuse

All parts shared with Layout space. ShaderEditor uses Sidebar (node categories) and
Custom (node graph) - same `part/sidebar.rs` and `part/custom.rs` views with different
store spec data.
