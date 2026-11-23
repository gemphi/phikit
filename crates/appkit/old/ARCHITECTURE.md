# rust-appkit/src — Module Architecture

## Layout

```
src/
├── lib.rs                   # Crate root, module declarations, re-exports
├── culling.rs               # BVH frustum culling (used by gpu + rust-kit-pro)
├── layout_constants.rs      # Token-driven geometry constants
│
├── Core
│   ├── application/         # NSApplication, run loop, app delegate
│   ├── event/               # NSEvent, keyboard/mouse input
│   ├── graphics/            # NSColor, NSRect, NSPoint, NSSize, NSImage
│   ├── math/                # SCNVector3, SCNBoundingBox, matrices
│   ├── render/              # Software renderer, RenderContext
│   ├── responder/           # NSResponder chain
│   ├── view/                # NSView, layout constraints, delegates
│   └── window/              # NSWindow, NSWindowController, delegates
│
├── Controls
│   ├── alert/               # NSAlert
│   ├── box_view/            # NSBox
│   ├── button/              # NSButton (bezel styles, actions, drawing)
│   ├── collection_view/     # NSCollectionView (grid layout)
│   ├── color_well/          # NSColorWell
│   ├── control/             # NSControl base
│   ├── disclosure_triangle/ # Expand/collapse triangle
│   ├── image_view/          # NSImageView
│   ├── outline_view/        # NSOutlineView (tree data source)
│   ├── popup_button/        # NSPopUpButton
│   ├── progress_indicator/  # NSProgressIndicator
│   ├── scroll_view/         # NSScrollView
│   ├── search_field/        # NSSearchField
│   ├── segmented_control/   # NSSegmentedControl
│   ├── slider/              # NSSlider
│   ├── split_view/          # NSSplitView, NSSplitViewController
│   ├── stack_view/          # NSStackView
│   ├── stepper/             # NSStepper
│   ├── switch/              # NSSwitch
│   ├── tab_view/            # NSTabView
│   ├── table_view/          # NSTableView
│   ├── text_field/          # NSTextField
│   └── text_view/           # NSTextView
│
├── Layout
│   └── layout_engine/       # Declarative ViewBuilder → FlexboxEngine → NSView
│
├── Theming & Rendering
│   ├── animation/           # Core Animation equivalents
│   ├── gpu/                 # wgpu-based 3D viewport renderer
│   ├── shaders/             # WGSL shader files (mesh, grid, glass, etc.)
│   ├── svg_icons/           # SVG icon rasterization
│   └── theme/               # Theme registry, tokens, appearance
│
├── Panels & Containers
│   ├── menu/                # Menu bar, dropdowns, delegates
│   ├── panels/              # Editor panels (hierarchy, viewport, timeline, etc.)
│   ├── shader_graph_view/   # Node-based shader editor
│   ├── timeline/            # NSTimelineView
│   └── toolbar/             # NSToolbar, NSToolbarItem
│
└── Services
    ├── dialog/              # Open/save file dialogs
    ├── ecs/                 # Entity-Component-System (Scene, Entity, World)
    └── project/             # Project management, asset import/export
```

## Conventions

- Each module is a directory with `mod.rs` + sub-files
- Standalone `.rs` files at root level are small utilities (`culling.rs`, `layout_constants.rs`)
- `prelude` module re-exports the most commonly used types
- Bottom-level `pub use` in `lib.rs` provides flat access to key types
- `shaders/` contains `.wgsl` files loaded via `include_str!()` from `gpu/`
