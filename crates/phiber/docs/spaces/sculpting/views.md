# Sculpting Space - View Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part → BView`

---

## Full View Tree

```
BSpaceGrid (Sculpting)
│
├─ Zone: Top
│  └─ BArea: { zone: Top, BHost: Topbar, fraction: 1.0 }
│       └─ Host: Topbar
│            └─ Header region (hflex, 32px)
│                 └─ Center pane (Relative) → Menubar part → part/menubar.rs
│
├─ Zone: Center
│  └─ BArea: { zone: Center, default_host: View3D, fraction: 1.0 }
│       └─ Host: View3D (Sculpt mode; switchable to ImageEditor)
│            ├─ Header region (hflex, 32px)
│            │    ├─ Pane: Left (Relative)   → Part: Modebar → part/modebar.rs (Sculpt/Paint tabs)
│            │    └─ Center pane (Relative) → Part: Menubar → part/menubar.rs
│            └─ Center region (hflex, flex-1 - 3 panes)
│                 ├─ Left pane (AbsLeft, 44px)   → Toolbar part → part/toolbar.rs (sculpt brushes)
│                 ├─ Center pane (Relative)        → Custom part → part/custom.rs → effect.custom_view()
│                 └─ Right pane (AbsRight, 300px) → Sidebar part → part/sidebar.rs
│                                                      → Panel part → part/panel.rs (brush settings)
│
└─ Zone: Right
   └─ BArea: { zone: Right, default_host: Properties, fraction: 1.0 }
        └─ Host: Properties (switchable to AssetBrowser)
             ├─ Header region (hflex, 32px)
             │    └─ Center pane (Relative) → Menubar part → part/menubar.rs
             └─ Center region (hflex, flex-1)
                  └─ Center pane (Relative) → Panel part → part/panel.rs (sculpt tools, brush settings)
```

---

## Render Walk

```rust
// BSpaceGrid::render() for Sculpting
let spec = store::spaces::space_spec(BSpaceType::Sculpting);
// spec.zones = [Top, Center, Right]
// spec.areas = [Top→Topbar, Center→View3D, Right→Properties]

for zone in spec.zones {
    for area in spec.areas.iter().filter(|a| a.zone == zone) {
        let host = state.active_host(zone).unwrap_or(area.default_host);
        let view = BHostView::new(host, theme);
        // BHostView walks: regions → panes → parts
    }
}
```

---

## Part Reuse

All parts shared with Layout space. View3D in Sculpt mode uses the same
part views - only the data (store specs) differ (sculpt brushes in toolbar,
sculpt brush settings in sidebar panel).
