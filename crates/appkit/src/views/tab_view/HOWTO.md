# TabView — How to Use

## Files
- `mod.rs` — `NSTabView` struct, constructors, selection logic
- `item.rs` — `NSTabViewItem` (label + content view pair)
- `delegate.rs` — `NSTabViewDelegate` handles tab bar rendering + click
- `builder.rs` — builder helpers for constructing tab views

## Quick Start
```rust
use rust_appkit::views::tab_view::{NSTabView, NSTabViewItem};

// Create a tab view
let mut tabs = NSTabView::new(rect);

// Add tabs — each holds a label and a content view
tabs.add_tab_view_item(NSTabViewItem::new("Project Browser", project_view));
tabs.add_tab_view_item(NSTabViewItem::new("Shader Graph", graph_view));

// Select a tab programmatically
tabs.set_selected_index(0);

// Get the underlying NSView for layout
let view = tabs.view();
```

## Key APIs
- `NSTabView::new(rect)` — create an empty tab view
- `tabs.add_tab_view_item(item)` — append a tab (auto-selects first)
- `tabs.set_selected_index(i)` — switch to tab at index
- `tabs.view()` / `tabs.view_mut()` — access underlying NSView
- `NSTabViewItem::new(label, view)` — create a tab item

## Rules
- First tab added is auto-selected
- `set_selected_index` triggers layout + redraw immediately
- Delegate owns hover state and renders the tab bar strip
- Content panes are subviews — only the selected one is visible
- Use theme tokens for tab bar colors, not hardcoded values
