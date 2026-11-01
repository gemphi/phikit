# NSMenu — Application Menus

## Apple Header: `NSMenu.h`

## Overview

`NSMenu` provides application menus, context menus, and the menu bar. Mirrors Apple's [`NSMenu`](https://developer.apple.com/documentation/appkit/nsmenu).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSMenu Creation                              │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Menu Bar Structure                              │
│  ├─ NSApp.mainMenu (top-level bar)                         │
│  │   ├── File Menu                                          │
│  │   ├── Edit Menu                                           │
│  │   ├── View Menu                                           │
│  │   └── Window Menu                                          │
│  └── Each menu contains NSMenuItems                          │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Menu Item Action                             │
│  ├─ User clicks menu item                                     │
│  ├─ NSMenuItem.target + action invoked                       │
│  └─ Menu closes, action executes                            │
└─────────────────────────────────────────────────────────────┘
```

## Menu Types

| Type | Description | Example |
|------|-------------|---------|
| **Main Menu** | Application menu bar | File, Edit, View |
| **Context Menu** | Right-click popup | Copy, Paste, Delete |
| **Submenu** | Nested menu | File → Open Recent → [files] |
| **Dock Menu** | App icon right-click | Recent documents |

## Connection to Other Modules

```
NSMenu
├── Contains: NSMenuItem[] (menu entries)
│   ├── Title + Key Equivalent (⌘S)
│   ├── Action (closure/target)
│   └── Submenu (optional nested menu)
├── Used by: NSApplication (mainMenu)
├── Used by: NSWindow (window menu)
└── Used by: NSControl (context menus)
```

## Usage Examples

### Menu Bar
```rust
use rust_appkit::prelude::*;

let mut app = NSApplication::shared();

// Create main menu
let main_menu = NSMenu::new();

// File menu
let mut file_menu = NSMenu::new("File");
file_menu.add_item(NSMenuItem::new("New", "n", || new_document()));
file_menu.add_item(NSMenuItem::new("Open...", "o", || open_document()));
file_menu.add_separator();
file_menu.add_item(NSMenuItem::new("Save", "s", || save_document()));
file_menu.add_item(NSMenuItem::new("Save As...", "S", || save_as()));
file_menu.add_separator();
file_menu.add_item(NSMenuItem::new("Quit", "q", || quit_app()));

main_menu.add_submenu(file_menu);

// Edit menu
let mut edit_menu = NSMenu::new("Edit");
edit_menu.add_item(NSMenuItem::new("Undo", "z", || undo()));
edit_menu.add_item(NSMenuItem::new("Redo", "Z", || redo()));
edit_menu.add_separator();
edit_menu.add_item(NSMenuItem::new("Cut", "x", || cut()));
edit_menu.add_item(NSMenuItem::new("Copy", "c", || copy()));
edit_menu.add_item(NSMenuItem::new("Paste", "v", || paste()));

main_menu.add_submenu(edit_menu);

app.set_main_menu(main_menu);
```

### Context Menu
```rust
// Right-click menu on a view
let mut context_menu = NSMenu::new();

context_menu.add_item(NSMenuItem::new("Cut", "", || cut_selection()));
context_menu.add_item(NSMenuItem::new("Copy", "", || copy_selection()));
context_menu.add_item(NSMenuItem::new("Paste", "", || paste_selection()));
context_menu.add_separator();
context_menu.add_item(NSMenuItem::new("Delete", "", || delete_selection()));

// Show at mouse position
context_menu.pop_up_at_point(mouse_location, view);
```

### Submenu
```rust
let mut file_menu = NSMenu::new("File");

// Open Recent submenu
let mut recent_menu = NSMenu::new("Open Recent");
for doc in recent_documents() {
    recent_menu.add_item(NSMenuItem::new(&doc.name, "", 
        move || open_specific(&doc.path)));
}
recent_menu.add_separator();
recent_menu.add_item(NSMenuItem::new("Clear Menu", "", || clear_recent()));

let recent_item = NSMenuItem::new("Open Recent", "", || {});
recent_item.set_submenu(recent_menu);
file_menu.add_item(recent_item);
```

## rust-kit-pro Menu Structure

```
File
├── New Project (⌘N)
├── Open Project... (⌘O)
├── Open Recent →
│   ├── [Project 1]
│   ├── [Project 2]
│   └── Clear Menu
├── ──────────────
├── Save (⌘S)
├── Save As... (⌘⇧S)
├── ──────────────
├── Import... (⌘I)
│   ├── USD/USDZ
│   ├── glTF/glb
│   └── Blender .blend
├── Export... (⌘E)
│   └── PHX
└── Quit (⌘Q)

Edit
├── Undo (⌘Z)
├── Redo (⌘⇧Z)
├── ──────────────
├── Cut (⌘X)
├── Copy (⌘C)
├── Paste (⌘V)
├── Delete (⌘⌫)
└── Select All (⌘A)

View
├── Show/Hide Sidebar (⌘⌥S)
├── Show/Hide Inspector (⌘⌥I)
├── Show/Hide Timeline (⌘⌥T)
├── ──────────────
├── Enter Full Screen (⌘⌃F)
└── Actual Size (⌘0)
```

## Key Equivalents

| Key | String | Modifier |
|-----|--------|----------|
| ⌘S | `"s"` | Command |
| ⌘⇧S | `"S"` | Command+Shift |
| ⌘⌥S | `"s"` + `NSAlternateKeyMask` | Command+Option |
| ⌃Tab | `"\t"` + `NSControlKeyMask` | Control |
| Esc | `"\u{1b}"` | None |
| Enter | `"\r"` | None |

## Testing & Verification

```rust
let mut menu = NSMenu::new("Test");

let item = NSMenuItem::new("Test Item", "t", || println!("clicked"));
menu.add_item(item);

assert_eq!(menu.items().len(), 1);
assert_eq!(menu.items()[0].title(), "Test Item");
```

## File Structure

```
src/menu/
├── mod.rs          # NSMenu, NSMenuItem
└── menu_bar.rs     # NSMenuBar (top-level bar)
```

## Missing from Apple's NSMenu

- **State items**: Checkmarks, radio groups
- **Attributed titles**: Rich text menu items
- **Images**: Icons in menu items
- **Tooltips**: Help text on hover
- **Alternates**: Different action with modifier
- **Indentation**: Hierarchical visual grouping
- **Tear-off**: Detachable menus (classic Mac)

These are stubbed for current use cases.
