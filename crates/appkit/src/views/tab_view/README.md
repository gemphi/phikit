# NSTabView — Tabbed Interface

## Apple Header: `NSTabView.h`

## Overview

`NSTabView` provides a tabbed interface for switching between content panes. Mirrors Apple's [`NSTabView`](https://developer.apple.com/documentation/appkit/nstabview).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSTabView Structure                          │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Tab Components                                    │
│  ├─ Tab bar: Row of selectable tabs                         │
│  ├─ Tab items: NSTabViewItem (label + view)                 │
│  └─ Content area: Shows selected tab's view                 │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Selection                                      │
│  ├─ User clicks tab                                          │
│  ├─ Old tab's view hidden                                    │
│  ├─ New tab's view shown                                     │
│  └─ Delegate notified (optional)                           │
└─────────────────────────────────────────────────────────────┘
```

## Tab Positions

| Position | Appearance |
|----------|------------|
| `Top` | Tabs above content (standard) |
| `Bottom` | Tabs below content |
| `Left` | Vertical tabs on left |
| `Right` | Vertical tabs on right |

## Tab Styles

| Style | Look |
|-------|------|
| `Segmented` | Pill-shaped buttons |
| `SegmentedOnLeft` | Left-aligned pills |
| `SegmentedOnCenter` | Centered pills |
| `Toolbar` | Toolbar style |

## Connection to Other Modules

```
NSTabView
├── Contains: NSTabViewItem[]
│   └── Each has: label + view
├── Used by: Preference panes
├── Used by: Document tabs
└── Used by: Inspector panels
```

## Usage Examples

### Basic Tabs
```rust
use rust_appkit::prelude::*;
use rust_appkit::tab_view::{NSTabView, NSTabViewItem};

let mut tab_view = NSTabView::new(NSRect::new(0.0, 0.0, 400.0, 300.0));

// Create tabs
let general_tab = NSTabViewItem::new("General", create_general_view());
let appearance_tab = NSTabViewItem::new("Appearance", create_appearance_view());
let advanced_tab = NSTabViewItem::new("Advanced", create_advanced_view());

// Add tabs
tab_view.add_tab_view_item(general_tab);
tab_view.add_tab_view_item(appearance_tab);
tab_view.add_tab_view_item(advanced_tab);

// Select first tab
tab_view.select_tab_view_item_at_index(0);
```

### Preferences Panel
```rust
let mut prefs = NSTabView::new(NSRect::new(0.0, 0.0, 600.0, 400.0));
prefs.set_tab_position(NSTabViewPosition::Top);
prefs.set_tab_style(NSTabViewStyle::Segmented);

// General settings
let general = NSTabViewItem::new("General", create_general_settings());
prefs.add_tab_view_item(general);

// Rendering settings
let rendering = NSTabViewItem::new("Rendering", create_render_settings());
prefs.add_tab_view_item(rendering);

// Keyboard shortcuts
let shortcuts = NSTabViewItem::new("Shortcuts", create_shortcut_settings());
prefs.add_tab_view_item(shortcuts);
```

### Tab Delegate
```rust
trait NSTabViewDelegate {
    fn tab_view_did_select_tab_view_item(&self, tab_view: &NSTabView, item: &NSTabViewItem);
    fn tab_view_should_select_tab_view_item(&self, tab_view: &NSTabView, item: &NSTabViewItem) -> bool;
}

struct MyTabDelegate;

impl NSTabViewDelegate for MyTabDelegate {
    fn tab_view_did_select_tab_view_item(&self, _tab_view: &NSTabView, item: &NSTabViewItem) {
        println!("Switched to tab: {}", item.label());
    }
}

tab_view.set_delegate(MyTabDelegate);
```

## Testing & Verification

```rust
let mut tab_view = NSTabView::new(NSRect::new(0.0, 0.0, 300.0, 200.0));

let tab1 = NSTabViewItem::new("Tab 1", NSView::new(NSRect::new(0.0, 0.0, 100.0, 100.0)));
let tab2 = NSTabViewItem::new("Tab 2", NSView::new(NSRect::new(0.0, 0.0, 100.0, 100.0)));

tab_view.add_tab_view_item(tab1);
tab_view.add_tab_view_item(tab2);

assert_eq!(tab_view.number_of_tab_view_items(), 2);
assert_eq!(tab_view.index_of_tab_view_item(tab_view.selected_tab_view_item()), 0);

// Select second tab
tab_view.select_tab_view_item_at_index(1);
assert_eq!(tab_view.selected_tab_view_item_index(), 1);
```

## File Structure

```
src/tab_view/
├── mod.rs          # NSTabView, NSTabViewItem, NSTabViewPosition
├── item.rs         # NSTabViewItem
└── delegate.rs     # NSTabViewDelegate
```

## Missing from Apple's NSTabView

- **Overflow menu**: More tabs than fit
- **Reorder tabs**: Drag to rearrange
- **Close buttons**: Per-tab close
- **Add button**: Plus button for new tabs
- **Detach tabs**: Move to new window
- **Animations**: Slide transitions

These are stubbed for current use cases.

## When to Use

| Use NSTabView | Use NSSegmentedControl |
|---------------|------------------------|
| Different content views | Switch modes on same view |
| Settings panels | View toggles |
| Document sections | Toolbar filters |
