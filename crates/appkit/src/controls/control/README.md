# NSControl — Interactive Control Base Class

## Apple Header: `NSControl.h`

## Overview

`NSControl` is the abstract base class for all interactive controls. Mirrors Apple's [`NSControl`](https://developer.apple.com/documentation/appkit/nscontrol).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSControl Inheritance                        │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Base Class Chain                                │
│  NSResponder                                               │
│     └── NSView (frame, bounds, drawing)                     │
│            └── NSControl (action, target, value)           │
│                   ├── NSButton                             │
│                   ├── NSTextField                          │
│                   └── NSSlider                             │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Target-Action Pattern                        │
│  ├─ Control stores: action closure                          │
│  ├─ User interacts (clicks, types, drags)                   │
│  └─ Control.sendAction() invokes the closure               │
└─────────────────────────────────────────────────────────────┘
```

## Key Concepts

| Concept | Description | Example |
|---------|-------------|---------|
| **Target** | Object receiving action | Usually `self` or controller |
| **Action** | Method/closure to invoke | `\|\| save_file()` |
| **Value** | Current control value | Button state, slider position |
| **Enabled** | Accepts user input | `setEnabled(true/false)` |

## Connection to Other Modules

```
NSControl
├── Inherits: NSView
├── Implemented by: NSButton, NSTextField, NSSlider
├── Uses: NSControlStateValue (int state)
└── Uses: NSFormatter (value formatting)
```

## Usage Examples

### Creating a Custom Control
```rust
use rust_appkit::prelude::*;
use rust_appkit::control::NSControl;

struct MyCustomControl {
    control: NSControl,
    value: f32,
}

impl MyCustomControl {
    pub fn new(frame: NSRect) -> Self {
        let mut control = NSControl::new(frame);
        control.set_action(|| {
            println!("Custom control activated");
        });
        
        Self {
            control,
            value: 0.0,
        }
    }
    
    pub fn set_value(&mut self, v: f32) {
        self.value = v;
        // Update display
        self.control.view_mut().set_needs_display();
    }
}
```

### Enabled/Disabled State
```rust
let mut button = NSButton::button_with_title("Save", || save());

// Disable during operation
button.control.view_mut().set_enabled(false);

// Re-enable after
button.control.view_mut().set_enabled(true);
```

### State Values
```rust
// NSControlStateValue values
const NS_CONTROL_STATE_VALUE_OFF: i64 = 0;
const NS_CONTROL_STATE_VALUE_ON: i64 = 1;
const NS_CONTROL_STATE_VALUE_MIXED: i64 = -1;

let checkbox = NSButton::checkbox_with_title("Option", || {});
checkbox.set_state(NS_CONTROL_STATE_VALUE_ON);
```

## Control State Machine

```
┌─────────┐     setEnabled(false)      ┌──────────┐
│ Enabled │ ─────────────────────────► │ Disabled │
│  (gray) │ ◄───────────────────────── │ (dimmed) │
└─────────┘     setEnabled(true)       └──────────┘
```

## File Structure

```
src/control/
└── mod.rs          # NSControl, NSControlStateValue
```

## Missing from Apple's NSControl

- **Formatter**: NSFormatter for value display
- **Validation**: NSUserInterfaceValidations protocol
- **Cell**: NSCell architecture (legacy)
- **Tag**: Integer identifier for finding controls
- **Baseline**: Text baseline alignment

These are stubbed for current use cases.
