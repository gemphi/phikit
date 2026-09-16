# Layout Space - Store Structure

> **Blender references:**
> - `DNA_screen_types.h` - `bScreen` (workspace layout), `ScrArea` (area), `ARegion` (region)
> - `BKE_screen.h` - `SpaceType` struct with `regiontypes` array
> - `space_view3d/space_view3d.c` - `ED_spacetype_view3d()` registers regions
> - Blender Manual: [`interface/window_system/regions.rst`](../../blender-manual/manual/interface/window_system/regions.rst)

---

## Core Principle: Each Level Owns ONLY Its Immediate Children

```
Space  → owns Zones (and Areas positioned at zones)
Zone   → owns Areas (areas are slots at zone positions)
BArea   → owns Hosts (switchable host list)
Host   → owns Regions (ONLY regions - NOT panes or parts)
Region → owns Panes (ONLY panes - NOT parts)
Pane   → owns Parts (ONLY parts)
Part   → produces BView
```

**No cross-level lookups.** The hierarchy walks itself:
```rust
for zone in space.zones:
  for area in space.areas_where(area.zone == zone):
    host = state.active_host(zone)
    host_spec = host_spec(host)
    for region in host_spec.regions:        // host → regions
      for pane in region.panes:             // region → panes
        for part in pane.parts:             // pane → parts
          render_part(part)                  // part → BView
```

No `REGION_PANES` lookup tables. No `PANE_PARTS` arrays on hosts.
Each struct carries its children inline. The hierarchy IS the data structure.

---

## Store File Layout

```
crates/phiber/src/store/
  mod.rs                    - re-exports

  spaces/                   - SPACE level (workspace tabs)
    mod.rs                  - BSpaceType enum, SpaceSpec, space_spec()
    def.rs                  - Space impl for SpaceSpec
    layout/
      mod.rs                - LayoutSpace spec: zones, BAreas, grid fractions
      def.rs                - zone→area→host assignments
    modeling/
      mod.rs, def.rs
    sculpting/
      mod.rs, def.rs
    ...

  zones/                    - ZONE level (grid positions)
    mod.rs                  - BZoneType enum, ZoneSpec

  areas/                    - AREA level (slots at zone positions)
    mod.rs                  - BArea struct, BBAreaSpec

  hosts/                    - HOST level (editor types)
    mod.rs                  - BHostType enum, BHostSpec (owns regions ONLY), BHost_spec()
    def.rs                  - Host impl, default region arrays per host

  regions/                  - REGION level
    mod.rs                  - BRegionType, RegionSpec (owns panes ONLY)
    def.rs                  - Region impl, default regions

  panes/                    - PANE level
    mod.rs                  - BPaneType, BDockType, PaneSpec (owns parts ONLY)
    def.rs                  - Pane impl, default panes

  parts/                    - PART level
    mod.rs                  - BPartType, PartSpec (produces BView)
    def.rs                  - Part impl, default part collections

  menus/
    mod.rs                  - MenuType, MenuItemSpec, MenuSpec

  modals/
    mod.rs                  - modal specs
```

---

## Space Level: `store/spaces/layout/mod.rs`

```rust
//! Layout space spec - the default Blender-like workspace.
//! Ref: Blender Manual - workspaces.rst lines 70-84

use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;
use crate::store::areas::BArea;

/// A space spec: defines the grid of zones for a workspace tab.
/// A space ONLY owns its zones and areas. Areas own hosts.
#[derive(Clone, Debug)]
pub struct SpaceSpec {
    pub space_type: BSpaceType,
    pub label: &'static str,
    /// Zones in this space, each holding areas.
    pub zones: &'static [BZoneType],
    /// Areas in this space, each positioned at a zone.
    pub areas: &'static [BArea],
    /// Grid column fractions (left-to-right).
    pub columns: &'static [f32],
}

/// An area at a zone position, holding switchable hosts.
/// An area ONLY owns its hosts. Hosts own regions.
#[derive(Clone, Debug)]
pub struct BArea {
    /// Which zone this area occupies.
    pub zone: BZoneType,
    /// Default host (shown first).
    pub default_host: BHostType,
    /// All hosts that can be switched to in this area.
    /// If only one entry, the area is not switchable.
    pub hosts: &'static [BHostType],
    /// Grid fraction (relative to other areas in same row/column).
    pub fraction: f32,
}
```

---

## Host Level: `store/hosts/mod.rs`

```rust
//! Host specs - editor host types and their specs.
//! A host ONLY owns its regions. Regions own panes, panes own parts.
//! The hierarchy walks itself - no cross-level lookups.

/// Static spec for a host.
/// A host ONLY owns its regions. Regions own panes, panes own parts.
#[derive(Clone, Debug)]
pub struct HostSpec {
    pub host_type: BHostType,
    pub label: &'static str,
    pub icon: &'static str,
    /// Regions this host has. Each RegionSpec owns its own panes.
    pub regions: &'static [RegionSpec],
}
```

---

## Region Level: `store/regions/mod.rs`

