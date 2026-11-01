# NSWindow — Top-Level Containers

## Apple Header: `NSWindow.h`

## Overview

`NSWindow` manages application windows. Mirrors Apple's [`NSWindow`](https://developer.apple.com/documentation/appkit/nswindow).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    NSWindow Lifecycle                        │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              initWithContentRect:styleMask:                  │
│  ├─ Create winit window                                      │
│  ├─ Set up event loop                                        │
│  ├─ Create contentView (NSView)                            │
│  └─ Initialize render context                                │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Event Loop                                │
│  ├─ NSEvent generated (mouse, keyboard)                     │
│  ├─ Route to firstResponder                                  │
│  ├─ Hit test to find target view                           │
│  └─ Delegate mouseDown/mouseUp/etc                         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Render Cycle                              │
│  ├─ Redraw requested                                         │
│  ├─ contentView drawRect called                              │
│  ├─ Recursively draw subviews                              │
│  └─ Swap buffers (GPU) or present (software)               │
└─────────────────────────────────────────────────────────────┘
```

## Window Styles (NSWindowStyle)

| Style | Features | Use Case |
|-------|----------|----------|
| `Titled` | Title bar + close/minimize/zoom | Standard document windows |
| `Closable` | Close button | All standard windows |
| `Miniaturizable` | Minimize button | Document windows |
| `Resizable` | Resize handles | Adjustable windows |
| `Borderless` | No decorations | Splash screens, overlays |

## Connection to Other Modules

```
NSWindow
├── contentView: NSView
│   ├── Menu bar (NSMenu)
│   ├── Toolbar (NSToolbar)
│   └── Split view (NSSplitView)
│       ├── Left sidebar
│       ├── Center viewport
│       └── Right inspector
├── firstResponder: NSResponder
│   └── Tracks keyboard focus
└── delegate: NSWindowDelegate
    └── Handles window events
```

## Usage Examples

### Standard Window
```rust
use rust_appkit::prelude::*;

let mut window = NSWindow::new(
    NSRect::new(100.0, 100.0, 800.0, 600.0),
    NSWindowStyle::Titled | NSWindowStyle::Closable | NSWindowStyle::Resizable
);
window.set_title("Reality Composer Pro");

// Set content
let content = create_main_interface();
window.content_view().add_subview(content);
```

### Panel Window
```rust
let panel = NSWindow::new(
    NSRect::new(400.0, 300.0, 300.0, 200.0),
    NSWindowStyle::Titled | NSWindowStyle::Closable
);
panel.set_title("Settings");
```

### Window with Delegate
```rust
struct MyWindowDelegate;

impl NSWindowDelegate for MyWindowDelegate {
    fn window_will_close(&mut self, _window: &mut NSWindow) {
        println!("Window closing!");
    }
}

window.set_delegate(MyWindowDelegate);
```

## Event Routing

```
NSEvent occurs
        │
        ▼
NSWindow.firstResponder?
        │ NO
        ▼
Hit test contentView
        │
        ▼
Find NSView at mouse position
        │
        ▼
Call view.delegate.mouseDown()
        │
        ▼
View can call setNeedsDisplay()
```

## File Structure

```
src/window/
├── mod.rs          # NSWindow, NSWindowStyle
└── delegate.rs     # NSWindowDelegate trait
```

## Missing from Apple's NSWindow

- **Sheet windows**: Modal child windows
- **Drawer windows**: Side drawers
- **Full screen**: Native full-screen support
- **Miniaturize**: Minimize to dock
- **Tabs**: macOS Sierra tabbed windows
- **Collection behavior**: Spaces/expose integration

These require platform-specific implementations.
