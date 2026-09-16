
# Composition Hierarchy — Loops + Enum Matches, No Logic Until Part

The BUI view tree is a pure composition of **static store relations** and
**runtime state**. Every level from `Space` down to `Pane` is a coordinator: it
loops the store's items and matches enums to hand off to the next level. Actual
view fragments — buttons, labels, icons, spatial canvases — are built **only at
the `Part` level** (and per-host `center_body`).

The single most important rule:

> **A coordinator emits exactly one `BView` variant with the same name as its
> level, and never speaks in BUI primitives.**
>
> `SpaceView` emits `BView::space`. `ZoneView` emits `BView::zone`. `AreaView`
> emits `BView::area`. `HostView` emits `BView::host`. `RegionView` emits
> `BView::region`. `PaneView` emits `BView::pane`. Only `Part` views compose
> primitives (`Row`, `Column`, `Panel`, `Icon`, `Label`, …).

And its corollary:

> **A zone is not a panel. An area is not a panel. A region is not a panel.
> A pane is not a panel.**
>
> `BPanelRole` is a primitive-body concept for theming `BView::Panel`. It
> does not describe zones, areas, hosts, regions, or panes. A zone's
> identity is `BZoneType::Top` — that is all the information the renderer
> needs to lay it out. Nothing in a coordinator ever writes
> `BView::panel(BPanelRole::TopArea, …)`; that reads as "wrap this zone's
> content in a themed rectangle called TopArea", which is exactly the
> primitive-thinking the rule forbids.

Rendering `Zone::Bottom` as a vertical stack or `Zone::Top` as a horizontal
strip is the **renderer's** job (`bui/render.rs`), not the coordinator's. The
coordinator only knows: "this is a zone, this is its kind, these are its
areas". Interpreting `BZoneType::Bottom` → column and `BZoneType::Top` → row is
part of turning `BView` into GPUI, and it lives in one file next to the theme.

If a `mod.rs` at `Space / Zone / Area / Host / Region / Pane` produces anything
other than its own level's `BView` variant, that logic belongs somewhere else:

- Structural shape (which zones exist, which areas sit in each zone, which
  hosts an area accepts, which regions a host has, which panes a region has,
  which parts a pane owns) → **`store/`** (static).
- What the user is currently looking at (active space, active host per area,
  visible zones, expanded parts, dropdown state) → **`state/`** + `bui/state/`
  (runtime).
- How each level lays out its children (row vs column, gaps, gradients,
  borders) → **`bui/render.rs`**, keyed off the level enum.
- Concrete widget composition (menus, toggles, tool icons, canvases, sidebar
  tabs, panels) → **`bui/views/part/`** and **`bui/views/host/<name>/`**.

## The 60-line rule

Every `bui/views/<level>/mod.rs` for `space / zone / area / host / region /
pane` targets **≤60 lines of body** (plus doc comments and use statements). If
a level grows beyond that, one of these is happening:

1. Structure that belongs in the **store** is being reinvented at runtime →
   move it into `store/<level>s/set.rs`.
2. Actual widget composition is leaking up → move it down into a `Part` view
   (or into the host-specific `center_body`).
3. BUI primitives (`Panel`, `Column`, `Row`) are being used at a coordinator
   → replace with the level's own `BView` variant and push the layout choice
   into `bui/render.rs`.

`Part` files (and `bui/views/host/<name>/mod.rs`) are the only place where the
line budget is content-driven — they hold the real widgets.

## The pipeline

```
BSpaceType ─┐
            ▼
    store::spaces::set     BSpaceSpec.items: &[BAreaSpec]
            │
            ▼  loop areas, group by zone
    store::zones           BZoneType.with_set → BZoneSpec (toggle data)
            │
            ▼  match zone kind only in the RENDERER
    store::areas           BAreaSpec { zone, default_host, items: &[BHostType] }
            │
            ▼  read state.get(BKey::AreaHost(default))
    store::hosts::set      BHostSpec.regions: &[BRegionSpec]
            │
            ▼  loop host regions
    store::regions         BRegionType (kind carried on BView::region)
            │
            ▼  panes_for(host, region) → &[BPaneSpec]
    store::panes           BPaneSpec.items: &[BPart]
            │
            ▼  loop pane parts
    store::parts::def      BPartType → concrete part view
            │
            ▼  match part
    bui/views/part/        ← view fragments live here
    bui/views/host/<n>/    ← per-host viewport content lives here
```

