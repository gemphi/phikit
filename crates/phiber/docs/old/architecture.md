# BUIApp Architecture - Blender-Faithful UI

> **Reference**: Blender source under `REFS/blender/blender/source/blender/`
> - `BKE_screen.h` - `SpaceType`, `ARegionType` structs
> - `space_api/spacetypes.c` - `ED_spacetypes_init()` registration
> - `space_view3d/space_view3d.c` - `ED_spacetype_view3d()` example
> - `DNA_screen_types.h` - `bScreen`, `ScrArea`, `ARegion`
> - `DNA_space_types.h` - `eSpace_Type` enum
> - `DNA_windowmanager_types.h` - `wmWindowManager`, `wmWindow`

## Blender's UI Hierarchy

```
wmWindowManager          (app singleton)
  └─ wmWindow            (window)
       └─ bScreen        (workspace layout)
            └─ ScrArea    (tiled editor area)
                 └─ ARegion  (sub-region: header, window, tools, ui)
                      └─ Panel    (collapsible section)
                           └─ uiLayout (row/column/split)
                                └─ uiBut     (button/slider/text/...)
```

## Registration Flow (Blender C → BUIKit Rust)

Blender registers each space type **once** at startup via `ED_spacetypes_init()`:

```c
// spacetypes.c - called once
ED_spacetype_outliner();
ED_spacetype_time();
ED_spacetype_view3d();
ED_spacetype_buttons();
// ... etc.
```

Each `ED_spacetype_*()` function:
1. Allocates a `SpaceType` struct
2. Sets `st->spaceid = SPACE_VIEW3D` (the enum)
3. Sets `st->name = "View3D"` (for menus - **not** hardcoded in draw code)
4. Sets `st->iconid` (icon for menus)
5. Registers function pointers: `new`, `free`, `init`, `listener`, `operatortypes`, `keymap`
6. Registers `ARegionType` entries for each region (`RGN_TYPE_WINDOW`, `RGN_TYPE_HEADER`, etc.)
7. Each `ARegionType` has its own `init`, `draw`, `listener` callbacks

**Key insight**: The name "View3D" is set **once** in the registration, never
hardcoded in draw functions. The draw code reads `st->name` from the
registered `SpaceType`.

## BUIKit Mapping (Unified Retained View Tree)

In BUIKit, we avoid creating complex object hierarchies or separate struct components for screens, BAreas, and regions. Instead, we use a single, unified, purely data-driven `BView` enum. No `NSView` is created until the render pass (`BRenderer::render_root`).

| Blender C           | BUIKit Rust (`BView` Unified Enum) |
|---------------------|---------------------------------|
| `wmWindowManager`   | `BApp`                          |
| `wmWindow`          | `BWindow`                       |
| `bScreen`           | `BView::Screen`                 |
| `ScrArea`           | `BView::BArea`                   |
| `ARegion`           | `BView::Region`                 |
| `SpaceType`         | `BSpaceHandler` trait           |
| `art->draw`         | `BSpaceHandler::region(...)`    |
| `bContext`          | `BContext` (shared model)       |

## Data-Driven Design

The app **never** hardcodes labels, menus, or viewport text. It only:

1. Declares screen layouts via nested `BView::screen` and `BView::area` trees.
2. Provides a `BContext` (shared mutable state)
3. Calls `BView::screen(name, BAreas)` to register layout groups.
4. Looks up the `BSpaceHandler` for each area's `BSpaceType` to construct header and content views.

```
┌─────────────────────────────────────────────────────┐
│                     BApp                             │
│  ┌───────────────────────────────────────────────┐  │
│  │                  BWindow                       │  │
│  │  ┌─────────────────────────────────────────┐   │  │
│  │  │            BView::Screen (data)         │   │  │
│  │  │  ┌──────────┬──────────┬──────────┐      │   │  │
│  │  │  │ BView::  │ BView::  │ BView::  │      │   │  │
│  │  │  │ BArea     │ BArea     │ BArea     │      │   │  │
│  │  │  │ Outliner │ View3D   │ Buts     │      │   │  │
│  │  │  ├──────────┼──────────┼──────────┤      │   │  │
│  │  │  │ Region   │ Region   │ Region   │      │   │  │
│  │  │  │ (Header) │ (Header) │ (Ui)     │      │   │  │
│  │  │  └──────────┴──────────┴──────────┘      │   │  │
│  │  └─────────────────────────────────────────┘   │  │
│  │           ↓ BSpaceRegistry lookup               │  │
│  │  ┌─────────────────────────────────────────┐   │  │
│  │  │  BSpaceHandler::header() → BView         │   │  │
│  │  │  BSpaceHandler::region() → BView         │   │  │
│  │  │  → BView tree → BRenderer → NSView       │   │  │
│  │  └─────────────────────────────────────────┘   │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## File Layout

```
crates/apple/src/buikit/
  bapp.rs              - BApp, BWindow (wraps NSApplication, NSWindow)
  view/mod.rs          - BView enum (retained UI tree definition)
  view/builder.rs      - unified builder methods on BView
  blayout/mod.rs       - BLayoutEngine entrypoint
  blayout/measure.rs   - computes intrinsic sizes
  blayout/layout.rs    - assigns final frames (NSRect)
  brender/mod.rs       - BRenderer entrypoint (BView → NSView)
  brender/controls.rs  - renders leaf control widgets (buttons, sliders, etc.)
  bspace.rs            - BSpaceHandler trait (replaces old BSpace + BRegionHandler)
  bspace_registry.rs   - BSpaceRegistry (lookup table, init once)
  bcontext.rs          - BContext (shared state, mirrors bContext)
  structure/mod.rs     - BSpaceType and BRegionType enum tags
  metrics.rs           - BUnit (UI_UNIT scaling)
  theme/mod.rs         - BTheme, BColor

crates/apple/apps/buiapp/
  main.rs              - app entry: BApp, BWindow, BScreen data-driven setup
  model/mod.rs         - BModel (app-specific state → BContext)
  screens/mod.rs       - workspace and viewport layouts
  docs/
    architecture.md    - this file
    registration.md    - how space types register (mirrors spacetypes.c)
    data_flow.md       - from BView tree → NSView layout pass
```
