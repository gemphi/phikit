# Layout Space - Specs & Pattern Matching Strategy

> **Blender references:**
> - `DNA_space_types.h` - `eSpace_Type` enum (SPACE_VIEW3D, SPACE_OUTLINER, etc.)
> - `BKE_screen.h` - `SpaceType.regiontypes` array (ARegionType list per space)
> - `space_api/spacetypes.c` - `ED_spacetypes_init()` registers all space types
> - Blender Manual: [`interface/window_system/areas.rst`](../../blender-manual/manual/interface/window_system/areas.rst)

---

## The Core Problem

The current codebase has too many `if` and `match` statements scattered
across the view layer. Each host effect has its own `region()` method
with a big match arm. The grid renderer has special cases per host type.
This doesn't scale.

## The Solution: Spec-Driven Pattern Matching

Instead of hardcoding behavior per host, we define **static specs** in the
store and **match on tuples** in the view layer. The tuples are
`(region, pane, part)` - and they come from store data, not from code.

---

## Spec Hierarchy (all static, all in store)

```
SpaceSpec
  ├─ space_type: BSpaceType
  ├─ label: &str
  ├─ zones: &[BZoneType]
  ├─ areas: &[BArea]
  │    ├─ zone: BZoneType
  │    ├─ default_host: BHostType
  │    ├─ hosts: &[BHostType]
  │    └─ fraction: f32
  └─ columns: &[f32]

HostSpec
  ├─ host_type: BHostType
  ├─ label: &str
  ├─ icon: &str
  ├─ regions: &[RegionSpec]
  │    └─ kind: BRegionType, visible: bool, scrollable: bool
  ├─ region_panes: &[(BRegionType, &[PaneSpec])]
  │    └─ position: BPanePosition, sizing: BPaneSizing
  ├─ pane_parts: &[(BRegionType, BPanePosition, BPartType, &[MenuItemSpec])]
  │    └─ items: static menu item specs for the BArea* component
  └─ menus: &[MenuIdx]
```

---

## Layout Space - Complete Spec

### Space Spec

```rust
// store/spaces/layout/def.rs

pub static LAYOUT_SPEC: SpaceSpec = SpaceSpec {
    space_type: BSpaceType::Layout,
    label: "Layout",
    zones: &[Top, Left, Center, Right, Bottom],
    areas: LAYOUT_AREAS,
    columns: &[0.18, 0.64, 0.18],
};

static LAYOUT_AREAS: &[BArea] = &[
    // Top: Topbar (full width, not switchable)
    BArea { zone: Top,    default_host: Topbar,
        hosts: &[Topbar], fraction: 1.0 },

    // Left: Outliner (switchable to File Browser, Asset Browser, Info)
    BArea { zone: Left,   default_host: Outliner,
        hosts: &[Outliner, FileBrowser, AssetBrowser, Info],
        fraction: 0.18 },

    // Center: View3D (switchable to all editors)
    BArea { zone: Center, default_host: View3D,
        hosts: BHostType::switchable(),
        fraction: 0.64 },

    // Right: Properties (switchable - 2+ hosts)
    BArea { zone: Right,  default_host: Properties,
        hosts: &[Properties, Outliner, AssetBrowser, Spreadsheet],
        fraction: 0.18 },

    // Bottom: Timeline (switchable to animation editors)
    BArea { zone: Bottom, default_host: Timeline,
        hosts: &[Timeline, DopeSheet, GraphEditor, Nla, Sequencer],
        fraction: 0.28 },
];
```

### Host Specs (one per host in the Layout space)

#### View3D Host Spec

