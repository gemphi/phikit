# Layout Space - Overview & New Hierarchy

> **Blender references:**
> - `DNA_screen_types.h` - `bScreen`, `ScrArea`, `ARegion`
> - `DNA_space_types.h` - `eSpace_Type` enum
> - `BKE_screen.h` - `SpaceType`, `ARegionType` structs
> - `space_api/spacetypes.c` - `ED_spacetypes_init()` registration
> - Blender Manual: [`interface/window_system/workspaces.rst`](../../blender-manual/manual/interface/window_system/workspaces.rst)
> - Blender Manual: [`interface/window_system/areas.rst`](../../blender-manual/manual/interface/window_system/areas.rst)
> - Blender Manual: [`interface/window_system/regions.rst`](../../blender-manual/manual/interface/window_system/regions.rst)

---

## The Problem

The current codebase conflates **Space** and **Host** types. `BAreaType::View3D` is treated as a "space type" but it's actually a **host type** (an editor that can be hosted in any area). Space types are the workspace tabs: Layout, Modeling, Sculpting, etc.

This leads to:
- `if` chains in grid rendering instead of pattern matching
- No clear separation between "where" (area position) and "what" (host type)
- Workspace layouts hardcoded as `BGridArea { space_type: BAreaType::View3D }` - wrong concept
- No way to express that the Layout space's right area has 2 switchable hosts (Outliner + Properties)

---

## New Hierarchy

```
Space (BSpaceType::Layout)
  └─ Zone (BZoneType: Top, Left, Right, Center, Bottom, Full)
       └─ BArea (BArea: zone, default_host, BHosts, fraction)
            └─ Host (BHostType: View3D, Outliner, Properties, ... - switchable)
                 └─ Region (BRegionType: Header, Center, Footer)
                      └─ Pane (BPaneType: Left, Center, Right + BDockType: Relative | AbsLeft | AbsRight)
                           └─ Part (BPartType: Menubar, Modebar, BHostSwitcher, Toolbar, Sidebar, Panel, Channels, Statusbar, ...)
                                └─ BView (Row, Column, Grid, Label, Button, ...)
```

**Key insight:** Zones are grid positions. Areas are slots at those positions
that hold hosts. Hosts don't sit directly in zones - they sit in areas, which
are positioned in zones. Regions are simple (only 3 types). The complexity lives
in panes and parts. A "Tools" region doesn't exist - instead, a `Center` region
has a `Left` pane with a `Toolbar` part. A "Channels" region doesn't exist -
instead, a `Center` region has a `Left` pane with a `Channels` part. The pane's
`BDockType` determines whether it's absolutely positioned (docked left/right)
or relative.

### Level 1: Space (`BSpaceType`)

A **Space** is a named workspace tab. It defines a grid of areas. Each space IS a grid.

> *"Workspaces are essentially predefined window layouts. Each Workspace consists of a set of Areas containing Editors, and is geared towards a specific task such as modeling, animating, or scripting."*
> - Blender Manual, `workspaces.rst` line 8-12

```rust
/// Workspace tab types - mirrors Blender's workspaces.
/// Ref: Blender Manual - workspaces.rst lines 70-97
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BSpaceType {
    Layout,
    Modeling,
    Sculpting,
    UvEditing,
    TexturePaint,
    Shading,
    Animation,
    Rendering,
    Compositing,
    GeometryNodes,
    Scripting,
}

impl BSpaceType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Layout       => "Layout",
            Self::Modeling     => "Modeling",
            Self::Sculpting    => "Sculpting",
            Self::UvEditing    => "UV Editing",
            Self::TexturePaint => "Texture Paint",
            Self::Shading      => "Shading",
            Self::Animation    => "Animation",
            Self::Rendering    => "Rendering",
            Self::Compositing  => "Compositing",
            Self::GeometryNodes => "Geometry Nodes",
            Self::Scripting    => "Scripting",
        }
    }

    pub fn all() -> &'static [BSpaceType] {
        &[
            Self::Layout, Self::Modeling, Self::Sculpting,
            Self::UvEditing, Self::TexturePaint, Self::Shading,
            Self::Animation, Self::Rendering, Self::Compositing,
            Self::GeometryNodes, Self::Scripting,
        ]
    }
}
```

