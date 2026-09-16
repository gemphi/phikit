# Page Documentation: Layout Workspace

## Overview
The **Layout** page is BUIApp's default workspace screen (`workspace_idx = 0`). It provides a multi-pane layout for 3D modeling, scene assembly, outliner hierarchy inspection, and property adjustments.

## UI Layout Composition
The screen is composed of 2 vertical columns declaring 4 tiled areas:

```
┌──────────────────────────────────────────────┬───────────────────────────────┐
│ Top Menu Bar Header (File, Edit, Render, ...)│ Workspace Tabs Strip          │
├──────────────────────────────────────────────┼───────────────────────────────┤
│                                              │ Outliner Space                │
│ 3D Viewport Space                            │ (Scene Tree Hierarchy)        │
│ (WGPU Real-time Shader Render)               ├───────────────────────────────┤
│                                              │ Properties Space              │
│                                              │ (Object, Mesh, Material)      │
├──────────────────────────────────────────────┤                               │
│ Timeline Space (Transport & Scrub Slider)    │                               │
└──────────────────────────────────────────────┴───────────────────────────────┘
```

## BArea Specifications
1. **Left Column - Upper**: 3D Viewport (`BSpaceType::View3D`).
2. **Left Column - Lower**: Timeline (`BSpaceType::Time`) with height dynamic from `ctx.timeline_height()`.
3. **Right Column - Upper**: Outliner (`BSpaceType::Outliner`) with width from `ctx.outliner_width()`.
4. **Right Column - Lower**: Properties (`BSpaceType::Buts`).

## Event & State Handlers
- Selecting an object in the Outliner highlights the corresponding item in `BContext` and refreshes the Properties space.
- Timeline frame changes dispatch render state updates to the 3D Viewport.
