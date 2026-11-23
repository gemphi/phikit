# NSSwitch — Modern Toggle Control

## Apple Header: `NSSwitch.h` (macOS 10.15+)

## Overview

`NSSwitch` provides a modern on/off toggle switch. Replaces checkbox for boolean settings. Mirrors Apple's [`NSSwitch`](https://developer.apple.com/documentation/appkit/nsswitch).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSSwitch Creation                            │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Visual State                                    │
│  ├─ isOn: false → Gray track, knob left                    │
│  └─ isOn: true  → Accent track, knob right                 │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 User Interaction                             │
│  ├─ Click anywhere on switch → Toggle state                 │
│  ├─ Animate knob position change                             │
│  └─ Fire action callback with new boolean value           │
└─────────────────────────────────────────────────────────────┘
```

## Comparison: NSSwitch vs NSButton

| Feature | NSSwitch | NSButton (Checkbox) |
|--------|----------|---------------------|
| Style | Modern iOS-style | Traditional macOS |
| Animation | Smooth slide | Instant |
| Label | Separate (add NSTextField) | Built-in title |
| Use Case | Settings, preferences | Forms, data entry |

## Connection to Other Modules

```
NSSwitch
├── Inherits: NSView (direct, not NSControl)
├── Used by: Settings panels (modern look)
├── Used by: Feature toggles
└── Paired with: NSTextField (for label)
```

## Usage Examples

### Basic Toggle (Actual API from mod.rs)
```rust
use rust_appkit::prelude::*;
use rust_appkit::switch::NSSwitch;

// Create (mod.rs:21-31)
let mut toggle = NSSwitch::new(NSRect::new(0.0, 0.0, 40.0, 24.0));

// Set state (mod.rs:33-36)
toggle.set_on(true);

// Get state (mod.rs:38)
let is_on = toggle.is_on();

// Set callback (mod.rs:40-42)
toggle.set_action(|is_on| {
    println!("Toggle is now: {}", is_on);
});

// View access
let view = toggle.view_mut();    // mod.rs:44
let view = toggle.view_clone();  // mod.rs:45
```

### With Label
```rust
let mut row = NSStackView::horizontal();
row.set_spacing(8.0);

let label = NSTextField::label("Dark Mode:");
let mut dark_mode_switch = NSSwitch::new(NSRect::new(0.0, 0.0, 40.0, 24.0));
dark_mode_switch.set_action(|on| {
    if on { enable_dark_mode(); } else { disable_dark_mode(); }
});

row.add_arranged_subview(label.view_clone());
row.add_arranged_subview(dark_mode_switch.view_clone());
```

### Settings Panel Pattern
```rust
fn settings_row(label: &str, initial: bool, action: impl Fn(bool) + Send + Sync + 'static) -> NSStackView {
    let mut row = NSStackView::horizontal();
    row.set_spacing(12.0);
    
    let label = NSTextField::label(label);
    label.view_mut().set_frame(NSRect::new(0.0, 0.0, 150.0, 22.0));
    
    let mut switch = NSSwitch::new(NSRect::new(0.0, 0.0, 40.0, 24.0));
    switch.set_on(initial);
    switch.set_action(action);
    
    // Push switch to right
    switch.view_mut().set_autoresizing_mask(
        NSAutoresizingMask::MIN_X_MARGIN
    );
    
    row.add_arranged_subview(label.view_clone());
    row.add_arranged_subview(switch.view_clone());
    row
}

// Usage
let auto_save = settings_row("Auto-save:", true, |on| set_auto_save(on));
let analytics = settings_row("Send analytics:", false, |on| set_analytics(on));
```

## Visual States

```
OFF State (is_on = false)
┌──────────────────────┐
│           ●          │  ← Knob on left
│    gray track        │
└──────────────────────┘

ON State (is_on = true)
┌──────────────────────┐
│           ●          │  ← Knob on right
│    blue/accent track │
└──────────────────────┘
```

## Actual Implementation (mod.rs)

```rust
// From src/switch/mod.rs
pub struct NSSwitch {
    view: NSView,
    inner: Arc<Mutex<NSSwitchState>>,
}

struct NSSwitchState {
    on: bool,
    action: Option<Arc<dyn Fn(bool) + Send + Sync>>,
}

// Public API:
impl NSSwitch {
    pub fn new(frame: NSRect) -> Self;           // line 21
    pub fn set_on(&mut self, on: bool);          // line 33
    pub fn is_on(&self) -> bool;                 // line 38
    pub fn set_action(&mut self, action: impl Fn(bool) + Send + Sync + 'static); // line 40
    pub fn view_mut(&mut self) -> &mut NSView;   // line 44
    pub fn view_clone(&self) -> NSView;          // line 45
}
```

## Testing & Verification

```rust
let mut switch = NSSwitch::new(NSRect::new(0.0, 0.0, 40.0, 24.0));
assert!(!switch.is_on());  // Default off

switch.set_on(true);
assert!(switch.is_on());
```

## File Structure

```
src/switch/
└── mod.rs          # NSSwitch, NSSwitchState
```

## Missing from Apple's NSSwitch

- **Accessibility**: Full VoiceOver support
- **Animation**: Smooth knob slide (current is instant)
- **Sizing**: Standard control sizes (small, regular, large)
- **Color variants**: Warning (orange), destructive (red)
- **Mixed state**: Indeterminate position

These are stubbed for current use cases.

## When to Use

| Use NSSwitch | Use NSButton (Checkbox) |
|--------------|---------------------------|
| Settings/preferences | Data entry forms |
| Feature toggles | Multi-select lists |
| Modern macOS apps | Legacy compatibility |
| iOS-style interface | Traditional desktop |
