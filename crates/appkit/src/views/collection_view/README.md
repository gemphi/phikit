# NSCollectionView — Grid of Items

## Apple Header: `NSCollectionView.h`

## Overview

`NSCollectionView` displays a scrollable grid of items. Mirrors Apple's [`NSCollectionView`](https://developer.apple.com/documentation/appkit/nscollectionview).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSCollectionView Setup                       │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Data Source Pattern                             │
│  ├─ DataSource: Provides number of sections/items           │
│  ├─ Delegate: Handles selection, drag/drop                  │
│  └─ Item: NSView subclass for each cell                   │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Layout Engine                                │
│  ├─ FlowLayout: Items in rows (like Photos app)             │
│  ├─ GridLayout: Fixed columns                              │
│  └─ CustomLayout: User-defined positioning                 │
└─────────────────────────────────────────────────────────────┘
```

## Layout Types

| Layout | Description | Use Case |
|--------|-------------|----------|
| **Flow** | Left-to-right, wrap to next row | Photo grid, icons |
| **Grid** | Fixed columns, equal spacing | Asset library |
| **Custom** | Delegate provides frames | Complex arrangements |

## Connection to Other Modules

```
NSCollectionView
├── Contains: NSScrollView (for scrolling)
├── Contains: NSCollectionViewItem[] (cells)
├── Uses: NSCollectionViewLayout (positioning)
├── Uses: NSImageView (for thumbnails)
└── Used by: Asset browsers, photo apps
```

## Data Source Protocol

```rust
trait NSCollectionViewDataSource {
    fn number_of_sections(&self) -> usize;
    fn number_of_items_in_section(&self, section: usize) -> usize;
    fn item_for_index_path(&self, index: NSIndexPath) -> NSCollectionViewItem;
}
```

## Usage Examples

### Basic Grid
```rust
use rust_appkit::prelude::*;
use rust_appkit::collection_view::{NSCollectionView, NSCollectionViewFlowLayout};

let mut collection = NSCollectionView::new(NSRect::new(0.0, 0.0, 400.0, 300.0));

// Flow layout: items in rows
let layout = NSCollectionViewFlowLayout::new();
layout.set_item_size(NSSize::new(64.0, 64.0));
layout.set_minimum_line_spacing(10.0);
layout.set_minimum_interitem_spacing(10.0);

collection.set_collection_view_layout(layout);

// Set data source
collection.set_data_source(MyDataSource::new());
```

### Thumbnail Item
```rust
struct ThumbnailItem {
    view: NSView,
    image_view: NSImageView,
    label: NSTextField,
}

impl ThumbnailItem {
    fn new() -> Self {
        let view = NSView::new(NSRect::new(0.0, 0.0, 64.0, 80.0));
        
        let image_view = NSImageView::new(NSRect::new(0.0, 16.0, 64.0, 64.0));
        let mut label = NSTextField::new(NSRect::new(0.0, 0.0, 64.0, 16.0));
        label.set_text_alignment(NSTextAlignment::Center);
        
        view.add_subview(image_view.view_clone());
        view.add_subview(label.view_clone());
        
        Self { view, image_view, label }
    }
    
    fn configure(&mut self, asset: &Asset) {
        self.image_view.set_image(Some(asset.thumbnail()));
        self.label.set_text(&asset.name);
    }
}
```

### Asset Browser (rust-kit-pro)
```rust
// Display project assets in a grid
let mut asset_browser = NSCollectionView::new(NSRect::new(0.0, 0.0, 320.0, 400.0));

let layout = NSCollectionViewFlowLayout::new();
layout.set_item_size(NSSize::new(80.0, 100.0)); // Thumbnail + label
layout.set_minimum_interitem_spacing(8.0);
layout.set_minimum_line_spacing(8.0);
layout.set_section_insets(NSEdgeInsets::uniform(12.0));

asset_browser.set_collection_view_layout(layout);
asset_browser.set_data_source(ProjectAssetDataSource::new(project));
asset_browser.set_delegate(AssetSelectionDelegate::new());
```

## Selection Handling

```rust
trait NSCollectionViewDelegate {
    fn collection_view_did_select_item(&mut self, index_path: NSIndexPath);
    fn collection_view_did_deselect_item(&mut self, index_path: NSIndexPath);
}
```

## Testing & Verification

```rust
let mut collection = NSCollectionView::new(NSRect::new(0.0, 0.0, 300.0, 200.0));

// Set up with test data
let data_source = TestDataSource::with_items(100);
collection.set_data_source(data_source);

// Layout should calculate positions
assert!(collection.collection_view_layout().is_some());
```

## File Structure

```
src/collection_view/
└── mod.rs          # NSCollectionView, NSCollectionViewLayout, NSCollectionViewItem
```

## Missing from Apple's NSCollectionView

- **Animations**: Insert/delete/move animations
- **Prefetching**: Background item loading
- **Reorder support**: Drag to rearrange
- **Compositional layout**: Advanced layouts (macOS 10.15+)
- **Diffable data source**: Automatic diffing
- **Supplementary views**: Headers, footers, badges

These are stubbed for current use cases.

## When to Use

| Use NSCollectionView | Use NSTableView |
|----------------------|-----------------|
| Grid of thumbnails | List with columns |
| Photo gallery | Spreadsheet data |
| Icon browser | File list |
| Variable item sizes | Uniform row heights |