```rust
// store/hosts/view3d/def.rs

// Regions - only 3 types
static REGIONS: &[(BRegionType, bool)] = &[
    (BRegionType::Header, true),
    (BRegionType::Center, true),
    // No Footer for View3D
];

// Panes per region
static REGION_PANES: &[(BRegionType, &[(BPaneType, BDockType, f32)])] = &[
    // Header: Left = Modebar (flex), Center = Header menus (flex)
    (BRegionType::Header, &[
        (BPaneType::Left,   BDockType::Relative,  0.0),   // modebar
        (BPaneType::Center, BDockType::Relative,  0.0),   // header menus
    ]),
    // Center: Left = Toolbar (fixed 44px), Center = Viewport (flex), Right = Sidebar (fixed 300px)
    (BRegionType::Center, &[
        (BPaneType::Left,   BDockType::AbsLeft,  44.0),  // toolbar rail
        (BPaneType::Center, BDockType::Relative,  0.0),   // viewport (flex fill)
        (BPaneType::Right,  BDockType::AbsRight, 300.0), // sidebar (N-panel)
    ]),
];

// Parts per pane (the key mapping)
static PANE_PARTS: &[(BRegionType, BPaneType, BPartType, &[MenuItemSpec])] = &[
    // Header region
    (BRegionType::Header, BPaneType::Left,   BPartType::Modebar, MODEBAR_MENUS),
    (BRegionType::Header, BPaneType::Center, BPartType::Menubar,  HEADER_MENUS),

    // Center region - toolbar, viewport, sidebar are all parts in panes
    (BRegionType::Center, BPaneType::Left,   BPartType::Toolbar, TOOLBAR_GROUPS),
    (BRegionType::Center, BPaneType::Right,  BPartType::Sidebar, SIDEBAR_TABS),
    (BRegionType::Center, BPaneType::Right,  BPartType::Panel,   PANEL_REGIONS),
    // Center pane uses Custom part (viewport) - no items needed
];
```

#### Outliner Host Spec

```rust
// store/hosts/outliner/def.rs

static REGIONS: &[(BRegionType, bool)] = &[
    (BRegionType::Header, true),
    (BRegionType::Center, true),
];

static REGION_PANES: &[(BRegionType, &[(BPaneType, BDockType, f32)])] = &[
    (BRegionType::Header, &[
        (BPaneType::Center, BDockType::Relative, 0.0),
    ]),
    (BRegionType::Center, &[
        (BPaneType::Center, BDockType::Relative, 0.0),
    ]),
];

static PANE_PARTS: &[(BRegionType, BPaneType, BPartType, &[MenuItemSpec])] = &[
    (BRegionType::Header, BPaneType::Center, BPartType::Menubar, HEADER_MENUS),
    // Center pane uses Custom part (outliner tree)
];
```

#### Properties Host Spec

```rust
// store/hosts/properties/def.rs

static REGIONS: &[(BRegionType, bool)] = &[
    (BRegionType::Header, true),
    (BRegionType::Center, true),
];

static REGION_PANES: &[(BRegionType, &[(BPaneType, BDockType, f32)])] = &[
    (BRegionType::Header, &[
        (BPaneType::Center, BDockType::Relative, 0.0),
    ]),
    (BRegionType::Center, &[
        (BPaneType::Center, BDockType::Relative, 0.0),
    ]),
];

static PANE_PARTS: &[(BRegionType, BPaneType, BPartType, &[MenuItemSpec])] = &[
    (BRegionType::Header, BPaneType::Center, BPartType::Menubar, HEADER_MENUS),
    (BRegionType::Center, BPaneType::Center, BPartType::Panel,  PANEL_REGIONS),
];
```

#### Timeline Host Spec

```rust
// store/hosts/timeline/def.rs

static REGIONS: &[(BRegionType, bool)] = &[
    (BRegionType::Header, true),
    (BRegionType::Center, true),
];

static REGION_PANES: &[(BRegionType, &[(BPaneType, BDockType, f32)])] = &[
    (BRegionType::Header, &[
        (BPaneType::Center, BDockType::Relative, 0.0),
    ]),
    (BRegionType::Center, &[
        (BPaneType::Center, BDockType::Relative, 0.0),
    ]),
];

static PANE_PARTS: &[(BRegionType, BPaneType, BPartType, &[MenuItemSpec])] = &[
    (BRegionType::Header, BPaneType::Center, BPartType::Menubar, HEADER_MENUS),
    // Center pane uses Custom part (timeline ruler)
];
```

#### Topbar Host Spec