Every arrow is either `for` over a `&'static [_]` or a `match` over an enum.
No cross-cutting lookups. No level walks anything but its own children. No
level renames a store term into a BUI primitive.

## BView surface — one variant per hierarchy level

```rust
pub enum BView {
    // ── Store-level containers: one per hierarchy tier ──
    //    Each carries only its store kind — no BPanelRole, no width, no gap.
    //    Physical layout for each kind is decided in bui/render.rs.
    Space  { children: Vec<BView> },
    Zone   { kind: BZoneType,   children: Vec<BView> },
    Area   { kind: BAreaType,   child:    Box<BView>  },   // area holds one active host
    Host   { kind: BHostType,   children: Vec<BView> },
    Region { kind: BRegionType, children: Vec<BView> },
    Pane   { kind: BPaneType,   children: Vec<BView> },
    Part   { kind: BPartType,   body:     Box<BView>  },   // body is the widget tree

    // ── Widget primitives — legal ONLY inside a Part.body ──
    //    These carry BPanelRole, gaps, widths, etc. because they are the
    //    actual visual atoms. Nothing above Part touches them.
    Row      { children: Vec<BView> },
    Column   { children: Vec<BView> },
    Panel    { role: BPanelRole, children: Vec<BView> },
    Label    { text: String },
    Icon     { name: String },
    Button   { icon: String, action: BAction },
    // …existing widget variants…
    Empty,
}

impl BView {
    pub fn space (children: Vec<BView>)                        -> Self { … }
    pub fn zone  (kind: BZoneType,   children: Vec<BView>)     -> Self { … }
    pub fn area  (kind: BAreaType,   child:    BView)          -> Self { … }
    pub fn host  (kind: BHostType,   children: Vec<BView>)     -> Self { … }
    pub fn region(kind: BRegionType, children: Vec<BView>)     -> Self { … }
    pub fn pane  (kind: BPaneType,   children: Vec<BView>)     -> Self { … }
    pub fn part  (kind: BPartType,   body:     BView)          -> Self { … }
}
```

The renderer is the sole owner of "how does a `Zone` lay out":

```rust
// bui/render.rs — the only file that interprets store terms into primitives.
fn render(node: &BView) -> gpui::AnyElement {
    match node {
        BView::Space { children }         => v_stack(children),
        BView::Zone  { kind, children }   => render_zone(*kind, children),
        BView::Area  { kind, child }      => render_area(*kind, child),
        BView::Host  { kind, children }   => render_host(*kind, children),
        BView::Region{ kind, children }   => render_region(*kind, children),
        BView::Pane  { kind, children }   => render_pane(*kind, children),
        BView::Part  { kind, body }       => render_part(*kind, body),
        BView::Row   { children }         => h_stack(children),
        BView::Column{ children }         => v_stack(children),
        BView::Panel { role, children }   => render_panel(*role, children),
        // …
    }
}

fn render_zone(kind: BZoneType, children: &[BView]) -> gpui::AnyElement {
    match kind {
        BZoneType::Top    | BZoneType::Bottom => v_stack_of_rows(children),
        BZoneType::Left   | BZoneType::Right  => v_stack(children),
        BZoneType::Center | BZoneType::Full   => h_stack(children),
    }
}
```

That switch is the only place `BZoneType` is decoded into physical layout.

## The List / Item split — one shape at every level

Each hierarchy tier is a pair of small views:

- **`<Level>ListView`** — receives a store collection (`&[BAreaSpec]`,
  `&[BRegionSpec]`, `&[BPaneSpec]`, `&[BPart]`, …), loops it, and returns
  `Vec<BView>`. It never emits a `BView` node itself — it produces the
  children of the enclosing `ItemView`. Pure iteration, zero state
  resolution.
