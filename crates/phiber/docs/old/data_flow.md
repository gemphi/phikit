# Data Flow - BScreen Data → BView Tree → NSView

> **Reference**: Blender's `bScreen` → `ScrArea` → `ARegion` → draw callbacks

## Flow Diagram

```
App (buiapp/main.rs)
  │
  │  1. Declare screen layout (data only via BView variants)
  │     let screen = BView::screen("Layout", vec![
  │         BView::area(BSpaceType::Outliner, vec![
  │             BView::region(BRegionType::Header, BView::spacer()),
  │             BView::region(BRegionType::Window, BView::spacer()),
  │         ]),
  │         BView::area(BSpaceType::View3D, vec![
  │             BView::region(BRegionType::Header, BView::spacer()),
  │             BView::region(BRegionType::Window, BView::spacer()),
  │         ]),
  │     ]);
  │
  │  2. Create BContext (shared state)
  │     let ctx = BContext::new(BModel::default());
  │
  │  3. Compose layout using Space Registry
  │     (Handler header & region content evaluated and substituted)
  │
  ▼
BScreen Composition Pass
  │
  │  For each BView::BArea:
  │    handler = registry.get(area.space)
  │    for each BView::Region in regions:
  │      if rtype == Header:  content = handler.header(ctx)
  │      else:                content = handler.region(rtype, ctx)
  │
  ▼
BView::host(w, h) (buikit/view/builder.rs)
  │
  │  BLayoutEngine::measure() - compute intrinsic sizes (measure.rs)
  │  BLayoutEngine::layout()  - assign frames (NSRect) (layout.rs)
  │  BRenderer::render_root() - BView → NSView hierarchy mapping (brender/mod.rs)
  │
  ▼
BWindow::with_content(view, w, h)
  │
  │  NSWindow.set_content_view(nsview)
  │
  ▼
BApp::run() - event loop
```

## On Resize / Rebuild

```
NSApplication::on_rebuild_layout(root_view, w, h)
  │
  │  1. Lock BContext, update window_w/window_h
  │  2. Rebuild BView tree dynamically matching current active workspaces/areas
  │  3. view.host(w, h) → new NSView hierarchy
  │  4. Replace root_view's subviews
  │
  ▼
Layout engine re-measures with new (w, h) bounds
Renderer produces fresh, native NSView hierarchy
```

## Key Principle

The app **never** calls `BView::label("3D Viewport")` directly.
The label text and headers are derived from the respective `BSpaceHandler` implementation registered for that `BSpaceType` - initialized once at registration time, matching Blender's C space registration paradigm.
