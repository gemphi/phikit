# Layout Space — Full Implementation (Blender 5.2 Reference)

Ground truth: `phiace/REFS/blender/blender` — `DNA_workspace_types.h`, `DNA_screen_types.h`,
`editors/space_view3d/space_view3d.cc`, `editors/space_buttons/space_buttons.cc`,
`editors/space_outliner/space_outliner.cc`, `editors/space_action/space_action.cc`,
`scripts/startup/bl_ui/space_view3d.py`.

Phiber side: `crates/phiber/src/store/**` (specs) + `crates/phiber/src/bui/views/**` (render).

---

## 1. Blender's Layout workspace composition

```
┌────────────────────────────────────────────────────────────┐
│ TOPBAR (global screen area)          File Edit … tabs      │
├───────────────────────────────────────────┬────────────────┤
│ VIEW3D (center, ~72%)                     │ OUTLINER       │
│   TOOL_HEADER (active tool settings)      │   HEADER       │
│   HEADER (menus/mode/snap/shading/…)      │   WINDOW tree  │
│   ┌────┬──────────────────────┬────┐      ├────────────────┤
│   │TOOL│ WINDOW (3D viewport) │NAV │      │ PROPERTIES     │
│   │BAR │  grid/axes/cube      │GIZ │      │   HEADER       │
│   │(T) │  HUD overlay         │MO  │      │   NAV_BAR tabs │
│   └────┴──────────────────────┴────┘      │   WINDOW       │
│   UI sidebar (N) — Item/Tool/View tabs    │   panels       │
├───────────────────────────────────────────┴────────────────┤
│ TIMELINE (bottom strip)                                    │
│   HEADER (transport) │ CHANNELS │ WINDOW (ruler/scrubber)  │
├────────────────────────────────────────────────────────────┤
│ STATUSBAR (global screen area)                             │
└────────────────────────────────────────────────────────────┘
```

Areas: `View3D` (center), `Outliner` (right-top), `Properties` (right-bottom),
`Timeline` (bottom), `Topbar` + `Statusbar` (global areas on `bScreen`, not in the area grid).

---

## 2. Region inventory per host (from `ED_spacetype_*`)

| Host | Blender regions | Blender file |
|---|---|---|
| View3D | HEADER, TOOL_HEADER, TOOLS (T), UI (N), WINDOW, NAV_BAR (gizmo overlay), HUD, ASSET_SHELF + ASSET_SHELF_HEADER, XR | `space_view3d.cc:1619-1721` |
| Properties | HEADER, NAV_BAR (context tabs), WINDOW | `space_buttons.cc:1106-1155` |
| Outliner | HEADER, WINDOW | `space_outliner.cc:692` |
| Timeline (space_action) | HEADER, CHANNELS, UI, WINDOW (+ FOOTER in 4.x) | `space_action.cc:988-1037` |
| Topbar | HEADER (global) | — |
| Statusbar | FOOTER (global) | — |

Phiber's `BRegionType` already models all 17 DNA region types (`store/regions/def.rs:7-29`)
and `BPartType` has all part kinds (`store/parts/def.rs:6-26`). The vocabulary is complete —
what's missing is **assignment and content**.

---

## 3. Full content spec per host

### 3.1 View3D — the core of the Layout space

**TOOL_HEADER** (`VIEW3D_HT_tool_header`): active tool name + its operator settings
(radius, strength, falloff). Hidden for tools without settings.

**HEADER** (`VIEW3D_HT_header.draw_xform_template`, `space_view3d.py`), left→right:
- Left: editor-type switcher; mode menus (`View Select Add Object`…, mode-dependent);
  mode dropdown (Object/Edit/Sculpt/…, `BObjectMode` already exists).
- Center: transform-orientation popover; pivot-point selector; snap toggle + snapping
  popover (`VIEW3D_PT_snapping`); proportional-editing toggle + falloff popover.
- Right: shading popover (`VIEW3D_PT_shading*`) + 4 shading buttons
  (Wireframe / Solid / Material Preview / Rendered = `ShadingType` in
  `store/hosts/view3d/def.rs:5-10`); overlay toggle + popover (`VIEW3D_PT_overlay*`);
  gizmo toggle + popover (`VIEW3D_PT_gizmo_display`).

