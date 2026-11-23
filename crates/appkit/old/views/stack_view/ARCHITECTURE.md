# NSStackView Architecture — Rust Design Decisions

## Automatic Child Positioning

```rust
pub fn add_arranged_subview(&mut self, view: NSView) {
    self.arranged_subviews.lock().unwrap().push(view.clone());
    self.view.add_subview(view);
    self.update_layout();
}
```

**Why `update_layout()` on every add:** The stack view is the first step toward reactive layout. Adding a child immediately positions it — no manual `set_frame()` needed by the caller. This matches Apple's `NSStackView` behavior.

## Flex Layout Model

```rust
let is_flex = view.autoresizing_mask().contains(NSAutoresizingMask::WIDTH_SIZABLE);
```

**Why autoresizing mask for flex detection:** Reuses an existing field on NSView. Views marked `WIDTH_SIZABLE` (horizontal stack) or `HEIGHT_SIZABLE` (vertical stack) expand to fill remaining space. Fixed-size views keep their intrinsic size.

**Distribution algorithm:**
1. Sum spacing between items
2. Sum fixed-size items
3. Remaining space / flex count = flex item size
4. Position items sequentially

This is a simplified flexbox — no wrapping, no grow/shrink ratios. Sufficient for toolbar layouts and panel arrangements.

## `Arc<Mutex<Vec<NSView>>>` for Arranged Subviews

**Why not just `Vec<NSView>` on the struct:** The `NSStackViewDelegate` (which implements `NSViewDelegate`) needs access to the arranged subviews during `layout()`. The delegate is behind `Arc<Mutex<_>>`, so it needs a shared reference to the subview list.

**Why not `Rc<RefCell<_>>`:** Must be `Send + Sync` because views can be built and rendered across thread boundaries.

## Builder Convenience

```rust
let stack = NSStackView::vertical_with_spacing(12.0)
    .with_frame(NSRect::new(0.0, 0.0, 300.0, 400.0));
stack.add_arranged_subview(button.view_clone());
stack.add_arranged_subview(label.view_clone());
// Children are positioned automatically — no set_frame() needed
```

**Why `with_frame()` returns `Self`:** The stack view needs a frame to compute child positions. Setting it via builder ensures layout runs immediately after frame is known.
