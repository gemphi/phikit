
# Layout Space — Composition Mock

A rendered walk of the Layout space through every hierarchy level, using the
actual store statics (`store::spaces::layout::set::LAYOUT_AREAS`,
`store::panes::set::panes_for`, per-host `hosts/<name>/set::REGIONS`). Every
level below is either a `for` over a `&'static [_]` or a `match` over an enum
— and each coordinator emits **only** its own level's `BView` variant.

Companion doc: [composition-hierarchy.md](../../composition-hierarchy.md).

Blender ground truth for the Layout workspace:
`DNA_workspace_types.h`, `editors/space_view3d/space_view3d.cc`,
`editors/space_outliner/space_outliner.cc`,
`editors/space_buttons/space_buttons.cc`,
`editors/space_action/space_action.cc`.

Rule of the mock: no coordinator says `Panel`, `Row`, `Column`, or any BUI
primitive. Each level emits `BView::space / zone / area / host / region /
pane / part` and nothing else. Primitives appear only inside `Part` bodies.

**A zone is not a panel.** `BPanelRole` (`TopArea`, `Work`, `Sidebar`, …) is
a theming tag for the `BView::Panel` primitive and lives inside `Part`
bodies. Zones, areas, hosts, regions, panes carry only their store kind
(`BZoneType::Top`, `BAreaType::Bottom`, `BRegionType::Center`, …). If you
see `BView::panel(BPanelRole::TopArea, …)` at zone level, that's the exact
bug this doc exists to prevent.

---

## 1. The tree — every box is a store-level `BView` variant

```
BView::space
├── BView::zone(Top)
│   └── BView::area(Top)
│       └── BView::host(Topbar)
│           └── BView::region(Header)
│               ├── BView::pane(Left)   → BView::part(Menubar)   (File Edit Render Window Help)
│               ├── BView::pane(Center) → BView::part(Spacebar)  (Layout* Modeling Sculpting …)
│               └── BView::pane(Right)  → BView::part(Statusbar) (scene picker, engine, save)
│
├── BView::zone(Center)
│   └── BView::area(Center)
│       └── BView::host(View3D)
│           ├── BView::region(Header)
│           │   ├── BView::pane(Left)   → BView::part(Menubar)   (View Select Add Object)
│           │   ├── BView::pane(Center) → BView::part(Modebar)   (Object Mode ▾)
│           │   └── BView::pane(Right)  → BView::part(Switcher)  (View3D ▾)
│           ├── BView::region(Center)
│           │   ├── BView::pane(Left)   → BView::part(Toolbar)   (T panel)
│           │   ├── BView::pane(Center) → BView::part(Custom)    → View3dView (canvas)
│           │   └── BView::pane(Right)  → BView::part(Sidebar)   (N panel)
│           └── BView::region(Footer)   → empty (panes_for returns EMPTY_PANES)
│
├── BView::zone(Right)
│   ├── BView::area(Right)
│   │   └── BView::host(Outliner)
│   │       ├── BView::region(Header)
│   │       │   ├── BView::pane(Left)   → BView::part(Menubar)
│   │       │   ├── BView::pane(Center) → BView::part(Modebar)   (Display ▾)
│   │       │   └── BView::pane(Right)  → BView::part(Switcher)
│   │       └── BView::region(Center)
│   │           └── BView::pane(Center) → BView::part(Custom)    → tree
│   └── BView::area(Right)
│       └── BView::host(Buts)
│           ├── BView::region(Header)
│           │   ├── BView::pane(Left)   → BView::part(Menubar)
│           │   └── BView::pane(Right)  → BView::part(Switcher)
│           └── BView::region(Center)
│               ├── BView::pane(Left)   → BView::part(NavBar)    (14 context tabs)
│               └── BView::pane(Center) → BView::part(Panel)
│
└── BView::zone(Bottom)
    ├── BView::area(Bottom)
    │   └── BView::host(Time)
    │       ├── BView::region(Header)
    │       │   ├── BView::pane(Left)   → BView::part(Menubar)
    │       │   ├── BView::pane(Center) → BView::part(Modebar)   (transport)
    │       │   └── BView::pane(Right)  → BView::part(Switcher)
    │       └── BView::region(Center)
    │           ├── BView::pane(Left)   → BView::part(Channels)
    │           └── BView::pane(Center) → BView::part(Custom)    → ruler + scrubber
    └── BView::area(Bottom)
        └── BView::host(Statusbar)
            └── BView::region(Header)
                └── BView::pane(Center) → BView::part(Statusbar) (keymap • op • mem • ver)
```

