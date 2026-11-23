# SplitView — How to Use

## Files
- `mod.rs` — `NSSplitView`, `NSSplitViewState`, divider drag logic
- `delegate.rs` — `NSSplitViewDelegate` draws dividers + performs layout
- `item.rs` — `NSSplitViewItem` wraps a child with collapse semantics
- `controller.rs` — `NSSplitViewController` (higher-level controller)
- `layout.rs` — layout calculation helpers
- `drag_bounds.rs` — min/max constraint computation during drag

## Quick Start
```rust
use rust_appkit::views::split_view::{NSSplitView, NSSplitViewDividerStyle};

// Create a vertical split (children side-by-side)
let mut split = NSSplitView::vertical();
split.add_arranged_subview(left_panel.view_clone());
split.add_arranged_subview(right_panel.view_clone());

// Set divider style
split.set_divider_style(NSSplitViewDividerStyle::Thin);

// Programmatically position a divider
split.set_divider_at_index(0, 300.0);

// Listen for divider changes
split.set_on_divider_changed(|positions, total_width| {
    println!("Dividers moved: {:?}", positions);
});
```

## Key APIs
- `NSSplitView::vertical()` — children left-to-right
- `NSSplitView::horizontal()` — children top-to-bottom
- `split.add_arranged_subview(view)` — add a pane
- `split.set_divider_at_index(i, pos)` — set divider pixel position
- `split.set_divider_style(style)` — Thin, Thick, PaneSplitter, None
- `split.set_on_divider_changed(cb)` — callback on drag end

## Rules
- Collapse a pane by calling `view.set_hidden(true)` on the child
- Divider hit region is ±4px (8px total)
- Drag respects `min_size` / `max_size` of adjacent subviews
- Delegate = drawing + layout only; drag state lives on `NSSplitView`
