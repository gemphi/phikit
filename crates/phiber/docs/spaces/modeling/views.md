# Modeling Space - View Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part → BView`

---

## Full View Tree

```
BSpaceGrid (Modeling)
│
├─ Zone: Top
│  └─ BArea: { zone: Top, BHost: Topbar, fraction: 1.0 }
│       └─ Host: Topbar
│            └─ Header region (hflex, 32px)
│                 └─ Center pane (Relative) → Menubar part → part/menubar.rs
│
├─ Zone: Center
│  └─ BArea: { zone: Center, default_host: View3D, fraction: 1.0 }
│       └─ Host: View3D (switchable to all editors)
│            ├─ Header region (hflex, 32px)
│            │    ├─ Pane: Left (Relative)   → Part: Modebar → part/modebar.rs
│            │    └─ Center pane (Relative) → Part: Menubar → part/menubar.rs
│            └─ Center region (hflex, flex-1 - 3 panes)
│                 ├─ Left pane (AbsLeft, 44px)   → Toolbar part → part/toolbar.rs
│                 ├─ Center pane (Relative)        → Custom part → part/custom.rs → effect.custom_view()
│                 └─ Right pane (AbsRight, 300px) → Sidebar part → part/sidebar.rs
│                                                      → Panel part → part/panel.rs
│
└─ Zone: Bottom
   └─ BArea: { zone: Bottom, default_host: Timeline, fraction: 0.28 }
        └─ Host: Timeline (switchable to DopeSheet, GraphEditor)
             ├─ Header region (hflex, 32px)
             │    └─ Center pane (Relative) → Menubar part → part/menubar.rs
             └─ Center region (hflex, flex-1)
                  └─ Center pane (Relative) → Custom part → part/custom.rs → effect.custom_view()
```

---

## Render Walk

```rust
// BSpaceGrid::render() for Modeling
let spec = store::spaces::space_spec(BSpaceType::Modeling);
// spec.zones = [Top, Center, Bottom]
// spec.areas = [Top→Topbar, Center→View3D, Bottom→Timeline]

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

All parts are shared with Layout space:
- `part/menubar.rs` - menu strip (View, Select, Add, Object menus)
- `part/modebar.rs` - mode tabs (Object, Edit, Sculpt, ...)
- `part/toolbar.rs` - tool buttons (select, transform, annotate)
- `part/sidebar.rs` - N-panel container (Item, Tool, View tabs)
- `part/panel.rs` - property panels
- `part/custom.rs` - delegates to `effect.custom_view()` per host