**Blender reference:** Our current `BWorkspaceTab` already maps to this concept.
Blender's default workspaces: Layout, Modeling, Sculpting, UV Editing, Texture Paint,
Shading, Animation, Rendering, Compositing, Geometry Nodes, Scripting.
(See: `workspaces.rst` lines 70-97)

---

### Level 2: Zone (`BZoneType`)

A **Zone** is a semantic grid position within a space. Not all spaces use all
positions. Zones define where areas can be placed in the grid.

```rust
/// Semantic grid position within a space.
/// Not all spaces use all positions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BZoneType {
    Top,       // Topbar (spans full width, always present)
    Left,      // Left column (e.g. Outliner in Layout)
    Center,    // Main viewport area (e.g. View3D in Layout)
    Right,     // Right column (e.g. Properties in Layout; can have multiple hosts)
    Bottom,    // Bottom strip (e.g. Timeline in Layout, Statusbar)
    Full,      // Single area fills entire space (e.g. Sculpting)
}
```

---

### Level 3: BArea (`BArea`)

An **BArea** is a slot at a zone position. It holds one or more switchable hosts.
The area defines the fraction (size relative to other areas in the same row/column).

> *"The Blender window is divided into a number of rectangles called Areas. Areas reserve screen space for Editors."*
> - Blender Manual, `areas.rst` lines 16-19

```rust
/// An area at a zone position, holding switchable hosts.
#[derive(Clone, Debug)]
pub struct BArea {
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

**Example - Layout space areas:**

| Zone     | BArea (default host) | Switchable hosts | Fraction |
|----------|---------------------|------------------|----------|
| Top      | Topbar              | Topbar           | 1.0      |
| Left     | Outliner            | Outliner, FileBrowser, AssetBrowser, Info | 0.18 |
| Center   | View3D              | All editors      | 0.64     |
| Right    | Properties          | Properties, Outliner, AssetBrowser, Spreadsheet | 0.18 |
| Bottom   | Timeline            | Timeline, DopeSheet, GraphEditor, Nla, Sequencer | 0.28 |

> *"Blender's 'Layout' Workspace with four editors: 3D Viewport (yellow), Outliner (green), Properties (blue) and Timeline (red)."*
> - Blender Manual, `workspaces.rst` lines 79-84

---

### Level 4: Host (`BHostType`)

A **Host** is an editor type. This was formerly `BAreaType`. Hosts are switchable
within an area. The host determines which regions are available and how they render.

> *"Each editor offers a specific piece of functionality."*
> - Blender Manual, `areas.rst` line 20

```rust
/// Editor host types - formerly BAreaType.
/// Ref: Blender `DNA_space_types.h` - `eSpace_Type` enum
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BHostType {
    Empty,
    View3D,
    Outliner,
    Properties,   // formerly Buts
    FileBrowser,
    ImageEditor,
    Info,
    Sequencer,
    TextEditor,
    DopeSheet,
    Nla,
    Timeline,
    NodeEditor,
    LogicEditor,
    PythonConsole,
    UvEditor,
    Compositor,
    ShaderEditor,
    GeoNodes,
    ClipEditor,
    Drivers,
    AssetBrowser,
    Spreadsheet,
    Preferences,
    Topbar,
    Statusbar,
}
```

**Key change:** `BAreaType` is renamed to `BHostType`. The `BZoneType` enum
takes over the "where" responsibility. `BHostType` only answers "what editor".

---

### Level 5: Region (`BRegionType`)

A **Region** is a top-level structural division of a host. Only 3 types exist.
Everything that used to be a separate region type (Tools, Channels, Sidebar, etc.)
is now a **Part** rendered inside a **Pane** within one of these 3 regions.

> *"Every Editor in Blender is divided into Regions."*
> - Blender Manual, `regions.rst` line 8

```rust
/// Region types - only 3. All complexity lives in panes and parts.
/// Ref: Blender Manual - regions.rst
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BRegionType {
    Header,   // Top strip with menus, modes, tabs
    Center,   // Main content area (viewport, outliner tree, timeline, etc.)
    Footer,   // Bottom strip with status, transport controls
}
```

**What happened to the old region types?**

| Old BRegionType   | New location                                      |
|-------------------|---------------------------------------------------|
| `Window`          | `Center` region, `Center` pane, `Custom` part     |
| `Tools`           | `Center` region, `Left` pane, `Toolbar` part      |
| `Pane` (sidebar)  | `Center` region, `Right` pane, `Sidebar` part     |
| `Channels`        | `Center` region, `Left` pane, `Channels` part     |
| `ToolProps`       | `Header` region, `Center` pane, `ToolProps` part  |
| `Preview`         | `Center` region, `Right` pane, `Preview` part     |
| `AssetShelf`      | `Center` region, `Bottom` pane, `AssetShelf` part |
| `AdjustLastOp`    | `Center` region, `Right` pane, `AdjustLastOp` part|
| `Temporary`       | (popup - not in the static grid)                  |

This simplification means the grid renderer only matches on 3 region types,
not 11. The pane/part system handles all the structural variety.

---

### Level 6: Pane (`BPaneType` + `BDockType`)

A **Pane** is a positional slot within a region. Panes have a **type** (position)
and a **float type** (docking mode). The float type determines whether the pane
is absolutely positioned (docked to a side) or relative (flex-based).

```rust
/// Pane position within a region.
/// Panes are the layout containers that hold parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPaneType {
    Left,     // Left pane (e.g. toolbar, channels)
    Center,   // Center pane (e.g. viewport, outliner tree)
    Right,    // Right pane (e.g. sidebar, property panels)
}