- **`<Level>ItemView`** — receives one store spec, resolves whatever runtime
  state matters at that level (visibility, active host, expansion), and
  emits exactly one `BView::<level>` variant, populated by calling the
  next tier's `ListView`.

The delegation chain is uniform top-to-bottom:

```
SpaceItemView   ─ BView::space   ← ZoneListView.items()
ZoneItemView    ─ BView::zone    ← AreaListView.items()
AreaItemView    ─ BView::area    ← HostItemView.build()      (area holds one active host)
HostItemView    ─ BView::host    ← RegionListView.items()
RegionItemView  ─ BView::region  ← PaneListView.items()
PaneItemView    ─ BView::pane    ← PartListView.items()
PartItemView    ─ BView::part    ← concrete part view (widget tree)
```

`ListView` is the "for-loop". `ItemView` is the "match + emit-one-node".
This is why every coordinator file stays trivially under 60 lines: each
file does exactly one of those two jobs.

### File layout

```
bui/views/
├── mod.rs                   — BViewBuilder trait, level re-exports
├── space/
│   ├── mod.rs
│   ├── list.rs              — SpaceListView (picks the active space)
│   └── item.rs              — SpaceItemView
├── zone/
│   ├── mod.rs
│   ├── list.rs              — ZoneListView (iterates the space's zones)
│   └── item.rs              — ZoneItemView
├── area/
│   ├── mod.rs
│   ├── list.rs              — AreaListView (iterates a zone's areas)
│   └── item.rs              — AreaItemView
├── host/
│   ├── mod.rs               — dispatcher for per-host center content
│   ├── list.rs              — HostListView (switchable-host tab strip)
│   ├── item.rs              — HostItemView
│   ├── view3d/              — concrete center content per host
│   ├── outliner/
│   └── …
├── region/
│   ├── mod.rs
│   ├── list.rs
│   └── item.rs
├── pane/
│   ├── mod.rs
│   ├── list.rs
│   └── item.rs
└── part/
    ├── mod.rs               — dispatcher for concrete part widgets
    ├── list.rs
    ├── item.rs
    ├── menubar/             — concrete widget composition
    ├── modebar/
    ├── toolbar/
    └── …
```

Every `list.rs` is 5–15 lines. Every `item.rs` is 10–25 lines. The
per-host and per-part concrete widget folders are where widget code
actually lives (unbudgeted).

## Per-level template

Each `list.rs` and `item.rs` collapses to this shape. Anything longer is a
smell — probably widget composition or role/palette leaking upward.

### Space

```rust
// space/list.rs — picks the active space, delegates
pub struct SpaceListView;
impl SpaceListView {
    pub fn items(&self, state: &BState) -> Vec<BView> {
        vec![SpaceItemView::new(active_space(state)).build(state)]
    }
}

// space/item.rs — one space
pub struct SpaceItemView { pub space: BSpaceType }
impl BViewBuilder for SpaceItemView {
    fn build(&self, state: &BState) -> BView {
        let spec = self.space.with_set();
        BView::space(ZoneListView::new(spec).items(state))
    }
}
```

### Zone

```rust
// zone/list.rs — iterate a space's zones
pub struct ZoneListView { space: &'static BSpaceSpec }
impl ZoneListView {
    pub fn items(&self, state: &BState) -> Vec<BView> {
        self.space.idx.zones().iter()
            .map(|&z| ZoneItemView::new(z, self.space.areas_in(z)).build(state))
            .filter(|v| !matches!(v, BView::Empty))
            .collect()
    }
}

// zone/item.rs — one zone
pub struct ZoneItemView { zone: BZoneType, areas: Vec<BAreaSpec> }
impl BViewBuilder for ZoneItemView {
    fn build(&self, state: &BState) -> BView {
        if self.areas.is_empty() || !self.zone.is_visible(state) {
            return BView::empty();
        }
        BView::zone(self.zone, AreaListView::new(&self.areas).items(state))
    }
}
```

