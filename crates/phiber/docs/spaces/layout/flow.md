# Layout Space - Flow Diagrams

> **Blender references:**
> - `BKE_screen.h` - `SpaceType` registration, `ARegionType` draw callbacks
> - `space_api/spacetypes.c` - `ED_spacetypes_init()` calls each `ED_spacetype_*()`
> - `DNA_screen_types.h` - `bScreen` → `ScrArea` → `ARegion` hierarchy
> - Blender Manual: [`interface/window_system/introduction.rst`](../../blender-manual/manual/interface/window_system/introduction.rst)

---

## 1. Boot / Registration Flow

```
┌─────────────────────────────────────────────────────────────┐
│                        APP STARTUP                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. BHostRegistry::init()                                    │
│     └─ registers all BHostEffect impls (View3D, Outliner,  │
│        Properties, Timeline, etc.) keyed by BHostType       │
│     └─ each effect provides: regions(), custom_view()      │
│                                                              │
│  2. Store::init()                                            │
│     └─ loads static specs from store/spaces/*/mod.rs        │
│     └─ loads static specs from store/hosts/*/mod.rs         │
│     └─ loads static specs from store/regions/mod.rs         │
│     └─ loads static specs from store/panes/mod.rs           │
│     └─ loads static specs from store/parts/mod.rs           │
│                                                              │
│  3. BContext::default()                                      │
│     └─ active_space = BSpaceType::Layout                     │
│     └─ active_area = BZoneType::Center                   │
│     └─ active_host = BHostType::View3D                       │
│     └─ mode = BMode::Object                                  │
│                                                              │
│  4. First Render                                             │
│     └─ BSpaceGrid::new(space_spec, BHost_registry, ...)      │
│     └─ walks space → areas → hosts → regions → panes → parts│
│     └─ produces BView tree                                   │
│     └─ BRenderer converts BView → GPUI elements             │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**Blender parallel:** `ED_spacetypes_init()` in `spacetypes.c` registers each
space type once at startup. Each `ED_spacetype_*()` function allocates a
`SpaceType`, sets its `spaceid`, `name`, `iconid`, and registers `ARegionType`
entries with `init`/`draw`/`listener` callbacks.
(See: `BKE_screen.h`, `space_api/spacetypes.c`)

---

## 2. Render Flow (per frame)

```
┌──────────────────────────────────────────────────────────────┐
│                       RENDER FRAME                            │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  BSpaceGrid::render(window, cx)                               │
│  │                                                            │
│  ├─ lookup space_spec for ctx.active_space                    │
│  │  (BSpaceType::Layout → store/spaces/layout/mod.rs)        │
│  │                                                            │
│  ├─ for each zone in space_spec.columns:                     │
│  │  │                                                         │
│  │  ├─ match zone:                                        │
│  │  │   BZoneType::Top    → render as full-width strip    │
│  │  │   BZoneType::Left   → render as left column         │
│  │  │   BZoneType::Center → render as center column       │
│  │  │   BZoneType::Right  → render as right column        │
│  │  │   BZoneType::Bottom → render as bottom strip        │
│  │  │                                                         │
│  │  ├─ for each area in zone:                                │
│  │  │  │                                                      │
│  │  │  ├─ determine active host for this area               │
│  │  │  │  (from SpaceState::active_host(area.zone))          │
│  │  │  │                                                      │
│  │  │  ├─ lookup BHostEffect from registry                  │
│  │  │  │  (BHostType::View3D → View3DHost effect)          │
│  │  │  │                                                      │
│  │  │  ├─ BHostView::new(effect, ...).render(window, cx)   │
│  │  │  │                                                      │
│  │  │  ├─ for each region in effect.regions():                │
│  │  │  │  │                                                   │
│  │  │  │  ├─ match region:                                    │
│  │  │  │  │   BRegionType::Header → render_header(...)        │
│  │  │  │  │   BRegionType::Center → render_center(...)        │
│  │  │  │  │   BRegionType::Footer → render_footer(...)        │
│  │  │  │  │                                                   │
│  │  │  │  ├─ for each pane in region_panes(region):           │
│  │  │  │  │  │                                                │
│  │  │  │  │  ├─ match (pane.pane_type, pane.fixed):            │
│  │  │  │  │   (Left,   BDockType::AbsLeft)  → docked left │
│  │  │  │  │   (Center, BDockType::Relative) → flex fill   │
│  │  │  │  │   (Right,  BDockType::AbsRight) → docked right│
│  │  │  │  │  │                                                │
│  │  │  │  │  ├─ for each part in pane_parts(pane):            │
│  │  │  │  │  │  │                                             │
│  │  │  │  │  │  ├─ match (region, pane_type, part):          │
│  │  │  │  │  │   (_, _, Menubar)  → BAreaMenubar::to_bview│
│  │  │  │  │  │   (_, _, Toolbar)  → BAreaToolbar::to_bview│
│  │  │  │  │  │   (_, _, Sidebar)  → BAreaSidebar::to_bview│
│  │  │  │  │  │   (_, _, Panel)    → BAreaPanel::to_bview  │
│  │  │  │  │  │   (_, _, Modebar)  → BAreaModebar::to_bview│
│  │  │  │  │  │   (_, _, Channels) → BAreaChannels::to_bview│
│  │  │  │  │  │   (_, _, Statusbar)→ BAreaStatusbar::to_bview│
│  │  │  │  │   │   (_, _, Custom)   → effect.custom_view() │
│  │  │  │  │  │  │                                             │
│  │  │  │  │  │  └─ produce BView node                         │
│  │  │  │  │  │                                                │
│  │  │  │  │  └─ pane container (grid/vflex/hflex) wraps parts│
│  │  │  │  │                                                   │
│  │  │  │  └─ region container wraps panes                     │
│  │  │  │                                                      │
│  │  │  └─ host container wraps regions → AnyElement           │
│  │  │                                                         │
│  │  └─ area container wraps host → grid child                 │
│  │                                                            │
│  └─ space grid wraps areas → final AnyElement                 │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

