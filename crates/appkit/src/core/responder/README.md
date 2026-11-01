# NSResponder — Event Handling Chain

## Apple Header: `NSResponder.h`

## Overview

`NSResponder` is the abstract base class for all objects that handle events. Mirrors Apple's [`NSResponder`](https://developer.apple.com/documentation/appkit/nsresponder).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 The Responder Chain                          │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Event Routing Path                                │
│                                                              │
│   User Event ──► First Responder ──► Next Responder ──► ... │
│        │              │                   │                   │
│        │              ▼                   ▼                   │
│        │         Current Focus       Parent View             │
│        │         (Text Field)        (Container)             │
│        │                               │                     │
│        │                               ▼                     │
│        │                         Window                       │
│        │                               │                     │
│        │                               ▼                     │
│        └────────────────────────►  Application                 │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Responder Chain Order

```
1. First Responder (focused view)
2. Parent view
3. Window content view
4. Window
5. Application
6. (event handled or discarded)
```

## Event Types Handled

| Event | Method | Description |
|-------|--------|-------------|
| **Mouse Down** | `mouseDown()` | Left click pressed |
| **Mouse Up** | `mouseUp()` | Left click released |
| **Right Mouse** | `rightMouseDown/Up()` | Right click |
| **Mouse Drag** | `mouseDragged()` | Move while pressed |
| **Mouse Move** | `mouseMoved()` | Hover tracking |
| **Scroll** | `scrollWheel()` | Trackpad/mouse wheel |
| **Key Down** | `keyDown()` | Keyboard press |
| **Key Up** | `keyUp()` | Keyboard release |
| **Flags Changed** | `flagsChanged()` | Modifiers (⌘, ⌥, etc) |

## Connection to Other Modules

```
NSResponder
├── Implemented by: NSView, NSWindow, NSApplication
├── NSView responders:
│   ├── NSControl (interactive)
│   └── Custom views
└── Event routing:
    ├── NSEvent (event data)
    └── NSWindow (first responder management)
```

## Usage Examples

### First Responder
```rust
use rust_appkit::prelude::*;

// Make text field accept keyboard input
let text_field = NSTextField::new(NSRect::new(0.0, 0.0, 200.0, 22.0));
text_field.set_editable(true);

// Window routes keyboard events to first responder
window.make_first_responder(text_field.view_clone());
```

### Custom Event Handling
```rust
struct CustomViewDelegate;

impl NSViewDelegate for CustomViewDelegate {
    fn mouse_down(&mut self, view: &mut NSView, event: &NSEvent) {
        let location = event.location_in_window();
        println!("Clicked at: {:?}", location);
        
        // Mark for redraw
        view.set_needs_display();
    }
    
    fn key_down(&mut self, _view: &mut NSView, event: &NSEvent) {
        match event.key_code() {
            53 => println!("Escape pressed"), // ESC key
            36 => println!("Enter pressed"),  // Return key
            _ => {}
        }
    }
}
```

### Responder Chain Navigation
```rust
// In a view, pass event to next responder
if !self.handle_event(event) {
    // Pass to parent
    self.next_responder().mouse_down(event);
}
```

## First Responder Chain

```
User presses key 'a'
        │
        ▼
Window.firstResponder? (some text field)
        │
        ▼
Text field accepts the key
        │
        ▼
Character 'a' appears in field
```

## Testing & Verification

```rust
// Test responder chain
let view = NSView::new(NSRect::new(0.0, 0.0, 100.0, 100.0));
let window = NSWindow::new(frame, styles);

window.make_first_responder(view.clone());
assert_eq!(window.first_responder().id, view.id);
```

## File Structure

```
src/responder/
└── mod.rs          # NSResponder, NSResponderChain
```

## Missing from Apple's NSResponder

- **Undo manager**: NSUndoManager integration
- **Menu validation**: validateMenuItem
- **User activities**: NSUserActivity handling
- **Touch events**: NSTouch (trackpad)
- **Pressure**: NSPressureConfiguration
- **Quick Look**: preview item

These are stubbed for current use cases.
