# NSApplication — Application Lifecycle

## Apple Header: `NSApplication.h`

## Overview

`NSApplication` manages the application lifecycle, event loop, and global state. Mirrors Apple's [`NSApplication`](https://developer.apple.com/documentation/appkit/nsapplication).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSApplication Lifecycle                      │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Startup Sequence                                │
│  1. NSApplication.shared() creates singleton               │
│  2. Set delegate for app events                              │
│  3. Create main window                                       │
│  4. app.run() starts event loop                              │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Event Loop                                   │
│  ├─ Wait for NSEvent (blocking)                            │
│  ├─ Dispatch to window/view/responder                      │
│  ├─ Process until termination                              │
│  └─ Cleanup on exit                                          │
└─────────────────────────────────────────────────────────────┘
```

## Application States

| State | Description |
|-------|-------------|
| `Active` | Frontmost, receiving events |
| `Inactive` | Background, another app active |
| `Hidden` | All windows hidden |

## Connection to Other Modules

```
NSApplication
├── Owns: NSWindow[] (all windows)
├── Owns: NSMenu (main menu bar)
├── Uses: NSResponder (event routing)
└── Delegates to: NSApplicationDelegate
```

## Usage Examples

### Basic App
```rust
use rust_appkit::prelude::*;

fn main() {
    let app = NSApplication::shared();
    
    // Set delegate for lifecycle events
    app.set_delegate(MyAppDelegate::new());
    
    // Create main window
    let mut window = NSWindow::standard("My App", 800.0, 600.0);
    window.set_content_view(create_main_view());
    window.make_key_and_order_front();
    
    // Run event loop
    app.run().unwrap();
}

struct MyAppDelegate;

impl NSApplicationDelegate for MyAppDelegate {
    fn application_did_finish_launching(&self) {
        println!("App launched!");
    }
    
    fn application_will_terminate(&self) {
        println!("App terminating...");
    }
}
```

### Menu Bar Setup
```rust
let app = NSApplication::shared();

let main_menu = NSMenu::new();
main_menu.add_submenu(create_file_menu());
main_menu.add_submenu(create_edit_menu());
main_menu.add_submenu(create_view_menu());

app.set_main_menu(main_menu);
```

## Delegate Methods

| Method | Trigger |
|--------|---------|
| `application_did_finish_launching` | Startup complete |
| `application_will_terminate` | About to quit |
| `application_should_terminate` | Return false to cancel quit |
| `application_did_become_active` | App came to front |
| `application_will_resign_active` | App going to background |

## Testing & Verification

```rust
let app = NSApplication::shared();
assert!(app.windows().is_empty());

let window = NSWindow::standard("Test", 400.0, 300.0);
app.add_window(&window);

assert_eq!(app.windows().len(), 1);
```

## File Structure

```
src/application/
├── mod.rs          # NSApplication
└── delegate.rs     # NSApplicationDelegate
```

## Missing from Apple's NSApplication

- **Dock menu**: Right-click on app icon
- **Services**: System-wide services menu
- **User notifications**: NSUserNotification
- **Touch bar**: NSTouchBar integration
- **URL handling**: Open documents by URL
- **Restoration**: Resume previous state

These are stubbed for current use cases.
