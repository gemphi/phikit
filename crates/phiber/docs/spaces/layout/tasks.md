# Layout Space - Implementation Tasks

> **Status (2026-09-16): historical plan.** This checklist targeted the
> pre-v2 `buikit` + `store/` / `state/` layout. `crates/buikit` has been
> deleted and the work was re-done in `crates/phiber/src/v2`, where the
> outcomes these phases aimed for exist: `BSpaceType`, `BZoneType`,
> `BHostType` (28 hosts), `BRegionType` (Header / Center / Footer plus
> Blender's Tools, UI, ToolProps, AssetShelf, Channels, HUD, NavBar,
> Execute, Scrubbing…), `BPaneType`, `BPartType`, host-scoped menus, the
> Switcher part and the modal layer (8 modals). Items below are left
> unticked because they name files that no longer exist — do not use this
> list to plan new work; use `MANUAL.md` and `blend-read-parity.md` §0.

> **Blender references:**
> - `space_api/spacetypes.c` - `ED_spacetypes_init()` registration flow
> - `BKE_screen.h` - `SpaceType`, `ARegionType` struct definitions
> - Blender Manual: [`interface/window_system/workspaces.rst`](../../blender-manual/manual/interface/window_system/workspaces.rst)

---

## Phase 1: Enum Refactoring (buikit)

### 1.1 Rename `BAreaType` → `BHostType`
- [ ] Rename enum `BAreaType` to `BHostType` in `buikit/src/structure.rs`
- [ ] Update all `BAreaType` references across buikit and phiber
- [ ] Update `title()`, `icon()`, `switchable()`, `modes()` methods
- [ ] Rename `BAreaState` trait to `BHostState` (or keep as `BAreaState` for compat)

### 1.2 Rename `BWorkspaceTab` → `BSpaceType`
- [ ] Rename enum `BWorkspaceTab` to `BSpaceType` in `buikit/src/structure.rs`
- [ ] Add `label()`, `all()`, `default_hosts()` methods
- [ ] Update all `BWorkspaceTab` references

### 1.3 Simplify `BRegionType` to 3 variants
- [ ] Replace 11-variant `BRegionType` with 3: `Header`, `Center`, `Footer`
- [ ] Update all `BRegionType` references across buikit and phiber
- [ ] Remove `Window`, `Tools`, `Pane`, `Channels`, `ToolProps`, `Preview`,
      `AssetShelf`, `AdjustLastOp`, `Temporary` variants
- [ ] These concepts move to `BPartType` (see 1.5)

### 1.4 Add `BPaneType` + `BDockType` enums (NEW)
- [ ] Create `BPaneType` enum: `Left`, `Center`, `Right`
- [ ] Create `BDockType` enum: `Relative`, `AbsLeft`, `AbsRight`
- [ ] Create `PaneSpec` struct: `{ pane_type, fixed, width }`
- [ ] Add to `buikit/src/structure.rs` or new `buikit/src/panes.rs`

### 1.5 Expand `BPartType` enum (NEW)
- [ ] Create `BPartType` enum with all part types:
      `Menubar`, `Modebar`, `Toolbar`, `Sidebar`, `Panel`,
      `Channels`, `Statusbar`, `ToolProps`, `Preview`, `AssetShelf`,
      `AdjustLastOp`, `Custom`
- [ ] Create `PartSpec` struct: `{ part_type, items }`
- [ ] Add to `buikit/src/structure.rs` or new `buikit/src/parts.rs`

### 1.6 Add `BZoneType` enum (NEW)
- [ ] Create `BZoneType` enum: `Top`, `Left`, `Center`, `Right`, `Bottom`, `Full`
- [ ] Add to `buikit/src/structure.rs`

---

## Phase 2: Store Refactoring (phiber/src/store)

### 2.1 Rename `areas/` → `hosts/`
- [ ] Rename `store/areas/` directory to `store/hosts/`
- [ ] Rename `BAreaSpec` → `HostSpec`, `BAreaSpecProvider` → `HostSpecProvider`
- [ ] Rename `area_spec()` → `host_spec()`
- [ ] Update all imports and references

### 2.2 Add pane/part specs to host stores
- [ ] Add `REGION_PANES` static array to each host's `mod.rs`
- [ ] Add `PANE_PARTS` static array to each host's `mod.rs`
- [ ] Add `PART_ITEMS` static array to each host's `def.rs`
- [ ] Update `HostSpec` struct to include `region_panes` and `pane_parts`

### 2.3 Create `store/panes/mod.rs` (NEW)
- [ ] Define `BPaneType`, `BDockType`, `PaneSpec`
- [ ] Re-export from store root

### 2.4 Create `store/parts/mod.rs` (NEW)
- [ ] Define `BPartType`, `PartSpec`
- [ ] Re-export from store root

