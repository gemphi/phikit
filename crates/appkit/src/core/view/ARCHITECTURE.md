# View Architecture — Rust Design Decisions

## NSView — Value Type with Identity

```rust
#[derive(Clone)]
pub struct NSView {
    pub id: u64,
    pub name: String,
    pub(crate) frame: NSRect,
    pub(crate) subviews: Vec<NSView>,
    pub(crate) delegate: Option<Arc<Mutex<dyn NSViewDelegate>>>,
    pub(crate) appearance: Option<NSAppearance>,
    // ...
}
```

**Why `Clone`:** Views are passed by value into `add_subview()`. The parent owns a clone. This avoids lifetime annotations on the entire view tree. The `id` field preserves identity across clones.

**Why `Vec<NSView>` not `Vec<Box<NSView>>`:** Views are small (~200 bytes). Storing them inline in the vec avoids pointer indirection and improves cache locality during rendering (linear scan of subviews).

**Trade-off:** Cloning a deep view tree is O(n). This is acceptable because:
- Trees are rebuilt on state change (not cloned for reads)
- The render pass borrows `&NSView` — no cloning during draw

## Delegate Pattern — `Arc<Mutex<dyn NSViewDelegate>>`

```rust
pub trait NSViewDelegate: Send + Sync {
    fn draw(&self, view: &NSView, ctx: &mut RenderContext) {}
    fn mouse_down(&mut self, view: &mut NSView, event: &NSEvent) {}
    fn layout(&mut self, view: &mut NSView) {}
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

**Why `Arc<Mutex<_>>` not `Box<_>`:** The delegate is shared between the view (which is cloned) and the original owner. `Arc` ensures the delegate lives as long as any view clone references it. `Mutex` allows mutable access from event handlers.

**Why `Send + Sync` bound:** Views can be built on one thread and rendered on another (wgpu submission). The delegate must be safe to access from any thread.

**Why `as_any()`:** Enables downcasting to concrete delegate types (e.g., `NSStackViewDelegate`) without unsafe code. Used by the layout system to detect stack views and trigger automatic child positioning.

## Appearance Inheritance

```rust
pub fn effective_appearance(&self) -> NSAppearance {
    self.appearance.unwrap_or_else(|| NSAppearance::current_drawing())
}
```

**Why `Option<NSAppearance>`:** Most views inherit from the system. Only views that explicitly override (e.g., a dark sidebar in a light app) store `Some(...)`. This saves 0 bytes for the common case (Option<Copy type> with niche optimization).

**Current limitation:** True parent traversal requires a parent reference or tree walk. We fall back to the global system appearance. This is correct for single-appearance apps (which Reality Composer Pro is).

## Frame-Based Layout (Current)

```rust
view.set_frame(NSRect::new(x, y, w, h));
```

**Why not auto-layout from the start:** Auto-layout requires a constraint solver (Cassowary algorithm). Frame-based layout is simple, predictable, and sufficient for the current panel system where the workspace layout engine computes positions.

**Migration path:** `NSStackView` already does automatic child positioning. As more views adopt stack views, manual `set_frame()` calls disappear. The `PLATFORM_PLAN.md` describes the full reactive layout engine.

## No Retain Cycles

In Objective-C, view hierarchies create retain cycles (parent → child → delegate → parent). In Rust:
- Parent owns children (`Vec<NSView>` — direct ownership)
- Delegate is `Arc<Mutex<_>>` — reference counted, dropped when all views are dropped
- No weak references needed because the ownership graph is a tree, not a cycle