How this tree becomes pixels is `bui/render.rs`'s job. That file is the
single place where `BZoneType::Bottom` becomes a vertical stack,
`BZoneType::Top` becomes a horizontal strip, `BRegionType::Center` places
its panes side-by-side, and `BAreaType::Bottom` gets the statusbar footer
height. Coordinators stay out of it.

---

## 2. The store data being read

```rust
// store/spaces/layout/set.rs — the single source of area truth
pub static LAYOUT_AREAS: &[BAreaSpec] = &[
    BAreaSpec { idx: Top,    zone: Top,    default_host: Topbar,    items: &[Topbar] },
    BAreaSpec { idx: Center, zone: Center, default_host: View3D,    items: SWITCHABLE_HOSTS },
    BAreaSpec { idx: Right,  zone: Right,  default_host: Outliner,  items: &[Outliner, File, Asset] },
    BAreaSpec { idx: Right,  zone: Right,  default_host: Buts,      items: &[Buts, Spreadsheet, Asset] },
    BAreaSpec { idx: Bottom, zone: Bottom, default_host: Time,      items: &[Time, Action, Nla, Seq] },
    BAreaSpec { idx: Bottom, zone: Bottom, default_host: Statusbar, items: &[Statusbar] },
];
```

Two `Right` slots stack in `Zone::Right` (Outliner on top, Properties below).
Two `Bottom` slots stack: Timeline strip + global Statusbar strip.

Every downstream lookup uses this static:

| Level  | Reads                                            | Kind          |
|--------|--------------------------------------------------|---------------|
| Space  | `LAYOUT_AREAS` grouped by `.zone`                | `for` filter  |
| Zone   | `self.areas` + `state.get(BKey::Zone(z))`        | visibility    |
| Area   | `state.get(BKey::AreaHost(default))`             | `unwrap_or`   |
| Host   | `host.with_set().iter()` (REGIONS)               | `for`         |
| Region | `panes_for(host, region)`                        | `for`         |
| Pane   | `pane.items` + `state.get(BKey::Expanded { … })` | `for`         |
| Part   | `match self.part` → concrete part view           | `match`       |

---

## 3. Level-by-level composition (with concrete Layout values)

### 3.1 Space — group `LAYOUT_AREAS` by zone, emit `BView::space`

Input: `state.get(BKey::CtxSpace) == "Layout"`.

```rust
impl BViewBuilder for SpaceView {
    fn build(&self, state: &BState) -> BView {
        let space = active_space(state).with_set();
        let zones: Vec<BView> = space.idx.zones().iter()
            .map(|&z| ZoneView::new(z, space.areas_in(z)).build(state))
            .filter(|v| !matches!(v, BView::Empty))
            .collect();
        BView::space(zones)
    }
}
```

`space.areas_in(z)` is a `&BSpaceSpec` method that returns
`space.items.iter().filter(|a| a.zone == z).copied().collect()` — pure
static filter, no widgets.

Zone → area count derived from `LAYOUT_AREAS`:

| Zone   | Area count | Areas (`default_host`)               |
|--------|-----------:|--------------------------------------|
| Top    |          1 | Topbar                               |
| Center |          1 | View3D                               |
| Right  |          2 | Outliner (top), Buts (bottom)        |
| Bottom |          2 | Time (strip), Statusbar (footer)     |

### 3.2 Zone — visibility only, emit `BView::zone`

Only `Left / Right / Bottom` are toggleable
(`store::zones::set::ALL_SPECS`). Layout has no `Left` zone. Toggler chrome
is a Space-level `Part`, not a Zone concern.

```rust
impl BViewBuilder for ZoneView {
    fn build(&self, state: &BState) -> BView {
        if self.areas.is_empty() || !self.zone.is_visible(state) {
            return BView::empty();
        }
        let areas: Vec<BView> = self.areas.iter()
            .map(|a| AreaView::new(*a).build(state)).collect();
        BView::zone(self.zone, areas)
    }
}
```