### 2.5 Update `store/spaces/`
- [ ] Update `SpaceSpec` to use `BZoneType` + `BArea` + `BHostType`
- [ ] Add `zones: &[BZoneType]` field to `SpaceSpec`
- [ ] Add `BArea` struct with `zone`, `default_host`, `hosts`, `fraction`
- [ ] Create `store/zones/mod.rs` with `BZoneType` enum
- [ ] Create `store/areas/mod.rs` with `BArea` struct
- [ ] Create `store/spaces/layout/mod.rs` with `LAYOUT_SPEC`
- [ ] Create `store/spaces/layout/def.rs` with zone/area/host assignments
- [ ] Update `space_spec()` function to use new types

### 2.6 Update `store/workspace.rs`
- [ ] Replace `workspace_grid()` to use new `SpaceSpec` + `BArea`
- [ ] Remove `BGridArea { space_type: BAreaType }` pattern
- [ ] Use `BGridArea { zone: BZoneType, BHost: BHostType }`

### 2.7 Update `store/regions/mod.rs`
- [ ] Simplify `BRegionType` to 3 variants
- [ ] Update `RegionSpec` to match new structure
- [ ] Remove `RegionIdx` if no longer needed (or update)

### 2.8 Update `store/mod.rs`
- [ ] Add `pub mod panes;` and `pub mod parts;`
- [ ] Update re-exports for new types
- [ ] Rename `pub mod areas` → `pub mod hosts`

---

## Phase 3: State Refactoring (phiber/src/state)

### 3.1 Restructure state to hierarchy (mod.rs/def.rs/ctx.rs/use.rs per level)
- [ ] Create `state/space/{mod.rs, def.rs, ctx.rs, use.rs}` - SpaceState (active zone, active host per zone)
- [ ] Create `state/zone/{mod.rs, def.rs, ctx.rs, use.rs}` - ZoneState (visible, active area index)
- [ ] Create `state/area/{mod.rs, def.rs, ctx.rs, use.rs}` - AreaState (host, focused)
- [ ] Create `state/host/{mod.rs, def.rs, ctx.rs, use.rs}` - HostState enum, per-host state files
- [ ] Create `state/region/{mod.rs, def.rs, ctx.rs, use.rs}` - RegionState (visible, height, active_pane)
- [ ] Create `state/pane/{mod.rs, def.rs, ctx.rs, use.rs}` - PaneState (visible, width, float_type, active_part)
- [ ] Create `state/part/{mod.rs, def.rs, ctx.rs, use.rs}` - PartState enum, per-part state files
- [ ] Move region visibility from host state to RegionState
- [ ] Move mode bar state to PartState::Modebar (with open/closed)
- [ ] Implement set methods (use.rs) as pure state mutations, NOT effects
- [ ] Implement context sync (ctx.rs) at each hierarchy level

### 3.2 Rename `area/` → `host/` (host state files)
- [ ] Rename `state/area/` directory to `state/host/`
- [ ] Rename `AreaState` → `HostState`
- [ ] Update `HostState::for_host()` match arms
- [ ] Remove region visibility fields from host state (moved to RegionState)
- [ ] Remove mode bar state from host state (moved to PartState::Modebar)

### 3.3 Rename `spaces/` → `hosts/` (effects)
- [ ] Rename `state/spaces/` directory to `state/hosts/`
- [ ] Rename `BSpaceHandler` trait → `BHostEffect`
- [ ] Update `space_type()` → `host_type()` method
- [ ] Replace `regions()` + `header()` + `region()` methods with `custom_view()`
- [ ] Add `pane_specs()` and `part_specs()` default methods

### 3.4 Update `state/context.rs`
- [ ] Update `BContext` to use hierarchical nested structs (SpaceCtx, ZoneCtx, BAreaCtx, BHostCtx, RegionCtx, PaneCtx)
- [ ] Replace flat fields with nested hierarchy
- [ ] Update all context accessors and mutators

### 3.5 Update `state/mod.rs`
- [ ] Add `pub mod space; pub mod zone; pub mod area; pub mod host; pub mod region; pub mod pane; pub mod part;`
- [ ] Update re-exports for new hierarchy

---

## Phase 4: View Layer Refactoring (buikit)

### 4.1 Create `BHostView` component (NEW)
- [ ] Create `buikit/src/components/host_view.rs`
- [ ] Implement generic region/pane/part rendering walk
- [ ] Implement `render_part()` with the core `(region, pane, part)` match
- [ ] Implement `render_pane()` with `BDockType` sizing match
- [ ] Implement `render_region()` with 3-variant `BRegionType` container match

