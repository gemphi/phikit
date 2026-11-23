# Toolbar — How to Use

## Files
- `mod.rs` — `NSToolbar` struct, `DisplayMode`, `SizeMode`
- `item.rs` — `NSToolbarItem` definition (identifier, label, view)
- `delegate.rs` — `NSToolbarDelegate` trait for item providers
- `drawing.rs` — rendering logic for toolbar background + items

## Quick Start
```rust
use rust_appkit::views::toolbar::{NSToolbar, NSToolbarItem, DisplayMode};

// Create a toolbar
let mut toolbar = NSToolbar::new("MainToolbar");
toolbar.set_display_mode(DisplayMode::IconAndLabel);

// Add items
let mut item = NSToolbarItem::new("save");
toolbar.add_item(item);

// Add flexible space to push items apart
toolbar.add_flexible_space();

// Add more items after the space
let run_item = NSToolbarItem::new("run");
toolbar.add_item(run_item);

// Insert at specific position
let new_item = NSToolbarItem::new("undo");
toolbar.insert_item(new_item, 0);
```

## Key APIs
- `NSToolbar::new(identifier)` — create with string ID
- `toolbar.add_item(item)` — append an item
- `toolbar.add_flexible_space()` — push items apart
- `toolbar.insert_item(item, index)` — insert at position
- `toolbar.remove_item(index)` — remove by index
- `toolbar.set_display_mode(mode)` — IconOnly, LabelOnly, IconAndLabel
- `toolbar.set_size_mode(mode)` — Default, Regular, Small
- `toolbar.set_visible(bool)` — show/hide
- `toolbar.height()` — returns height from theme

## Rules
- Height comes from `theme::keys::SIZE_TITLE_BAR_HEIGHT` (default 44px)
- Flexible space item ID = `"NSToolbarFlexibleSpaceItem"`
- Delegate provides items dynamically; direct `add_item` for static toolbars
- Drawing is a separate struct — keeps rendering isolated
