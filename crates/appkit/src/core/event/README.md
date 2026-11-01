# NSEvent — User Input Events

## Apple Header: `NSEvent.h`

## Overview

`NSEvent` represents user input events. Mirrors Apple's [`NSEvent`](https://developer.apple.com/documentation/appkit/nsevent).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 Event Creation                               │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Event Types                                       │
│  ├─ Mouse: Left, right, other buttons                     │
│  ├─ Keyboard: Key down/up, flags changed                  │
│  ├─ Scroll: Wheel, trackpad gesture                         │
│  └─ Special: Periodic, application defined                │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Event Routing                                │
│  ├─ Window receives event from platform                      │
│  ├─ Dispatches to first responder or hit-tested view       │
│  └─ View handles via NSResponder methods                   │
└─────────────────────────────────────────────────────────────┘
```

## Event Types

| Type | Description | Properties |
|------|-------------|------------|
| `LeftMouseDown` | Left click press | `locationInWindow`, `clickCount` |
| `LeftMouseUp` | Left click release | `locationInWindow` |
| `RightMouseDown` | Right click press | `locationInWindow` |
| `KeyDown` | Key pressed | `keyCode`, `characters` |
| `KeyUp` | Key released | `keyCode` |
| `ScrollWheel` | Scroll gesture | `scrollingDeltaX/Y` |
| `FlagsChanged` | Modifiers changed | `modifierFlags` |

## Modifier Flags

| Flag | Key |
|------|-----|
| `NSEventModifierFlagCommand` | ⌘ Command |
| `NSEventModifierFlagOption` | ⌥ Option |
| `NSEventModifierFlagControl` | ⌃ Control |
| `NSEventModifierFlagShift` | ⇧ Shift |

## Connection to Other Modules

```
NSEvent
├── Created by: NSWindow (from platform)
├── Handled by: NSResponder (mouseDown, keyDown, etc)
├── Uses: NSPoint (locations)
└── Uses: NSWindow (window reference)
```

## Usage Examples

### Mouse Event
```rust
use rust_appkit::prelude::*;

impl NSViewDelegate for MyView {
    fn mouse_down(&mut self, view: &mut NSView, event: &NSEvent) {
        let location = event.location_in_window();
        let click_count = event.click_count();
        
        if click_count == 2 {
            println!("Double click at {:?}", location);
        } else {
            println!("Click at {:?}", location);
        }
    }
}
```

### Keyboard Event
```rust
fn key_down(&mut self, _view: &mut NSView, event: &NSEvent) {
    match event.key_code() {
        53 => println!("Escape"),
        36 => println!("Return/Enter"),
        48 => println!("Tab"),
        51 => println!("Delete"),
        123 => println!("Left arrow"),
        124 => println!("Right arrow"),
        125 => println!("Down arrow"),
        126 => println!("Up arrow"),
        _ => println!("Key: {}", event.key_code()),
    }
}
```

### Modifier Check
```rust
fn flags_changed(&mut self, _view: &mut NSView, event: &NSEvent) {
    let flags = event.modifier_flags();
    
    if flags.contains(NSEventModifierFlagCommand) {
        println!("Command pressed");
    }
    
    if flags.contains(NSEventModifierFlagShift) {
        println!("Shift pressed");
    }
}
```

### Scroll Event
```rust
fn scroll_wheel(&mut self, view: &mut NSView, event: &NSEvent) {
    let dx = event.scrolling_delta_x();
    let dy = event.scrolling_delta_y();
    
    // Update scroll position
    self.scroll_x += dx;
    self.scroll_y += dy;
    
    view.set_needs_display();
}
```

## Key Codes (Common)

| Code | Key |
|------|-----|
| 53 | Escape |
| 36 | Return |
| 48 | Tab |
| 51 | Delete |
| 117 | Forward Delete |
| 123 | Left Arrow |
| 124 | Right Arrow |
| 125 | Down Arrow |
| 126 | Up Arrow |
| 116 | Home |
| 121 | End |
| 115 | Page Up |
| 119 | Page Down |

## Testing & Verification

```rust
let event = NSEvent::mouse_event(
    NSEventType::LeftMouseDown,
    NSPoint::new(100.0, 200.0),
    0, // modifier flags
    1.0, // timestamp
    1, // click count
);

assert_eq!(event.event_type(), NSEventType::LeftMouseDown);
assert_eq!(event.location_in_window(), NSPoint::new(100.0, 200.0));
```

## File Structure

```
src/event/
└── mod.rs          # NSEvent, NSEventType, NSEventModifierFlags
```

## Missing from Apple's NSEvent

- **Touch events**: NSTouch (trackpad)
- **Pressure**: Force touch
- **Rotation**: Trackpad rotate gesture
- **Magnification**: Pinch zoom
- **Swipe**: Three-finger swipe
- **Phase**: Gesture begin/change/end
- **Precise scrolling**: High-res trackpad deltas

These are stubbed for current use cases.