No `match self.zone` here. `BZoneType::Bottom` vs `BZoneType::Top` travels
on the `BView::zone` node and the renderer decodes it.

### 3.3 Area — resolve active host, emit `BView::area`

For each area, `state.get(BKey::AreaHost(default))` may override the visible
host. Default paths for Layout:

| Area (zone, idx) | `default_host` | Active host after boot |
|------------------|----------------|------------------------|
| (Top, Top)       | Topbar         | Topbar                 |
| (Center, Center) | View3D         | View3D                 |
| (Right, Right) #1| Outliner       | Outliner               |
| (Right, Right) #2| Buts           | Properties (Buts)      |
| (Bottom, Bottom) | Time           | Timeline               |
| (Bottom, Bottom) | Statusbar      | Statusbar              |

```rust
impl BViewBuilder for AreaView {
    fn build(&self, state: &BState) -> BView {
        let host = state.get(&BKey::AreaHost(self.area.default_host))
            .and_then(BHostType::from_name)
            .unwrap_or(self.area.default_host);
        BView::area(self.area.idx, HostView::new(host).build(state))
    }
}
```

### 3.4 Host — loop regions, emit `BView::host`

Regions declared per Layout host:

| Host      | REGIONS                              |
|-----------|--------------------------------------|
| Topbar    | Header                               |
| View3D    | Header, Center, Footer               |
| Outliner  | Header, Center                       |
| Buts      | Header, Center                       |
| Time      | Header, Center                       |
| Statusbar | Header                               |

```rust
impl BViewBuilder for HostView {
    fn build(&self, state: &BState) -> BView {
        let regions: Vec<BView> = self.host.with_set().iter()
            .map(|r| RegionView::new(self.host, r.idx).build(state))
            .filter(|v| !matches!(v, BView::Empty))
            .collect();
        BView::host(self.host, regions)
    }
}
```

### 3.5 Region — `panes_for(host, region)`, emit `BView::region`

`store::panes::set::panes_for(host, region)` is a pure static match. Layout
mapping:

| Host      | Region  | Pane names       | Pane parts                                    |
|-----------|---------|------------------|-----------------------------------------------|
| Topbar    | Header  | Left/Center/Right| Menubar / Spacebar / Statusbar                |
| View3D    | Header  | Left/Center/Right| Menubar / Modebar / Switcher                  |
| View3D    | Center  | Left/Center/Right| Toolbar / Custom / Sidebar                    |
| View3D    | Footer  | —                | `EMPTY_PANES`                                 |
| Outliner  | Header  | Left/Center/Right| Menubar / Modebar (display mode) / Switcher   |
| Outliner  | Center  | Center           | Custom (tree)                                 |
| Buts      | Header  | Left/(—)/Right   | Menubar / — / Switcher                        |
| Buts      | Center  | Left/Center      | NavBar (14 context tabs) / Panel              |
| Time      | Header  | Left/Center/Right| Menubar / Modebar (transport) / Switcher      |
| Time      | Center  | Left/Center      | Channels / Custom (ruler)                     |
| Statusbar | Header  | Center           | Statusbar                                     |

```rust
impl BViewBuilder for RegionView {
    fn build(&self, state: &BState) -> BView {
        let panes = panes_for(self.host, self.region);
        if panes.is_empty() { return BView::empty(); }
        let cells: Vec<BView> = panes.iter()
            .map(|p| PaneView::new(*p, self.host).build(state)).collect();
        BView::region(self.region, cells)
    }
}
```

Whether `BRegionType::Center` renders its panes side-by-side (viewport row)
or stacked (sidebar drawer) is the renderer's decision, keyed off the
`BRegionType` carried on `BView::region`. The coordinator stays out.

### 3.6 Pane — loop parts, emit `BView::pane`

```rust
impl BViewBuilder for PaneView {
    fn build(&self, state: &BState) -> BView {
        if self.pane.items.is_empty() { return BView::empty(); }
        let parts: Vec<BView> = self.pane.items.iter()
            .map(|p| PartView::new(*p, self.host).build(state)).collect();
        BView::pane(self.pane.idx, parts)
    }
}
```

`BKey::Expanded { host, part }` is read by the renderer when it lays out a
`BView::part(Toolbar | Sidebar, …)`. It never enters the coordinator.

