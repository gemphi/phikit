# NSToolbar — Window Toolbar

## Apple Header: `NSToolbar.h`

## Overview

`NSToolbar` provides a customizable bar at the top of windows for quick actions. Items include buttons, search fields, flexible spaces, and custom views. Mirrors Apple's [`NSToolbar`](https://developer.apple.com/documentation/appkit/nstoolbar).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSToolbar Setup                              │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Toolbar Items                                   │
│  ├─ NSButton: Actions (Save, Open, New)                    │
│  ├─ NSSearchField: Search/filter                           │
│  ├─ NSPopUpButton: Mode selection                          │
│  ├─ NSView: Custom controls                                │
│  └─ NSToolbarFlexibleSpaceItem: Push items apart            │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Display Modes                                │
│  ├─ Icon only: Just icons (default)                        │
│  ├─ Icon and Label: Below each icon                        │
│  ├─ Label only: Text buttons                               │
│  └─ Custom: User-defined                                     │
└─────────────────────────────────────────────────────────────┘
```

## Toolbar Item Types

| Item Type | Purpose | Icon |
|-----------|---------|------|
| `Button` | Action trigger | ▶️ Save, ➕ Add |
| `Search` | Filter/search | 🔍 Search field |
| `Menu` | Dropdown selection | ▼ Mode |
| `Space` | Fixed gap | `   ` |
| `FlexibleSpace` | Expanding gap | `~~~~` |
| `Separator` | Visual divider | `|` |

## Connection to Other Modules

```
NSToolbar
├── Contains: NSToolbarItem[]
│   ├── NSButton (action items)
│   ├── NSSearchField (filtering)
│   └── NSView (custom)
├── Attached to: NSWindow
└── Used by: Document-based apps
```

## Usage Examples

### Basic Toolbar
```rust
use rust_appkit::prelude::*;
use rust_appkit::toolbar::NSToolbar;

let mut toolbar = NSToolbar::new("MainToolbar");

// Add items
let save_item = toolbar.create_button_item("save", "Save", save_icon, || save());
let open_item = toolbar.create_button_item("open", "Open", open_icon, || open());
let flexible_space = toolbar.create_flexible_space_item();
let search_item = toolbar.create_search_item("search", |query| filter_results(query));

toolbar.set_items(&[
    save_item,
    open_item,
    flexible_space,
    search_item,
]);

window.set_toolbar(toolbar);
```

### rust-kit-pro Toolbar
```rust
let mut toolbar = NSToolbar::new("EditorToolbar");

// Mode selector (Add | Select | Move)
let mode_popup = create_mode_selector();
toolbar.add_item(NSToolbarItem::with_view("mode", mode_popup));

// Spacer
toolbar.add_item(toolbar.create_flexible_space_item());

// Play controls
let play = toolbar.create_button_item("play", "Play", play_icon, || start_playback());
let pause = toolbar.create_button_item("pause", "Pause", pause_icon, || pause_playback());
toolbar.add_items(&[play, pause]);

// Another spacer
toolbar.add_item(toolbar.create_flexible_space_item());

// Search
let search = toolbar.create_search_item("search", |q| search_assets(q));
toolbar.add_item(search);

window.set_toolbar(toolbar);
```

### Centered Title
```rust
toolbar.set_shows_toolbar_button(false);
toolbar.set_centered_item("title");

let title_view = NSTextField::label("Untitled Project");
toolbar.add_item(NSToolbarItem::with_view("title", title_view.view_clone()));
```

## Toolbar Item Factory

```rust
impl NSToolbar {
    fn create_button_item(
        &mut self,
        identifier: &str,
        label: &str,
        image: Arc<NSImage>,
        action: impl Fn() + Send + Sync + 'static
    ) -> NSToolbarItem {
        let button = NSButton::button_with_image(image, action);
        NSToolbarItem::with_view(identifier, button.view_clone())
    }
    
    fn create_search_item(
        &mut self,
        identifier: &str,
        on_search: impl Fn(String) + Send + Sync + 'static
    ) -> NSToolbarItem {
        let mut search = NSSearchField::new(NSRect::new(0.0, 0.0, 200.0, 22.0));
        search.set_action(on_search);
        NSToolbarItem::with_view(identifier, search.view_clone())
    }
}
```

## Display Modes

```rust
// Icon only (compact)
toolbar.set_display_mode(NSToolbarDisplayMode::IconOnly);

// Icon and label
 toolbar.set_display_mode(NSToolbarDisplayMode::IconAndLabel);

// Label only
toolbar.set_display_mode(NSToolbarDisplayMode::LabelOnly);
```

## Customization

```rust
// Allow user to customize
toolbar.set_allows_user_customization(true);

// Default items
let default_items = vec!["save", "open", "flexible_space", "search"];
toolbar.set_default_item_identifiers(&default_items);

// All allowed items
let allowed_items = vec!["save", "open", "new", "flexible_space", "search", "mode"];
toolbar.set_allowed_item_identifiers(&allowed_items);
```

## Testing & Verification

```rust
let mut toolbar = NSToolbar::new("Test");

let item = NSToolbarItem::with_view("test", NSView::new(NSRect::new(0.0, 0.0, 44.0, 32.0)));
toolbar.add_item(item);

assert_eq!(toolbar.items().len(), 1);
assert_eq!(toolbar.items()[0].identifier(), "test");
```

## File Structure

```
src/toolbar/
└── mod.rs          # NSToolbar, NSToolbarItem, NSToolbarDisplayMode
```

## Missing from Apple's NSToolbar

- **Overflow menu**: Items that don't fit
- **Selectable items**: Radio-button behavior
- **Group items**: Section separators with labels
- **Full-width items**: Stretch to fill
- **Touch Bar**: NSTouchBar integration
- **Sheet attachment**: Per-window customization sheet

These are stubbed for current use cases.

## Common Patterns

| Pattern | Implementation |
|---------|----------------|
| **Left actions, right search** | Button items + flexibleSpace + search |
| **Centered mode selector** | flexibleSpace + popup + flexibleSpace |
| **Document controls** | New, Open, Save, flexibleSpace, Share |
| **Editing tools** | Undo, Redo, flexibleSpace, Search |
