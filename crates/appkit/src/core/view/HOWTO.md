# Core View (NSView) — How to Use

## Files
- `mod.rs` — `NSView` struct, `NSViewDelegate` trait, `NSThemeable` trait
- `drawing.rs` — rendering helpers and draw traversal
- `events.rs` — event dispatch (mouse, keyboard, drag-drop)
- `layout.rs` — autoresizing mask logic, frame-based layout
- `constraints.rs` — `NSLayoutConstraint` (Auto Layout support)

## Quick Start
```rust
use rust_appkit::core::view::{NSView, NSViewDelegate};

// Create a view
let mut view = NSView::new(NSRect::new(0.0, 0.0, 400.0, 300.0));
view.set_background_color(Some(NSColor::white()));
view.set_corner_radius(8.0);

// Add subviews
view.add_subview(child_view);

// Custom delegate for drawing + events
struct MyDelegate;
impl NSViewDelegate for MyDelegate {
    fn draw(&self, view: &NSView, ctx: &mut RenderContext) {
        let bounds = view.bounds();
        ctx.set_color(NSColor::red());
        ctx.fill_rect(bounds);
    }
    fn mouse_down(&mut self, view: &mut NSView, event: &NSEvent) {
        println!("Clicked at {:?}", event.location_in_window());
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
view.set_delegate(MyDelegate);
```

## Key APIs
- `NSView::new(frame)` — create with frame rect
- `NSView::sized(w, h)` — create at origin (0,0)
- `view.set_delegate(impl NSViewDelegate)` — custom draw/events
- `view.add_subview(child)` — build view hierarchy
- `view.set_frame(rect)` — position + size (triggers layout)
- `view.set_hidden(bool)` — visibility
- `view.set_background_color(color)` — solid background
- `view.set_glass(true)`, `set_glass_style`, `set_glass_tint` — glass background (see `src/glass`)
- `view.set_glass_container(Some(spacing))` — render a subtree's glass as one merged layer
- `view.find_view_by_name(name)` — tree search

## Rules
- Delegate = all custom behavior (draw, events, layout)
- Delegate protocol: `draw`, `mouse_down/up/dragged`, `key_down/up`, `layout`, `scroll_wheel`, `gpu_draw`
- `as_any()` / `as_any_mut()` required for downcasting
- Use theme tokens (`set_theme_token`, `apply_theme`) not hardcoded colors
- `set_frame` triggers autoresize + layout cascade on children
- `set_needs_display()` requests a window redraw