### 3.7 Part — dispatcher (view fragments live inside)

The `match` is exhaustive over `BPartType`. Each arm returns a concrete
widget composition; only `Custom` re-routes to `host::center_body(host,
state)` for the per-host viewport content (View3D canvas, Outliner tree,
timeline ruler, properties panels, …).

```rust
impl BViewBuilder for PartView {
    fn build(&self, state: &BState) -> BView {
        let body = match self.part {
            BPartType::Menubar   => MenubarView::new(self.host, state).build(),
            BPartType::Modebar   => ModebarView::new(self.host, state).build(),
            BPartType::Switcher  => SwitcherView::new(self.host, state).build(),
            BPartType::Spacebar  => SpacebarView::new(self.host, state).build(),
            BPartType::Toolbar   => ToolbarView::new(self.host, state).build(),
            BPartType::Sidebar   => SidebarView::new(self.host, state).build(),
            BPartType::Panel     => PanelView::new(self.host, state).build(),
            BPartType::Channels  => ChannelsView::new(self.host, state).build(),
            BPartType::Statusbar => StatusbarView::new(self.host, state).build(),
            BPartType::NavBar    => NavBarView::new(self.host, state).build(),
            BPartType::Custom    => super::host::center_body(self.host, state),
            _ => BView::empty(),
        };
        BView::part(self.part, body)
    }
}
```

Inside each `*View::build` (and inside `center_body`), primitives are legal:

```rust
// bui/views/part/menubar/mod.rs — legal to use primitives here
BView::row(vec![
    BView::button("File",   BAction::Open(BKey::MenuItem { .. })),
    BView::button("Edit",   BAction::Open(BKey::MenuItem { .. })),
    BView::button("Render", BAction::Open(BKey::MenuItem { .. })),
    BView::button("Window", BAction::Open(BKey::MenuItem { .. })),
    BView::button("Help",   BAction::Open(BKey::MenuItem { .. })),
])
```

That's the only tier where a coordinator may say `Row`, `Column`, `Panel`,
`Icon`, `Label`, `Button`.

---

## 4. State reads observed while rendering Layout

Every side-effect input the mock consumes, in the order it's read:

| Level  | Key                                              | Default   | Purpose                        |
|--------|--------------------------------------------------|-----------|--------------------------------|
| Space  | `BKey::CtxSpace`                                 | `Layout`  | which space is active          |
| Zone   | `BKey::Zone(BZoneType::Right)`                   | `true`    | right column visible?          |
| Zone   | `BKey::Zone(BZoneType::Bottom)`                  | `true`    | bottom stack visible?          |
| Area   | `BKey::AreaHost(Outliner)`                       | `Outliner`| override for right-top slot    |
| Area   | `BKey::AreaHost(Buts)`                           | `Buts`    | override for right-bottom slot |
| Area   | `BKey::AreaHost(Time)`                           | `Time`    | override for bottom strip      |
| Renderer | `BKey::Expanded { host: View3D, part: Toolbar }` | `false` | T-panel expansion              |
| Renderer | `BKey::Expanded { host: View3D, part: Sidebar }` | `false` | N-panel expansion              |
| Part   | `BKey::Tab { host: View3D, part: Sidebar }`      | `Item`    | active N-panel tab             |
| Part   | `BKey::Tab { host: Buts,   part: NavBar }`       | `Object`  | properties context tab         |
| Part   | (part-specific)                                  | —         | dropdown / menu / section keys |

Everything above uses `BKey::key()` under the hood; no view constructs a raw
`&str` key. `state/context/def.rs::BContext::default()` seeds the boot
values.

---

## 5. Line-budget audit against the ≤60-line rule

Target vs. current for the coordinator layer (body LOC of `mod.rs`,
excluding doc comments and use statements):

