# Layout Space - Documentation Index

> **Blender references:**
> - `DNA_screen_types.h` - `bScreen`, `ScrArea`, `ARegion`
> - `DNA_space_types.h` - `SpaceLink`, `View3D` struct
> - `BKE_screen.h` - `SpaceType`, `ARegionType`
> - `BKE_context.h` - `bContext` struct
> - Blender Manual: [`interface/window_system/`](../../blender-manual/manual/interface/window_system/)

---

## UI Hierarchy

```
Space → Zone → BArea → Host → Region → Pane → Part → BView
```

- **Space** - workspace tab (Layout, Modeling, Sculpting, ...)
- **Zone** - grid position (Top, Left, Center, Right, Bottom, Full)
- **BArea** - slot at a zone position, holds switchable hosts
- **Host** - editor type (View3D, Outliner, Properties, Timeline, ...)
- **Region** - container (Header, Center, Footer)
- **Pane** - layout slot (Left, Center, Right) with sizing (AbsLeft, Relative, AbsRight)
- **Part** - UI component (Menubar, Modebar, Toolbar, Sidebar, Panel, Channels, Statusbar, ...)
- **BView** - render primitive (Row, Column, Grid, Label, Button, ...)

---

## Architecture Layers

| Layer | What | Where | Docs |
|-------|------|-------|------|
| **Store** | Static specs - what CAN exist | `crates/phiber/src/store/` | [store.md](store.md) |
| **State** | Per-instance runtime data - what IS happening | `crates/phiber/src/state/` | [state.md](state.md) |
| **Context** | Hierarchical active-focus path - what's FOCUSED | `crates/phiber/src/state/context.rs` | [ctx.md](ctx.md) |
| **Views** | Render walk - BView production | `crates/buikit/src/` | [views.md](views.md) |

Each layer reads from the one above it:
1. Store specs tell the view WHAT to render
2. State tells the view HOW to render (visibility, zoom, open/closed, ...)
3. Context tells the view what to HIGHLIGHT (active mode, active object, ...)

---

## File Structure (mod.rs / def.rs / ctx.rs / use.rs per level)

Each hierarchy level in the state and store layers has four files:
- **`mod.rs`** - struct definition, trait impl, enum dispatch
- **`def.rs`** - constants, defaults, min/max bounds
- **`ctx.rs`** - context sync (`sync_from_context` / `sync_to_context`)
- **`use.rs`** - set methods (get/set, not effects)

---

## Doc List

| Doc | Description |
|-----|-------------|
| [overview.md](overview.md) | New hierarchy, enum definitions, concrete example tree |
| [store.md](store.md) | Static store specs: SpaceSpec, BHostSpec, PaneSpec, PartSpec |
| [state.md](state.md) | Per-instance state: mod.rs/def.rs/ctx.rs/use.rs per hierarchy level |
| [ctx.md](ctx.md) | Hierarchical BContext: SpaceCtx → AreaCtx → HostCtx → RegionCtx → PaneCtx |
| [views.md](views.md) | View layer: render walk, part reuse, BHostEffect trait |
| [flow.md](flow.md) | Boot, render, data, and state mutation flow diagrams |
| [specs.md](specs.md) | Spec-driven pattern matching strategy, complete Layout space spec |
| [tasks.md](tasks.md) | Implementation phases and task checklist |

---

## Key Concepts

### BPartType Variants

| Variant | Description | State File |
|---------|-------------|------------|
| `Menubar` | Menu strip (File, Edit, View, ...) | `part/menubar.rs` |
| `Modebar` | Mode selector tabs with open/closed state | `part/modebar.rs` |
| `Toolbar` | Left tool rail (select, move, rotate, ...) | `part/toolbar.rs` |
| `Sidebar` | N-panel tab strip (Item, Tool, View) | `part/sidebar.rs` |
| `Panel` | Collapsible property panels | `part/panel.rs` |
| `Channels` | Animation channel list | `part/channels.rs` |
| `Statusbar` | Footer status items | `part/status_bar.rs` |
| `ToolProps` | Tool settings strip | `part/tool_props.rs` |
| `Preview` | Preview panel | `part/preview.rs` |
| `AssetShelf` | Asset browser strip | `part/asset_shelf.rs` |
| `AdjustLastOp` | Redo last operation panel | `part/adjust_last_op.rs` |
| `Custom` | Host-specific BView (viewport, node graph, ...) | `part/custom.rs` |

### Set Methods (use.rs)

Set methods are pure state mutations - NOT effects. The caller (keymap/action
dispatcher) invokes `set_*` methods on state, then calls `cx.notify()` to
trigger re-render.

Naming convention: `set_<field>` or `set_<field>_<action>`.

Examples:
- `SpaceState::set_host(zone, BHost)` - switch host in a zone
- `HostState::set_zoom_in()` - zoom in
- `HostState::set_mode(mode)` - set interaction mode
- `RegionState::set_visible(visible)` - set region visibility
- `PaneState::set_width(width)` - resize pane
- `ModebarState::set_open()` - open mode dropdown
- `ModebarState::set_mode(mode)` - select mode (closes dropdown)
- `SidebarState::set_tab(idx)` - select sidebar tab
- `PanelState::set_panel_toggle(id)` - toggle panel expansion

### Region Types (only 3)

| Region | Container | Height | Direction |
|--------|-----------|--------|-----------|
| Header | hflex | fixed 32px | horizontal |
| Center | hflex | flex-1 | horizontal |
| Footer | hflex | fixed 28px | horizontal |

### Pane Sizing (BDockType)

| BDockType | Width | Behavior |
|------------|-------|----------|
| AbsLeft | fixed px | Docked left, doesn't shrink |
| Relative | flex-1 | Fills remaining space |
| AbsRight | fixed px | Docked right, doesn't shrink |
