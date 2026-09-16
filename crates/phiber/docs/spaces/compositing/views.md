# Compositing Space - View Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part → BView`

---

## Full View Tree

```
BSpaceGrid (Compositing)
│
├─ Zone: Top
│  └─ BArea: { zone: Top, BHost: Topbar, fraction: 1.0 }
│       └─ Host: Topbar
│            └─ Header region (hflex, 32px)
│                 └─ Center pane (Relative) → Menubar part → part/menubar.rs
│
└─ Zone: Center
   └─ BArea: { zone: Center, default_host: Compositor, fraction: 1.0 }
        └─ Host: Compositor
             ├─ Header region (hflex, 32px)
             │    └─ Center pane (Relative) → Menubar part → part/menubar.rs (Node/Add/Select/View menus)
             └─ Center region (hflex, flex-1 - 3 panes)
                  ├─ Left pane (AbsLeft, 200px)   → Sidebar part → part/sidebar.rs (node categories)
                  ├─ Center pane (Relative)         → Custom part → part/custom.rs → effect.custom_view()
                  │                                    (compositor node graph)
                  └─ Right pane (AbsRight, 300px)  → Panel part → part/panel.rs (node properties)
                                                       → Preview part → part/preview.rs (backdrop viewer)
```

---

## Render Walk

```rust
// BSpaceGrid::render() for Compositing
let spec = store::spaces::space_spec(BSpaceType::Compositing);
// spec.zones = [Top, Center]
// spec.areas = [Top→Topbar, Center→Compositor]

for zone in spec.zones {
    for area in spec.areas.iter().filter(|a| a.zone == zone) {
        let host = state.active_host(zone).unwrap_or(area.default_host);
        let view = BHostView::new(host, theme);
    }
}
```

---

## Part Reuse

All parts shared with Layout space. Compositor uses Sidebar (node categories),
Custom (node graph), Panel (node properties), and Preview (backdrop viewer) -
same part views with different store spec data.
