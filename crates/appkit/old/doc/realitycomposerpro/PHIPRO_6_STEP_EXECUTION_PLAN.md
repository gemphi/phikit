# PhiPro / Rust Kit Pro — Reality Composer Pro Parity (6-Step Execution Plan)

This document is the **implementation checklist** that maps Apple Reality Composer Pro features (as documented under `app/phipro/doc/read/realitycomposerpro/**/*.md`) to the current Rust implementation and a **6-step path to parity**.

## Source of Truth (Docs)

- `-realitycomposerpro.md`
- `configure-the-reality-composer-pro-project-window/--configure-the-reality-composer-pro-project-window.md`
- `editor_projectbrowser/--editor_projectbrowser.md`
- `STEP_BY_STEP_ASSET_IMPORT_GUIDE.md`
- `STEP_BY_STEP_SHADER_GRAPH_GUIDE.md`
- `STEP_BY_STEP_ANIMATION_EDITOR.md`

## Current UI Shell (Observed in Code)

Top area (Scene workspace):

- Hierarchy (left)
- Viewport (center)
- Inspector (right)

Bottom tabs (`phi/crates/rust-kit-pro/src/config.rs` `BOTTOM_TABS`):

- Timeline
- Project
- Assets
- Shader Graph

Missing vs Apple RCP:

- Audio Mixer tab
- Statistics tab

## Current Import Reality (Why “Import does nothing” happens)

- Many `.usdz` files contain a **binary USDC** payload.
- The project’s lightweight USD parsing cannot fully parse USDC without OpenUSD.
- Without a fallback, importing Botanist `.usdz` often fails.

## 6-Step Plan (Outcome-Oriented)

### Step 1 — UI Layout + Theme Tokens Everywhere

Goal: switching themes must affect **every panel and control**, including center viewport and bottom editors.

Deliverables:

- All major regions use theme tokens (no hard-resolved colors for backgrounds)
- View-name based token mapping is deterministic and has precedence
- Viewport clear color is theme-driven

Primary code:

- `phi/crates/rust-kit-pro/src/theme_helper.rs`
- `phi/crates/rust-kit-pro/src/config.rs` (`THEMED_VIEW_ROLES`)
- `phi/crates/rust-appkit/src/gpu/renderer.rs`

Test checklist:

- Switch to light theme
- Verify: hierarchy, inspector, viewport, bottom tab bar, toolbars, timeline repaint

### Step 2 — Project Browser (AppKit Data Source Model)

Goal: implement the full three-pane Project Browser:

- Navigator view (folders/collections)
- File grid view (assets)
- Inspector view (tabs: hierarchy + info)

Deliverables:

- A first-class `Project` model with:
  - folders/collections
  - assets with metadata (type, path, imported/converted path, preview)
- `NSOutlineViewDataSource` for navigator
- `NSCollectionViewDataSource` for file grid
- selection propagation: navigator -> grid -> inspector

Primary code:

- `phi/crates/rust-kit-pro/src/panels/project.rs`
- `phi/crates/rust-kit-pro/src/data.rs`
- `phi/crates/rust-kit-pro/src/project.rs`

Test checklist:

- Create folder
- Drag asset between folders
- Search filters grid
- Icon size slider changes grid layout

### Step 3 — Asset Import Pipeline (USD/GLTF/BLEND) + Clear User Feedback

Goal: importing assets must:

- always show progress / errors in the UI
- produce a stable “internal imported asset” representation
- be robust on Windows

Deliverables:

- Import supports:
  - `.gltf`/`.glb`
  - `.usd`/`.usda`
  - `.usdz` via zip parsing where possible
  - `.usdz` (binary USDC) via **Blender fallback**
  - `.blend` via Blender
- Visible “Import failed: …” message in Project Browser if conversion fails

Primary code:

- `phi/crates/rust-kit-pro/src/asset_importer.rs`
- `phi/crates/rust-kit-pro/src/app_actions.rs` (`file_import`)
- `phi/ext/phx/src/importers/usd.rs`

Test checklist:

- Import `assets/botanist/usd/environment/mesh/volume.usd` (USDA)
- Import one Botanist `.usdz` and confirm:
  - if OpenUSD not available, Blender fallback is attempted
  - error message is shown if Blender is missing

### Step 4 — Viewport Parity (Tools, Modes, Manipulators)

Goal: match RCP viewport workflows:

- selection mode + gizmos
- navigation modes
- environment picker
- grid toggle + snapping

Deliverables:

- viewport toolbar with icon buttons
- camera mode switching (orbit/pan/zoom/etc)
- entity manipulator handles

Primary code:

- `phi/crates/rust-appkit/src/gpu/viewport.rs`
- `phi/crates/rust-kit-pro/src/panels/viewport.rs`

### Step 5 — Shader Graph Parity

Goal: node-based editor usable end-to-end:

- create graph
- connect nodes
- preview material
- assign material to model

Deliverables:

- shader graph panel supports:
  - node palette
  - connections
  - parameter editing
  - serialization

Primary docs:

- `STEP_BY_STEP_SHADER_GRAPH_GUIDE.md`

### Step 6 — Timelines + Audio Mixer + Statistics (Bottom Editors)

Goal: implement the remaining editor tabs.

Deliverables:

- Audio Mixer panel
- Statistics panel
- Timeline is wired to selected entity + animations

Primary docs:

- `STEP_BY_STEP_ANIMATION_EDITOR.md`
- `STEP_BY_STEP_AUDIO_SETUP.md`

## SVG Icons (Known Gap)

Apple’s UI relies heavily on small glyph icons (viewport toolbar, panels, etc.).

Deliverables:

- A minimal cross-platform SVG pipeline (load + rasterize) OR a replacement icon system
- A small set of core icons required by the viewport toolbar + panels

## What Has Been Verified vs Not

Verified:

- `cargo check -p rust-kit-pro` passes

Not yet verified (requires running the app interactively):

- Importing Botanist USD/USDZ files through the UI
- Blender fallback availability on this machine (`blender` in PATH)
- Full theme repaint on all panels in a live session