```rust
// store/hosts/topbar/def.rs

static REGIONS: &[(BRegionType, bool)] = &[
    (BRegionType::Header, true),
    (BRegionType::Center, true),
];

static REGION_PANES: &[(BRegionType, &[(BPaneType, BDockType, f32)])] = &[
    (BRegionType::Header, &[
        (BPaneType::Left,   BDockType::Relative, 0.0),   // menus (flex)
        (BPaneType::Center, BDockType::Relative, 0.0),   // workspace tabs (flex)
        (BPaneType::Right,  BDockType::Relative, 0.0),   // status (flex)
    ]),
    (BRegionType::Center, &[
        (BPaneType::Center, BDockType::Relative, 0.0),
    ]),
];

static PANE_PARTS: &[(BRegionType, BPaneType, BPartType, &[MenuItemSpec])] = &[
    (BRegionType::Header, BPaneType::Left, BPartType::Menubar, MENUBAR_ITEMS),
    // Center pane: workspace tabs (rendered as Custom)
    // Right pane: status info (rendered as Custom)
];
```

---

## Pattern Matching in the View Layer

### Grid Level - match on BZoneType

```rust
/// Render the space grid.
fn render_space(spec: &SpaceSpec, state: &SpaceState, ctx: &BContext, cx: &App) -> AnyElement {
    // The space IS a grid - columns laid out by fractions
    let mut grid = hflex().size_full();

    for (col_idx, &fraction) in spec.columns.iter().enumerate() {
        let mut col = vflex()
            .flex_shrink_0()
            .w_f(fraction);  // relative width

        // Find areas in this column
        let areas_in_col = spec.areas.iter().filter(|a| {
            // Map area position to column index
            area_column(a.position) == col_idx
        });

        for area in areas_in_col {
            let host = state.active_host(area.position).unwrap_or(area.default_host);
            let effect = registry.get(host).unwrap();
            let host_view = BHostView::new(effect, theme);
            col = col.child(host_view.render(cx));
        }

        grid = grid.child(col);
    }

    grid.into_any_element()
}

/// Map area position to grid column index.
/// This is the ONLY match needed for grid layout.
fn area_column(pos: BZoneType) -> usize {
    match pos {
        BZoneType::Top    => 0,  // spans all columns (special-cased)
        BZoneType::Left   => 0,
        BZoneType::Center => 1,
        BZoneType::Right  => 2,
        BZoneType::Bottom => 1,  // under center
        BZoneType::Full   => 0,  // fills everything
    }
}
```

### Region Level - match on BRegionType (only 3!)

```rust
/// Determine the container type for a region.
/// ONE match with 3 arms - covers ALL hosts.
fn region_container(region: BRegionType) -> impl Styled {
    match region {
        BRegionType::Header => hflex().size_full(),  // horizontal
        BRegionType::Center => hflex().size_full(),  // horizontal
        BRegionType::Footer => hflex().size_full(),  // horizontal
    }
}
```

### Pane Level - match on (BPaneType, BDockType)

/// Apply pane sizing based on fixed type.
/// ONE match - covers ALL panes in ALL hosts.
fn pane_sizing(pane: &PaneSpec) -> impl Styled {
    match pane.fixed {
        BDockType::AbsLeft | BDockType::AbsRight => {
            vflex().w(px(pane.width))  // absolute, fixed width
        }
        BDockType::Relative => {
            vflex().flex_1()  // relative, flex fill
        }
    }
}

// ...

Pane sizing comes from `BDockType` enum in the store:

```rust
match pane.fixed {
    BDockType::AbsLeft | BDockType::AbsRight => vflex().w(px(pane.width)),  // absolute
    BDockType::Relative                  => vflex().flex_1(),           // relative
}
```

No per-host width calculations. No `if host == View3D { 44.0 } else { ... }`.
The fixed type and width come from static store specs.

### 5. Grid Props Can Be Passed Through

The pane spec can carry additional grid props:

```rust
pub struct PaneSpec {
    pub pane_type: BPaneType,    // Left, Center, Right
    pub fixed: BDockType,       // Relative, AbsLeft, AbsRight
    pub width: f32,              // fixed pixel width (when fixed != Relative)
    pub min_size: Option<f32>,   // minimum width/height
    pub max_size: Option<f32>,   // maximum width/height
    pub gap: Option<f32>,        // gap between parts in this pane
}

These are read generically by the pane container - no host-specific logic.
