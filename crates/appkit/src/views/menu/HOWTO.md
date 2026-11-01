# Menu — How to Use

## Files
- `mod.rs` — re-exports, `MenuAction` type alias
- `menu.rs` — `Menu`, `MenuBar` structs + constructors
- `item.rs` — `MenuItem` definition (title, action, shortcut)
- `delegate.rs` — `MenuBarDelegate` for rendering the menu bar
- `dropdown.rs` — `MenuDropdownDelegate`, `MenuOverlayDelegate` for popups
- `view.rs` — `MenuBarView` rendering logic
- `overlay.rs` — overlay layer for dropdown display
- `macros.rs` — helper macros for menu construction

## Quick Start
```rust
use rust_appkit::views::menu::{Menu, MenuBar, MenuItem, MenuBarView};

// Create a menu with items
let mut file_menu = Menu::new("File");
file_menu.add_item(MenuItem::new("New", || create_new()));
file_menu.add_item(MenuItem::new("Open", || open_file()));
file_menu.add_item(MenuItem::separator());
file_menu.add_item(MenuItem::new("Quit", || quit_app()));

// Build the menu bar
let mut menu_bar = MenuBar::new();
menu_bar.add_menu(file_menu);

// Attach to a MenuBarView for rendering
let menu_view = MenuBarView::new(frame, menu_bar);
```

## Key APIs
- `Menu::new(title)` — create a named menu
- `MenuItem::new(title, action)` — create a clickable item
- `MenuItem::separator()` — visual divider between items
- `MenuBar::new()` — top-level menu bar container
- `MenuBar::add_menu(menu)` — append a menu to the bar
- `MenuBarView::new(frame, bar)` — renderable menu bar view

## Rules
- `MenuAction` = `Arc<dyn Fn() + Send + Sync>` — actions are thread-safe closures
- Delegate handles drawing and hover/click state
- Dropdown overlays are separate views layered on top
- Use `MenuItem::separator()` for visual grouping
