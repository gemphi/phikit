# OutlineView — How to Use

## Files
- `mod.rs` — `NSOutlineView`, `NSOutlineViewDataSource` trait, `NSOutlineViewDelegate` trait, internal rendering delegate
- `builder.rs` — builder helpers for constructing outline views

## Quick Start
```rust
use rust_appkit::views::outline_view::{NSOutlineView, NSOutlineViewDataSource};

struct MyDataSource { /* your tree data */ }

impl NSOutlineViewDataSource for MyDataSource {
    fn number_of_children_of_item(&self, item: Option<&String>) -> usize {
        match item {
            None => 3,           // root has 3 children
            Some(name) => 0,    // leaves have no children
        }
    }
    fn child_of_item(&self, item: Option<&String>, index: usize) -> String {
        format!("Item {}", index)
    }
    fn is_item_expandable(&self, _item: &String) -> bool { false }
    fn item_entity_id(&self, _item: &String) -> Option<u64> { None }
}

let mut outline = NSOutlineView::new(frame);
outline.set_data_source(MyDataSource {});
```

## Key APIs
- `NSOutlineView::new(frame)` — create the view
- `outline.set_data_source(ds)` — install data source (also sets render delegate)
- `outline.view()` / `outline.view_mut()` — access underlying NSView
- `NSOutlineViewDataSource` trait — provide tree structure
- `NSOutlineViewDelegate` trait — respond to selection/expand/collapse

## Rules
- Data source is the single source of truth for tree structure
- Setting a data source auto-installs the internal rendering delegate
- Tree connector lines are drawn automatically (vertical + horizontal branches)
- Icons come from `icon_for_item()` — returns optional `IconId`
- Colors come from theme tokens (`CLR_BG_CONTENT`, `CLR_SEPARATOR`, etc.)
