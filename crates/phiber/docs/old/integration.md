# BUIApp ↔ Blender Integration Guide

## 1. Application Architecture

`BUIApp` is an executable shell (`crates/apple/apps/buiapp/main.rs`) built using `BUIKit` widgets and data-driven screen layouts:

```
┌────────────────────────────────────────────────────────┐
│ BUIApp (main.rs)                                       │
├────────────────────────────────────────────────────────┤
│  1. Initialize BLog & Theme (apple/dark/solid)         │
│  2. Instantiate BModel (Scene state storage)           │
│  3. Wrap BModel into thread-safe BContext              │
│  4. Initialize BSpaceRegistry (View3D, Outliner, ...)  │
│  5. Build active workspace screen via BScreen          │
│  6. Mount into BWindow and run BApp event loop         │
└────────────────────────────────────────────────────────┘
```

## 2. Data-Driven Layout Engine

In `buiapp`, screens are declared purely as data structures without explicit layout view hierarchy construction in application code:

```rust
// buiapp/screens/mod.rs
pub fn default_screen(ctx: &BContext) -> Vec<Vec<BArea>> {
    let left_col = vec![
        BArea::new(BSpaceType::View3D),
        BArea::new(BSpaceType::Time).with_height(timeline_h),
    ];
    let right_col = vec![
        BArea::new(BSpaceType::Outliner).with_width(outliner_w),
        BArea::new(BSpaceType::Buts),
    ];
    vec![left_col, right_col]
}
```

When `BScreen::build_layout(&registry, &columns, menu, &ctx)` is invoked:
1. `BScreen` queries `BSpaceRegistry` for the registered `BSpaceHandler` matching each `BSpaceType`.
2. Each `BSpaceHandler` constructs the appropriate `BView` subtree for its area.
3. Resizable column view stacks are composed into the top-level application layout.

## 3. Dynamic Workspace Switching

`BUIApp` provides 10 standard Blender workspaces in the top header tab strip:
1. Layout
2. Modeling
3. Sculpting
4. UV Editing
5. Shading
6. Animation
7. Rendering
8. Compositing
9. Geometry Nodes
10. Scripting

Clicking a workspace tab updates `ctx.with_mut(|d| d.set_workspace_idx(i))`, which triggers the workspace setup closure registered on `BApp` to rebuild the layout instantly.