| File                                | Current | Target | Fix                                                                                              |
|-------------------------------------|--------:|-------:|--------------------------------------------------------------------------------------------------|
| `bui/views/space/mod.rs`            |     ~78 |    ≤60 | Emit only `BView::space`. Move zone togglers into `part/zone_toggler`. Move modal overlay + activity bar into their `Part`s. |
| `bui/views/zone/mod.rs`             |     ~72 |    ≤60 | Emit only `BView::zone`. Delete `chrome` field, delete the `match self.zone` panel/column decision — move into `bui/render.rs`. |
| `bui/views/area/mod.rs`             |     ~32 |    ≤60 | Emit only `BView::area(kind, host_view)`.                                                        |
| `bui/views/host/mod.rs`             |     ~28 |    ≤60 | Emit only `BView::host`. `center_body` grows with hosts — expected and enumerable.               |
| `bui/views/region/mod.rs`           |     ~50 |    ≤60 | Emit only `BView::region`. Delete the Center-vs-non-Center panel decision — move into renderer. |
| `bui/views/pane/mod.rs`             |     ~55 |    ≤60 | Emit only `BView::pane`. Delete `PartContainer` sizing — width lookup lives in the renderer.     |
| `bui/views/part/mod.rs`             |     ~55 |    n/a | Enumerable dispatcher; wraps every arm in `BView::part(self.part, body)`.                        |

`center_body` and `part/*` are not budgeted: their content is enumerable
(one arm per host, one arm per part) and any splitting would just hide
the switch.

---

## 6. Where the current logic goes (mapping current → correct home)

The "Current leak" column is exactly the anti-pattern we're eliminating:
BUI primitives (`BView::panel`, `BView::column`, `BPanelRole`, `BPalette`
widths) surfacing at a coordinator level. Every leak has a single correct
home: either the renderer (`bui/render.rs`, keyed off the store enum on
the `BView` node) or a `Part` (widget composition).

| Current location                        | Current leak (anti-pattern)                                     | Correct home                                        |
|-----------------------------------------|-----------------------------------------------------------------|-----------------------------------------------------|
| `SpaceView::zone_togglers`              | Builds an `IconButton` row inline                               | `part/zone_toggler` (new part) or `part/activity`   |
| `SpaceView::build` (Full zone takeover) | `matches!(top, BView::Empty)` + zone reshuffling                | `render_zone(kind, children)` in `bui/render.rs`    |
| `ZoneView::build` (`match self.zone`)   | `BView::panel(BPanelRole::TopArea …)` / `BView::column(...)`    | `render_zone` in `bui/render.rs`                    |
| `ZoneView::build` (statusbar height)    | Reads `BPalette::h_footer` at zone level                        | `render_area(BAreaType::Bottom, …)` in renderer     |
| `AreaView::build`                       | `BView::panel_sized(self.area.idx.panel_role(), …)`             | `BView::area(self.area.idx, host_view)` — renderer decides height/role |
| `RegionView::build` (Center split)      | `flat_map` panes → `BView::panel(BPanelRole::Work, …)`          | `render_region(BRegionType::Center, …)` in renderer |
| `PaneView::build`                       | `BView::panel(self.pane.idx.panel_role(), …)`                   | `BView::pane(self.pane.idx, parts)` — renderer paints |
| `PartContainer::build` (Toolbar width)  | Reads `BKey::Expanded` + `BPalette::w_*` at pane wrapper level  | `render_part(BPartType::Toolbar, …)` in renderer    |
| `HostView::build` (empty filter)        | `.filter(|v| !matches!(v, BView::Empty))`                       | keep at coordinator — enum-only check               |

None of these require new store types. All rows are pure moves — the
information is already there, it just needs to travel on the store-level
`BView` variant instead of being re-encoded as `BPanelRole`.

---

## 7. Cross-reference: enforced by tests

- `tests/store_blender_tests.rs::test_bspec_hierarchy_and_roles` walks the
  `BSpec` trait to assert every level round-trips `BSpecType`. `BPanelRole`
  is a Part-body concern; it is asserted only against `BView::Panel` inside
  part views, never against a coordinator.
- `spaces_and_hosts.md::print_hierarchy()` produces a text dump of the whole
  tree via nested `for` loops — the same walk the views do at render time.
- The `Layout` output in `spaces_and_hosts.md` matches the mock in §1
  section-for-section.
- New lint proposal `views_no_primitives_above_part.rs` enforces the
  coordinator purity rule: inside `bui/views/{space,zone,area,host,region,
  pane}/`, the only allowed `BView::*` calls are the matching level's
  variant plus `empty`. Any `Panel / Row / Column / Separator / Label /
  Icon / Button` above `Part` is a lint failure.
