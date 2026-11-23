# StackView — How to Use

## Files
- `mod.rs` — `NSStackView`, `NSStackViewDistribution`, `NSStackViewDelegate`, layout engine

## Quick Start
```rust
use rust_appkit::views::stack_view::{NSStackView, NSStackViewDistribution};

// Horizontal stack with spacing
let mut hstack = NSStackView::horizontal_with_spacing(12.0);
hstack.add_arranged_subview(button_a.view_clone());
hstack.add_arranged_subview(button_b.view_clone());

// Vertical stack
let mut vstack = NSStackView::vertical_with_spacing(8.0);
vstack.add_arranged_subview(label.view_clone());
vstack.add_arranged_subview(text_field.view_clone());

// Configure distribution and alignment
hstack.set_distribution(NSStackViewDistribution::FillEqually);
hstack.set_edge_insets(NSEdgeInsets::new(4.0, 8.0, 4.0, 8.0));

// Position the stack within a parent
let stack = hstack.with_frame(NSRect::new(0.0, 0.0, 400.0, 44.0));
```

## Key APIs
- `NSStackView::horizontal()` — left-to-right layout
- `NSStackView::vertical()` — top-to-bottom layout
- `NSStackView::horizontal_with_spacing(f32)` — with custom gap
- `stack.add_arranged_subview(view)` — add a child
- `stack.remove_all_arranged_subviews()` — clear children
- `stack.set_spacing(f32)` — gap between items
- `stack.set_orientation(orient)` — Horizontal or Vertical
- `stack.set_distribution(dist)` — Fill, FillEqually, EqualSpacing
- `stack.set_edge_insets(insets)` — padding around content
- `stack.set_alignment(attr)` — Leading, CenterY, Fill, etc.

## Rules
- Flexible children use `NSAutoresizingMask::WIDTH_SIZABLE` (horizontal) or `HEIGHT_SIZABLE` (vertical)
- Layout is recalculated on every `add_arranged_subview` call
- Delegate performs layout automatically when parent frame changes
- Use `with_frame()` to position — children are auto-arranged inside