No `match self.zone` anywhere. `BZoneType::Top` vs `BZoneType::Bottom`
travels on the `BView::zone` node and the renderer decodes orientation and
theming once, in one place.

### Area

```rust
// area/list.rs — iterate a zone's areas
pub struct AreaListView<'a> { areas: &'a [BAreaSpec] }
impl<'a> AreaListView<'a> {
    pub fn items(&self, state: &BState) -> Vec<BView> {
        self.areas.iter().map(|a| AreaItemView::new(*a).build(state)).collect()
    }
}

// area/item.rs — one area, resolves active host
pub struct AreaItemView { area: BAreaSpec }
impl BViewBuilder for AreaItemView {
    fn build(&self, state: &BState) -> BView {
        let host = state.get(&BKey::AreaHost(self.area.default_host))
            .and_then(BHostType::from_name)
            .unwrap_or(self.area.default_host);
        BView::area(self.area.idx, HostItemView::new(host).build(state))
    }
}
```

### Host

```rust
// host/list.rs — switchable-host tab strip (used by the Switcher part)
pub struct HostListView<'a> { hosts: &'a [BHostType] }
impl<'a> HostListView<'a> {
    pub fn items(&self, state: &BState) -> Vec<BView> {
        self.hosts.iter().map(|&h| HostItemView::new(h).build(state)).collect()
    }
}

// host/item.rs — one host
pub struct HostItemView { host: BHostType }
impl BViewBuilder for HostItemView {
    fn build(&self, state: &BState) -> BView {
        BView::host(self.host, RegionListView::new(self.host).items(state))
    }
}
```

Per-host center content (viewport canvas, tree, ruler, …) lives in
`bui/views/host/<name>/` and is invoked by the `Part` dispatcher via the
`Custom` arm — the host coordinator never touches viewport pixels.

### Region

```rust
// region/list.rs
pub struct RegionListView { host: BHostType }
impl RegionListView {
    pub fn items(&self, state: &BState) -> Vec<BView> {
        self.host.with_set().iter()
            .map(|r| RegionItemView::new(self.host, r.idx).build(state))
            .filter(|v| !matches!(v, BView::Empty))
            .collect()
    }
}

// region/item.rs
pub struct RegionItemView { host: BHostType, region: BRegionType }
impl BViewBuilder for RegionItemView {
    fn build(&self, state: &BState) -> BView {
        let panes = panes_for(self.host, self.region);
        if panes.is_empty() { return BView::empty(); }
        BView::region(self.region, PaneListView::new(self.host, panes).items(state))
    }
}
```

Whether `BRegionType::Center` renders its panes side-by-side (viewport row)
or stacked (sidebar drawer) is the renderer's decision, keyed off the
`BRegionType` carried on `BView::region`.

### Pane

```rust
// pane/list.rs
pub struct PaneListView { host: BHostType, panes: &'static [BPaneSpec] }
impl PaneListView {
    pub fn items(&self, state: &BState) -> Vec<BView> {
        self.panes.iter().map(|p| PaneItemView::new(*p, self.host).build(state)).collect()
    }
}

// pane/item.rs
pub struct PaneItemView { pane: BPaneSpec, host: BHostType }
impl BViewBuilder for PaneItemView {
    fn build(&self, state: &BState) -> BView {
        if self.pane.items.is_empty() { return BView::empty(); }
        BView::pane(self.pane.idx, PartListView::new(self.host, self.pane.items).items(state))
    }
}
```

`BKey::Expanded { host, part }` is read by the renderer when it lays out a
`BView::part(Toolbar | Sidebar, …)`. It never enters the coordinator.

### Part — the only dispatcher, primitives live below

