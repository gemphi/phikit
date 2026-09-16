# Animation Space - View Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part → BView`

---

## Full View Tree

```
BSpaceGrid (Animation)
│
├─ Zone: Top
│  └─ BArea: { zone: Top, BHost: Topbar, fraction: 1.0 }
│       └─ Host: Topbar
│            └─ Header region (hflex, 32px)
│                 └─ Center pane (Relative) → Menubar part → part/menubar.rs
│
├─ Zone: Left
│  └─ BArea: { zone: Left, default_host: Outliner, fraction: 1.0 }
│       └─ Host: Outliner (switchable to Spreadsheet)
│            ├─ Header region (hflex, 32px)
│            │    └─ Center pane (Relative) → Menubar part → part/menubar.rs
│            └─ Center region (hflex, flex-1)
│                 └─ Center pane (Relative) → Custom part → part/custom.rs → effect.custom_view()
│
├─ Zone: Center
│  └─ BArea: { zone: Center, default_host: View3D, fraction: 0.5 }
│       └─ Host: View3D
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
   └─ BArea: { zone: Bottom, default_host: DopeSheet, fraction: 0.5 }
        └─ Host: DopeSheet (switchable to GraphEditor, Nla, Timeline)
             ├─ Header region (hflex, 32px)
             │    └─ Center pane (Relative) → Menubar part → part/menubar.rs (Key/Select/Marker menus)
             └─ Center region (hflex, flex-1 - 2 panes)
                  ├─ Left pane (AbsLeft, 200px)  → Channels part → part/channels.rs (channel list)
                  └─ Center pane (Relative)        → Custom part → part/custom.rs → effect.custom_view()
                                                       (dope sheet grid with keyframes)
```

---

## Render Walk

```rust
// BSpaceGrid::render() for Animation
let spec = store::spaces::space_spec(BSpaceType::Animation);
// spec.zones = [Top, Left, Center, Bottom]
// spec.areas = [Top→Topbar, Left→Outliner, Center→View3D, Bottom→DopeSheet]

for zone in spec.zones {
    for area in spec.areas.iter().filter(|a| a.zone == zone) {
        let host = state.active_host(zone).unwrap_or(area.default_host);
        let view = BHostView::new(host, theme);
    }
}
```

---

## Part Reuse

All parts shared with Layout space. DopeSheet uses Channels (channel list) and
Custom (dope sheet grid) - same `part/channels.rs` and `part/custom.rs` views.