/// Docking/sizing mode for a pane.
/// BDockType determines whether a pane is absolutely positioned or relative.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BDockType {
    Relative,  // Relative - flex-based, grows/shrinks with parent
    AbsLeft,   // Absolute - docked to left, fixed width
    AbsRight,  // Absolute - docked to right, fixed width
}
```

**How it works:**
- A pane with `BDockType::Relative` is a flex child - it grows to fill space.
- A pane with `BDockType::AbsLeft` is docked left with a fixed pixel width.
- A pane with `BDockType::AbsRight` is docked right with a fixed pixel width.

**Example:**
```rust
// View3D Center region panes:
static CENTER_PANES: &[PaneSpec] = &[
    PaneSpec { pane_type: BPaneType::Left,   fixed: BDockType::AbsLeft,   width: 44.0  }, // toolbar
    PaneSpec { pane_type: BPaneType::Center, fixed: BDockType::Relative,  width: 0.0   }, // viewport
    PaneSpec { pane_type: BPaneType::Right,  fixed: BDockType::AbsRight,  width: 300.0 }, // sidebar
];
```

**Design principle:** Panes are generic containers. You can pass grid props
(fraction, min_size, max_size) to drive the grid layout. The pane itself
doesn't know what it contains - it just knows how to lay out its parts.

---

### Level 7: Part (`BPartType`)

A **Part** is a typed UI component that lives inside a pane. Parts are matched
by defs (static specs) into their panes. Each part type maps to a `BArea*`
component in buikit.

```rust
/// Part types - UI components that live inside panes.
/// Each maps to a BArea* component in buikit.
/// These are what used to be separate region types (Tools, Channels, etc.)
/// but are now parts rendered inside panes within the 3 simple regions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPartType {
    // Content parts (map to BArea* components)
    Menubar,      // BAreaMenubar - menu strip (File, Edit, View, etc.)
    Modebar,      // BAreaModebar - mode selector tabs (open/closed state)
    Switcher, // BSwitcher - host type picker menu (editor type dropdown)
    Toolbar,      // BAreaToolbar - left tool rail
    Sidebar,      // BAreaSidebar - N-panel tabs + panels
    Panel,        // BAreaPanel - collapsible property panels
    Channels,     // BAreaChannels - animation channel list
    Statusbar,    // BAreaStatusbar - status items in footer region
    ToolProps,    // Tool settings strip
    Preview,      // Preview panel
    AssetShelf,   // Asset shelf
    AdjustLastOp, // Redo last operation panel
    Custom,       // Custom BView (viewport, node graph, text editor, etc.)
}
```

**Matching parts to panes by def:**

```rust
// In store spec - which parts go in which panes for the Center region:
static CENTER_PANE_PARTS: &[(BPaneType, BPartType)] = &[
    (BPaneType::Left,   BPartType::Modebar),   // modebar in left pane
    (BPaneType::Center, BPartType::Custom),     // viewport in center pane
    (BPaneType::Right,  BPartType::Sidebar),    // sidebar in right pane
];
```

This is the key to eliminating `if` chains: instead of checking `if rtype == Tools`,
we match on `(region, pane, part)` tuples from static defs. The old region types
like `Tools` and `Channels` are now just part types placed in panes.

---

## Layout Space - Concrete Example

```
BSpaceType::Layout (grid)
├─ Zone: Top
│  └─ BArea: { zone: Top, BHost: Topbar, fraction: 1.0 }
│       └─ Host: Topbar
│            └─ Region: Header
│                 └─ Pane: Center (Relative) → Part: Menubar (File, Edit, Render, Window, Help)
│
├─ Zone: Left
│  └─ BArea: { zone: Left, default_host: Outliner, fraction: 0.18 }
│       └─ Host: Outliner (default; switchable to FileBrowser, AssetBrowser, Info)
│            ├─ Region: Header
│            │    └─ Pane: Center (Relative) → Part: Menubar (View, Select menus)
│            └─ Region: Center
│                 └─ Pane: Center (Relative) → Part: Custom (outliner tree)
│
├─ Zone: Center
│  └─ BArea: { zone: Center, default_host: View3D, fraction: 0.64 }
│       └─ Host: View3D (default; switchable to all editors)
│            ├─ Region: Header
│            │    ├─ Pane: Left (Relative)   → Part: Modebar (Object, Edit, Sculpt, ...)
│            │    └─ Pane: Center (Relative) → Part: Menubar (View, Select, Add, Object menus)
│            └─ Region: Center
│                 ├─ Pane: Left (AbsLeft, 44px)   → Part: Toolbar (select, transform, annotate)
│                 ├─ Pane: Center (Relative)        → Part: Custom (3D viewport)
│                 └─ Pane: Right (AbsRight, 300px) → Part: Sidebar (Item, Tool, View tabs)
│                                               → Part: Panel (property panels)
│
├─ Zone: Right
│  └─ BArea: { zone: Right, default_host: Properties, fraction: 0.18 }
│       └─ Host: Properties (default; switchable to Outliner, AssetBrowser, Spreadsheet)
│            ├─ Region: Header
│            │    └─ Pane: Center (Relative) → Part: Menubar (Render, Output, Scene, ... tabs)
│            └─ Region: Center
│                 └─ Pane: Center (Relative) → Part: Panel (property panels)
│
└─ Zone: Bottom
   └─ BArea: { zone: Bottom, default_host: Timeline, fraction: 0.28 }
        └─ Host: Timeline (default; switchable to DopeSheet, GraphEditor, Nla, Sequencer)
             ├─ Region: Header
             │    └─ Pane: Center (Relative) → Part: Menubar (View, Marker menus)
             └─ Region: Center
                  └─ Pane: Center (Relative) → Part: Custom (timeline ruler)