**Key principle:** Every `match` arm is driven by enum values from static specs.
No `if` statements. No runtime type checking. Just pattern matching.

---

## 3. Data Flow (store → state → view)

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│    STORE     │     │    STATE     │     │    VIEWS     │
│ (static specs)│    │ (runtime)    │     │ (BView tree) │
├──────────────┤     ├──────────────┤     ├──────────────┤
│              │     │              │     │              │
│ SpaceSpec    │────▶│ SpaceState   │────▶│ BSpaceGrid   │
│  .areas      │     │  .active_    │     │  .columns    │
│  .grid       │     │   host[]     │     │  .areas      │
│              │     │              │     │              │
│ HostSpec     │────▶│ HostState    │────▶│ BHostView    │
│  .regions    │     │  .region_    │     │  .regions    │
│  .menus      │     │   visible{}  │     │              │
│  .panes      │     │  .mode       │     │              │
│  .parts      │     │  .zoom       │     │              │
│              │     │              │     │              │
│ RegionSpec   │────▶│ (visibility  │────▶│ BView::Region│
│  .kind       │     │  flags only) │     │  (3 types)   │
│  (3 types)   │     │              │     │              │
│              │     │              │     │              │
│ PaneSpec     │────▶│ (sizing from │────▶│ BView::      │
│  .pane_type  │     │  state +     │     │  Grid/Flex   │
│  .fixed      │     │  fixed type) │     │              │
│  .width      │     │              │     │              │
│              │     │              │     │              │
│ PartSpec     │────▶│ (no runtime  │────▶│ BArea*::     │
│  .part_type  │     │  state -     │     │  to_bview()  │
│  .items      │     │  pure data)  │     │              │
│              │     │              │     │              │
└──────────────┘     └──────────────┘     └──────────────┘
     │                     │                     │
     │     ┌───────────────┘                     │
     │     │                                     │
     │     ▼                                     │
     │  ┌──────────┐                             │
     │  │ BContext │◀──────── user input ────────│
     │  │ (active  │                             │
     │  │  space,  │                             │
     │  │  area,   │                             │
     │  │  host,   │                             │
     │  │  mode)   │                             │
     │  └──────────┘                             │
     │     │                                     │
     └─────┘                                     │
     (store specs are read                       │
      during render, not mutated)                │
                                               │
                                    BRenderer renders
                                    BView → GPUI elements
```

**Blender parallel:** Blender's `bContext` is passed to every draw callback.
The `bContext` provides access to `scene`, `area`, `region`, `space_data`.
Our `BContext` mirrors this: it tracks what's active, and the render walk
reads from both store specs and state.

---

## 4. State Mutation Flow (user interaction)

```
┌─────────────────────────────────────────────────────────────┐
│                    USER INTERACTION                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  User clicks area header icon                                │
│  │                                                           │
│  ├─ on_area_click(idx, x, y, window, cx)                    │
│  │   └─ ctx.focus_area(zone, BHost_type)            │
│  │      └─ updates BContext:                                 │
│  │         .active_area = BZoneType::Center              │
│  │         .active_host = BHostType::View3D                  │
│  │         .mode = BMode::for_host(BHostType::View3D)       │
│  │                                                           │
│  ├─ cx.notify() → triggers re-render                         │
│  │                                                           │
│  └─ Render flow runs again (see diagram 2)                   │
│                                                              │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ │
│                                                              │
│  User presses 'N' (toggle sidebar)                           │
│  │                                                           │
│  ├─ keymap action: toggle_pane(BHostType::View3D,           │
│  │                       BRegionType::Center, BPaneType::Right)│
│  │   └─ PaneState::set_visible_toggle()    │
│  │  │   └─ pane.visible = !pane.visible                     │
│  │                                                           │
│  ├─ cx.notify() → re-render                                  │
│  │                                                           │
│  └─ Render flow: Right pane in Center region hidden/visible │
│                                                              │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ │
│                                                              │
│  User switches host in right area (Properties → Outliner)    │
│  │                                                           │
│  ├─ SpaceState::set_host(BZoneType::Right,                │
│  │                              BHostType::Outliner)        │
│  │   └─ active_hosts[Right] = BHostType::Outliner           │
│  │                                                           │
│  ├─ cx.notify() → re-render                                  │
│  │                                                           │
│  └─ Render flow: Right area now shows Outliner effect        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 5. Store → State → View Mapping (Layout Space)