**TOOLS (T)**: vertical tool palette — Select, Cursor, Move, Rotate, Scale, Transform,
Annotate, Measure, Add Cube. Icon-only collapsed, labels expanded (current toggle behavior).

**UI (N)**: tab strip `Item | Tool | View`:
- Item: Transform panel (location/rotation/scale/dimensions of active object).
- Tool: active-tool settings + workspace tool list.
- View: focal length, clip start/end, lock, 3D cursor (`VIEW3D_PT_view3d_*`).

**WINDOW** (the `Spatial` canvas): gradient background (theme `space.gradients`),
grid floor, X/Y axes, default cube + camera + light wireframes, origin marker,
nav-gizmo (axis balls, top-right), view label ("User Perspective"), object/stats text
(top-left), FPS overlay. All driven by `View3DConfig` (`store/hosts/view3d/def.rs:12-24`):
`show_floor`, `show_axis_*`, `show_overlays`, `show_gizmos`, `shading`, `lens`, `clip_*`.

**HUD**: last-operator panel + redo; **ASSET_SHELF**: brush shelf (paint modes only).

### 3.2 Outliner
- HEADER: display-mode dropdown (Scenes / View Layer / Collections / Data API),
  filter text box (expand on focus), filter popover.
- WINDOW: tree — Scene Collection → Collection → Cube / Camera / Light, each row with
  icon + name + restriction toggles (viewport eye, render camera, select cursor).

### 3.3 Properties (Buts)
- HEADER: context title + search.
- NAV_BAR: vertical context tab strip — Tool, Render, Output, View Layer, Scene, World,
  Object, Modifiers, Particles, Physics, Constraints, Data, Material, Texture.
- WINDOW: panels for the active context tab (e.g. Object → Transform, Visibility, …).

### 3.4 Timeline
- HEADER: transport — jump-to-start, prev-keyframe, play-reverse, play, next-keyframe,
  jump-to-end; current-frame field; start/end range; playback popover; keying set popover;
  auto-keying toggle.
- CHANNELS: (empty in Timeline mode; used in Dope Sheet).
- WINDOW: frame ruler with numbers, current-frame scrubber line, marker rows.

### 3.5 Topbar / Statusbar (global)
- Topbar: app menus (File Edit Render Window Help) | workspace tabs (Layout, Modeling,
  Sculpting, UV Editing, Texture Paint, Shading, Animation, Rendering, Compositing,
  Geometry Nodes, Scripting) | right: scene picker, render engine, save indicator.
- Statusbar: keymap hints (left), modal operator status (center), right: memory, version,
  selection stats.

---

## 4. Gap analysis — phiber vs Blender

### Space level (`store/spaces/layout/set.rs`, `bui/views/space/mod.rs`)
| # | Gap | Fix |
|---|---|---|
| G1 | **No Statusbar area** — `LAYOUT_AREAS` has Top/Center/Right×2/Bottom; Blender adds a global statusbar strip under everything | Add `BAreaSpec { zone: Bottom, default_host: Statusbar }` as a *second* bottom row in `SpaceView::build` (statusbar is screen-global, renders below the Bottom zone) |
| G2 | `SpaceView` hardcodes zone order and right-column split heights; Blender areas come from a vert/edge layout (`ScrAreaMap`) | Acceptable simplification — document as such; keep fractions in `layout::set::COLUMNS` |
| G3 | Zone toggler row sits inside the Top panel; Blender hides zones via keyboard (T/N) + region overlap | Keep togglers (dev aid), but drive T/N from state keys too |