**Note:** The old `Tools`, `Window`, and `Pane` regions are gone. View3D now has
only `Header` and `Center` regions. The toolbar, viewport, and sidebar are all
---

## Naming Migration Table

| Old (current)         | New                   | Level  |
|-----------------------|-----------------------|--------|
| `BWorkspaceTab`       | `BSpaceType`          | Space  |
| (none)                | `BZoneType`           | Zone   |
| (none)                | `BArea`               | BArea   |
| `BAreaType`           | `BHostType`           | Host   |
| `BRegionType` (11 variants) | `BRegionType` (3: Header, Center, Footer) | Region |
| (none)                | `BPaneType` + `BDockType` | Pane   |
| (none)                | `BPartType`           | Part   |
| `BGridArea.space_type`| `BBAreaSpec.hosts`     | BArea   |
| `SpaceSpec`           | `HostSpec`            | Host   |
| `SpaceState`          | `HostState`           | Host   |
| `BSpaceHandler`       | `BHostEffect`         | Host   |
| `BSpaceRegistry`      | `BHostRegistry`       | Host   |

---

## File Structure (proposed)

```
crates/phiber/src/
  store/
    spaces/
      mod.rs              - BSpaceType enum, SpaceSpec
      layout/
        mod.rs            - LayoutSpace spec (areas + grid)
        def.rs            - area/host assignments, grid fractions
      modeling/
        mod.rs, def.rs
      sculpting/
        mod.rs, def.rs
      ...
    areas/                → renamed to hosts/
      mod.rs              - BHostType enum, BHostSpec, BArea_spec → host_spec
      view3d/
        mod.rs            - View3DHost spec (regions, menus)
        def.rs            - region/pane/part definitions
        header/mod.rs     - header menu specs
        toolbar/mod.rs    - toolbar group specs
        sidebar/mod.rs    - sidebar tab specs
        panel/mod.rs      - panel region specs
        modebar/mod.rs     - mode bar specs
      outliner/...
      properties/...
      ...
    regions/
      mod.rs              - BRegionType (Header, Center, Footer), RegionSpec
    panes/                → NEW
      mod.rs              - BPaneType (Left, Center, Right), BDockType (Relative, AbsLeft, AbsRight), PaneSpec
    parts/                → NEW
      mod.rs              - BPartType (Menubar, Modebar, BHostSwitcher, Toolbar, Sidebar, Panel, Channels, Statusbar, ...), PartSpec
    menus/
      mod.rs              - MenuType, MenuItemSpec, MenuSpec

  state/
    space/
      mod.rs              - SpaceState (active area, active host per area)
    area/                 → renamed to host/
      mod.rs              - HostState enum, per-host state
      view3d.rs           - View3DState
      outliner.rs         - OutlinerState
      ...
    context.rs            - BContext (active space, active area, active host)
    spaces/               → renamed to hosts/
      view3d.rs           - View3DHost effect (BSpaceHandler → BHostEffect)
      outliner.rs         - OutlinerHost effect
      ...
```

---

## The Grid Principle

> *"Think of the entire space as a grid."*

Each space is a grid. The grid is driven by **enum pattern matching**, not `if` chains:

```rust
// BEFORE (current - lots of ifs):
let inner_pad = match space_type {
    BAreaType::View3D | BAreaType::Image | BAreaType::Uv => 0.0,
    _ => theme.content_padding,
};

// AFTER (new - pattern match on pane defs):
// Region is simple (Header/Center/Footer), pane float type drives layout:
let pane_layout = match (region, pane.fixed) {
    (BRegionType::Center, BDockType::AbsLeft)  => vflex().w(px(pane.width)),  // docked left
    (BRegionType::Center, BDockType::AbsRight) => vflex().w(px(pane.width)),  // docked right
    (BRegionType::Center, BDockType::Relative) => vflex().flex_1(),           // flex fill
    _ => vflex().flex_1(),
};
```

The grid renders by walking the space → zone → area → host → region → pane → part tree,
matching each level by enum, and producing `BView` nodes. No `if` statements -
just `match` arms driven by static defs.