### 4.2 Update `BGrid` / `BAreaView`
- [ ] Update `BGrid` to use `BZoneType` for column mapping
- [ ] Update `BAreaView` to delegate to `BHostView` instead of `BSpaceEffect`
- [ ] Remove per-host `if` chains in grid rendering
- [ ] Update `BGridArea` to use `BZoneType` + `BHostType`

### 4.3 Rename `BSpaceRegistry` → `BHostRegistry`
- [ ] Rename registry struct
- [ ] Update lookup methods to use `BHostType`
- [ ] Update all callers

### 4.4 Update area_parts components
- [ ] Add `BAreaMenubar` component (if not exists)
- [ ] Add `BAreaModebar` component (renamed from `BAreaModes`)
- [ ] Ensure all `BArea*` components accept `&[BPartItem]` and produce `BView`
- [ ] Update `BAreaToolbar` to read width from `PaneSpec` instead of hardcoded 44.0

---

## Phase 5: Integration & Testing

### 5.1 Update BUIApp root
- [ ] Update `BUIApp` to use new `BSpaceType` for workspace tabs
- [ ] Update workspace tab rendering to use `BSpaceType::all()`
- [ ] Update grid rendering to use new `SpaceSpec` + `BArea`

### 5.2 Update keymap actions
- [ ] Update region toggle to work with 3-variant `BRegionType`
- [ ] Update host switching to use `BZoneType` + `BHostType`
- [ ] Update context updates on area/host focus changes

### 5.3 Verify compilation
- [ ] `cargo build` - fix all type errors
- [ ] `cargo test` - fix all test failures
- [ ] Manual verification: Layout space renders correctly

### 5.4 Update remaining spaces
- [ ] Create store specs for Modeling, Sculpting, etc.
- [ ] Verify each space renders with correct area/host layout
- [ ] Test host switching within areas

---

## Phase 6: Modal Refactoring (phiber/src/state/modals + store/modals)

### 6.1 Restructure modal state (mod.rs/def.rs/ctx.rs/use.rs per modal)
- [ ] Create `state/modals/mod.rs` - ModalState enum (Closed, Settings, Help, Profile)
- [ ] Create `state/modals/def.rs` - default modal state
- [ ] Create `state/modals/ctx.rs` - modal context sync (bypasses BContext - overlays)
- [ ] Create `state/modals/use.rs` - set_open, set_close, set_toggle
- [ ] Create `state/modals/settings/{mod.rs, def.rs, ctx.rs, use.rs}` - SettingsState
- [ ] Create `state/modals/help/{mod.rs, def.rs, ctx.rs, use.rs}` - HelpState
- [ ] Create `state/modals/profile/{mod.rs, def.rs, ctx.rs, use.rs}` - ProfileState

### 6.2 Implement modal view layer
- [ ] Create `buikit/src/components/b_modal.rs` - modal container (header + body + footer)
- [ ] Implement click-outside-to-dismiss (Zed pattern)
- [ ] Implement focus save/restore across modal open/close
- [ ] Implement DismissDecision (Dismiss(true/false), Pending) for async confirmations
- [ ] Add fade-out background behind modal

### 6.3 Expand settings modal
- [ ] Add missing categories: Viewport, Keymap, File Paths, Save & Load
- [ ] Implement category sidebar with active highlighting
- [ ] Implement row rendering: name + value per setting
- [ ] Add search filter across all categories
- [ ] Add reset-to-default per field

### 6.4 Implement Switcher part (NOT a modal)
- [ ] Add `BPartType::Switcher` to `store/parts/mod.rs`
- [ ] Create `buikit/src/components/area_parts/host_switcher.rs` - dropdown menu part
- [ ] Show host type list with icons (filtered by switchable hosts for zone)
- [ ] Highlight current host
- [ ] Dispatch `set_host(zone, BHost)` on click
- [ ] Place in header region's left pane, next to Modebar

---

## Migration Order (dependency-safe)

```
1. buikit enums (Phase 1)          ← no dependencies
2. store refactoring (Phase 2)     ← depends on Phase 1
3. state refactoring (Phase 3)     ← depends on Phase 1 + 2
4. view layer (Phase 4)            ← depends on Phase 1 + 2 + 3
5. integration (Phase 5)           ← depends on all above
6. modal refactoring (Phase 6)     ← depends on Phase 1 + 2 + 3
```

Each phase can be compiled and tested independently if done in order.

---

## Risk Areas

- **`BRegionType` simplification** is the biggest breaking change - affects
  every host effect, every store spec, every state file. Must be done carefully.
- **`BAreaType` → `BHostType` rename** touches many files but is mechanical.
- **`BWorkspaceTab` → `BSpaceType` rename** is straightforward.
- **New pane/part system** is additive - can be introduced alongside existing
  code before removing old patterns.
