# NSSearchField — Search Input Control

## Apple Header: `NSSearchField.h`

## Overview

`NSSearchField` provides a dedicated search input with built-in search/cancel buttons and recents menu. Mirrors Apple's [`NSSearchField`](https://developer.apple.com/documentation/appkit/nssearchfield).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSSearchField Structure                      │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Components                                        │
│  ├─ Text field: User input                                  │
│  ├─ Search button: Magnifying glass (left)                  │
│  ├─ Cancel button: X to clear (right, when text)            │
│  └─ Recents menu: Dropdown of recent searches               │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Search Flow                                  │
│  ├─ User types query                                         │
│  ├─ Optional: Live search on text change                    │
│  ├─ User presses Return: Execute search                     │
│  ├─ Query added to recents                                  │
│  └─ Delegate/search action invoked                          │
└─────────────────────────────────────────────────────────────┘
```

## Search Modes

| Mode | Behavior |
|------|----------|
| **Instant** | Search on every keystroke |
| **On Return** | Search when user presses Return |
| **Delayed** | Debounced (wait for pause) |

## Connection to Other Modules

```
NSSearchField
├── Inherits: NSTextField
│   └── Inherits: NSControl
├── Used by: Toolbars (global search)
├── Used by: Filters (content filtering)
└── Used by: Find panels
```

## Usage Examples

### Basic Search
```rust
use rust_appkit::prelude::*;
use rust_appkit::search_field::NSSearchField;

let mut search = NSSearchField::new(NSRect::new(0.0, 0.0, 200.0, 22.0));

// Set placeholder
search.set_placeholder("Search assets...");

// Search action
search.set_action(|query| {
    if query.is_empty() {
        show_all_assets();
    } else {
        filter_assets(&query);
    }
});
```

### Toolbar Search
```rust
let mut toolbar_search = NSSearchField::new(NSRect::new(0.0, 0.0, 200.0, 22.0));
toolbar_search.set_placeholder("Search...");

// Live search (search as user types)
toolbar_search.set_sends_search_string_immediately(true);

toolbar_search.set_action(|query| {
    search_all(query);
});

toolbar.add_item(NSToolbarItem::with_view("search", toolbar_search.view_clone()));
```

### With Recents
```rust
let mut search = NSSearchField::new(NSRect::new(0.0, 0.0, 200.0, 22.0));

// Enable recents menu
search.set_recent_searches(vec![
    "player mesh".to_string(),
    "environment light".to_string(),
    "wood material".to_string(),
]);
search.set_maximum_recents(10);
search.set_sends_whole_search_string(true);
```

## Delegate Methods

```rust
trait NSSearchFieldDelegate {
    // Called when search initiated
    fn search_field_search_string_changed(&self, search_field: &NSSearchField, search_string: &str);
    
    // Called when user selects from recents
    fn search_field_did_select_recent_search(&self, search_field: &NSSearchField, search_string: &str);
}
```

## Testing & Verification

```rust
let mut search = NSSearchField::new(NSRect::new(0.0, 0.0, 150.0, 22.0));
search.set_placeholder("Search...");

assert_eq!(search.placeholder(), "Search...");

// Simulate search
search.set_text("test query");
search.perform_search();
// Action should fire with "test query"
```

## File Structure

```
src/search_field/
└── mod.rs          # NSSearchField, NSSearchFieldDelegate
```

## Missing from Apple's NSSearchField

- **Scope buttons**: Category filters (All, Images, Files)
- **Token field**: Search with tokens/chips
- **Autocompletion**: Suggested completions
- **Search history**: Persistent recents
- **Clear button modes**: Always/never/when editing

These are stubbed for current use cases.