```rust
// part/list.rs
pub struct PartListView { host: BHostType, parts: &'static [BPart] }
impl PartListView {
    pub fn items(&self, state: &BState) -> Vec<BView> {
        self.parts.iter().map(|p| PartItemView::new(p.idx, self.host).build(state)).collect()
    }
}

// part/item.rs
pub struct PartItemView { part: BPartType, host: BHostType }
impl BViewBuilder for PartItemView {
    fn build(&self, state: &BState) -> BView {
        let body = match self.part {
            BPartType::Menubar   => menubar::MenubarView::new(self.host, state).build(),
            BPartType::Modebar   => modebar::ModebarView::new(self.host, state).build(),
            BPartType::Switcher  => switcher::SwitcherView::new(self.host, state).build(),
            BPartType::Spacebar  => spacebar::SpacebarView::new(self.host, state).build(),
            BPartType::Toolbar   => toolbar::ToolbarView::new(self.host, state).build(),
            BPartType::Sidebar   => sidebar::SidebarView::new(self.host, state).build(),
            BPartType::Panel     => panel::PanelView::new(self.host, state).build(),
            BPartType::Channels  => channels::ChannelsView::new(self.host, state).build(),
            BPartType::Statusbar => statusbar::StatusbarView::new(self.host, state).build(),
            BPartType::NavBar    => navbar::NavBarView::new(self.host, state).build(),
            BPartType::Custom    => super::host::dispatch_center(self.host, state),
            // …
            _ => BView::empty(),
        };
        BView::part(self.part, body)
    }
}
```

Inside each concrete part view (`part/menubar/mod.rs`, `part/toolbar/mod.rs`,
…), primitives (`Row`, `Column`, `Panel`, `Icon`, `Label`, `Button`,
`Dropdown`) are legal — that's where widgets are made.

## Cross-cutting

- **`store::spec::BSpec`** — every spec exposes `idx()` and `items()`. Enables
  a generic `walk` for tests, debug printouts, and structural audits.
- **`store::map::BMap<K, S>`** — key → spec lookup used by `set.rs` modules
  to expose `spec_for(key)`.
- **`store::panel_role::BPanelRole`** — semantic tag on the `BView::Panel`
  **primitive**. It is a Part-body concept, used for theming
  (`bg_card_header`, `bg_status`, …). Zones, areas, hosts, regions, panes
  do **not** carry a role — their identity is their store kind
  (`BZoneType::Top`, `BAreaType::Bottom`, …). A coordinator that reaches for
  `BPanelRole` is a bug.
- **`state::context::BContext`** — active `space / zone / host / region` for
  operators.
- **`bui::state::BState` + `BKey`** — hierarchy-bound state keys. Every view
  reads state through `state.get(&BKey::...)`.

## Why this shape holds

- **The store is exhaustive.** `BSpaceType.zones()`, `BAreaSpec.items`,
  `BHostSpec::with_set()`, `panes_for(host, region)`, `BPaneSpec.items` cover
  every branch. Nothing left to compute at runtime except which host the area
  is currently showing and which zones are toggled off.
- **Enums are closed.** `match` on `BZoneType`, `BAreaType`, `BHostType`,
  `BRegionType`, `BPaneType`, `BPartType` is exhaustive by construction.
  There is no default arm to silently absorb new levels. Coordinators
  don't even do these matches — the renderer does, once, per level.
- **State is generic.** `BState` is a `HashMap<String, String>` keyed by
  `BKey`. Every view reads through the same accessor.
- **Widgets live at the tail.** By pinning "actual view fragments" to the
  `Part` level and per-host `center_body`, the rest of the tree is
  composition by construction. A new space is a new `set.rs`, not a new
  coordinator.
- **The renderer owns physical layout.** `BZoneType::Bottom → column`,
  `BZoneType::Top → row`, `BRegionType::Center → side-by-side panes` are
  all decisions made once, in one file, keyed off the store enum.

## Test surface

Three lints keep this shape enforced:

- `structural_reuse_no_duplicate_types.rs` — no duplicate spec types.
- `structural_no_dangling_fn.rs` — every helper is reachable through the
  hierarchy.
- `file_line_cap.rs` — coordinators stay under the line budget.

`tests/store_blender_tests.rs::test_bspec_hierarchy_and_roles` walks the
`BSpec` trait to assert every level round-trips through `BSpecType`.
`BPanelRole` mapping is asserted at the primitive layer only (i.e. for
`BView::Panel` used inside part bodies).

