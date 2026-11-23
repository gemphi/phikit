# CollectionView — How to Use

## Files
- `mod.rs` — `NSCollectionView` struct, constructors, selection
- `traits.rs` — `NSCollectionViewDataSource`, `NSCollectionViewDelegate` traits
- `delegate.rs` — `CollectionViewDelegate` internal rendering delegate
- `builder/` — specialized builders (e.g., `project_browser.rs`)

## Quick Start
```rust
use rust_appkit::views::collection_view::{
    NSCollectionView, NSCollectionViewDataSource, NSCollectionViewDelegate,
};

struct MyGridData { items: Vec<String> }

impl NSCollectionViewDataSource for MyGridData {
    fn number_of_items(&self) -> usize { self.items.len() }
    fn item_label(&self, index: usize) -> String { self.items[index].clone() }
    fn collection_view_did_select_item(&mut self, index: usize) {
        println!("Selected: {}", self.items[index]);
    }
}

let mut grid = NSCollectionView::new(frame);
grid.set_data_source(MyGridData { items: vec!["A".into(), "B".into()] });

// Programmatic selection
grid.select_item_at_index(Some(0));
```

## Key APIs
- `NSCollectionView::new(frame)` — create with default 90×90 item size
- `grid.set_data_source(ds)` — provide items + install render delegate
- `grid.set_delegate(del)` — respond to interaction events
- `grid.select_item_at_index(Some(i))` — programmatic selection
- `grid.selected_index()` — current selection
- `grid.view()` / `grid.view_mut()` — underlying NSView

## Rules
- Data source provides item count and labels
- Setting data source auto-installs the internal view delegate
- Delegate (optional) receives selection change notifications
- Default item size is 90×90 — set via `item_size` field
- Both data source and delegate are `Arc<Mutex<...>>` internally