### View3D host (`store/hosts/view3d/set.rs`, `panes/set.rs`, `bui/views/host/view3d/mod.rs`)
| # | Gap | Fix |
|---|---|---|
| G4 | **No ToolHeader region** — only Header/Center/Footer; Footer doesn't exist in 5.2 View3D | Replace Footer with `BRegionType::ToolHeader` in `view3d::set::REGIONS`; panes: active-tool row |
| G5 | Toolbar/Sidebar modeled as parts of Center region; Blender has separate TOOLS/UI regions with own visibility flags (`RGN_FLAG_HIDDEN`) | Add real `Tools`/`UI` region specs to `view3d::set::REGIONS` with pane→part mapping; keep Center = viewport only |
| G6 | **No NavBar gizmo overlay** (axis balls top-right) | Add `NavBar` region (Float align) → `BPartType::NavBar`; render gizmo overlay on the Spatial canvas |
| G7 | **No HUD / AssetShelf regions** | Add region specs, default hidden (`RGN_FLAG_HIDDEN` analog in state) |
| G8 | **Header content is generic** — Menubar/Modebar/Switcher only; missing orientation, pivot, snap, proportional editing, shading buttons, overlay/gizmo toggles (`VIEW3D_HT_header`) | Extend `part/activity`-style parts: add View3D header part set (`transform_orient`, `pivot`, `snap`, `prop_edit`, `shading_btns`, `overlay_toggle`, `gizmo_toggle`) wired to `BKey` state |
| G9 | **`View3dView::build` ignores `BState`** — `View3DConfig` (shading, floor, axes, overlays, gizmos, lens, clip) is dead data | Read config keys from state in `build`; pass into `BView::spatial` |
| G10 | **`SpatialView` draws its own title bar** (`core/spatial.rs:29-43`) duplicating the region Header | Delete the inner header — the Header region already shows title/controls; canvas is content only |
| G11 | **Hardcoded viewport colors** — bg `0x1e1e24`, grid/axis colors inline (`core/spatial.rs:154,161,197`) | Move to `BPalette`: add `viewport_bg`, `viewport_grid`, `axis_x`, `axis_y`, `axis_z`, `wire_active` tokens (Blender: `ThemeSpace` gradient/grid/wire) |
| G12 | Missing default scene items: camera + light wireframes, view label, nav gizmo, stats text | Extend Spatial canvas painters (all behind `show_overlays`/`show_gizmos` flags) |
| G13 | No sidebar tabs (Item/Tool/View) in the UI part | Sidebar part for View3D gets a tab strip + per-tab panels |
| G14 | No toolbar tool palette content | Toolbar part for View3D gets the 9-tool list from §3.1 |

### Outliner / Properties / Timeline hosts
| # | Gap | Fix |
|---|---|---|
| G15 | Outliner: no display-mode dropdown, no filter box, no restriction columns | Extend `views/host/outliner/mod.rs` header part + row items with toggle icons |
| G16 | Properties: no NAV_BAR region / context tabs | Add `NavBar` region spec to `properties::set::REGIONS`; part = 14-tab context strip; center panels switch on active tab |
| G17 | Timeline: no CHANNELS region, no transport controls | Header part = transport cluster (7 buttons + frame field + range + keying); center = ruler + scrubber canvas |

### Missing primitives (block content work)
| # | Primitive | Needed by |
|---|---|---|
| G18 | `BView::popover(trigger, panel)` | Snapping, shading, overlays, gizmo popovers |
| G19 | `BView::dropdown(value, items)` | Mode selector, orientation, pivot, outliner display mode |
| G20 | `BView::tabs(active, items)` | Sidebar Item/Tool/View, Properties context nav |
| G21 | `BView::tree(nodes)` / expandable rows | Outliner |
| G22 | `BView::toggle_group(buttons)` | Shading-mode quad, transport cluster |
| G23 | `BView::ruler(range, cursor)` / scrubber canvas | Timeline |
| G24 | `BView::gizmo_overlay` region layer | NavBar/HUD overlays on Spatial |

### Missing state keys
`BKey` needs: `View3dShading`, `View3dShowFloor`, `View3dShowAxisX/Y/Z`,
`View3dShowOverlays`, `View3dShowGizmos`, `View3dToolHeader`, `View3dSidebarTab`,
`OutlinerDisplayMode`, `OutlinerFilter`, `PropsContextTab`, `TimelineFrame`,
`TimelineRange`, `TimelinePlaying`.

---

## 5. Implementation order (non-breaking, like the views migration)

1. **Theme tokens** (G11) — palette first, everything else consumes it.
2. **Spatial canvas split** (G10, G12) — remove inner header; add camera/light/gizmo/label/stats painters behind flags.
3. **State keys + config wiring** (G9, §Missing keys) — `View3dView::build` reads config.
4. **View3D regions** (G4-G7) — ToolHeader/Tools/UI/NavBar region specs + pane maps.
5. **Header parts** (G8, G18-G20, G22) — orientation/pivot/snap/shading/overlays/gizmos.
6. **Toolbar + Sidebar content** (G13, G14).
7. **Statusbar area** (G1).
8. **Outliner / Properties / Timeline content** (G15-G17, G21, G23).
9. **HUD + AssetShelf** (G7) — last, both hidden by default.