A fourth lint enforces the coordinator purity rule:

- **`views_no_primitives_above_part.rs`** (new) — inside
  `bui/views/{space,zone,area,host,region,pane}/`, the only allowed
  `BView::*` calls are `space`, `zone`, `area`, `host`, `region`, `pane`,
  and `empty`. Any use of `Panel`, `Row`, `Column`, `Separator`, `Label`,
  `Icon`, `Button`, or the `BPanelRole` / `BPalette` type names above the
  `Part` level is a lint failure.

- **`views_list_item_shape.rs`** (new) — inside
  `bui/views/<level>/`, exactly two structs are expected: `<Level>ListView`
  and `<Level>ItemView`. `ListView` must expose `fn items(&self, state:
  &BState) -> Vec<BView>` and must not call `BView::<level>` itself.
  `ItemView` must implement `BViewBuilder` and its `build` must return
  `BView::<level>(kind, list.items(state))`. Any other shape at these
  files is a lint failure.

## Appendix — current violation inventory

Enumerated 2026-08-30 against `phiber/src/bui/views/`. Each row is a
concrete piece of code to move; none require a new store type. Applying
these leaves every coordinator file under the 60-line budget.

### `space/mod.rs` (13)

1. `use crate::store::BPanelRole;` at coordinator.
2. `use super::super::theme::BPalette;` at coordinator.
3. `fn zone_togglers(state, space, p)` composes `BView::button()
   .with_icon().with_tooltip().with_action(BAction::Toggle(BKey::Zone(z)))`
   — widget composition at space.
4. `BView::row(Some(p.gap_xs), items)` — primitive + palette.
5. `BView::panel(BPanelRole::Work, "", main_row)` — role at space.
6. `BView::panel(BPanelRole::CenterArea, "", children)` — role at space.
7. `BView::panel(BPanelRole::App, "", vec![activity_bar, main_content])`
   — role at space.
8. `matches!(top, BView::Empty)` — coordinator inspects BView shape.
9. `matches!(full, BView::Empty)` — same.
10. Hardcoded `[BZoneType::Left, Center, Right]` ordering for `main_row`
    — layout decision at space.
11. Inline `ActivityView::new(state).build()` — activity bar is a `Part`.
12. Inline `ModalView::new(modal, state).build()` — modal wrap at space.
13. `Self::zone_togglers(state, space_type, &p)` invoked twice — dead code
    if #11 succeeds.

**Fix:** replace with `SpaceItemView { space }` reading
`state.get(&BKey::CtxSpace)`. Emit only `BView::space(zones)`. Move
`zone_togglers` into a new `part/zone_toggler`. Move activity bar into
`part/activity`. Move modal overlay into `part/modal` (or a top-level
overlay layer the renderer picks up).

### `zone/mod.rs` (10)

1. `use super::super::theme::BPalette;`.
2. `use crate::store::BPanelRole;`.
3. `chrome: Option<BView>` field.
4. `with_chrome(chrome: BView)` builder.
5. `area_views` interleaves `BView::separator()` between areas.
6. `match self.zone` returns `BView::panel(BPanelRole::TopArea)`,
   `BView::panel(BPanelRole::LeftArea)`,
   `BView::panel(BPanelRole::CenterArea)`,
   `BView::panel(BPanelRole::RightColumn)` — the primary bug.
7. `BView::column(None, vec![…])` for Bottom.
8. `BView::column(None, …)` inside Top wrapping chrome.
9. `BView::vseparator()` inside Top.
10. Peeks `default_host == BHostType::Statusbar` and reaches for
    `BPalette::from_state(state).h_footer` — palette + neighbour peek.

**Fix:** replace with `ZoneItemView { zone, areas }`. Emit only
`BView::zone(self.zone, AreaListView::new(&self.areas).items(state))`.
Delete `chrome`, `area_views`, the whole `match self.zone`, and the
statusbar height hack. All layout choices move to `render_zone` in
`bui/render.rs`.

