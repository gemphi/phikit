# Scripting Space - View Structure

> **Hierarchy:** `Space → Zone → BArea → Host → Region → Pane → Part → BView`

---

## Full View Tree

```
BSpaceGrid (Scripting)
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
│  └─ BArea: { zone: Center, default_host: TextEditor, fraction: 1.0 }
│       └─ Host: TextEditor
│            ├─ Header region (hflex, 32px)
│            │    └─ Center pane (Relative) → Menubar part → part/menubar.rs (File/Edit/Format/View menus)
│            └─ Center region (hflex, flex-1)
│                 └─ Center pane (Relative) → Custom part → part/custom.rs → effect.custom_view()
│                                                      (text editor with syntax highlighting)
│
└─ Zone: Right
   └─ BArea: { zone: Right, default_host: Info, fraction: 1.0 }
        └─ Host: Info (switchable to PythonConsole)
             ├─ Header region (hflex, 32px)
             │    └─ Center pane (Relative) → Menubar part → part/menubar.rs
             └─ Center region (hflex, flex-1)
                  └─ Center pane (Relative) → Custom part → part/custom.rs → effect.custom_view()
                                                       (info log / console output)
```

---

## Render Walk

```rust
// BSpaceGrid::render() for Scripting
let spec = store::spaces::space_spec(BSpaceType::Scripting);
// spec.zones = [Top, Left, Center, Right]
// spec.areas = [Top→Topbar, Left→Outliner, Center→TextEditor, Right→Info]

for zone in spec.zones {
    for area in spec.areas.iter().filter(|a| a.zone == zone) {
        let host = state.active_host(zone).unwrap_or(area.default_host);
        let view = BHostView::new(host, theme);
    }
}
```

---

## Part Reuse

All parts shared with Layout space. TextEditor and Info both use Custom part
(`part/custom.rs`) - the same view, just delegating to different `effect.custom_view()`
implementations (text editor vs. info log).