```
STORE                          STATE                    VIEW
─────                          ─────                    ────

spaces/layout/                 space/mod.rs             BSpaceGrid
  mod.rs                        SpaceState               ├─ TopArea
  def.rs                        .active_hosts = {        │  └─ TopbarHost
    areas: [                      Right: Properties,     │     ├─ Header region
      Top → [Topbar],             Left: Outliner,        │     │  └─ Center pane → Menubar part
      Left → [Outliner],          Center: View3D,        │     └─ Center region
      Center → [View3D],          Bottom: Timeline,     │        └─ Center pane → Custom part
      Right → [Properties,       }                       │
               Outliner],        .region_visible = {     ├─ LeftArea
      Bottom → [Timeline],         (View3D, Header): true,│  └─ OutlinerHost
    ]                              (View3D, Center): true,│     ├─ Header region
                                 }                       │     │  └─ Center pane → Menubar part
                                                        │     └─ Center region
hosts/view3d/                  area/view3d.rs            │        └─ Center pane → Custom part
  mod.rs                        View3DState              │
  def.rs                        .mode = Object           ├─ CenterArea
    regions: [                  .zoom = 1.0              │  └─ View3DHost
      Header → panes: [         .rails = {40, 40}       │     ├─ Header region
        Left → [Modebar],       .shading = Solid         │     │  ├─ Left pane → Modebar part
        Center → [Menubar],     .menubar = true          │     │  └─ Center pane → Menubar part
      ],                                                 │     └─ Center region
      Center → panes: [                                  │        ├─ Left pane (AbsLeft, 44px)
        Left → [Toolbar],                                │        │  └─ Toolbar part
        Center → [Custom],                               │        ├─ Center pane (Relative)
        Right → [Sidebar],                               │        │  └─ Custom part (viewport)
      ],                                                 │        └─ Right pane (AbsRight, 300px)
    ]                                                    │           └─ Sidebar part
                                                           ├─ RightArea
                                                           │  └─ PropertiesHost (switchable)
                                                           │     ├─ Header region
                                                           │     │  └─ Center pane → Menubar part
                                                           │     └─ Center region
                                                           │        └─ Center pane → Panel part
                                                           │
                                                           └─ BottomArea
                                                              └─ TimelineHost
                                                                 ├─ Header region
                                                                 └─ Center region
```

---

## 6. Pane Layout Flow (the grid within)

```
Region: Header (hflex)
┌─────────────────────────────────────────────────┐
│  Pane: Left (BDockType::Relative)  │  Pane:    │
│  ┌─────────────────────────────────┐│  Center   │
│  │  Part: Modebar                  ││  (BFloat  │
│  │  [Object][Edit][Sculpt]         ││  Type::   │
│  └─────────────────────────────────┘│  Relative)│
│                                      │  ┌──────┐ │
│                                      │  │Part: │ │
│                                      │  │Header│ │
│                                      │  └──────┘ │
└─────────────────────────────────────────────────┘

Region: Center (hflex - 3 panes: toolbar | viewport | sidebar)
┌──────────┬─────────────────────────────┬──────────────────┐
│ Pane:    │ Pane: Center                │ Pane: Right      │
│ Left     │ (BDockType::Relative       │ (BDockType::    │
│(BFloat:: │  → flex fill)               │  AbsRight,       │
│ AbsLeft, │                             │  300px)          │
│ 44px)    │ ┌─────────────────────────┐ │ ┌──────────────┐ │
│┌────────┐│ │ Part: Custom            │ │ │Part: Sidebar │ │
││Part:   ││ │ (3D Viewport)           │ │ │[Item][Tool]  │ │
││Toolbar ││ │                         │ │ │[View]        │ │
││[Select]││ │ ┌─────────────────────┐ │ │ ├──────────────┤ │
││[Move]  ││ │ │ (GPUI raw element)  │ │ │ │Part: Panel   │ │
││[Rotate]││ │ │                     │ │ │ │ - Transform  │ │
││[Scale] ││ │ │                     │ │ │ │ - Delta      │ │
││[Annot] ││ │ └─────────────────────┘ │ │ └──────────────┘ │
│└────────┘│ └─────────────────────────┘ │                  │
└──────────┴─────────────────────────────┴──────────────────┘

**Key:** The old Tools/Window/Pane regions are gone. Everything is in
the Center region with 3 panes: Left (toolbar, BDockType::AbsLeft, 44px),
Center (viewport, BDockType::Relative), Right (sidebar, BDockType::AbsRight, 300px).
```

**Pane float types:**
- `BDockType::AbsLeft` - docked left with fixed pixel width (e.g. toolbar 44px)
- `BDockType::AbsRight` - docked right with fixed pixel width (e.g. sidebar 300px)
- `BDockType::Relative` - relative, flex-fills remaining space

The pane container itself is a `BView::Grid`, `BView::Row` (hflex), or
`BView::Column` (vflex) depending on the region's layout direction.
