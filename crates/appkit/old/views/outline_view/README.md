# NSOutlineView — Hierarchical Lists

## Apple Header: `NSOutlineView.h`

## Overview

`NSOutlineView` displays hierarchical data in a collapsible tree structure. Mirrors Apple's [`NSOutlineView`](https://developer.apple.com/documentation/appkit/nsoutlineview).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSOutlineView Setup                          │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Data Source Pattern                             │
│  ├─ isItemExpandable(item): Can this row expand?            │
│  ├─ numberOfChildrenOfItem(item): Child count               │
│  ├─ childOfItem(item, index): Get specific child            │
│  └─ viewForItem(item): NSView for the row                  │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 User Interaction                             │
│  ├─ Click disclosure triangle: Expand/collapse             │
│  ├─ Click row: Select item                                 │
│  ├─ Double-click: Edit or default action                   │
│  └─ Drag rows: Reorder or move in hierarchy                │
└─────────────────────────────────────────────────────────────┘
```

## Data Source Protocol

```rust
trait NSOutlineViewDataSource {
    // Required
    fn outline_view_number_of_children_of_item(&self, item: Option<&dyn Any>) -> i64;
    fn outline_view_child_of_item(&self, index: i64, item: Option<&dyn Any>) -> &dyn Any;
    fn outline_view_is_item_expandable(&self, item: &dyn Any) -> bool;
    
    // Optional
    fn outline_view_view_for_item(&self, item: &dyn Any) -> NSView;
}
```

## Connection to Other Modules

```
NSOutlineView
├── Inherits: NSTableView (column/row structure)
│   └── Inherits: NSControl
├── Contains: NSScrollView (for scrolling)
├── Uses: NSImageView (for row icons)
└── Used by: Scene graph, file browsers
```

## Usage Examples

### Basic Outline
```rust
use rust_appkit::prelude::*;
use rust_appkit::outline_view::{NSOutlineView, NSOutlineViewDataSource};

let mut outline = NSOutlineView::new(NSRect::new(0.0, 0.0, 300.0, 400.0));
outline.set_data_source(FileSystemDataSource::new("/projects"));
```

### Scene Graph Data Source
```rust
struct SceneGraphDataSource {
    root: SceneNode,
}

impl NSOutlineViewDataSource for SceneGraphDataSource {
    fn outline_view_number_of_children_of_item(&self, item: Option<&dyn Any>) -> i64 {
        match item {
            Some(node) => node.children.len() as i64,
            None => 1, // Root node
        }
    }
    
    fn outline_view_child_of_item(&self, index: i64, item: Option<&dyn Any>) -> &dyn Any {
        match item {
            Some(node) => &node.children[index as usize],
            None => &self.root,
        }
    }
    
    fn outline_view_is_item_expandable(&self, item: &dyn Any) -> bool {
        let node = item.downcast_ref::<SceneNode>().unwrap();
        !node.children.is_empty()
    }
}
```

### Hierarchy Panel (rust-kit-pro)
```rust
// Scene hierarchy with entities and components
let mut hierarchy = NSOutlineView::new(NSRect::new(0.0, 0.0, 320.0, 400.0));

// Configure columns
let name_column = NSTableColumn::new("Name", 200.0);
let type_column = NSTableColumn::new("Type", 100.0);
hierarchy.add_table_column(name_column);
hierarchy.add_table_column(type_column);

// Set data source
hierarchy.set_data_source(SceneHierarchyDataSource::new(ecs_world));

// Expand all by default
hierarchy.expand_item(None, true); // true = expand children too
```

## Row View

```rust
fn create_row_view(node: &SceneNode) -> NSView {
    let mut row = NSStackView::horizontal();
    row.set_spacing(4.0);
    
    // Icon based on type
    let icon = match node.node_type {
        NodeType::Entity => entity_icon(),
        NodeType::Mesh => mesh_icon(),
        NodeType::Light => light_icon(),
        NodeType::Camera => camera_icon(),
    };
    
    let icon_view = NSImageView::new(NSRect::new(0.0, 0.0, 16.0, 16.0));
    icon_view.set_image(Some(icon));
    
    let name_label = NSTextField::label(&node.name);
    
    row.add_arranged_subview(icon_view.view_clone());
    row.add_arranged_subview(name_label.view_clone());
    
    row.view_clone()
}
```

## Expand/Collapse

```rust
// Expand specific item
outline.expand_item(&entity, false); // false = don't expand children

// Collapse
outline.collapse_item(&entity);

// Expand all
outline.expand_item(None, true); // nil item + recursive
```

## Selection

```rust
// Single selection
outline.select_row_indexes(&[5], false);

// Multiple selection
outline.select_row_indexes(&[1, 3, 5], true); // true = extend selection

// Get selected
let selected = outline.selected_row();
let item = outline.item_at_row(selected);
```

## Testing & Verification

```rust
let mut outline = NSOutlineView::new(NSRect::new(0.0, 0.0, 200.0, 300.0));

let data = TestHierarchy::with_depth(3, 3); // 3 levels, 3 children each
outline.set_data_source(data);

// Test expand
assert!(!outline.is_item_expanded(&root));
outline.expand_item(&root, false);
assert!(outline.is_item_expanded(&root));
```

## File Structure

```
src/outline_view/
└── mod.rs          # NSOutlineView, NSOutlineViewDataSource
```

## Missing from Apple's NSOutlineView

- **Drag & drop**: Reorder and reparent items
- **Autosave**: Remember expanded state
- **Indentation markers**: Visual tree depth
- **Grid lines**: Row/column separators
- **Cell-based**: NSCell support (legacy)
- **Animations**: Smooth expand/collapse

These are stubbed for current use cases.

## When to Use

| Use NSOutlineView | Use NSTableView |
|-------------------|-----------------|
| Hierarchical data | Flat list |
| File browser | Spreadsheet |
| Scene graph | Database table |
| XML/JSON tree | Simple array |