```rust
//! Region specs - only 3 region types: Header, Center, Footer.
//! A region ONLY owns its panes. Panes own parts.

/// Static spec for a region.
/// A region ONLY owns its panes. Each PaneSpec owns its own parts.
#[derive(Clone, Copy, Debug)]
pub struct RegionSpec {
    pub region: BRegionType,
    pub visible: bool,
    pub units: Option<f32>,
    /// Panes within this region. Each pane owns its parts.
    pub panes: &'static [PaneSpec],
}
```

---

## Pane Level: `store/panes/mod.rs`

```rust
//! Pane specs - positional slots within regions.
//! A pane ONLY owns its parts. Parts produce BView nodes.

/// Static spec for a pane within a region.
/// A pane ONLY owns its parts. Each PartSpec produces a BView node.
#[derive(Clone, Copy, Debug)]
pub struct PaneSpec {
    pub pane: BPaneType,
    pub float: BDockType,
    /// Width in pixels for absolute panes (None for relative).
    pub width: Option<f32>,
    /// Parts within this pane. Each part produces a BView node.
    pub parts: &'static [PartSpec],
}
```

---

## Part Level: `store/parts/mod.rs`

```rust
//! Part specs - typed UI components that live inside panes.
//! Parts produce BView nodes. They are the leaves of the hierarchy.

/// Part types - UI components that live inside panes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPartType {
    Menubar, Modebar, BHostSwitcher, Toolbar, Sidebar, Panel,
    Channels, Statusbar, ToolProps, Preview,
    AssetShelf, AdjustLastOp, Custom,
}

/// Static spec for a part within a pane.
#[derive(Clone, Copy, Debug)]
pub struct PartSpec {
    pub part: BPartType,
    pub visible: bool,
}
```

---

## Store `mod.rs` - Exports

```rust
//! Store - all static specs for the Phiber editor.
//!
//! Hierarchy (top-down):
//!   spaces/   → BSpaceType, SpaceSpec (workspace tabs, zone/area grids)
//!   zones/    → BZoneType enum, ZoneSpec (grid positions)
//!   areas/    → BArea struct (zone, fraction, default_host, BHosts)
//!   hosts/    → BHostType, BHostSpec (owns regions ONLY)
//!   regions/  → BRegionType (Header, Center, Footer), RegionSpec (owns panes)
//!   panes/    → BPaneType, BDockType, PaneSpec (owns parts)
//!   parts/    → BPartType, PartSpec (produces BView)
//!   modals/   → help, profile, settings modal specs

pub mod spaces;
pub mod zones;
pub mod hosts;
pub mod areas;
pub mod regions;
pub mod panes;
pub mod parts;
pub mod modals;
```

---

## How the Store Eliminates If-Chains

The key insight: **the hierarchy IS the data structure.** Each level owns
its immediate children inline. The renderer walks the tree naturally -
no lookup tables, no cross-level arrays, no `if` chains.

```rust
// The render walk - just follow the children:
fn render_space(space: &SpaceSpec, state: &SpaceState, cx: &Context) -> BView {
    BView::Grid {
        columns: space.columns,
        children: space.zones.iter().map(|&zone| {
            render_zone(zone, space, state, cx)
        }).collect(),
    }
}

fn render_zone(zone: BZoneType, space: &SpaceSpec, state: &SpaceState, cx: &Context) -> BView {
    let areas: Vec<_> = space.areas.iter().filter(|a| a.zone == zone).collect();
    BView::Column {
        children: areas.iter().map(|area| {
            render_area(area, state, cx)
        }).collect(),
    }
}

fn render_area(area: &BArea, state: &SpaceState, cx: &Context) -> BView {
    let host = state.active_host(area.zone).unwrap_or(area.default_host);
    let spec = host_spec(host);
    BView::Host {
        children: spec.regions.iter().filter(|r| r.visible).map(|region| {
            render_region(region, cx)
        }).collect(),
    }
}

fn render_region(region: &RegionSpec, cx: &Context) -> BView {
    BView::Region {
        region_type: region.region,
        children: region.panes.iter().map(|pane| {
            render_pane(pane, cx)
        }).collect(),
    }
}

fn render_pane(pane: &PaneSpec, cx: &Context) -> BView {
    BView::Pane {
        children: pane.parts.iter().map(|part| {
            render_part(part, cx)
        }).collect(),
    }
}

fn render_part(part: &PartSpec, cx: &Context) -> BView {
    match part.part {
        BPartType::Menubar      => BAreaMenubar::new(cx).to_bview(),
        BPartType::Modebar      => BAreaModebar::new(cx).to_bview(),
        BPartType::Switcher => BSwitcher::new(cx).to_bview(),
        BPartType::Toolbar      => BAreaToolbar::new(cx).to_bview(),
        BPartType::Sidebar      => BAreaSidebar::new(cx).to_bview(),
        BPartType::Panel        => BAreaPanel::new(cx).to_bview(),
        BPartType::Channels     => BAreaChannels::new(cx).to_bview(),
        BPartType::Statusbar    => BAreaStatusbar::new(cx).to_bview(),
        BPartType::Custom       => effect.custom_view(cx).unwrap_or(BView::empty()),
        // ... exhaustive match on BPartType
    }
}
```

No `REGION_PANES` lookup. No `PANE_PARTS` array. No `(region, pane, part)`
tuples. The hierarchy walks itself because each struct carries its children.