### `area/mod.rs` (2)

1. `BView::panel_sized(self.area.idx.panel_role(), "", …, None, self.height)`
   — role + width + height at area.
2. `height: Option<f32>` field — sizing propagated from zone.

**Fix:** `AreaItemView { area }`. Emit `BView::area(self.area.idx,
HostItemView::new(host).build(state))`. Statusbar footer height moves to
`render_area(BAreaType::Bottom, …)`.

### `host/mod.rs` (3 + smell)

1. `use crate::store::BPanelRole;`.
2. `.with_set().with_set()` double call — the outer call is redundant;
   `BHostType::with_set() -> &BHostSpec` already, then `.with_set()` is a
   no-op alias that returns the regions. Rename or delete one call.
3. `BView::panel(BPanelRole::Host, "", children)` — role at host.

**Fix:** `HostItemView { host }`. Emit `BView::host(self.host,
RegionListView::new(self.host).items(state))`. Rename
`center_body(host, state)` to `dispatch_center(host, state)` for
consistency with the Part dispatcher.

### `region/mod.rs` (3)

1. `use crate::store::BPanelRole;`.
2. Center/Window branch: `flat_map` panes → parts → `BView::panel(
   BPanelRole::Work, "", parts)` — the coordinator flattens the pane
   layer.
3. Non-Center branch: `BView::panel(self.region.panel_role(), "",
   children)` — role at region.

**Fix:** `RegionItemView { host, region }`. Emit `BView::region(
self.region, PaneListView::new(self.host, panes).items(state))`. The
Center-vs-strip decision moves to `render_region(BRegionType::Center, …)`
in the renderer.

### `pane/mod.rs` (6)

1. `use super::super::theme::BPalette;`.
2. `PartContainer` type — off-pattern wrapper with sizing logic.
3. `BView::panel_sized(part.idx.panel_role(), "", vec![item], w, h)`
   inside `PartContainer` — role + width + height.
4. `BView::panel(part.idx.panel_role(), "", vec![item])` — role.
5. `BView::row(Some(p.gap_xs), part_views)` when >1 part — primitive +
   palette.
6. `BView::panel(self.pane.idx.panel_role(), "", children)` — role at pane.

**Fix:** delete `PartContainer` entirely. `PaneItemView { pane, host }`
emits `BView::pane(self.pane.idx, PartListView::new(self.host,
self.pane.items).items(state))`. Toolbar / Sidebar width lookup moves to
`render_part(BPartType::Toolbar, …)` in the renderer, keyed off
`BKey::Expanded`.

### `part/mod.rs` (mostly clean)

- Uses `BView::empty()` for `_ =>` arm — fine (enum-only).
- `impl<'a> BViewBuilder for ActivityView<'a>` — `ActivityView` has a
  lifetime while other part views don't; unify their shapes (`new(host,
  state)` vs `new(state)`) so the macro `impl_part_builder!` covers all
  of them.

### Cross-cutting naming inconsistencies

- No List/Item split anywhere → introduce it uniformly.
- `PartContainer` breaks the `<Level>View` naming → deleted by the pane
  fix.
- `HostView::center_body` is a free function, `PartView` is a method →
  make both `dispatch_*` free functions in their level's `mod.rs`.
- Concrete part views live under `bui/part/<name>/` while the dispatcher
  lives at `bui/views/part/mod.rs` → move each `bui/part/<name>/` under
  `bui/views/part/<name>/` to parallel `bui/views/host/<name>/`.
- Some code uses `BView::Empty` variant directly, some uses
  `BView::empty()` constructor → always use the constructor.

None of the above requires a new store type or a new `BView` variant
beyond the seven store-level ones (`Space / Zone / Area / Host / Region /
Pane / Part`). Every fix is a pure move of an existing value onto the
correct level.

## When to break the rule

Never at `space / zone / area / host / region / pane`. The only files that
grow organically are `part/mod.rs` (one arm per `BPartType`) and per-host
`center_body` (one arm per `BHostType`). Both are exhaustive enum matches
and cannot be split further without hiding the switch.
